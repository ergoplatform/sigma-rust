use ergo_lib::ast::expr::Expr;
use ergo_lib::ast::global_vars::GlobalVars;

use crate::hir;

#[derive(Debug, PartialEq)]
pub struct MirError {}

pub fn lower(e: hir::Expr) -> Result<Expr, MirError> {
    Ok(match &e.kind {
        hir::ExprKind::GlobalVars(hir) => match hir {
            hir::GlobalVars::Height => GlobalVars::Height.into(),
        },
        _ => todo!("{:?}", e),
    })
}
