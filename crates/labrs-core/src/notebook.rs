//! Notebook data model.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Stable cell identifier (function / const name).
pub type CellId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Cell,
    Helper,
    Definition,
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: CellId,
    pub name: String,
    pub docs: Option<String>,
    /// Full function source including signature and body.
    pub source: String,
    /// Body only (inside braces), used when regenerating.
    pub body: String,
    pub params: Vec<Param>,
    pub return_type: String,
    /// Byte span in the original file [start, end).
    pub span: (usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    /// Type as written, e.g. `&String` or `&u16`.
    pub ty: String,
    /// Inner type without leading reference, e.g. `String` or `u16`.
    pub inner_ty: String,
    pub is_ref: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Helper {
    pub name: String,
    pub docs: Option<String>,
    pub source: String,
    pub explicit: bool,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedDef {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownCell {
    pub id: CellId,
    pub name: String,
    pub content: String,
    pub source: String,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub path: PathBuf,
    pub source: String,
    pub cells: Vec<Cell>,
    pub helpers: Vec<Helper>,
    pub definitions: Vec<SharedDef>,
    pub markdown: Vec<MarkdownCell>,
    /// Ordered display items as they appear in the file.
    pub order: Vec<OrderEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrderEntry {
    Cell { id: CellId },
    Helper { name: String },
    Definition { name: String },
    Markdown { id: CellId },
}

impl Notebook {
    pub fn cell(&self, name: &str) -> Option<&Cell> {
        self.cells.iter().find(|c| c.name == name)
    }

    pub fn cell_mut(&mut self, name: &str) -> Option<&mut Cell> {
        self.cells.iter_mut().find(|c| c.name == name)
    }
}
