//! Integration-style execution smoke test.

use labrs_core::{graph, parse, ExecuteOptions, Executor};
use std::fs;

#[test]
fn runs_helper_and_cells() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nb.rs");
    fs::write(
        &path,
        r#"
fn double(val: u16) -> u16 { 2 * val }

#[labrs::cell]
pub fn val() -> u16 { 4 }

#[labrs::cell]
pub fn report(val: &u16) -> String {
    let d = double(*val);
    format!("{val}->{d}")
}
"#,
    )
    .unwrap();

    let opts = ExecuteOptions {
        rustfmt: false,
        cache_dir: Some(dir.path().join("cache")),
    };
    let ex = Executor::new(&opts).unwrap();
    let outs = ex.run_notebook(&path, &opts).unwrap();
    assert_eq!(outs.len(), 2);
    assert!(outs.iter().all(|o| o.success));
    assert_eq!(outs[1].value, serde_json::json!("4->8"));
}

#[test]
fn rejects_unbound_param() {
    let src = r#"
#[labrs::cell]
pub fn double(val: &u16) -> u16 { 2 * (*val) }
"#;
    let nb = parse::parse_notebook_source("t.rs", src.into()).unwrap();
    let g = graph::build_graph(&nb);
    assert!(!g.is_ok());
}
