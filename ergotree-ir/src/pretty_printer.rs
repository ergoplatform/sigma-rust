//! Pretty printer for ErgoTree IR

use std::fmt::Write;

use thiserror::Error;

use crate::mir::block::BlockValue;
use crate::mir::coll_append::Append;
use crate::mir::constant::Constant;
use crate::mir::expr::Expr;
use crate::mir::val_def::ValDef;
use crate::source_span::Span;
use crate::source_span::Spanned;

/// Print error
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Debug, Clone, Error)]
pub enum PrintError {
    #[error("fmt error: {0:?}")]
    FmtError(#[from] std::fmt::Error),
}

/// Print trait for Expr that sets the source span for the resulting Expr
pub trait Print {
    /// Print the expression and return the resulting expression with source span
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError>;
}

impl Print for BlockValue {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let start = w.current_pos();
        writeln!(w, "{{")?;
        w.inc_ident();
        for item in &self.items {
            item.print(w)?;
            writeln!(w)?;
        }
        self.result.print(w)?;
        w.dec_ident();
        write!(w, "\n}}")?;
        let end = w.current_pos();
        Ok(Spanned {
            source_span: Span { start, end },
            expr: self.clone(),
        }
        .into())
    }
}

impl Print for ValDef {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let start = w.current_pos();
        write!(w, "val v{} = ", self.id)?;
        self.rhs.print(w)?;
        let end = w.current_pos();
        writeln!(w)?;
        Ok(Spanned {
            source_span: Span { start, end },
            expr: self.clone(),
        }
        .into())
    }
}

impl Print for Constant {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        // TODO: implement Display for Literal
        write!(w, "{:?}", self.v)?;
        Ok(self.clone().into())
    }
}

impl Print for Append {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        let start = w.current_pos();
        self.input.print(w)?;
        write!(w, ".append(")?;
        self.col_2.print(w)?;
        write!(w, ")")?;
        let end = w.current_pos();
        Ok(Spanned {
            source_span: Span { start, end },
            expr: self.clone(),
        }
        .into())
    }
}

#[allow(clippy::panic)]
impl Print for Expr {
    fn print(&self, w: &mut dyn Printer) -> Result<Expr, PrintError> {
        match self {
            Expr::Append(v) => v.expr().print(w),
            Expr::BlockValue(v) => v.expr().print(w),
            Expr::ValDef(v) => v.expr().print(w),
            Expr::Const(v) => v.print(w),
            e => panic!("Not implemented: {:?}", e),
        }
    }
}

// TODO: extract to a separate module
/// Printer trait with tracking of current position and indent
pub trait Printer: Write {
    /// Current position (last printed char)
    fn current_pos(&self) -> usize;
    /// Increase indent
    fn inc_ident(&mut self);
    /// Decrease indent
    fn dec_ident(&mut self);
}

/// Printer implementation with tracking of current position and indent
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

    /// Create new printer
    pub fn new() -> Self {
        Self {
            print_buf: String::new(),
            current_pos: 0,
            current_indent: 0,
        }
    }

    /// Get printed buffer
    pub fn get_buf(&self) -> &str {
        &self.print_buf
    }
}

impl Default for PosTrackingWriter {
    fn default() -> Self {
        Self::new()
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
        let print_buf = String::new();
        let mut w = PosTrackingWriter {
            print_buf,
            current_pos: 0,
            current_indent: 0,
        };
        let _ = expr.print(&mut w).unwrap();
        expected_tree.assert_eq(w.get_buf());
        // todo!("check source spans");
    }

    #[test]
    fn print_block() {
        let val_id = 2.into();
        let expr = Expr::BlockValue(
            BlockValue {
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
            }
            .into(),
        );
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
