//! Source position for an IR node in the source code

use crate::mir::coll_append::Append;
use crate::mir::expr::Expr;

/// Source position for the Expr
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Span {
    /// Start position in the source code
    pub start: usize,
    /// End position in the source code
    pub end: usize,
}

impl Span {
    /// Empty span
    pub fn empty() -> Self {
        Span { start: 0, end: 0 }
    }
}

/// Wrapper for Expr with source position
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Spanned<T> {
    /// Source position
    pub source_span: Span,
    /// Wrapped value
    pub expr: T,
}

impl<T> Spanned<T> {
    /// Expression
    pub fn expr(&self) -> &T {
        &self.expr
    }
}

// TODO: can be a macros
impl From<Append> for Expr {
    fn from(v: Append) -> Self {
        Expr::Append(Spanned {
            source_span: Span::empty(),
            expr: v,
        })
    }
}

impl<T> From<T> for Spanned<T> {
    fn from(v: T) -> Self {
        Spanned {
            source_span: Span::empty(),
            expr: v,
        }
    }
}
