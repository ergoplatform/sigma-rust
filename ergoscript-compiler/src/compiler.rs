//! ErgoScript compiler

use super::binder::BinderError;
use super::hir::HirLoweringError;
use crate::ast;
use crate::binder::Binder;
use crate::hir;
use crate::mir;
use crate::parser::parse_error::ParseError;
use crate::script_env::ScriptEnv;
use crate::type_infer::assign_type;
use crate::type_infer::TypeInferenceError;

extern crate derive_more;
use derive_more::From;
use ergotree_ir::ergo_tree::ErgoTree;
use mir::lower::MirLoweringError;
use mir::type_check::TypeCheckError;

/// Compilation errors
#[derive(Debug, PartialEq, From)]
pub enum CompileError {
    /// Parser error
    ParseError(Vec<ParseError>),
    /// Error on AST to HIR lowering
    HirLoweringError(HirLoweringError),
    /// Error on binder pass
    BinderError(BinderError),
    /// Error on type inference pass
    TypeInferenceError(TypeInferenceError),
    /// Error on HIT to MIR lowering
    MirLoweringError(MirLoweringError),
    /// Error on type checking
    TypeCheckError(TypeCheckError),
}

impl CompileError {
    /// Pretty formatted error with CST/AST/IR, etc.
    pub fn pretty_desc(&self, source: &str) -> String {
        match self {
            CompileError::ParseError(errors) => {
                errors.iter().map(|e| e.pretty_desc(source)).collect()
            }
            CompileError::HirLoweringError(e) => e.pretty_desc(source),
            CompileError::BinderError(e) => e.pretty_desc(source),
            CompileError::TypeInferenceError(e) => e.pretty_desc(source),
            CompileError::MirLoweringError(e) => e.pretty_desc(source),
            CompileError::TypeCheckError(e) => e.pretty_desc(),
        }
    }
}

/// Compiles given source code to [`ErgoTree`], or returns an error
pub fn compile(source: &str, env: ScriptEnv) -> Result<ErgoTree, CompileError> {
    let parse = super::parser::parse(&source);
    dbg!(parse.debug_tree());
    if !parse.errors.is_empty() {
        return Err(CompileError::ParseError(parse.errors));
    }
    let syntax = parse.syntax();
    dbg!(&syntax);
    let root = ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root)?;
    dbg!(&hir);
    let binder = Binder::new(env);
    let bind = binder.bind(hir)?;
    let typed = assign_type(bind)?;
    dbg!(typed.debug_tree());
    let mir = mir::lower::lower(typed)?;
    let res = mir::type_check::type_check(mir)?;
    Ok(res.into())
}

#[cfg(test)]
fn check(input: &str, expected_tree: expect_test::Expect) {
    let res = compile(input, ScriptEnv::new());

    let expected_out = res
        .map(|tree| tree.debug_tree())
        .unwrap_or_else(|e| e.pretty_desc(input));
    expected_tree.assert_eq(&expected_out);
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
            ErgoTree {
                header: ErgoTreeHeader(
                    16,
                ),
                tree: Ok(
                    ParsedTree {
                        constants: [],
                        root: Ok(
                            GlobalVars(
                                Height,
                            ),
                        ),
                    },
                ),
            }"#]],
        );
    }

    #[test]
    fn test_parser_error() {
        check(
            "HSB.HEIGHT",
            expect![[r#"
            error: expected ‘+’, ‘-’, ‘*’, ‘/’, ‘val’, number, identifier, ‘-’ or ‘(’, but found an unrecognized token
            line: 1
            HSB.HEIGHT
              ^^"#]],
        );
    }
}
