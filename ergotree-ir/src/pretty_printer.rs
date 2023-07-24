use crate::mir::expr::Expr;

pub(crate) struct PrintExpr {
    expr: Expr,
    text: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum PrintError {}

#[allow(clippy::todo)]
pub(crate) fn print_expr(expr: Expr) -> Result<PrintExpr, PrintError> {
    todo!()
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {

    use expect_test::expect;

    use crate::mir::block::BlockValue;
    use crate::mir::val_def::ValDef;
    use crate::mir::val_use::ValUse;
    use crate::types::stype::SType;

    use super::*;

    fn check(expr: Expr, expected_tree: expect_test::Expect) {
        let expected_out = print_expr(expr).unwrap();
        expected_tree.assert_eq(&expected_out.text);
    }

    #[test]
    fn smoke() {
        let val_id = 2.into();
        let body = Expr::BlockValue(BlockValue {
            items: vec![ValDef {
                id: val_id,
                rhs: Box::new(Expr::Const(1i32.into())),
            }
            .into()],
            result: Box::new(
                ValUse {
                    val_id,
                    tpe: SType::SInt,
                }
                .into(),
            ),
        });
        let expr = Expr::Const(1i32.into());
        check(
            expr,
            expect![[r#"
            {
                val v1 = 1
                v1
            }
            "#]],
        );
    }
}
