//! Procedural macros for labrs notebooks.
//!
//! Attributes are intentionally pass-through so notebooks remain valid Rust for
//! rust-analyzer. labrs-core parses the attributes with `syn` at runtime.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemConst, ItemFn};

/// Marks a function as a reactive notebook cell.
///
/// The function name is the notebook binding. Parameters are injected
/// dependencies that must match other cell names.
#[proc_macro_attribute]
pub fn cell(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(quote! { #input })
}

/// Marks a plain helper function for UI discovery (optional; plain `fn` works too).
#[proc_macro_attribute]
pub fn helper(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(quote! { #input })
}

/// Marks a markdown cell stored as a string constant.
///
/// ```ignore
/// #[labrs::markdown]
/// pub const intro: &str = r#"# Hello"#;
/// ```
#[proc_macro_attribute]
pub fn markdown(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemConst);
    TokenStream::from(quote! { #input })
}
