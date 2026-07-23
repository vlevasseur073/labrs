//! Helpers for UI ↔ `.rs` source translation (attributes live only on disk).

/// Remove `#[labrs::…]` / `#[labrs_macros::…]` lines from source shown in the editor.
pub fn strip_labrs_attrs(source: &str) -> String {
    source
        .lines()
        .filter(|l| {
            let t = l.trim();
            !(t.starts_with("#[labrs::")
                || t.starts_with("#[labrs_macros::")
                || t == "#[cell]"
                || t == "#[helper]"
                || t == "#[markdown]")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Ensure a `#[labrs::{attr}]` line is present before the item (for writing to disk).
pub fn with_labrs_attr(source: &str, attr: &str) -> String {
    let stripped = strip_labrs_attrs(source);
    let marker = format!("#[labrs::{attr}]");
    let lines: Vec<&str> = stripped.lines().collect();
    let mut insert_at = 0usize;
    while insert_at < lines.len() {
        let t = lines[insert_at].trim();
        if t.is_empty() || t.starts_with("///") || t.starts_with("//!") {
            insert_at += 1;
        } else {
            break;
        }
    }
    let mut out_lines: Vec<String> = lines.iter().map(|s| (*s).to_string()).collect();
    out_lines.insert(insert_at, marker);
    let mut out = out_lines.join("\n");
    out.push('\n');
    out
}

/// Pick an unused name like `prefix`, `prefix_2`, …
pub fn fresh_name(existing: &[String], prefix: &str) -> String {
    if !existing.iter().any(|n| n == prefix) {
        return prefix.to_string();
    }
    for i in 2..10_000 {
        let candidate = format!("{prefix}_{i}");
        if !existing.iter().any(|n| n == &candidate) {
            return candidate;
        }
    }
    format!("{prefix}_new")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_and_restores_cell_attr() {
        let src = "#[labrs::cell]\npub fn foo() -> i32 {\n    1\n}\n";
        let bare = strip_labrs_attrs(src);
        assert!(!bare.contains("labrs::"));
        assert!(bare.contains("pub fn foo"));
        let back = with_labrs_attr(&bare, "cell");
        assert!(back.lines().next().unwrap().contains("#[labrs::cell]"));
        assert!(back.contains("pub fn foo"));
    }

    #[test]
    fn keeps_docs_before_attr() {
        let bare = "/// docs\npub fn foo() -> i32 {\n    1\n}";
        let back = with_labrs_attr(bare, "cell");
        let lines: Vec<_> = back.lines().collect();
        assert_eq!(lines[0], "/// docs");
        assert_eq!(lines[1], "#[labrs::cell]");
    }
}
