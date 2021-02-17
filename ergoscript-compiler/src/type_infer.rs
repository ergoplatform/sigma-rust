use crate::hir;
use crate::hir::Binary;
use crate::hir::Expr;
use crate::hir::ExprKind;

#[derive(Debug, PartialEq)]
pub struct TypeInferenceError {}

// TODO: do only type inference? Type check on MIR?
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
