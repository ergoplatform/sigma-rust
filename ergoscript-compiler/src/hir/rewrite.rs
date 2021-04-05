use super::Binary;
use super::Expr;
use super::ExprKind;

// pub fn hir_rewrite_safe<F: Fn(&Expr) -> Option<Expr>>(e: Expr, f: F) -> Expr {
//     let f_wrap = |e| Result::<Option<Expr>, BinderError>::Ok(f(e));
//     hir_rewrite(e, f_wrap).unwrap()
// }

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
        ExprKind::Ident(_) => f(&e)?.unwrap_or(e), // TODO: duplicate call to f?
        ExprKind::GlobalVars(_) => f(&e)?.unwrap_or(e),
        ExprKind::Literal(_) => f(&e)?.unwrap_or(e),
    })
}
