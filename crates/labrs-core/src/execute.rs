//! Compile and execute notebook cells via a temporary Cargo project.

use crate::fmt::rustfmt_file;
use crate::graph::{self, DependencyGraph};
use crate::notebook::{Cell, Notebook};
use crate::parse::parse_notebook;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellOutput {
    pub cell: String,
    pub value: Value,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub error: Option<String>,
    pub value_hash: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExecuteOptions {
    pub rustfmt: bool,
    pub cache_dir: Option<PathBuf>,
}

pub struct Executor {
    cache_dir: PathBuf,
    _temp: Option<TempDir>,
}

impl Executor {
    pub fn new(opts: &ExecuteOptions) -> Result<Self> {
        if let Some(dir) = &opts.cache_dir {
            fs::create_dir_all(dir)?;
            Ok(Self {
                cache_dir: dir.clone(),
                _temp: None,
            })
        } else {
            let temp = tempfile::tempdir()?;
            let cache_dir = temp.path().to_path_buf();
            Ok(Self {
                cache_dir,
                _temp: Some(temp),
            })
        }
    }

    pub fn run_notebook(&self, path: &Path, opts: &ExecuteOptions) -> Result<Vec<CellOutput>> {
        if opts.rustfmt {
            let _ = rustfmt_file(path);
        }
        let notebook = parse_notebook(path)?;
        let graph = graph::build_graph(&notebook);
        if !graph.is_ok() {
            let msgs: Vec<_> = graph.errors().map(|d| d.message.clone()).collect();
            bail!("notebook validation failed:\n  - {}", msgs.join("\n  - "));
        }
        self.execute_all(&notebook, &graph)
    }

    pub fn execute_all(
        &self,
        notebook: &Notebook,
        graph: &DependencyGraph,
    ) -> Result<Vec<CellOutput>> {
        let project = self.materialize_runner(notebook, graph)?;
        let output = Command::new("cargo")
            .arg("run")
            .arg("--quiet")
            .current_dir(&project)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("failed to run cargo for notebook")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            bail!("notebook compilation/execution failed:\n{stderr}\n{stdout}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_all_cells(&stdout))
    }

    pub fn execute_cell(
        &self,
        notebook: &Notebook,
        cell_name: &str,
        deps: &HashMap<String, Value>,
    ) -> Result<CellOutput> {
        let cell = notebook
            .cell(cell_name)
            .with_context(|| format!("unknown cell `{cell_name}`"))?;

        for param in &cell.params {
            if !deps.contains_key(&param.name) {
                bail!(
                    "cell `{cell_name}` requires dependency `{}` to have been run first",
                    param.name
                );
            }
        }

        let project = self.materialize_single_cell(notebook, cell)?;
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--quiet")
            .current_dir(&project)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn cell runner")?;

        {
            let mut stdin = child.stdin.take().context("no stdin")?;
            let payload = serde_json::to_vec(deps)?;
            stdin.write_all(&payload)?;
        }

        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Ok(CellOutput {
                cell: cell_name.to_string(),
                value: Value::Null,
                stdout,
                stderr: stderr.clone(),
                success: false,
                error: Some(format!("execution failed:\n{stderr}")),
                value_hash: hash_value(&Value::Null),
            });
        }

        let parsed = parse_all_cells(&stdout);
        Ok(parsed
            .into_iter()
            .find(|o| o.cell == cell_name)
            .unwrap_or_else(|| CellOutput {
                cell: cell_name.to_string(),
                value: Value::Null,
                stdout,
                stderr,
                success: false,
                error: Some("could not parse cell output protocol".into()),
                value_hash: hash_value(&Value::Null),
            }))
    }

    fn materialize_runner(
        &self,
        notebook: &Notebook,
        graph: &DependencyGraph,
    ) -> Result<PathBuf> {
        let hash = short_hash(&notebook.source);
        let dir = self.cache_dir.join(format!("run_all_{hash}"));
        fs::create_dir_all(dir.join("src"))?;
        write_cargo_toml(&dir, "labrs_notebook_runner")?;
        let code = generate_run_all(notebook, graph)?;
        fs::write(dir.join("src/main.rs"), code)?;
        Ok(dir)
    }

    fn materialize_single_cell(&self, notebook: &Notebook, cell: &Cell) -> Result<PathBuf> {
        let hash = short_hash(&format!("{}:{}", &notebook.source, cell.name));
        let dir = self.cache_dir.join(format!("cell_{}_{hash}", cell.name));
        fs::create_dir_all(dir.join("src"))?;
        write_cargo_toml(&dir, &format!("labrs_cell_{}", cell.name))?;
        let code = generate_single_cell(notebook, cell)?;
        fs::write(dir.join("src/main.rs"), code)?;
        Ok(dir)
    }
}

fn write_cargo_toml(dir: &Path, name: &str) -> Result<()> {
    let toml = format!(
        r#"[workspace]

[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
gag = "1"
"#
    );
    fs::write(dir.join("Cargo.toml"), toml)?;
    Ok(())
}

fn shared_prelude(notebook: &Notebook) -> String {
    let mut out = String::new();
    out.push_str("#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]\n");
    out.push_str("use serde_json::Value;\n\n");

    for def in &notebook.definitions {
        if def.kind == "use" && def.source.contains("labrs") {
            continue;
        }
        if def.source.trim().is_empty() {
            continue;
        }
        out.push_str(&def.source);
        if !def.source.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    for helper in &notebook.helpers {
        let cleaned = strip_labrs_attrs(&helper.source);
        if cleaned.trim().is_empty() {
            continue;
        }
        out.push_str(&cleaned);
        out.push_str("\n\n");
    }
    out
}

fn strip_labrs_attrs(source: &str) -> String {
    crate::source_util::strip_labrs_attrs(source)
}

fn cell_fn_source(cell: &Cell) -> String {
    let cleaned = strip_labrs_attrs(&cell.source);
    let src = if cleaned.trim().is_empty() {
        let params: Vec<String> = cell
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.ty))
            .collect();
        format!(
            "pub fn {}({}) -> {} {{\n{}\n}}",
            cell.name,
            params.join(", "),
            cell.return_type,
            cell.body
        )
    } else {
        cleaned
    };
    // Rename `fn cellname` → `fn __cell_cellname` so bindings can use the cell name.
    rename_fn(&src, &cell.name, &format!("__cell_{}", cell.name))
}

fn rename_fn(source: &str, from: &str, to: &str) -> String {
    // Replace function name in signature only (first occurrence of `fn from`).
    if let Some(idx) = source.find(&format!("fn {from}")) {
        let mut out = String::new();
        out.push_str(&source[..idx]);
        out.push_str(&format!("fn {to}"));
        out.push_str(&source[idx + format!("fn {from}").len()..]);
        out
    } else {
        source.to_string()
    }
}

fn generate_run_all(notebook: &Notebook, graph: &DependencyGraph) -> Result<String> {
    let mut code = shared_prelude(notebook);
    code.push_str("use gag::BufferRedirect;\n");
    code.push_str("use std::io::Read;\n\n");

    for cell in &notebook.cells {
        code.push_str(&cell_fn_source(cell));
        code.push_str("\n\n");
    }

    code.push_str("fn main() {\n");

    for name in &graph.order {
        let cell = notebook.cell(name).expect("cell in order");
        let args: Vec<String> = cell
            .params
            .iter()
            .map(|p| {
                if p.is_ref {
                    format!("&{}", p.name)
                } else {
                    p.name.clone()
                }
            })
            .collect();
        let call = format!("__cell_{}({})", cell.name, args.join(", "));

        // Bind results at main scope (no nested block) so dependents can see them.
        code.push_str("    let mut stdout_buf = BufferRedirect::stdout().unwrap();\n");
        code.push_str("    let mut stderr_buf = BufferRedirect::stderr().unwrap();\n");
        code.push_str(&format!("    let __result_{} = (|| {{ {call} }})();\n", cell.name));
        code.push_str(&format!("    let mut __stdout_{} = String::new();\n", cell.name));
        code.push_str(&format!("    let mut __stderr_{} = String::new();\n", cell.name));
        code.push_str(&format!(
            "    stdout_buf.read_to_string(&mut __stdout_{}).ok();\n",
            cell.name
        ));
        code.push_str(&format!(
            "    stderr_buf.read_to_string(&mut __stderr_{}).ok();\n",
            cell.name
        ));
        code.push_str("    drop(stdout_buf);\n");
        code.push_str("    drop(stderr_buf);\n");
        code.push_str(&format!(
            "    let __value_{} = serde_json::to_value(&__result_{}).unwrap_or(serde_json::Value::Null);\n",
            cell.name, cell.name
        ));
        code.push_str(&format!(
            "    println!(\"___LABRS_CELL_START___{}___\");\n",
            cell.name
        ));
        code.push_str(&format!(
            "    println!(\"___LABRS_VALUE___{{}}\", serde_json::to_string(&__value_{}).unwrap());\n",
            cell.name
        ));
        code.push_str(&format!(
            "    println!(\"___LABRS_STDOUT___{{}}\", serde_json::to_string(&__stdout_{}).unwrap());\n",
            cell.name
        ));
        code.push_str(&format!(
            "    println!(\"___LABRS_STDERR___{{}}\", serde_json::to_string(&__stderr_{}).unwrap());\n",
            cell.name
        ));
        code.push_str(&format!(
            "    println!(\"___LABRS_CELL_END___{}___\");\n",
            cell.name
        ));
        code.push_str(&format!("    let {} = __result_{};\n", cell.name, cell.name));
        code.push_str(&format!(
            "    let _ = (&__stdout_{}, &__stderr_{}, &__value_{}, &{});\n",
            cell.name, cell.name, cell.name, cell.name
        ));
    }

    code.push_str("}\n");
    Ok(code)
}

fn generate_single_cell(notebook: &Notebook, cell: &Cell) -> Result<String> {
    let mut code = shared_prelude(notebook);
    code.push_str("use gag::BufferRedirect;\n");
    code.push_str("use std::io::{self, Read};\n\n");
    code.push_str(&cell_fn_source(cell));
    code.push_str("\n\n");

    code.push_str("fn main() {\n");
    code.push_str("    let __deps: serde_json::Map<String, Value> = {\n");
    code.push_str("        let mut s = String::new();\n");
    code.push_str("        io::Read::read_to_string(&mut io::stdin(), &mut s).unwrap();\n");
    code.push_str("        if s.trim().is_empty() {\n");
    code.push_str("            serde_json::Map::new()\n");
    code.push_str("        } else {\n");
    code.push_str("            serde_json::from_str::<serde_json::Map<String, Value>>(&s).unwrap_or_default()\n");
    code.push_str("        }\n");
    code.push_str("    };\n");

    for param in &cell.params {
        code.push_str(&format!(
            "    let {}: {} = serde_json::from_value(__deps.get(\"{}\").cloned().unwrap_or(Value::Null)).expect(\"deserialize {}\");\n",
            param.name, param.inner_ty, param.name, param.name
        ));
    }

    let args: Vec<String> = cell
        .params
        .iter()
        .map(|p| {
            if p.is_ref {
                format!("&{}", p.name)
            } else {
                p.name.clone()
            }
        })
        .collect();
    let call = format!("__cell_{}({})", cell.name, args.join(", "));

    code.push_str("    let mut stdout_buf = BufferRedirect::stdout().unwrap();\n");
    code.push_str("    let mut stderr_buf = BufferRedirect::stderr().unwrap();\n");
    code.push_str(&format!("    let __result = (|| {{ {call} }})();\n"));
    code.push_str("    let mut __stdout = String::new();\n");
    code.push_str("    let mut __stderr = String::new();\n");
    code.push_str("    stdout_buf.read_to_string(&mut __stdout).ok();\n");
    code.push_str("    stderr_buf.read_to_string(&mut __stderr).ok();\n");
    code.push_str("    drop(stdout_buf);\n");
    code.push_str("    drop(stderr_buf);\n");
    code.push_str("    let __value = serde_json::to_value(&__result).unwrap_or(Value::Null);\n");
    code.push_str(&format!(
        "    println!(\"___LABRS_CELL_START___{}___\");\n",
        cell.name
    ));
    code.push_str(
        "    println!(\"___LABRS_VALUE___{}\", serde_json::to_string(&__value).unwrap());\n",
    );
    code.push_str(
        "    println!(\"___LABRS_STDOUT___{}\", serde_json::to_string(&__stdout).unwrap());\n",
    );
    code.push_str(
        "    println!(\"___LABRS_STDERR___{}\", serde_json::to_string(&__stderr).unwrap());\n",
    );
    code.push_str(&format!(
        "    println!(\"___LABRS_CELL_END___{}___\");\n",
        cell.name
    ));
    code.push_str("}\n");
    Ok(code)
}

fn parse_all_cells(stdout: &str) -> Vec<CellOutput> {
    let mut outputs = Vec::new();
    let mut current: Option<String> = None;
    let mut value = Value::Null;
    let mut cell_stdout = String::new();
    let mut cell_stderr = String::new();

    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("___LABRS_CELL_START___") {
            let name = rest.strip_suffix("___").unwrap_or(rest).to_string();
            current = Some(name);
            value = Value::Null;
            cell_stdout.clear();
            cell_stderr.clear();
        } else if let Some(rest) = line.strip_prefix("___LABRS_VALUE___") {
            value = serde_json::from_str(rest).unwrap_or(Value::Null);
        } else if let Some(rest) = line.strip_prefix("___LABRS_STDOUT___") {
            cell_stdout = serde_json::from_str(rest).unwrap_or_default();
        } else if let Some(rest) = line.strip_prefix("___LABRS_STDERR___") {
            cell_stderr = serde_json::from_str(rest).unwrap_or_default();
        } else if line.starts_with("___LABRS_CELL_END___") {
            if let Some(name) = current.take() {
                let value_hash = hash_value(&value);
                outputs.push(CellOutput {
                    cell: name,
                    value: value.clone(),
                    stdout: cell_stdout.clone(),
                    stderr: cell_stderr.clone(),
                    success: true,
                    error: None,
                    value_hash,
                });
            }
        }
    }
    outputs
}

pub fn hash_value(v: &Value) -> String {
    let s = serde_json::to_string(v).unwrap_or_default();
    short_hash(&s)
}

fn short_hash(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(&hasher.finalize()[..8])
}

pub fn replace_cell_source(
    notebook_source: &str,
    cell: &Cell,
    new_fn_source: &str,
) -> Result<String> {
    replace_named_fn(notebook_source, &cell.name, &cell.source, new_fn_source)
}

pub fn replace_helper_source(
    notebook_source: &str,
    helper_name: &str,
    old_source: &str,
    new_fn_source: &str,
) -> Result<String> {
    replace_named_fn(notebook_source, helper_name, old_source, new_fn_source)
}

fn replace_named_fn(
    notebook_source: &str,
    name: &str,
    old_source: &str,
    new_fn_source: &str,
) -> Result<String> {
    if !old_source.is_empty() {
        if let Some(idx) = notebook_source.find(old_source) {
            let mut out = String::new();
            out.push_str(&notebook_source[..idx]);
            out.push_str(new_fn_source.trim());
            out.push('\n');
            out.push_str(&notebook_source[idx + old_source.len()..]);
            return Ok(out);
        }
    }
    if let Some((start, end)) = find_fn_span(notebook_source, name) {
        let start = extend_back_over_attrs(notebook_source, start);
        let mut out = String::new();
        out.push_str(&notebook_source[..start]);
        out.push_str(new_fn_source.trim());
        out.push('\n');
        out.push_str(&notebook_source[end..]);
        return Ok(out);
    }
    bail!("could not locate item `{name}` in notebook source");
}

/// Replace a markdown const's string contents (keeps name and attribute).
pub fn replace_markdown_content(
    notebook_source: &str,
    name: &str,
    new_content: &str,
) -> Result<String> {
    let lit = serde_json::to_string(new_content).unwrap_or_else(|_| "\"\"".into());
    let block = format!("#[labrs::markdown]\npub const {name}: &str = {lit};\n");
    let patterns = [format!("pub const {name}"), format!("const {name}")];
    let mut pos = None;
    for p in &patterns {
        if let Some(i) = notebook_source.find(p) {
            pos = Some(i);
            break;
        }
    }
    let start_sig = pos.context(format!("markdown const `{name}` not found"))?;
    let start = extend_back_over_attrs(notebook_source, start_sig);
    let semi = notebook_source[start_sig..]
        .find(';')
        .map(|i| start_sig + i + 1)
        .context("markdown const missing `;`")?;
    let mut out = String::new();
    out.push_str(&notebook_source[..start]);
    out.push_str(block.trim());
    out.push('\n');
    let rest = notebook_source[semi..].trim_start_matches('\n');
    if !rest.is_empty() {
        out.push('\n');
        out.push_str(rest);
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

pub fn append_item(notebook_source: &str, item: &str) -> String {
    let mut out = notebook_source.trim_end().to_string();
    out.push_str("\n\n");
    out.push_str(item.trim());
    out.push('\n');
    out
}

pub fn prepend_item(notebook_source: &str, item: &str) -> String {
    let mut out = String::new();
    out.push_str(item.trim());
    out.push_str("\n\n");
    out.push_str(notebook_source.trim_start());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Insert `item` immediately after the named notebook item (`cell` / `helper` / `markdown`).
pub fn insert_after_item(
    notebook_source: &str,
    after_kind: &str,
    after_name: &str,
    item: &str,
) -> Result<String> {
    let (_start, end) = item_full_span(notebook_source, after_kind, after_name)?;
    let mut out = String::new();
    out.push_str(&notebook_source[..end]);
    // skip trailing newlines then add clean separation
    let rest = notebook_source[end..].trim_start_matches('\n');
    out.push_str("\n\n");
    out.push_str(item.trim());
    out.push_str("\n\n");
    out.push_str(rest);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// Replace an entire item (cell/helper/markdown) with a new block.
pub fn replace_item_block(
    notebook_source: &str,
    kind: &str,
    name: &str,
    new_block: &str,
) -> Result<String> {
    let (start, end) = item_full_span(notebook_source, kind, name)?;
    let mut out = String::new();
    out.push_str(&notebook_source[..start]);
    out.push_str(new_block.trim());
    out.push('\n');
    let rest = notebook_source[end..].trim_start_matches('\n');
    if !rest.is_empty() {
        out.push('\n');
        out.push_str(rest);
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn item_full_span(source: &str, kind: &str, name: &str) -> Result<(usize, usize)> {
    match kind {
        "cell" | "helper" => {
            let (start, end) = find_fn_span(source, name)
                .with_context(|| format!("function `{name}` not found"))?;
            let start = extend_back_over_attrs(source, start);
            Ok((start, end))
        }
        "markdown" => {
            let patterns = [format!("pub const {name}"), format!("const {name}")];
            let mut pos = None;
            for p in &patterns {
                if let Some(i) = source.find(p) {
                    pos = Some(i);
                    break;
                }
            }
            let start_sig = pos.context(format!("markdown const `{name}` not found"))?;
            let start = extend_back_over_attrs(source, start_sig);
            let end = source[start_sig..]
                .find(';')
                .map(|i| start_sig + i + 1)
                .context("markdown const missing `;`")?;
            Ok((start, end))
        }
        other => bail!("unknown item kind `{other}`"),
    }
}

fn find_fn_span(source: &str, name: &str) -> Option<(usize, usize)> {
    let patterns = [
        format!("pub fn {name}"),
        format!("fn {name}"),
        format!("pub(crate) fn {name}"),
    ];
    let mut pos = None;
    for p in &patterns {
        if let Some(i) = source.find(p) {
            pos = Some(i);
            break;
        }
    }
    let start = pos?;
    let brace = source[start..].find('{')? + start;
    let mut depth = 0i32;
    let mut end = brace;
    for (i, ch) in source[brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = brace + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    Some((start, end))
}

fn extend_back_over_attrs(source: &str, mut start: usize) -> usize {
    let bytes = source.as_bytes();
    loop {
        let mut i = start;
        while i > 0 && (bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
            i -= 1;
        }
        if i > 0 && bytes[i - 1] == b'\n' {
            i -= 1;
        }
        let line_start = source[..i].rfind('\n').map(|x| x + 1).unwrap_or(0);
        let line = source[line_start..i].trim();
        if line.starts_with("#[") || line.starts_with("///") || line.starts_with("//!") {
            start = line_start;
            continue;
        }
        break;
    }
    start
}

pub fn write_notebook(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents)?;
    Ok(())
}

pub fn reload_notebook(path: &Path) -> Result<(Notebook, DependencyGraph)> {
    let nb = parse_notebook(path)?;
    let g = graph::build_graph(&nb);
    Ok((nb, g))
}
