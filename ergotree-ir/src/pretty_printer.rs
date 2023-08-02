use std::fmt::Write;

use thiserror::Error;

use crate::mir::coll_append::Append;
use crate::mir::expr::Expr;
use crate::source_span::Span;
use crate::source_span::Spanned;

#[derive(PartialEq, Eq, Debug, Clone, Error)]
pub(crate) enum PrintError {
    #[error("fmt error: {0:?}")]
    FmtError(#[from] std::fmt::Error),
}

trait Print {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError>;
}

impl Print for Append {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let start_pos = w.current_pos();
        self.input.print(w)?;
        write!(w, ".append(")?;
        self.col_2.print(w)?;
        write!(w, ")")?;
        let end_pos = w.current_pos();
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

#[allow(clippy::todo)]
impl Print for Expr {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        match self {
            Expr::Append(a) => a.expr().print(w),
            _ => todo!(),
        }
    }
}

// TODO: extract to a separate module
pub trait Printer: Write {
    fn current_pos(&self) -> usize;
    fn inc_ident(&mut self);
    fn dec_ident(&mut self);
}

pub struct PosTrackingWriter {
    print_buf: String,
    current_pos: usize,
    current_indent: usize,
}

impl Write for PosTrackingWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let indented_str = s
            .lines()
            .map(|l| {
                let mut indent = String::new();
                for _ in 0..self.current_indent {
                    indent.push(' ');
                }
                format!("{}{}", indent, l)
            })
            .collect::<Vec<String>>()
            .join("\n");
        let len = s.len();
        self.current_pos += len;
        write!(self.print_buf, "{}", indented_str)
    }
}

impl Printer for PosTrackingWriter {
    fn current_pos(&self) -> usize {
        self.current_pos
    }

    fn inc_ident(&mut self) {
        self.current_indent += Self::INDENT;
    }

    fn dec_ident(&mut self) {
        self.current_indent -= Self::INDENT;
    }
}

impl PosTrackingWriter {
    const INDENT: usize = 4;

    fn get_buf(&self) -> &str {
        &self.print_buf
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
        let print_buf = String::new();
        let mut w = PosTrackingWriter {
            print_buf,
            current_pos: 0,
            current_indent: 0,
        };
        let spanned_expr = expr.print(&mut w).unwrap();
        expected_tree.assert_eq(w.get_buf());
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
