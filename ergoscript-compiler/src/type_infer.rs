use rowan::TextRange;

use crate::error::pretty_error_desc;
use crate::hir;
use crate::hir::Binary;
use crate::hir::Expr;
use crate::hir::ExprKind;

#[derive(Debug, PartialEq, Eq)]
pub struct TypeInferenceError {
    msg: String,
    span: TextRange,
}

impl TypeInferenceError {
    pub fn new(msg: String, span: TextRange) -> Self {
        Self { msg, span }
    }

    pub fn pretty_desc(&self, source: &str) -> String {
        pretty_error_desc(source, self.span, &self.msg)
    }
}

pub fn assign_type(expr: Expr) -> Result<Expr, TypeInferenceError> {
    hir::rewrite(expr, |e| {
        Ok(match &e.kind {
            ExprKind::Binary(Binary { op, lhs, rhs }) => match op.node {
                hir::BinaryOp::Plus => {
                    let l = assign_type(*lhs.clone())?;
                    let r = assign_type(*rhs.clone())?;
                    let tpe = l.tpe.clone();
                    Some(Expr {
                        kind: Binary {
                            op: op.clone(),
                            lhs: l.into(),
                            rhs: r.into(),
                        }
                        .into(),
                        span: e.span,
                        tpe,
                    })
                }
                _ => todo!(),
            },
            _ => None,
        })
    })
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = super::parser::parse(input);
    let syntax = parse.syntax();
    let root = crate::ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root).unwrap();
    let binder = crate::binder::Binder::new(crate::script_env::ScriptEnv::new());
    let bind = binder.bind(hir).unwrap();
    let res = assign_type(bind).unwrap();
    expected_tree.assert_eq(&res.debug_tree());
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn bin_smoke() {
        check(
            "HEIGHT + HEIGHT",
            expect![[r#"
            Expr {
                kind: Binary(
                    Binary {
                        op: Spanned {
                            node: Plus,
                            span: 7..8,
                        },
                        lhs: Expr {
                            kind: GlobalVars(
                                Height,
                            ),
                            span: 0..7,
                            tpe: Some(
                                SInt,
                            ),
                        },
                        rhs: Expr {
                            kind: GlobalVars(
                                Height,
                            ),
                            span: 9..15,
                            tpe: Some(
                                SInt,
                            ),
                        },
                    },
                ),
                span: 0..15,
                tpe: Some(
                    SInt,
                ),
            }"#]],
        );
    }
}
