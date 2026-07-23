//! labrs — reactive notebooks for Rust.
//!
//! Notebooks are ordinary `.rs` files. Mark bindings with `#[labrs::cell]`;
//! use plain functions as helpers (not part of the dependency graph).

pub use labrs_core::prelude;
pub use labrs_core::{
    parse_notebook, Cell, CellOutput, DependencyGraph, Diagnostic, Executor, Notebook, Session,
};
pub use labrs_macros::{cell, helper, markdown};
