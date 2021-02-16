//! ErgoScript compiler

use super::binder::BinderError;
use super::binder::ScriptEnv;
use super::hir::HirError;
use crate::ast;
use crate::binder;
use crate::hir;

extern crate derive_more;
use derive_more::From;

// TODO: convert to struct and add span, message?
/// Compilation errors
#[derive(Debug, PartialEq, From)]
pub enum CompileError {
    /// HIR lowering error
    HirError(HirError),
    /// Error on binder pass
    BinderError(BinderError),
}

/// Compiles given source code to HIR, or returns an error
pub fn compile_hir(source: String, env: ScriptEnv) -> Result<hir::Expr, CompileError> {
    let parse = super::parser::parse(&source);
    println!("{}", parse.debug_tree());
    let syntax = parse.syntax();
    let root = ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root)?;
    let binder = binder::Binder::new(env);
    let res = binder.bind(hir)?;
    Ok(res)
}
