//! WebSocket protocol between UI and server.

use labrs_core::{CellOutput, SessionSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    GetState,
    /// List directory under the server root (relative path).
    ListDir {
        #[serde(default)]
        path: Option<String>,
    },
    /// Open an existing `.rs` notebook.
    OpenNotebook {
        path: String,
    },
    /// Create a new notebook (`name` stem or `name.rs`) under `dir` (relative).
    CreateNotebook {
        name: String,
        #[serde(default)]
        dir: Option<String>,
    },
    /// Return to the welcome / file browser (unload current notebook).
    CloseNotebook,
    EditCell {
        name: String,
        source: String,
    },
    EditHelper {
        name: String,
        source: String,
    },
    EditDefinition {
        name: String,
        source: String,
    },
    EditPreamble {
        source: String,
    },
    EditMarkdown {
        name: String,
        content: String,
    },
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
    DeleteItem {
        kind: String,
        name: String,
    },
    MoveItem {
        kind: String,
        name: String,
        direction: String,
    },
    RunCell {
        name: String,
    },
    SetAuto {
        enabled: bool,
    },
    RunAll,
    /// Clear all cell outputs (return values and logs).
    ClearOutputs,
    Reload,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// No notebook open — show welcome / file browser.
    Welcome {
        root: String,
        cwd: String,
        entries: Vec<DirEntry>,
        auto_react: bool,
    },
    DirListing {
        path: String,
        entries: Vec<DirEntry>,
    },
    NotebookState {
        snapshot: SessionSnapshot,
        notebook_source: String,
        cells_detail: Vec<CellDetail>,
        helpers_detail: Vec<HelperDetail>,
        markdown_detail: Vec<MarkdownDetail>,
        definitions_detail: Vec<DefinitionDetail>,
        /// Absolute filesystem path for the LSP workspace root (Cargo project).
        lsp_root: String,
        /// Absolute path of the document rust-analyzer should analyze.
        lsp_document: String,
    },
    CellFormatted {
        name: String,
        source: String,
    },
    HelperFormatted {
        name: String,
        source: String,
    },
    DefinitionFormatted {
        name: String,
        source: String,
    },
    PreambleFormatted {
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
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_notebook: bool,
}

#[derive(Debug, Serialize)]
pub struct ParamDetail {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Serialize)]
pub struct CellDetail {
    pub name: String,
    pub source: String,
    pub docs: Option<String>,
    pub return_type: String,
    pub params: Vec<ParamDetail>,
    /// Byte span in the notebook file [start, end).
    pub span: (usize, usize),
}

#[derive(Debug, Serialize)]
pub struct HelperDetail {
    pub name: String,
    pub source: String,
    pub docs: Option<String>,
    pub span: (usize, usize),
}

#[derive(Debug, Serialize)]
pub struct MarkdownDetail {
    pub name: String,
    pub content: String,
    pub source: String,
    pub span: (usize, usize),
}

#[derive(Debug, Serialize)]
pub struct DefinitionDetail {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub span: (usize, usize),
}
