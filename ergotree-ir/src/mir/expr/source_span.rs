use crate::mir::coll_append::Append;

use super::Expr;

/// Source position for the Expr
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SourceSpan {
    /// Start position in the source code
    pub start: usize,
    /// End position in the source code
    pub end: usize,
}

impl SourceSpan {
    /// Empty span
    pub fn empty() -> Self {
        SourceSpan { start: 0, end: 0 }
    }
}

/// Wrapper for Expr with source position
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SourceSpanWrapper<T> {
    /// Source position
    pub source_span: SourceSpan,
    /// Wrapped value
    pub expr: T,
}

impl<T> SourceSpanWrapper<T> {
    /// Expression
    pub fn expr(&self) -> &T {
        &self.expr
    }
}

// TODO: can be a macros
impl From<Append> for Expr {
    fn from(v: Append) -> Self {
        Expr::Append(SourceSpanWrapper {
            source_span: SourceSpan::empty(),
            expr: v,
        })
    }
}

impl<T> From<T> for SourceSpanWrapper<T> {
    fn from(v: T) -> Self {
        SourceSpanWrapper {
            source_span: SourceSpan::empty(),
            expr: v,
        }
    }
}

// TODO: draft pretty printer and how it's sets source span for every expr
// TODO: draft enriching eval errors with source span and hightlight it in the source code piece
