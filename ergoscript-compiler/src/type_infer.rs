use crate::hir;
use crate::hir::Binary;
use crate::hir::Expr;
use crate::hir::ExprKind;

#[derive(Debug, PartialEq)]
pub struct TypeInferenceError {}

pub fn assign_type(expr: Expr) -> Result<Expr, TypeInferenceError> {
    hir::rewrite(expr, |e| {
        Ok(match &e.kind {
            ExprKind::Binary(Binary { op, lhs, rhs }) => match op.node {
                hir::BinaryOp::Add => {
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
mod tests {
    use expect_test::expect;

    use crate::compiler::check;

    #[test]
    fn bin_smoke() {
        check(
            "HEIGHT + HEIGHT",
            expect![[r#"
            Expr {
                kind: Binary(
                    Binary {
                        op: Spanned {
                            node: Add,
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
