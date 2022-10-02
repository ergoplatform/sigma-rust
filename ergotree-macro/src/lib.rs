//! Procedural macro to generate ergotree instances
//!

use ergotree_ir::mir::expr::Expr;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn ergo_tree(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    TokenStream::from(quote! { #expr })
}
