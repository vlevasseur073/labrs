//! WebSocket protocol between UI and server.

use labrs_core::{CellOutput, SessionSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    GetState,
    EditCell { name: String, source: String },
    EditHelper { name: String, source: String },
    EditMarkdown { name: String, content: String },
    /// Append or insert after an existing item.
    AddItem {
        kind: String,
        /// Optional: insert after this item (`after_kind` + `after_name`).
        #[serde(default)]
        after_kind: Option<String>,
        #[serde(default)]
        after_name: Option<String>,
    },
    ChangeKind {
        name: String,
        from: String,
        to: String,
    },
    RunCell { name: String },
    SetAuto { enabled: bool },
    RunAll,
    Reload,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    NotebookState {
        snapshot: SessionSnapshot,
        notebook_source: String,
        cells_detail: Vec<CellDetail>,
        helpers_detail: Vec<HelperDetail>,
        markdown_detail: Vec<MarkdownDetail>,
    },
    CellFormatted {
        name: String,
        source: String,
    },
    HelperFormatted {
        name: String,
        source: String,
    },
    CellOutput {
        output: CellOutput,
    },
    CellRunning {
        name: String,
    },
    CellsDirty {
        cells: Vec<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Serialize)]
pub struct CellDetail {
    pub name: String,
    pub source: String,
    pub docs: Option<String>,
    pub return_type: String,
    pub params: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HelperDetail {
    pub name: String,
    pub source: String,
    pub docs: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MarkdownDetail {
    pub name: String,
    pub content: String,
    pub source: String,
}
