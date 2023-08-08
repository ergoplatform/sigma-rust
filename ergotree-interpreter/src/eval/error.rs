use bounded_vec::BoundedVecOutOfBounds;
use ergotree_ir::ergo_tree::ErgoTreeError;
use ergotree_ir::mir::constant::TryExtractFromError;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::source_span::SourceSpan;
use sigma_ser::ScorexParsingError;
use sigma_ser::ScorexSerializationError;
use thiserror::Error;

use super::cost_accum::CostError;
use super::env::Env;

/// Interpreter errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum EvalError {
    /// AVL tree errors
    #[error("AvlTree: {0}")]
    AvlTree(String),
    /// Only boolean or SigmaBoolean is a valid result expr type
    #[error("Only boolean or SigmaBoolean is a valid result expr type")]
    InvalidResultType,
    /// Unexpected Expr encountered during the evaluation
    #[error("Unexpected Expr: {0}")]
    UnexpectedExpr(String),
    /// Error on cost calculation
    #[error("Error on cost calculation: {0:?}")]
    CostError(#[from] CostError),
    /// Unexpected value type
    #[error("Unexpected value type: {0:?}")]
    TryExtractFrom(#[from] TryExtractFromError),
    /// Not found (missing value, argument, etc.)
    #[error("Not found: {0}")]
    NotFound(String),
    /// Register id out of bounds
    #[error("{0}")]
    RegisterIdOutOfBounds(String),
    /// Unexpected value
    #[error("Unexpected value: {0}")]
    UnexpectedValue(String),
    /// Arithmetic exception error
    #[error("Arithmetic exception: {0}")]
    ArithmeticException(String),
    /// Misc error
    #[error("error: {0}")]
    Misc(String),
    /// Sigma serialization error
    #[error("Serialization error: {0}")]
    SigmaSerializationError(#[from] SigmaSerializationError),
    /// Sigma serialization parsing error
    #[error("Serialization parsing error: {0}")]
    SigmaParsingError(#[from] SigmaParsingError),
    /// ErgoTree error
    #[error("ErgoTree error: {0}")]
    ErgoTreeError(#[from] ErgoTreeError),
    /// Not yet implemented
    #[error("evaluation is not yet implemented: {0}")]
    NotImplementedYet(&'static str),
    /// Invalid item quantity for BoundedVec
    #[error("Invalid item quantity for BoundedVec: {0}")]
    BoundedVecError(#[from] BoundedVecOutOfBounds),
    /// Scorex serialization error
    #[error("Serialization error: {0}")]
    ScorexSerializationError(#[from] ScorexSerializationError),
    /// Scorex serialization parsing error
    #[error("Serialization parsing error: {0}")]
    ScorexParsingError(#[from] ScorexParsingError),
    /// Wrapped error with source span and environment
    #[error("eval error: {error}, details: {details:?}")]
    Wrapped {
        /// eval error
        error: Box<EvalError>,
        /// error details
        details: EvalErrorDetails,
    },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EvalErrorDetails {
    /// source span
    source_span: SourceSpan,
    /// environment after evaluation
    env: Env,
    /// source code
    source: Option<String>,
}

impl EvalError {
    /// Wrap eval error with source span
    pub fn wrap(self, source_span: SourceSpan, env: Env) -> Self {
        EvalError::Wrapped {
            error: Box::new(self),
            details: EvalErrorDetails {
                source_span,
                env,
                source: None,
            },
        }
    }

    /// Wrap eval error with source code
    pub fn wrap_with_src(self, source: String) -> Self {
        match self {
            EvalError::Wrapped { error, details } => EvalError::Wrapped {
                error,
                details: EvalErrorDetails {
                    source_span: details.source_span,
                    env: details.env,
                    source: Some(source),
                },
            },
            e => EvalError::Wrapped {
                error: Box::new(e),
                details: EvalErrorDetails {
                    source_span: SourceSpan::empty(),
                    env: Env::empty(),
                    source: Some(source),
                },
            },
        }
    }
}

pub trait ExtResultEvalError<T> {
    fn enrich_err(self, span: SourceSpan, env: Env) -> Result<T, EvalError>;
}

impl<T> ExtResultEvalError<T> for Result<T, EvalError> {
    fn enrich_err(self, span: SourceSpan, env: Env) -> Result<T, EvalError> {
        self.map_err(|e| match e {
            // skip already wrapped errors
            w @ EvalError::Wrapped {
                error: _,
                details: _,
            } => w,
            e => e.wrap(span, env),
        })
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use expect_test::expect;

    use ergotree_ir::mir::bin_op::ArithOp;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::block::BlockValue;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::val_def::ValDef;
    use ergotree_ir::mir::val_use::ValUse;
    use ergotree_ir::pretty_printer::PosTrackingWriter;
    use ergotree_ir::pretty_printer::Print;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    use crate::eval::context::Context;
    use crate::eval::tests::try_eval_out;

    fn check(expr: Expr, expected_tree: expect_test::Expect) {
        let mut w = PosTrackingWriter::new();
        let spanned_expr = expr.print(&mut w).unwrap();
        dbg!(&spanned_expr);
        let ctx = Rc::new(force_any_val::<Context>());
        let err_raw = try_eval_out::<i32>(&spanned_expr, ctx).err().unwrap();
        // let err = err_raw.wrap_with_src(w.get_buf().to_string());
        let err_msg = format!("{:?}", err_raw);
        expected_tree.assert_eq(&err_msg);
    }

    #[test]
    fn pretty_binop_div_zero() {
        let val_id = 1.into();
        let expr = Expr::BlockValue(
            BlockValue {
                items: vec![ValDef {
                    id: val_id,
                    rhs: Box::new(
                        BinOp {
                            kind: ArithOp::Divide.into(),
                            left: Expr::Const(1i32.into()).into(),
                            right: Expr::Const(0i32.into()).into(),
                        }
                        .into(),
                    ),
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
            "#]],
        )
    }
}
