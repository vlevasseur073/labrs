# labrs

Reactive notebook environment for Rust — real `.rs` files, not `.ipynb`.

labrs fills the Venus gap: **cells are named bindings**, **plain functions are helpers** (compile into shared scope, never appear in the dependency graph). Mis-tagged cells with unbound parameters get clear diagnostics.

## Quick start

```bash
cargo install --path crates/labrs

labrs new hello
labrs graph hello.rs
labrs run hello.rs
labrs serve hello.rs   # http://127.0.0.1:8080
```

## Notebook model

```rust
use labrs::prelude::*;

/// Helper — not a notebook variable
fn double(val: u16) -> u16 {
    2 * val
}

#[labrs::markdown]
pub const intro: &str = r#"# Hello"#;

#[labrs::cell]
pub fn val() -> u16 {
    4
}

#[labrs::cell]
pub fn report(val: &u16) -> String {
    let double_val = double(*val);
    format!("Double of {val} is {double_val}")
}
```

| Kind | Syntax | In DAG? | Runnable? |
|------|--------|---------|-----------|
| Cell | `#[labrs::cell] fn name(deps) -> T` | Yes | Yes |
| Helper | plain `fn` | No | No |
| Markdown | `#[labrs::markdown] const name: &str = r#"..."#` | No | No |

Dependencies are inferred from **parameter names** matching other cell names.

## CLI

| Command | Description |
|---------|-------------|
| `labrs new <name>` | Scaffold notebook |
| `labrs graph <file>` | Print DAG + diagnostics |
| `labrs run <file>` | Topo-run all cells (rustfmt first) |
| `labrs fmt <file>` | rustfmt |
| `labrs serve <file>` | Web UI |

## Reactivity

By default the UI **auto-runs** dependent cells when an upstream cell’s output changes (Pluto-style cascade, gated by output hash). Toggle **Auto-run** in the toolbar, or start with:

```bash
labrs serve notebook.rs --no-auto   # manual dirty mode
```

