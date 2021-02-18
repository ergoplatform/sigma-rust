use ergo_lib::ast::bin_op::ArithOp;
use ergo_lib::ast::bin_op::BinOp;
use ergo_lib::ast::bin_op::BinOpKind;
use ergo_lib::ast::expr::Expr;
use ergo_lib::ast::global_vars::GlobalVars;
use hir::BinaryOp;

use crate::hir;

#[derive(Debug, PartialEq)]
pub struct MirError {}

pub fn lower(e: hir::Expr) -> Result<Expr, MirError> {
    Ok(match &e.kind {
        hir::ExprKind::GlobalVars(hir) => match hir {
            hir::GlobalVars::Height => GlobalVars::Height.into(),
        },
        hir::ExprKind::Ident(_) => return Err(MirError {}),
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
    })
}

impl From<hir::BinaryOp> for BinOpKind {
    fn from(op: hir::BinaryOp) -> Self {
        // TODO: rename BinaryOp variants to be inline with MIR
        match op {
            BinaryOp::Add => ArithOp::Plus.into(),
            BinaryOp::Sub => ArithOp::Minus.into(),
            BinaryOp::Mul => ArithOp::Multiply.into(),
            BinaryOp::Div => ArithOp::Divide.into(),
        }
    }
}

#[cfg(test)]
pub fn check(input: &str, expected_tree: expect_test::Expect) {
    let parse = super::parser::parse(input);
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
