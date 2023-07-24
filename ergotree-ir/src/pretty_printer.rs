use std::fmt::Write;

use crate::mir::coll_append::Append;
use crate::mir::expr::Expr;
use crate::source_span::Span;
use crate::source_span::Spanned;

pub(crate) struct PrintExpr {
    spanned_expr: Expr,
    text: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) enum PrintError {}

struct PrintContext {
    current_pos: usize,
    current_indent: usize,
}

trait Print {
    fn print(&self, w: &mut dyn Write, pc: &mut PrintContext) -> Result<Expr, PrintError>;
}

impl Print for Append {
    fn print(&self, w: &mut dyn Write, pc: &mut PrintContext) -> Result<Expr, PrintError> {
        let start_pos = pc.current_pos;
        self.input.print(w, pc)?;
        // TODO: set indent
        write!(w, ".append(")?;
        // TODO: need a custom writer to bundle output and moving current pos
        pc.current_pos += 8;
        self.col_2.print(w, pc)?;
        write!(w, ")")?;
        pc.current_pos += 1;
        let end_pos = pc.current_pos;
        Ok(Spanned {
            source_span: Span {
                start: start_pos,
                end: end_pos,
            },
            expr: self.clone(),
        }
        .into())
    }
}

impl Print for Expr {
    fn print(&self, w: &mut dyn Write, pc: &mut PrintContext) -> Result<Expr, PrintError> {
        todo!()
    }
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
        // TODO: create a formatter and grab it's output
        let expected_out = todo!();
        let mut pc = PrintContext {
            current_pos: 0,
            current_indent: 0,
        };
        let spanned_expr = expr.print(w, &mut pc).unwrap();
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
