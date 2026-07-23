//! Dependency graph construction and validation.

use crate::diagnose::{Diagnostic, Severity};
use crate::notebook::{Cell, Notebook};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("dependency graph contains a cycle involving: {0}")]
    Cycle(String),
    #[error("notebook has validation errors")]
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub param_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub edges: Vec<DependencyEdge>,
    /// Topological levels (cells that can run in parallel share a level).
    pub levels: Vec<Vec<String>>,
    /// Full topological order.
    pub order: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

impl DependencyGraph {
    pub fn is_ok(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
    }
}

/// Build the dependency graph from cell parameter names.
///
/// Edge `A → B` when cell `B` has a parameter named `A`.
pub fn build_graph(notebook: &Notebook) -> DependencyGraph {
    let cell_map: HashMap<&str, &Cell> = notebook
        .cells
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut diagnostics = Vec::new();
    let mut edges = Vec::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut indegree: HashMap<String, usize> = HashMap::new();

    for cell in &notebook.cells {
        indegree.entry(cell.name.clone()).or_insert(0);
        adj.entry(cell.name.clone()).or_default();
    }

    for cell in &notebook.cells {
        for param in &cell.params {
            match cell_map.get(param.name.as_str()) {
                None => {
                    diagnostics.push(Diagnostic::error_in(
                        &cell.name,
                        format!(
                            "parameter `{}` is not a cell. Use a plain `fn` for reusable logic, \
                             or add `#[labrs::cell] fn {}() -> ...`.",
                            param.name, param.name
                        ),
                    ));
                }
                Some(dep) => {
                    // Soft type check: compare inner types ignoring whitespace
                    let expected = normalize_ty(&dep.return_type);
                    let got = normalize_ty(&param.inner_ty);
                    if expected != got {
                        diagnostics.push(Diagnostic::error_in(
                            &cell.name,
                            format!(
                                "parameter `{}` has type `{}` but cell `{}` returns `{}`",
                                param.name, param.ty, dep.name, dep.return_type
                            ),
                        ));
                    }
                    if !param.is_ref && param.inner_ty != "()" {
                        diagnostics.push(Diagnostic::warning(format!(
                            "cell `{}`: prefer `&{}` for dependency parameter `{}`",
                            cell.name, param.inner_ty, param.name
                        )));
                    }
                    edges.push(DependencyEdge {
                        from: dep.name.clone(),
                        to: cell.name.clone(),
                        param_name: param.name.clone(),
                    });
                    adj.entry(dep.name.clone())
                        .or_default()
                        .push(cell.name.clone());
                    *indegree.entry(cell.name.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    // Kahn topological sort
    let mut queue: VecDeque<String> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(k, _)| k.clone())
        .collect();
    // Stable order by appearance in notebook
    let appearance: HashMap<&str, usize> = notebook
        .cells
        .iter()
        .enumerate()
        .map(|(i, c)| (c.name.as_str(), i))
        .collect();
    let mut queue_vec: Vec<_> = queue.drain(..).collect();
    queue_vec.sort_by_key(|n| appearance.get(n.as_str()).copied().unwrap_or(usize::MAX));
    queue.extend(queue_vec);

    let mut order = Vec::new();
    let mut levels = Vec::new();
    let mut remaining = indegree.clone();

    while !queue.is_empty() {
        let mut level = Vec::new();
        let level_size = queue.len();
        for _ in 0..level_size {
            if let Some(node) = queue.pop_front() {
                level.push(node.clone());
                order.push(node.clone());
                if let Some(children) = adj.get(&node) {
                    for child in children {
                        if let Some(d) = remaining.get_mut(child) {
                            *d = d.saturating_sub(1);
                            if *d == 0 {
                                queue.push_back(child.clone());
                            }
                        }
                    }
                }
            }
        }
        level.sort_by_key(|n| appearance.get(n.as_str()).copied().unwrap_or(usize::MAX));
        // Re-sort queue for stability
        let mut qv: Vec<_> = queue.drain(..).collect();
        qv.sort_by_key(|n| appearance.get(n.as_str()).copied().unwrap_or(usize::MAX));
        // dedup
        let mut seen = HashSet::new();
        qv.retain(|n| seen.insert(n.clone()));
        queue.extend(qv);
        levels.push(level);
    }

    if order.len() != notebook.cells.len()
        && diagnostics.iter().all(|d| d.severity != Severity::Error)
    {
        let in_order: HashSet<_> = order.iter().collect();
        let cycle_nodes: Vec<_> = notebook
            .cells
            .iter()
            .map(|c| &c.name)
            .filter(|n| !in_order.contains(n))
            .cloned()
            .collect();
        diagnostics.push(Diagnostic::error(format!(
            "dependency cycle involving: {}",
            cycle_nodes.join(", ")
        )));
    }

    DependencyGraph {
        edges,
        levels,
        order,
        diagnostics,
    }
}

fn normalize_ty(ty: &str) -> String {
    ty.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Direct dependents of a cell (cells that take it as a parameter).
pub fn dependents(graph: &DependencyGraph, cell: &str) -> Vec<String> {
    graph
        .edges
        .iter()
        .filter(|e| e.from == cell)
        .map(|e| e.to.clone())
        .collect()
}

/// All transitive dependents of `cell`, ordered by the graph's topological order.
pub fn transitive_dependents(graph: &DependencyGraph, cell: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut stack = dependents(graph, cell);
    while let Some(n) = stack.pop() {
        if !seen.insert(n.clone()) {
            continue;
        }
        stack.extend(dependents(graph, &n));
    }
    let mut result: Vec<String> = seen.into_iter().collect();
    result.sort_by_key(|n| {
        graph
            .order
            .iter()
            .position(|x| x == n)
            .unwrap_or(usize::MAX)
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_notebook_source;

    #[test]
    fn builds_graph_and_flags_unbound() {
        let src = r#"
#[labrs::cell]
pub fn greeting() -> String { "hi".into() }

#[labrs::cell]
pub fn process(greeting: &String) -> String { format!("P: {greeting}") }

#[labrs::cell]
pub fn double(val: &u16) -> u16 { 2 * (*val) }
"#;
        let nb = parse_notebook_source("t.rs", src.to_string()).unwrap();
        let g = build_graph(&nb);
        assert!(g
            .edges
            .iter()
            .any(|e| e.from == "greeting" && e.to == "process"));
        assert!(g.errors().any(|d| d.cell.as_deref() == Some("double")));
        assert!(g.errors().any(|d| d.message.contains("plain `fn`")));
    }

    #[test]
    fn topo_order() {
        let src = r#"
#[labrs::cell]
pub fn a() -> i32 { 1 }

#[labrs::cell]
pub fn b(a: &i32) -> i32 { *a + 1 }

#[labrs::cell]
pub fn c(b: &i32) -> i32 { *b + 1 }
"#;
        let nb = parse_notebook_source("t.rs", src.to_string()).unwrap();
        let g = build_graph(&nb);
        assert!(g.is_ok());
        assert_eq!(g.order, vec!["a", "b", "c"]);
    }

    #[test]
    fn transitive_dependents_topo() {
        let src = r#"
#[labrs::cell]
pub fn a() -> i32 { 1 }

#[labrs::cell]
pub fn b(a: &i32) -> i32 { *a + 1 }

#[labrs::cell]
pub fn c(b: &i32) -> i32 { *b + 1 }
"#;
        let nb = parse_notebook_source("t.rs", src.to_string()).unwrap();
        let g = build_graph(&nb);
        assert_eq!(transitive_dependents(&g, "a"), vec!["b", "c"]);
        assert_eq!(transitive_dependents(&g, "b"), vec!["c"]);
        assert!(transitive_dependents(&g, "c").is_empty());
    }
}
