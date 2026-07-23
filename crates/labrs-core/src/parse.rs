//! Parse a labrs notebook `.rs` file into structured items.

use crate::notebook::{Cell, Helper, MarkdownCell, Notebook, OrderEntry, Param, SharedDef};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use syn::spanned::Spanned;
use syn::{
    Attribute, Expr, FnArg, Item, ItemConst, ItemFn, Meta, Pat, ReturnType, Type, Visibility,
};

/// Parse notebook source from a file path.
pub fn parse_notebook(path: impl AsRef<Path>) -> Result<Notebook> {
    let path = path.as_ref().to_path_buf();
    let source = fs::read_to_string(&path)
        .with_context(|| format!("failed to read notebook {}", path.display()))?;
    parse_notebook_source(path, source)
}

/// Parse notebook source from an in-memory string.
pub fn parse_notebook_source(
    path: impl Into<std::path::PathBuf>,
    source: String,
) -> Result<Notebook> {
    let path = path.into();
    let file = syn::parse_file(&source)
        .with_context(|| format!("failed to parse Rust syntax in {}", path.display()))?;

    let mut cells = Vec::new();
    let mut helpers = Vec::new();
    let mut definitions = Vec::new();
    let mut markdown = Vec::new();
    let mut order = Vec::new();

    for item in file.items {
        match item {
            Item::Fn(func) => classify_fn(&source, func, &mut cells, &mut helpers, &mut order)?,
            Item::Const(item_const) => {
                if has_labrs_attr(&item_const.attrs, "markdown") {
                    let md = parse_markdown(&source, item_const)?;
                    order.push(OrderEntry::Markdown { id: md.id.clone() });
                    markdown.push(md);
                } else {
                    push_def(
                        &source,
                        "const",
                        item_name_const(&item_const),
                        &item_const,
                        &mut definitions,
                        &mut order,
                    );
                }
            }
            Item::Struct(s) => {
                push_def_item(
                    &source,
                    "struct",
                    s.ident.to_string(),
                    Item::Struct(s),
                    &mut definitions,
                    &mut order,
                );
            }
            Item::Enum(e) => {
                push_def_item(
                    &source,
                    "enum",
                    e.ident.to_string(),
                    Item::Enum(e),
                    &mut definitions,
                    &mut order,
                );
            }
            Item::Type(t) => {
                push_def_item(
                    &source,
                    "type",
                    t.ident.to_string(),
                    Item::Type(t),
                    &mut definitions,
                    &mut order,
                );
            }
            Item::Trait(t) => {
                push_def_item(
                    &source,
                    "trait",
                    t.ident.to_string(),
                    Item::Trait(t),
                    &mut definitions,
                    &mut order,
                );
            }
            Item::Impl(i) => {
                let name = format!("impl_{}", definitions.len());
                push_def_item(
                    &source,
                    "impl",
                    name,
                    Item::Impl(i),
                    &mut definitions,
                    &mut order,
                );
            }
            Item::Use(u) => {
                let name = format!("use_{}", definitions.len());
                push_def_item(
                    &source,
                    "use",
                    name,
                    Item::Use(u),
                    &mut definitions,
                    &mut order,
                );
            }
            Item::Mod(m) => {
                push_def_item(
                    &source,
                    "mod",
                    m.ident.to_string(),
                    Item::Mod(m),
                    &mut definitions,
                    &mut order,
                );
            }
            Item::Static(s) => {
                push_def_item(
                    &source,
                    "static",
                    s.ident.to_string(),
                    Item::Static(s),
                    &mut definitions,
                    &mut order,
                );
            }
            other => {
                // Keep unknown items as opaque definitions so we don't drop them on rewrite.
                let name = format!("item_{}", definitions.len());
                push_def_item(&source, "item", name, other, &mut definitions, &mut order);
            }
        }
    }

    // Unique cell names
    let mut seen = std::collections::HashSet::new();
    for cell in &cells {
        if !seen.insert(cell.name.clone()) {
            bail!("duplicate cell name `{}`", cell.name);
        }
    }

    Ok(Notebook {
        path,
        source,
        cells,
        helpers,
        definitions,
        markdown,
        order,
    })
}

fn classify_fn(
    source: &str,
    func: ItemFn,
    cells: &mut Vec<Cell>,
    helpers: &mut Vec<Helper>,
    order: &mut Vec<OrderEntry>,
) -> Result<()> {
    let is_cell = has_labrs_attr(&func.attrs, "cell");
    let is_helper = has_labrs_attr(&func.attrs, "helper");

    if is_cell {
        let cell = parse_cell(source, func)?;
        order.push(OrderEntry::Cell {
            id: cell.id.clone(),
        });
        cells.push(cell);
    } else {
        let name = func.sig.ident.to_string();
        let span = item_span(source, func.span());
        let docs = extract_docs(&func.attrs);
        let src = prettyplease::unparse(&syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![Item::Fn(func)],
        })
        .trim()
        .to_string();
        order.push(OrderEntry::Helper { name: name.clone() });
        helpers.push(Helper {
            name,
            docs,
            source: src,
            explicit: is_helper,
            span,
        });
    }
    Ok(())
}

fn parse_cell(source: &str, func: ItemFn) -> Result<Cell> {
    let name = func.sig.ident.to_string();
    let span = item_span(source, func.span());
    let docs = extract_docs(&func.attrs);
    let full_source = prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![Item::Fn(func.clone())],
    })
    .trim()
    .to_string();

    let mut params = Vec::new();
    for input in &func.sig.inputs {
        match input {
            FnArg::Receiver(_) => {
                bail!("cell `{name}` cannot take `self`");
            }
            FnArg::Typed(pat_type) => {
                let pname = match pat_type.pat.as_ref() {
                    Pat::Ident(id) => id.ident.to_string(),
                    other => bail!(
                        "cell `{name}`: unsupported parameter pattern `{}`",
                        quote::ToTokens::to_token_stream(other)
                    ),
                };
                let ty = type_to_string(&pat_type.ty);
                let (is_ref, inner_ty) = strip_ref(&pat_type.ty);
                params.push(Param {
                    name: pname,
                    ty,
                    inner_ty,
                    is_ref,
                });
            }
        }
    }

    let return_type = match &func.sig.output {
        ReturnType::Default => "()".to_string(),
        ReturnType::Type(_, ty) => type_to_string(ty),
    };

    let body = {
        let stmts = &func.block.stmts;
        if stmts.is_empty() {
            String::new()
        } else {
            let block_span = func.block.span();
            let start = block_span.start();
            let end = block_span.end();
            // Rough body extraction from braces content via pretty-print fallback
            let pretty = prettyplease::unparse(&syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![Item::Fn(ItemFn {
                    attrs: vec![],
                    vis: Visibility::Inherited,
                    sig: func.sig.clone(),
                    block: func.block.clone(),
                })],
            });
            // Extract between first `{` and last `}`
            if let (Some(a), Some(b)) = (pretty.find('{'), pretty.rfind('}')) {
                pretty[a + 1..b].trim().to_string()
            } else {
                let _ = (start, end);
                String::new()
            }
        }
    };

    Ok(Cell {
        id: name.clone(),
        name,
        docs,
        source: full_source,
        body,
        params,
        return_type,
        span,
    })
}

fn parse_markdown(source: &str, item: ItemConst) -> Result<MarkdownCell> {
    let name = item.ident.to_string();
    let span = item_span(source, item.span());
    let full_source = prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![Item::Const(item.clone())],
    })
    .trim()
    .to_string();
    let content = match item.expr.as_ref() {
        Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => s.value(),
        other => bail!(
            "markdown cell `{name}` must be a string literal, got {}",
            quote::ToTokens::to_token_stream(other)
        ),
    };
    Ok(MarkdownCell {
        id: name.clone(),
        name,
        content,
        source: full_source,
        span,
    })
}

fn push_def(
    source: &str,
    kind: &str,
    name: String,
    item: &ItemConst,
    definitions: &mut Vec<SharedDef>,
    order: &mut Vec<OrderEntry>,
) {
    let span = item_span(source, item.span());
    order.push(OrderEntry::Definition { name: name.clone() });
    definitions.push(SharedDef {
        name,
        kind: kind.to_string(),
        source: prettyplease::unparse(&syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![Item::Const(item.clone())],
        })
        .trim()
        .to_string(),
        span,
    });
}

fn push_def_item(
    source: &str,
    kind: &str,
    name: String,
    item: Item,
    definitions: &mut Vec<SharedDef>,
    order: &mut Vec<OrderEntry>,
) {
    let span = match &item {
        Item::Struct(s) => item_span(source, s.span()),
        Item::Enum(e) => item_span(source, e.span()),
        Item::Type(t) => item_span(source, t.span()),
        Item::Trait(t) => item_span(source, t.span()),
        Item::Impl(i) => item_span(source, i.span()),
        Item::Use(u) => item_span(source, u.span()),
        Item::Mod(m) => item_span(source, m.span()),
        Item::Static(s) => item_span(source, s.span()),
        _ => (0, 0),
    };
    order.push(OrderEntry::Definition { name: name.clone() });
    definitions.push(SharedDef {
        name,
        kind: kind.to_string(),
        source: prettyplease::unparse(&syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![item],
        })
        .trim()
        .to_string(),
        span,
    });
}

fn item_name_const(item: &ItemConst) -> String {
    item.ident.to_string()
}

fn has_labrs_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        // labrs::cell, labrs_macros::cell, cell
        let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        match segs.as_slice() {
            [n] if n == name => true,
            [crate_name, n] if n == name && (crate_name == "labrs" || crate_name == "labrs_macros") => {
                true
            }
            _ => {
                // Also accept #[labrs::cell(...)] via meta path
                matches!(
                    &attr.meta,
                    Meta::Path(p) | Meta::List(syn::MetaList { path: p, .. })
                        if {
                            let s: Vec<_> = p.segments.iter().map(|x| x.ident.to_string()).collect();
                            matches!(s.as_slice(), [n] if n == name)
                                || matches!(s.as_slice(), [c, n] if n == name && (c == "labrs" || c == "labrs_macros"))
                        }
                )
            }
        }
    })
}

fn extract_docs(attrs: &[Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    let line = s.value();
                    lines.push(line.strip_prefix(' ').unwrap_or(&line).to_string());
                }
            }
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn type_to_string(ty: &Type) -> String {
    quote::ToTokens::to_token_stream(ty)
        .to_string()
        .replace(" :: ", "::")
        .replace(" < ", "<")
        .replace(" > ", ">")
        .replace(" ,", ",")
        .replace(" & ", "&")
        .replace("& ", "&")
}

fn strip_ref(ty: &Type) -> (bool, String) {
    match ty {
        Type::Reference(r) => (true, type_to_string(&r.elem)),
        other => (false, type_to_string(other)),
    }
}

fn item_span(source: &str, span: proc_macro2::Span) -> (usize, usize) {
    // syn spans from parse_file are byte offsets when using proc_macro2 with span locations
    // From source files via syn::parse_file, we need procmacro2 span locations feature.
    // Fallback: search is unreliable; use 0,0 and store full pretty source instead.
    let start = span.start();
    let end = span.end();
    byte_offset(source, start.line, start.column)
        .and_then(|s| byte_offset(source, end.line, end.column).map(|e| (s, e)))
        .unwrap_or((0, 0))
}

fn byte_offset(source: &str, line: usize, column: usize) -> Option<usize> {
    if line == 0 {
        return None;
    }
    let mut current_line = 1usize;
    let mut offset = 0usize;
    for l in source.split_inclusive('\n') {
        if current_line == line {
            let col = column.saturating_sub(1);
            return Some(offset + col.min(l.len()));
        }
        offset += l.len();
        current_line += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cells_helpers_and_markdown() {
        let src = r##"
use labrs::prelude::*;

fn double(val: u16) -> u16 {
    2 * val
}

#[labrs::cell]
pub fn val() -> u16 {
    4
}

#[labrs::cell]
pub fn report(val: &u16) -> String {
    let double_val = double(*val);
    format!("Double of {val} is {double_val}")
}

#[labrs::markdown]
pub const intro: &str = r#"# Hello"#;
"##;
        let nb = parse_notebook_source("test.rs", src.to_string()).unwrap();
        assert_eq!(nb.cells.len(), 2);
        assert_eq!(nb.helpers.len(), 1);
        assert_eq!(nb.helpers[0].name, "double");
        assert_eq!(nb.cells[0].name, "val");
        assert_eq!(nb.cells[1].params[0].name, "val");
        assert_eq!(nb.markdown.len(), 1);
        assert_eq!(nb.markdown[0].content, "# Hello");
    }
}
