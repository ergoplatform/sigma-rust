use super::Apply;
use super::Binary;
use super::Expr;
use super::ExprKind;
use super::FieldAccessExpr;
use super::IfExprHir;
use super::LambdaExpr;
use super::ValDef;

pub fn rewrite<E, F: Fn(&Expr) -> Result<Option<Expr>, E>>(e: Expr, f: F) -> Result<Expr, E> {
    let e = f(&e)?.unwrap_or(e);
    Ok(match &e.kind {
        ExprKind::Binary(binary) => match (f(&binary.lhs)?, f(&binary.rhs)?) {
            (None, None) => e,
            (l, r) => Expr {
                kind: Binary {
                    op: binary.op.clone(),
                    lhs: Box::new(l.unwrap_or(*binary.lhs.clone())),
                    rhs: Box::new(r.unwrap_or(*binary.rhs.clone())),
                }
                .into(),
                ..e
            },
        },
        ExprKind::Ident(_) => f(&e)?.unwrap_or(e),
        ExprKind::GlobalVars(_) => f(&e)?.unwrap_or(e),
        ExprKind::Literal(_) => f(&e)?.unwrap_or(e),
        ExprKind::Apply(apply) => {
            let new_func = f(&apply.func)?;
            let new_args: Result<Vec<Expr>, E> = apply
                .args
                .iter()
                .map(|arg| {
                    let rewritten = f(arg)?;
                    Ok(rewritten.unwrap_or_else(|| arg.clone()))
                })
                .collect();
            let new_args = new_args?;
            let new_func = new_func.unwrap_or(*apply.func.clone());
            Expr {
                kind: ExprKind::Apply(Apply {
                    func: Box::new(new_func),
                    args: new_args,
                    type_arg: apply.type_arg.clone(),
                }),
                ..e
            }
        }
        ExprKind::Block(exprs) => {
            let new_exprs: Result<Vec<Expr>, E> = exprs
                .iter()
                .map(|expr| {
                    let rewritten = f(expr)?;
                    Ok(rewritten.unwrap_or_else(|| expr.clone()))
                })
                .collect();
            let new_exprs = new_exprs?;
            if new_exprs
                .iter()
                .zip(exprs.iter())
                .all(|(new, old)| new == old)
            {
                e
            } else {
                Expr {
                    kind: ExprKind::Block(new_exprs),
                    ..e
                }
            }
        }
        ExprKind::ValDef(val_def) => {
            let new_rhs = f(&val_def.rhs)?;
            match new_rhs {
                Some(new_rhs) => Expr {
                    kind: ExprKind::ValDef(ValDef {
                        name: val_def.name.clone(),
                        id: val_def.id,
                        tpe: val_def.tpe.clone(),
                        rhs: Box::new(new_rhs),
                    }),
                    ..e
                },
                None => e,
            }
        }
        ExprKind::ValUse(_) => e,
        ExprKind::FieldAccess(fa) => {
            let new_obj = f(&fa.object)?;
            match new_obj {
                Some(new_obj) => Expr {
                    kind: ExprKind::FieldAccess(FieldAccessExpr {
                        object: Box::new(new_obj),
                        field: fa.field.clone(),
                        type_args: fa.type_args.clone(),
                    }),
                    ..e
                },
                None => e,
            }
        }
        ExprKind::If(if_expr) => {
            let new_cond = f(&if_expr.condition)?;
            let new_then = f(&if_expr.then_branch)?;
            let new_else = f(&if_expr.else_branch)?;
            match (&new_cond, &new_then, &new_else) {
                (None, None, None) => e,
                _ => Expr {
                    kind: ExprKind::If(IfExprHir {
                        condition: Box::new(new_cond.unwrap_or(*if_expr.condition.clone())),
                        then_branch: Box::new(new_then.unwrap_or(*if_expr.then_branch.clone())),
                        else_branch: Box::new(new_else.unwrap_or(*if_expr.else_branch.clone())),
                    }),
                    ..e
                },
            }
        }
        ExprKind::Lambda(lambda) => {
            let new_body = f(&lambda.body)?;
            match new_body {
                Some(new_body) => Expr {
                    kind: ExprKind::Lambda(LambdaExpr {
                        params: lambda.params.clone(),
                        param_ids: lambda.param_ids.clone(),
                        body: Box::new(new_body),
                    }),
                    ..e
                },
                None => e,
            }
        }
        ExprKind::Negation(inner) => {
            let new_inner = f(inner)?;
            match new_inner {
                Some(new_inner) => Expr {
                    kind: ExprKind::Negation(Box::new(new_inner)),
                    ..e
                },
                None => e,
            }
        }
        ExprKind::LogicalNot(inner) => {
            let new_inner = f(inner)?;
            match new_inner {
                Some(new_inner) => Expr {
                    kind: ExprKind::LogicalNot(Box::new(new_inner)),
                    ..e
                },
                None => e,
            }
        }
        ExprKind::BitInversion(inner) => {
            let new_inner = f(inner)?;
            match new_inner {
                Some(new_inner) => Expr {
                    kind: ExprKind::BitInversion(Box::new(new_inner)),
                    ..e
                },
                None => e,
            }
        }
        ExprKind::Context => e,
        ExprKind::Tuple(items) => {
            let new_items: Result<Vec<Expr>, E> = items
                .iter()
                .map(|item| {
                    let rewritten = f(item)?;
                    Ok(rewritten.unwrap_or_else(|| item.clone()))
                })
                .collect();
            let new_items = new_items?;
            if new_items
                .iter()
                .zip(items.iter())
                .all(|(new, old)| new == old)
            {
                e
            } else {
                Expr {
                    kind: ExprKind::Tuple(new_items),
                    ..e
                }
            }
        }
    })
}
