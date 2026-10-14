use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::Debug;
use core::fmt::Display;
use ergo_chain_types::autolykos_pow_scheme::AutolykosPowSchemeError;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::expr::SubstDeserializeError;

use bounded_vec::BoundedVecOutOfBounds;
use derive_more::TryInto;
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
#[derive(Error, PartialEq, Eq, Debug, Clone, TryInto)]
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
    /// Invalid item quantity for BoundedVec
    #[error("Invalid item quantity for BoundedVec: {0}")]
    BoundedVecError(#[from] BoundedVecOutOfBounds),
    /// Scorex serialization error
    #[error("Serialization error: {0}")]
    ScorexSerializationError(#[from] ScorexSerializationError),
    /// Scorex serialization parsing error
    #[error("Serialization parsing error: {0}")]
    ScorexParsingError(#[from] ScorexParsingError),
    /// Wrapped error with source span and source code
    #[error("eval error: {0}")]
    SpannedWithSource(SpannedWithSourceEvalError),
    /// Wrapped error with source span
    #[error("eval error: {0:?}")]
    Spanned(SpannedEvalError),
    /// Script version error
    #[error("Method requires at least version {required_version:?}, but activated version is {activated_version:?}")]
    ScriptVersionError {
        /// Opcode/method call requires this version
        required_version: ErgoTreeVersion,
        /// Currently activated script version on network
        activated_version: ErgoTreeVersion,
    },
    /// Deserialize substitution error, see [`ergotree_ir::mir::expr::Expr::substitute_deserialize`]
    #[error("DeserializeRegister/DeserializeContext error: {0}")]
    SubstDeserializeError(#[from] SubstDeserializeError),
    /// Autolykos PoW error
    #[error("Autolykos PoW error: {0}")]
    AutolykosPowSchemeError(#[from] AutolykosPowSchemeError),
}

/// Wrapped error with source span
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SpannedEvalError {
    /// eval error
    pub error: Box<EvalError>,
    /// source span for the expression where error occurred
    pub source_span: SourceSpan,
    /// environment at the time when error occurred
    pub env: Env<'static>,
}

/// Wrapped error with source span and source code
#[derive(PartialEq, Eq, Clone)]
pub struct SpannedWithSourceEvalError {
    /// eval error
    error: Box<EvalError>,
    /// source span for the expression where error occurred
    source_span: SourceSpan,
    /// environment at the time when error occurred
    env: Env<'static>,
    /// source code
    source: String,
}

#[cfg(feature = "std")]
impl Display for SpannedWithSourceEvalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _ = miette::set_hook(Box::new(|_| {
            Box::new(
                miette::MietteHandlerOpts::new()
                    .terminal_links(false)
                    .unicode(false)
                    .color(false)
                    .context_lines(5)
                    .tab_width(2)
                    .build(),
            )
        }));
        let err_msg = self.error.to_string();
        let report = miette::miette!(
            labels = vec![miette::LabeledSpan::at(self.source_span, err_msg,)],
            // help = "Help msg",
            "Evaluation error"
        )
        .with_source_code(self.source.clone());
        write!(f, "{:?}", report)
    }
}

#[cfg(not(feature = "std"))]
impl Display for SpannedWithSourceEvalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(feature = "std")]
impl Debug for SpannedWithSourceEvalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _ = miette::set_hook(Box::new(|_| {
            Box::new(
                miette::MietteHandlerOpts::new()
                    .terminal_links(false)
                    .unicode(false)
                    .color(false)
                    .context_lines(5)
                    .tab_width(2)
                    .build(),
            )
        }));
        let err_msg = self.error.to_string();
        let report = miette::miette!(
            labels = vec![miette::LabeledSpan::at(self.source_span, err_msg,)],
            // help = "Help msg",
            "Evaluation error"
        )
        .with_source_code(self.source.clone());
        write!(f, "{:?}", report)?;
        write!(f, "Env:\n{}", self.env)
    }
}

#[cfg(not(feature = "std"))]
impl Debug for SpannedWithSourceEvalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")?;
        write!(f, "Env:\n{}", self.env)
    }
}

impl EvalError {
    /// Wrap eval error with source span
    pub fn wrap(self, source_span: SourceSpan, env: Env) -> Self {
        EvalError::Spanned(SpannedEvalError {
            error: Box::new(self),
            source_span,
            env: env.to_static(),
        })
    }

    /// Wrap eval error with source code.
    ///
    /// `Spanned` variants are unwrapped and re-wrapped with `source` attached.
    /// Other variants are wrapped with an empty span/env — reachable from the
    /// diagnostic retry path in `reduce_to_crypto` when the deeper eval returns
    /// a bare variant (e.g. `InvalidResultType`, `UnexpectedExpr`) without
    /// having gone through `enrich_err`.
    pub fn wrap_spanned_with_src(self, source: String) -> Self {
        match self {
            EvalError::Spanned(e) => EvalError::SpannedWithSource(SpannedWithSourceEvalError {
                error: e.error,
                source_span: e.source_span,
                env: e.env,
                source,
            }),
            e => EvalError::SpannedWithSource(SpannedWithSourceEvalError {
                error: Box::new(e),
                source_span: SourceSpan::empty(),
                env: Env::empty().to_static(),
                source,
            }),
        }
    }
}

pub trait ExtResultEvalError<T> {
    fn enrich_err(self, span: SourceSpan, env: &Env) -> Result<T, EvalError>;
}

impl<T> ExtResultEvalError<T> for Result<T, EvalError> {
    fn enrich_err<'ctx>(self, span: SourceSpan, env: &Env<'ctx>) -> Result<T, EvalError> {
        self.map_err(|e| match e {
            // skip already wrapped errors
            w @ EvalError::Spanned { .. } => w,
            e => e.wrap(span, env.clone()),
        })
    }
}

#[allow(clippy::unwrap_used, unused_imports, dead_code)]
#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use ergotree_ir::mir::coll_by_index::ByIndex;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::source_span::SourceSpan;
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

    use crate::eval::error::SpannedEvalError;
    use crate::eval::error::SpannedWithSourceEvalError;
    use crate::eval::test_util::try_eval_out;
    use ergotree_ir::chain::context::Context;

    fn check(expr: Expr, expected_tree: expect_test::Expect) {
        let mut w = PosTrackingWriter::new();
        let spanned_expr = expr.print(&mut w).unwrap();
        let ctx = force_any_val::<Context>();
        let err_raw: SpannedEvalError = try_eval_out::<i32>(&spanned_expr, &ctx)
            .err()
            .unwrap()
            .try_into()
            .unwrap();
        let err = SpannedWithSourceEvalError {
            error: err_raw.error,
            source_span: err_raw.source_span,
            env: err_raw.env,
            source: w.get_buf().to_string(),
        };
        expected_tree.assert_eq(&err.to_string());
    }

    fn check_error_span(expr: Expr, expected_span: SourceSpan) {
        let mut w = PosTrackingWriter::new();
        let spanned_expr = expr.print(&mut w).unwrap();
        let ctx = force_any_val::<Context>();
        let err_raw: SpannedEvalError = try_eval_out::<i32>(&spanned_expr, &ctx)
            .err()
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(err_raw.source_span, expected_span);
    }

    #[test]
    fn pretty_binop_div_zero() {
        let lhs_val_id = 1.into();
        let rhs_val_id = 2.into();
        let res_val_id = 3.into();
        let expr = Expr::BlockValue(
            BlockValue {
                items: vec![
                    ValDef {
                        id: lhs_val_id,
                        rhs: Box::new(Expr::Const(42i32.into())),
                    }
                    .into(),
                    ValDef {
                        id: rhs_val_id,
                        rhs: Box::new(Expr::Const(0i32.into())),
                    }
                    .into(),
                    ValDef {
                        id: res_val_id,
                        rhs: Box::new(
                            BinOp {
                                kind: ArithOp::Divide.into(),
                                left: Box::new(
                                    ValUse {
                                        val_id: lhs_val_id,
                                        tpe: SType::SInt,
                                    }
                                    .into(),
                                ),
                                right: Box::new(
                                    ValUse {
                                        val_id: rhs_val_id,
                                        tpe: SType::SInt,
                                    }
                                    .into(),
                                ),
                            }
                            .into(),
                        ),
                    }
                    .into(),
                ],
                result: Box::new(
                    ValUse {
                        val_id: res_val_id,
                        tpe: SType::SInt,
                    }
                    .into(),
                ),
            }
            .into(),
        );
        // check(
        //     expr,
        //     expect![[r#"
        //           x Evaluation error
        //            ,-[1:1]
        //          1 | {
        //          2 |   val v1 = 42
        //          3 |   val v2 = 0
        //          4 |   val v3 = v1 / v2
        //            :            ^^^|^^^
        //            :               `-- Arithmetic exception: (42) / (0) resulted in exception
        //          5 |   v3
        //          6 | }
        //            `----
        //     "#]],
        // );
        check_error_span(expr, (40, 7).into());
    }

    #[test]
    fn pretty_out_of_bounds() {
        let v1_id = 1.into();
        let expr = Expr::BlockValue(
            BlockValue {
                items: vec![ValDef {
                    id: v1_id,
                    rhs: Box::new(Expr::Const(99999999i32.into())),
                }
                .into()],
                result: Box::new(Expr::ByIndex(
                    ByIndex::new(
                        Expr::GlobalVars(GlobalVars::Outputs),
                        ValUse {
                            val_id: v1_id,
                            tpe: SType::SInt,
                        }
                        .into(),
                        None,
                    )
                    .unwrap()
                    .into(),
                )),
            }
            .into(),
        );
        // check(
        //     expr,
        //     expect![[r#"
        //           x Evaluation error
        //            ,-[1:1]
        //          1 | {
        //          2 |   val v1 = 99999999
        //          3 |   OUTPUTS(v1)
        //            :          ^^|^
        //            :            `-- error: ByIndex: index Int(99999999) out of bounds for collection size 1
        //          4 | }
        //            `----
        //     "#]],
        // );
        check_error_span(expr, (31, 4).into());
    }

    #[test]
    fn wrap_spanned_with_src_handles_bare_variant() {
        use super::EvalError;
        let bare = EvalError::InvalidResultType;
        let wrapped = bare.wrap_spanned_with_src("script source".to_string());
        assert!(
            matches!(&wrapped, EvalError::SpannedWithSource(_)),
            "expected SpannedWithSource, got {:?}",
            wrapped
        );
        if let EvalError::SpannedWithSource(swse) = wrapped {
            assert_eq!(swse.source, "script source");
            assert!(matches!(*swse.error, EvalError::InvalidResultType));
        }
    }
}
