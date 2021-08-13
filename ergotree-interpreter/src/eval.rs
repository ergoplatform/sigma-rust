//! Interpreter
use std::rc::Rc;

use ergotree_ir::ergo_tree::ErgoTreeError;
use ergotree_ir::ir_ergo_box::IrErgoBoxArenaError;
use ergotree_ir::mir::constant::TryExtractFromError;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::value::Value;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

use cost_accum::CostAccumulator;
use ergotree_ir::types::smethod::SMethod;
use thiserror::Error;

use self::context::Context;
use self::cost_accum::CostError;
use self::env::Env;

/// Context(blockchain) for the interpreter
pub mod context;
/// Environment for
pub mod env;

pub(crate) mod and;
pub(crate) mod apply;
pub(crate) mod bin_op;
pub(crate) mod block;
pub(crate) mod bool_to_sigma;
pub(crate) mod byte_array_to_bigint;
pub(crate) mod byte_array_to_long;
pub(crate) mod calc_blake2b256;
pub(crate) mod calc_sha256;
pub(crate) mod coll_append;
pub(crate) mod coll_by_index;
pub(crate) mod coll_exists;
pub(crate) mod coll_filter;
pub(crate) mod coll_fold;
pub(crate) mod coll_forall;
pub(crate) mod coll_map;
pub(crate) mod coll_size;
pub(crate) mod coll_slice;
pub(crate) mod collection;
pub(crate) mod cost_accum;
pub(crate) mod costs;
pub(crate) mod create_prove_dh_tuple;
pub(crate) mod create_provedlog;
pub(crate) mod decode_point;
mod deserialize_context;
mod deserialize_register;
pub(crate) mod exponentiate;
pub(crate) mod expr;
pub(crate) mod extract_amount;
pub(crate) mod extract_creation_info;
pub(crate) mod extract_id;
pub(crate) mod extract_reg_as;
pub(crate) mod extract_script_bytes;
pub(crate) mod func_value;
pub(crate) mod get_var;
pub(crate) mod global_vars;
pub(crate) mod if_op;
pub(crate) mod logical_not;
pub(crate) mod long_to_byte_array;
pub(crate) mod method_call;
pub(crate) mod multiply_group;
pub(crate) mod negation;
pub(crate) mod option_get;
pub(crate) mod option_get_or_else;
pub(crate) mod option_is_defined;
pub(crate) mod or;
pub(crate) mod property_call;
pub(crate) mod sbox;
pub(crate) mod scoll;
pub(crate) mod scontext;
pub(crate) mod select_field;
pub(crate) mod sgroup_elem;
pub(crate) mod sigma_and;
pub(crate) mod sigma_or;
pub(crate) mod sigma_prop_bytes;
pub(crate) mod tuple;
pub(crate) mod upcast;
pub(crate) mod val_use;
pub(crate) mod xor;

/// Interpreter errors
#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum EvalError {
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
    /// Cannot find ErgoBox in Context
    #[error("Cannot find ErgoBox in Context: {0:?}")]
    ErgoBoxNotFound(#[from] IrErgoBoxArenaError),
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
}

/// Result of expression reduction procedure (see `reduce_to_crypto`).
pub struct ReductionResult {
    /// value of SigmaProp type which represents a statement verifiable via sigma protocol.
    pub sigma_prop: SigmaBoolean,
    /// estimated cost of expression evaluation
    pub cost: u64,
}

/// Interpreter
pub trait Evaluator {
    /// Evaluate the given expression by reducing it to SigmaBoolean value.
    fn reduce_to_crypto(
        &self,
        expr: &Expr,
        env: &Env,
        ctx: Rc<Context>,
    ) -> Result<ReductionResult, EvalError> {
        let cost_accum = CostAccumulator::new(0, None);
        let mut ectx = EvalContext::new(ctx, cost_accum);
        expr.eval(env, &mut ectx)
            .and_then(|v| -> Result<ReductionResult, EvalError> {
                match v {
                    Value::Boolean(b) => Ok(ReductionResult {
                        sigma_prop: SigmaBoolean::TrivialProp(b),
                        cost: 0,
                    }),
                    Value::SigmaProp(sp) => Ok(ReductionResult {
                        sigma_prop: sp.value().clone(),
                        cost: 0,
                    }),
                    _ => Err(EvalError::InvalidResultType),
                }
            })
    }
}

#[derive(Debug)]
pub(crate) struct EvalContext {
    pub(crate) ctx: Rc<Context>,
    pub(crate) cost_accum: CostAccumulator,
}

impl EvalContext {
    pub fn new(ctx: Rc<Context>, cost_accum: CostAccumulator) -> Self {
        EvalContext { ctx, cost_accum }
    }
}

/// Expression evaluation.
/// Should be implemented by every node that can be evaluated.
pub(crate) trait Evaluable {
    /// Evaluation routine to be implement by each node
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError>;
}

type EvalFn = fn(env: &Env, ctx: &mut EvalContext, Value, Vec<Value>) -> Result<Value, EvalError>;

fn smethod_eval_fn(method: &SMethod) -> Result<EvalFn, EvalError> {
    use ergotree_ir::types::*;
    Ok(match method.obj_type.type_id() {
        scontext::TYPE_ID if method.method_id() == scontext::DATA_INPUTS_PROPERTY_METHOD_ID => {
            self::scontext::DATA_INPUTS_EVAL_FN
        }
        sbox::TYPE_ID => match method.method_id() {
            sbox::VALUE_METHOD_ID => self::sbox::VALUE_EVAL_FN,
            sbox::GET_REG_METHOD_ID => self::sbox::GET_REG_EVAL_FN,
            sbox::TOKENS_METHOD_ID => self::sbox::TOKENS_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SBox: {:?}",
                    method_id
                )))
            }
        },
        scoll::TYPE_ID => match method.method_id() {
            scoll::INDEX_OF_METHOD_ID => self::scoll::INDEX_OF_EVAL_FN,
            scoll::FLATMAP_METHOD_ID => self::scoll::FLATMAP_EVAL_FN,
            scoll::ZIP_METHOD_ID => self::scoll::ZIP_EVAL_FN,
            scoll::INDICES_METHOD_ID => self::scoll::INDICES_EVAL_FN,
            scoll::UPDATED_METHOD_ID => self::scoll::UPDATED_EVAL_FN,
            scoll::UPDATE_MANY_METHOD_ID => self::scoll::UPDATE_MANY_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SCollection: {:?}",
                    method_id
                )))
            }
        },
        sgroup_elem::TYPE_ID => match method.method_id() {
            sgroup_elem::GET_ENCODED_METHOD_ID => self::sgroup_elem::GET_ENCODED_EVAL_FN,
            sgroup_elem::NEGATE_METHOD_ID => self::sgroup_elem::NEGATE_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SGroupElement: {:?}",
                    method_id
                )))
            }
        },
        type_id => {
            return Err(EvalError::NotFound(format!(
                "Eval fn: unknown type id {:?}",
                type_id
            )))
        }
    })
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests {

    #![allow(dead_code)]

    use super::env::Env;
    use super::*;
    use ergotree_ir::mir::constant::TryExtractFrom;
    use ergotree_ir::mir::constant::TryExtractInto;
    use sigma_test_util::force_any_val;

    pub fn eval_out_wo_ctx<T: TryExtractFrom<Value>>(expr: &Expr) -> T {
        let ctx = Rc::new(force_any_val::<Context>());
        eval_out(expr, ctx)
    }

    pub fn eval_out<T: TryExtractFrom<Value>>(expr: &Expr, ctx: Rc<Context>) -> T {
        let cost_accum = CostAccumulator::new(0, None);
        let mut ectx = EvalContext::new(ctx, cost_accum);
        expr.eval(&Env::empty(), &mut ectx)
            .unwrap()
            .try_extract_into::<T>()
            .unwrap()
    }

    pub fn try_eval_out<T: TryExtractFrom<Value>>(
        expr: &Expr,
        ctx: Rc<Context>,
    ) -> Result<T, EvalError> {
        let cost_accum = CostAccumulator::new(0, None);
        let mut ectx = EvalContext::new(ctx, cost_accum);
        expr.eval(&Env::empty(), &mut ectx)
            .and_then(|v| v.try_extract_into::<T>().map_err(EvalError::TryExtractFrom))
    }

    pub fn try_eval_out_wo_ctx<T: TryExtractFrom<Value>>(expr: &Expr) -> Result<T, EvalError> {
        let ctx = Rc::new(force_any_val::<Context>());
        try_eval_out(expr, ctx)
    }
}
