//! Interactive notebook session with dirty tracking.

use crate::diagnose::Diagnostic;
use crate::execute::{
    append_item, delete_item_block, insert_after_item, prepend_item, replace_cell_source,
    replace_definition_source, replace_helper_source, replace_item_block, replace_markdown_content,
    replace_preamble_block, swap_item_blocks, write_notebook, CellOutput, ExecuteOptions, Executor,
};
use crate::fmt::{rustfmt_cell_source, rustfmt_file};
use crate::graph::{self, dependents, transitive_dependents, DependencyGraph};
use crate::notebook::{Notebook, OrderEntry};
use crate::parse::parse_notebook;
use crate::source_util::{fresh_name, strip_labrs_attrs, with_labrs_attr};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellStatus {
    Pristine,
    Running,
    Success,
    Error,
    Dirty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellState {
    pub name: String,
    pub status: CellStatus,
    pub dirty: bool,
    pub output: Option<CellOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub path: PathBuf,
    pub cells: Vec<CellState>,
    pub helpers: Vec<String>,
    pub definitions: Vec<String>,
    pub markdown: Vec<(String, String)>,
    pub graph: DependencyGraph,
    pub diagnostics: Vec<Diagnostic>,
    pub order: Vec<crate::notebook::OrderEntry>,
    /// When true, running a cell auto-runs dependents whose upstream output changed.
    pub auto_react: bool,
}

pub struct Session {
    pub path: PathBuf,
    pub notebook: Notebook,
    pub graph: DependencyGraph,
    pub outputs: HashMap<String, CellOutput>,
    pub dirty: HashMap<String, bool>,
    /// Pluto-style automatic cascade (default: true).
    pub auto_react: bool,
    executor: Executor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddKind {
    Cell,
    Helper,
    Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveDirection {
    Up,
    Down,
}

impl AddKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AddKind::Cell => "cell",
            AddKind::Helper => "helper",
            AddKind::Markdown => "markdown",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "cell" => Ok(AddKind::Cell),
            "helper" => Ok(AddKind::Helper),
            "markdown" | "md" => Ok(AddKind::Markdown),
            other => bail!("unknown kind `{other}`"),
        }
    }
}

/// Ensure a helper looks like a runnable cell function.
fn ensure_cellish_fn(source: &str, name: &str) -> String {
    let s = source.trim();
    if s.contains(&format!("fn {name}")) {
        // Promote to pub if needed
        if s.contains(&format!("pub fn {name}")) {
            s.to_string()
        } else {
            s.replacen(&format!("fn {name}"), &format!("pub fn {name}"), 1)
        }
    } else {
        format!("pub fn {name}() -> i32 {{\n    0\n}}\n")
    }
}

/// True if `ident` appears as a whole word in `src`.
fn ident_used_in(src: &str, ident: &str) -> bool {
    if ident.is_empty() {
        return false;
    }
    let bytes = src.as_bytes();
    let id = ident.as_bytes();
    let mut i = 0;
    while i + id.len() <= bytes.len() {
        if &bytes[i..i + id.len()] == id {
            let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            let after = i + id.len();
            let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

impl Session {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let notebook = parse_notebook(&path)?;
        let graph = graph::build_graph(&notebook);
        let cache = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".labrs")
            .join("cache");
        let executor = Executor::new(&ExecuteOptions {
            rustfmt: false,
            cache_dir: Some(cache),
        })?;
        let mut dirty = HashMap::new();
        for c in &notebook.cells {
            dirty.insert(c.name.clone(), false);
        }
        Ok(Self {
            path,
            notebook,
            graph,
            outputs: HashMap::new(),
            dirty,
            auto_react: true,
            executor,
        })
    }

    pub fn set_auto_react(&mut self, enabled: bool) {
        self.auto_react = enabled;
    }

    pub fn delete_item(&mut self, kind: AddKind, name: &str) -> Result<()> {
        let new_file = delete_item_block(&self.notebook.source, kind.as_str(), name)?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.outputs.remove(name);
        self.dirty.remove(name);
        // Dependents of a deleted cell become dirty / invalid
        if kind == AddKind::Cell {
            for d in dependents(&self.graph, name) {
                self.dirty.insert(d, true);
            }
        }
        self.reload()?;
        Ok(())
    }

    /// Move an item up or down among notebook display items (cells / helpers / markdown).
    pub fn move_item(&mut self, kind: AddKind, name: &str, direction: MoveDirection) -> Result<()> {
        let movable: Vec<(AddKind, String)> = self
            .notebook
            .order
            .iter()
            .filter_map(|e| match e {
                OrderEntry::Cell { id } => Some((AddKind::Cell, id.clone())),
                OrderEntry::Helper { name } => Some((AddKind::Helper, name.clone())),
                OrderEntry::Markdown { id } => Some((AddKind::Markdown, id.clone())),
                OrderEntry::Definition { .. } => None,
            })
            .collect();

        let idx = movable
            .iter()
            .position(|(k, n)| *k == kind && n == name)
            .with_context(|| format!("item `{name}` not found in order"))?;

        let swap_with = match direction {
            MoveDirection::Up => {
                if idx == 0 {
                    return Ok(());
                }
                idx - 1
            }
            MoveDirection::Down => {
                if idx + 1 >= movable.len() {
                    return Ok(());
                }
                idx + 1
            }
        };

        let (k2, n2) = &movable[swap_with];
        let new_file = swap_item_blocks(
            &self.notebook.source,
            kind.as_str(),
            name,
            k2.as_str(),
            n2,
        )?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.notebook = parse_notebook(&self.path)?;
        self.graph = graph::build_graph(&self.notebook);
        for c in &self.notebook.cells {
            self.dirty.entry(c.name.clone()).or_insert(false);
        }
        Ok(())
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        let cells = self
            .notebook
            .cells
            .iter()
            .map(|c| {
                let output = self.outputs.get(&c.name).cloned();
                let is_dirty = self.dirty.get(&c.name).copied().unwrap_or(false);
                let status = if is_dirty && output.is_some() {
                    CellStatus::Dirty
                } else if let Some(o) = &output {
                    if o.success {
                        CellStatus::Success
                    } else {
                        CellStatus::Error
                    }
                } else {
                    CellStatus::Pristine
                };
                CellState {
                    name: c.name.clone(),
                    status,
                    dirty: is_dirty,
                    output,
                }
            })
            .collect();

        SessionSnapshot {
            path: self.path.clone(),
            cells,
            helpers: self.notebook.helpers.iter().map(|h| h.name.clone()).collect(),
            definitions: self
                .notebook
                .definitions
                .iter()
                .map(|d| d.name.clone())
                .collect(),
            markdown: self
                .notebook
                .markdown
                .iter()
                .map(|m| (m.name.clone(), m.content.clone()))
                .collect(),
            graph: self.graph.clone(),
            diagnostics: self.graph.diagnostics.clone(),
            order: self.notebook.order.clone(),
            auto_react: self.auto_react,
        }
    }

    /// Edit a cell source from the UI (no `#[labrs::cell]` — added on write).
    pub fn edit_cell(&mut self, name: &str, new_source: &str) -> Result<String> {
        let cell = self
            .notebook
            .cell(name)
            .with_context(|| format!("unknown cell `{name}`"))?
            .clone();
        let for_disk = with_labrs_attr(new_source, "cell");
        let formatted = rustfmt_cell_source(&for_disk);
        let new_file = replace_cell_source(&self.notebook.source, &cell, &formatted)?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        if self.outputs.contains_key(name) {
            self.dirty.insert(name.to_string(), true);
        }
        Ok(strip_labrs_attrs(&formatted))
    }

    /// Edit a helper function (attributes optional; not required on disk).
    /// Marks cells that reference the helper (and their dependents) dirty.
    pub fn edit_helper(&mut self, name: &str, new_source: &str) -> Result<String> {
        let helper = self
            .notebook
            .helpers
            .iter()
            .find(|h| h.name == name)
            .with_context(|| format!("unknown helper `{name}`"))?
            .clone();
        let stripped = strip_labrs_attrs(new_source);
        let formatted = rustfmt_cell_source(&stripped);
        let new_file = replace_helper_source(
            &self.notebook.source,
            name,
            &helper.source,
            &formatted,
        )?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        for cell_name in self.cells_affected_by_helper(name) {
            self.dirty.insert(cell_name, true);
        }
        Ok(strip_labrs_attrs(&formatted))
    }

    /// Edit a shared definition (struct, use, impl, …) by parsed name.
    /// Marks all cells dirty (compile universe may change).
    pub fn edit_definition(&mut self, name: &str, new_source: &str) -> Result<String> {
        let def = self
            .notebook
            .definitions
            .iter()
            .find(|d| d.name == name)
            .with_context(|| format!("unknown definition `{name}`"))?
            .clone();
        let formatted = rustfmt_cell_source(new_source.trim());
        let new_file = replace_definition_source(
            &self.notebook.source,
            name,
            def.span,
            &def.source,
            &formatted,
        )?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        for cell in &self.notebook.cells {
            self.dirty.insert(cell.name.clone(), true);
        }
        let refreshed = self
            .notebook
            .definitions
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.source.clone())
            .unwrap_or_else(|| formatted.clone());
        Ok(refreshed)
    }

    /// Edit all preamble items (`use`, etc.) as a single source block.
    pub fn edit_preamble(&mut self, new_source: &str) -> Result<String> {
        let preamble: Vec<(String, (usize, usize), String)> = self
            .notebook
            .definitions
            .iter()
            .filter(|d| d.kind == "use" || d.kind == "item")
            .map(|d| (d.name.clone(), d.span, d.source.clone()))
            .collect();
        let formatted = rustfmt_cell_source(new_source.trim());
        let new_file = replace_preamble_block(&self.notebook.source, &preamble, &formatted)?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        for cell in &self.notebook.cells {
            self.dirty.insert(cell.name.clone(), true);
        }
        Ok(Self::join_preamble(&self.notebook.definitions))
    }

    fn join_preamble(definitions: &[crate::notebook::SharedDef]) -> String {
        definitions
            .iter()
            .filter(|d| d.kind == "use" || d.kind == "item")
            .map(|d| d.source.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Cells that reference `helper` by name, plus their transitive dependents (topo order).
    pub fn cells_affected_by_helper(&self, helper: &str) -> Vec<String> {
        let mut roots: Vec<String> = self
            .notebook
            .cells
            .iter()
            .filter(|c| ident_used_in(&c.source, helper) || ident_used_in(&c.body, helper))
            .map(|c| c.name.clone())
            .collect();
        if roots.is_empty() {
            // Conservative: shared universe — any cell may be affected.
            roots = self.notebook.cells.iter().map(|c| c.name.clone()).collect();
        }
        let mut set: std::collections::HashSet<String> = roots.iter().cloned().collect();
        for r in &roots {
            for d in transitive_dependents(&self.graph, r) {
                set.insert(d);
            }
        }
        self.graph
            .order
            .iter()
            .filter(|c| set.contains(*c))
            .cloned()
            .collect()
    }

    /// Run all currently dirty cells in topological order (deps must be ready).
    pub fn run_dirty_cells(&mut self) -> Result<Vec<CellOutput>> {
        let order = self.graph.order.clone();
        let mut results = Vec::new();
        for name in order {
            if !self.dirty.get(&name).copied().unwrap_or(false) {
                continue;
            }
            let cell = match self.notebook.cell(&name) {
                Some(c) => c.clone(),
                None => continue,
            };
            let ready = cell.params.iter().all(|p| {
                self.outputs
                    .get(&p.name)
                    .map(|o| o.success)
                    .unwrap_or(false)
            });
            // Root cells with no params are always ready.
            let ready = ready || cell.params.is_empty();
            if !ready {
                continue;
            }
            match self.run_cell_once(&name) {
                Ok((out, _)) => results.push(out),
                Err(e) => {
                    results.push(CellOutput {
                        cell: name.clone(),
                        value: Value::Null,
                        stdout: String::new(),
                        stderr: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                        value_hash: String::new(),
                    });
                }
            }
        }
        Ok(results)
    }

    pub fn edit_markdown(&mut self, name: &str, content: &str) -> Result<()> {
        let new_file = replace_markdown_content(&self.notebook.source, name, content)?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        Ok(())
    }

    pub fn add_item(
        &mut self,
        kind: AddKind,
        after: Option<(AddKind, String)>,
    ) -> Result<String> {
        let mut names: Vec<String> = self.notebook.cells.iter().map(|c| c.name.clone()).collect();
        names.extend(self.notebook.helpers.iter().map(|h| h.name.clone()));
        names.extend(self.notebook.markdown.iter().map(|m| m.name.clone()));

        let (name, block) = self.make_new_block(kind, &names)?;
        let new_file = match after {
            None => append_item(&self.notebook.source, &block),
            Some((ak, aname)) if aname == "__start__" || ak.as_str() == "__start__" => {
                prepend_item(&self.notebook.source, &block)
            }
            Some((ak, aname)) => {
                insert_after_item(&self.notebook.source, ak.as_str(), &aname, &block)?
            }
        };
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        Ok(name)
    }

    /// Change an item's kind (cell ↔ helper ↔ markdown), keeping the same name when possible.
    pub fn change_kind(&mut self, name: &str, from: AddKind, to: AddKind) -> Result<()> {
        if from == to {
            return Ok(());
        }
        let block = match (from, to) {
            (AddKind::Cell, AddKind::Helper) => {
                let cell = self
                    .notebook
                    .cell(name)
                    .with_context(|| format!("unknown cell `{name}`"))?;
                format!("{}\n", strip_labrs_attrs(&cell.source))
            }
            (AddKind::Helper, AddKind::Cell) => {
                let helper = self
                    .notebook
                    .helpers
                    .iter()
                    .find(|h| h.name == name)
                    .with_context(|| format!("unknown helper `{name}`"))?;
                let bare = strip_labrs_attrs(&helper.source);
                // Ensure it looks like a cell: add pub and return type if missing is hard;
                // just attach the attribute and hope signature is valid.
                with_labrs_attr(&ensure_cellish_fn(&bare, name), "cell")
            }
            (AddKind::Cell, AddKind::Markdown) => {
                let cell = self
                    .notebook
                    .cell(name)
                    .with_context(|| format!("unknown cell `{name}`"))?;
                let content = cell
                    .docs
                    .clone()
                    .unwrap_or_else(|| format!("# {name}\n"));
                let lit = serde_json::to_string(&content).unwrap();
                format!("#[labrs::markdown]\npub const {name}: &str = {lit};\n")
            }
            (AddKind::Markdown, AddKind::Cell) => {
                let md = self
                    .notebook
                    .markdown
                    .iter()
                    .find(|m| m.name == name)
                    .with_context(|| format!("unknown markdown `{name}`"))?;
                let lit = serde_json::to_string(&md.content).unwrap();
                format!(
                    "#[labrs::cell]\npub fn {name}() -> String {{\n    {lit}.to_string()\n}}\n"
                )
            }
            (AddKind::Helper, AddKind::Markdown) => {
                let content = format!("# {name}\n");
                let lit = serde_json::to_string(&content).unwrap();
                format!("#[labrs::markdown]\npub const {name}: &str = {lit};\n")
            }
            (AddKind::Markdown, AddKind::Helper) => {
                format!("fn {name}() {{\n    // helper\n}}\n")
            }
            (AddKind::Cell, AddKind::Cell)
            | (AddKind::Helper, AddKind::Helper)
            | (AddKind::Markdown, AddKind::Markdown) => return Ok(()),
        };

        let new_file = replace_item_block(&self.notebook.source, from.as_str(), name, &block)?;
        write_notebook(&self.path, &new_file)?;
        let _ = rustfmt_file(&self.path);
        // Clear outputs for this name; dependents may be dirty
        self.outputs.remove(name);
        for c in &self.notebook.cells {
            if self.outputs.contains_key(&c.name) {
                self.dirty.insert(c.name.clone(), true);
            }
        }
        self.reload()?;
        Ok(())
    }

    fn make_new_block(&self, kind: AddKind, names: &[String]) -> Result<(String, String)> {
        Ok(match kind {
            AddKind::Cell => {
                let name = fresh_name(names, "cell");
                let block =
                    format!("#[labrs::cell]\npub fn {name}() -> i32 {{\n    0\n}}\n");
                (name, block)
            }
            AddKind::Helper => {
                let name = fresh_name(names, "helper");
                let block = format!("fn {name}() {{\n    // helper\n}}\n");
                (name, block)
            }
            AddKind::Markdown => {
                let name = fresh_name(names, "md");
                let block = format!(
                    "#[labrs::markdown]\npub const {name}: &str = \"# New markdown\\n\\nEdit me.\";\n"
                );
                (name, block)
            }
        })
    }

    /// Run a single cell (no cascade). Returns `(output, output_changed)`.
    pub fn run_cell_once(&mut self, name: &str) -> Result<(CellOutput, bool)> {
        let cell = self
            .notebook
            .cell(name)
            .with_context(|| format!("unknown cell `{name}`"))?
            .clone();

        let mut deps = HashMap::new();
        for param in &cell.params {
            let out = self.outputs.get(&param.name).with_context(|| {
                format!(
                    "dependency `{}` has not been run yet (required by `{name}`)",
                    param.name
                )
            })?;
            if !out.success {
                anyhow::bail!(
                    "dependency `{}` failed; fix it before running `{name}`",
                    param.name
                );
            }
            deps.insert(param.name.clone(), out.value.clone());
        }

        let output = self.executor.execute_cell(&self.notebook, name, &deps)?;
        let old_hash = self.outputs.get(name).map(|o| o.value_hash.clone());
        self.outputs.insert(name.to_string(), output.clone());
        self.dirty.insert(name.to_string(), false);

        let mut changed = true;
        if output.success {
            let new_hash = output.value_hash.clone();
            changed = old_hash.as_ref() != Some(&new_hash);
            if changed {
                for dep in dependents(&self.graph, name) {
                    // Mark all dependents dirty (including never-run / pristine).
                    self.dirty.insert(dep, true);
                }
            }
        }
        Ok((output, changed))
    }

    /// Run a cell; when `auto_react` is on and the output changed, cascade to
    /// transitive dependents in topological order (Pluto-style).
    pub fn run_cell(&mut self, name: &str) -> Result<Vec<CellOutput>> {
        let (first, changed) = self.run_cell_once(name)?;
        let mut results = vec![first.clone()];

        if !self.auto_react || !first.success || !changed {
            return Ok(results);
        }

        let cascade = transitive_dependents(&self.graph, name);
        for dep_name in cascade {
            // Skip if no longer dirty (upstream in this cascade produced same hash).
            if !self.dirty.get(&dep_name).copied().unwrap_or(false) {
                continue;
            }
            // All parameters must have successful outputs.
            let cell = match self.notebook.cell(&dep_name) {
                Some(c) => c.clone(),
                None => continue,
            };
            let ready = cell.params.iter().all(|p| {
                self.outputs
                    .get(&p.name)
                    .map(|o| o.success)
                    .unwrap_or(false)
            });
            if !ready {
                continue;
            }

            match self.run_cell_once(&dep_name) {
                Ok((out, _dep_changed)) => {
                    results.push(out.clone());
                }
                Err(e) => {
                    results.push(CellOutput {
                        cell: dep_name.clone(),
                        value: Value::Null,
                        stdout: String::new(),
                        stderr: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                        value_hash: String::new(),
                    });
                    self.dirty.insert(dep_name, true);
                }
            }
        }
        Ok(results)
    }

    pub fn run_all(&mut self) -> Result<Vec<CellOutput>> {
        let _ = rustfmt_file(&self.path);
        self.reload()?;
        if !self.graph.is_ok() {
            anyhow::bail!("cannot run: notebook has validation errors");
        }
        let outputs = self.executor.execute_all(&self.notebook, &self.graph)?;
        for o in &outputs {
            self.outputs.insert(o.cell.clone(), o.clone());
            self.dirty.insert(o.cell.clone(), false);
        }
        Ok(outputs)
    }

    pub fn dep_values(&self) -> HashMap<String, Value> {
        self.outputs
            .iter()
            .filter(|(_, o)| o.success)
            .map(|(k, o)| (k.clone(), o.value.clone()))
            .collect()
    }
}
