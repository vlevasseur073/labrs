//! Core engine for labrs reactive notebooks.

pub mod diagnose;
pub mod execute;
pub mod fmt;
pub mod graph;
pub mod notebook;
pub mod parse;
pub mod session;
pub mod source_util;

pub use diagnose::{Diagnostic, Severity};
pub use execute::{CellOutput, ExecuteOptions, Executor};
pub use graph::{DependencyGraph, GraphError};
pub use notebook::{Cell, CellId, Helper, ItemKind, MarkdownCell, Notebook, OrderEntry, SharedDef};
pub use parse::parse_notebook;
pub use session::{AddKind, CellState, CellStatus, MoveDirection, Session, SessionSnapshot};
pub use source_util::{strip_labrs_attrs, with_labrs_attr};

/// Common imports for labrs notebooks.
pub mod prelude {
    pub use labrs_macros::{cell, helper, markdown};
}
