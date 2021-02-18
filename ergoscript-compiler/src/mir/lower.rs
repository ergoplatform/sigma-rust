use ergo_lib::ast::bin_op::ArithOp;
use ergo_lib::ast::bin_op::BinOp;
use ergo_lib::ast::bin_op::BinOpKind;
use ergo_lib::ast::expr::Expr;
use ergo_lib::ast::global_vars::GlobalVars;
use hir::BinaryOp;

use crate::hir;

#[derive(Debug, PartialEq)]
pub struct MirLoweringError {}

pub fn lower(hir_expr: hir::Expr) -> Result<Expr, MirLoweringError> {
    let mir: Expr = match &hir_expr.kind {
        hir::ExprKind::GlobalVars(hir) => match hir {
            hir::GlobalVars::Height => GlobalVars::Height.into(),
        },
        hir::ExprKind::Ident(_) => return Err(MirLoweringError {}),
        hir::ExprKind::Binary(hir) => {
            let l = lower(*hir.lhs.clone())?;
            let r = lower(*hir.rhs.clone())?;
            BinOp {
                kind: hir.op.node.clone().into(),
                left: l.into(),
                right: r.into(),
            }
            .into()
        }
    };
    if mir.tpe() == hir_expr.tpe.ok_or(MirLoweringError {})? {
        Ok(mir)
    } else {
        Err(MirLoweringError {})
    }
}

impl From<hir::BinaryOp> for BinOpKind {
    fn from(op: hir::BinaryOp) -> Self {
        match op {
            BinaryOp::Plus => ArithOp::Plus.into(),
            BinaryOp::Minus => ArithOp::Minus.into(),
            BinaryOp::Multiply => ArithOp::Multiply.into(),
            BinaryOp::Divide => ArithOp::Divide.into(),
        }
    }
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = crate::parser::parse(input);
    let syntax = parse.syntax();
    let root = crate::ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root).unwrap();
    let binder = crate::binder::Binder::new(crate::ScriptEnv::new());
    let bind = binder.bind(hir).unwrap();
    let typed = crate::type_infer::assign_type(bind).unwrap();
    let res = lower(typed).unwrap();
    expected_tree.assert_eq(&res.debug_tree());
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn bin_smoke() {
        check(
            "HEIGHT + HEIGHT",
            expect![[r#"
            BinOp(
                BinOp {
                    kind: Arith(
                        Plus,
                    ),
                    left: GlobalVars(
                        Height,
                    ),
                    right: GlobalVars(
                        Height,
                    ),
                },
            )"#]],
        )
    }
}
