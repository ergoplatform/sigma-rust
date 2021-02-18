//! ErgoScript compiler

use super::binder::BinderError;
use super::hir::HirError;
use crate::ast;
use crate::binder::Binder;
use crate::hir;
use crate::mir;
use crate::type_infer::assign_type;
use crate::type_infer::TypeInferenceError;
use crate::ScriptEnv;

extern crate derive_more;
use derive_more::From;
use mir::MirError;

// TODO: convert to struct and add span, message?
/// Compilation errors
#[derive(Debug, PartialEq, From)]
pub enum CompileError {
    /// HIR lowering error
    HirError(HirError),
    /// Error on binder pass
    BinderError(BinderError),
    /// Error on type inference pass
    TypeInferenceError(TypeInferenceError),
    MirError(MirError),
}

/// Compiles given source code to MIR, or returns an error
pub fn compile(source: &str, env: ScriptEnv) -> Result<ergo_lib::ast::expr::Expr, CompileError> {
    let parse = super::parser::parse(&source);
    dbg!(parse.debug_tree());
    let syntax = parse.syntax();
    dbg!(&syntax);
    let root = ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root)?;
    dbg!(&hir);
    let binder = Binder::new(env);
    let bind = binder.bind(hir)?;
    let typed = assign_type(bind)?;
    let p = typed.debug_tree();
    println!("{}", p);
    let res = mir::lower(typed)?;
    Ok(res)
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let res = compile(input, ScriptEnv::new());
    expected_tree.assert_eq(&res.unwrap().debug_tree());
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_height() {
        check(
            "HEIGHT",
            expect![[r#"
            GlobalVars(
                Height,
            )"#]],
        );
    }
}
