//! Interpreter
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Display;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
use snumeric::numeric_method_evalfn;

use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::value::Value;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

use ergotree_ir::types::smethod::SMethod;

use self::env::Env;
use ergotree_ir::chain::context::Context;

/// Environment for
pub mod env;

pub(crate) mod and;
pub(crate) mod apply;
pub(crate) mod atleast;
pub(crate) mod bin_op;
pub(crate) mod bit_inversion;
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
pub(crate) mod create_avl_tree;
pub(crate) mod create_prove_dh_tuple;
pub(crate) mod create_provedlog;
pub(crate) mod decode_point;
mod deserialize_context;
mod deserialize_register;
pub(crate) mod downcast;
mod error;
pub(crate) mod exponentiate;
pub(crate) mod expr;
pub(crate) mod extract_amount;
pub(crate) mod extract_bytes;
pub(crate) mod extract_bytes_with_no_ref;
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
pub(crate) mod savltree;
pub(crate) mod sbox;
pub(crate) mod scoll;
pub(crate) mod scontext;
pub(crate) mod select_field;
pub(crate) mod sglobal;
pub(crate) mod sgroup_elem;
pub(crate) mod sheader;
pub(crate) mod sigma_and;
pub(crate) mod sigma_or;
pub(crate) mod sigma_prop_bytes;
pub(crate) mod snumeric;
pub(crate) mod soption;
pub(crate) mod spreheader;
pub(crate) mod subst_const;
pub(crate) mod tree_lookup;
pub(crate) mod tuple;
pub(crate) mod upcast;
pub(crate) mod val_use;
pub(crate) mod xor;
pub(crate) mod xor_of;

pub use error::EvalError;

/// Diagnostic information about the reduction (pretty printed expr and/or env)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ReductionDiagnosticInfo {
    /// environment after the evaluation
    pub env: Env<'static>,
    /// expression pretty-printed
    pub pretty_printed_expr: Option<String>,
}

impl Display for ReductionDiagnosticInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(expr_str) = &self.pretty_printed_expr {
            writeln!(f, "Pretty printed expr:\n{}", expr_str)?;
        }
        write!(f, "Env:\n{}", self.env)
    }
}

/// Result of expression reduction procedure (see `reduce_to_crypto`).
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ReductionResult {
    /// value of SigmaProp type which represents a statement verifiable via sigma protocol.
    pub sigma_prop: SigmaBoolean,
    /// estimated cost of expression evaluation
    pub cost: u64,
    /// Diagnostic information about the reduction (pretty printed expr and/or env)
    pub diag: ReductionDiagnosticInfo,
}

/// Evaluate the given expression by reducing it to SigmaBoolean value.
pub fn reduce_to_crypto(tree: &ErgoTree, ctx: &Context) -> Result<ReductionResult, EvalError> {
    fn inner<'ctx>(expr: &'ctx Expr, ctx: &Context<'ctx>) -> Result<ReductionResult, EvalError> {
        let mut env_mut = Env::empty();
        expr.eval(&mut env_mut, ctx)
            .and_then(|v| -> Result<ReductionResult, EvalError> {
                match v {
                    Value::Boolean(b) => Ok(ReductionResult {
                        sigma_prop: SigmaBoolean::TrivialProp(b),
                        cost: 0,
                        diag: ReductionDiagnosticInfo {
                            env: env_mut.to_static(),
                            pretty_printed_expr: None,
                        },
                    }),
                    Value::SigmaProp(sp) => Ok(ReductionResult {
                        sigma_prop: sp.value().clone(),
                        cost: 0,
                        diag: ReductionDiagnosticInfo {
                            env: env_mut.to_static(),
                            pretty_printed_expr: None,
                        },
                    }),
                    _ => Err(EvalError::InvalidResultType),
                }
            })
    }

    let expr = tree.proposition()?;
    let expr = if tree.has_deserialize() {
        expr.substitute_deserialize(ctx)?
    } else {
        expr
    };
    let res = inner(&expr, ctx);
    if let Ok(reduction) = res {
        if reduction.sigma_prop == SigmaBoolean::TrivialProp(false) {
            let (_, printed_expr_str) = expr
                .pretty_print()
                .map_err(|e| EvalError::Misc(e.to_string()))?;
            let new_reduction = ReductionResult {
                sigma_prop: SigmaBoolean::TrivialProp(false),
                cost: reduction.cost,
                diag: ReductionDiagnosticInfo {
                    env: reduction.diag.env,
                    pretty_printed_expr: Some(printed_expr_str),
                },
            };
            return Ok(new_reduction);
        } else {
            return Ok(reduction);
        }
    }
    let (spanned_expr, printed_expr_str) = expr
        .pretty_print()
        .map_err(|e| EvalError::Misc(e.to_string()))?;
    inner(&spanned_expr, ctx).map_err(|e| e.wrap_spanned_with_src(printed_expr_str.to_string()))
}

/// Expects SigmaProp constant value and returns it's value. Otherwise, returns an error.
pub fn extract_sigma_boolean(expr: &Expr) -> Result<SigmaBoolean, EvalError> {
    match expr {
        Expr::Const(c) => Ok(c.clone().try_extract_into::<SigmaProp>()?.into()),
        _ => Err(EvalError::InvalidResultType),
    }
}

/// Expression evaluation.
/// Should be implemented by every node that can be evaluated.
pub(crate) trait Evaluable {
    /// Evaluation routine to be implement by each node
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
        // TODO for JIT costing: cost_accum: &mut CostAccumulator,
    ) -> Result<Value<'ctx>, EvalError>;
}

type EvalFn = for<'ctx> fn(
    mc: &SMethod,
    env: &mut Env<'ctx>,
    ctx: &Context<'ctx>,
    Value<'ctx>,
    Vec<Value<'ctx>>,
) -> Result<Value<'ctx>, EvalError>;

fn smethod_eval_fn(method: &SMethod) -> Result<EvalFn, EvalError> {
    use ergotree_ir::types::*;
    Ok(match method.obj_type.type_code() {
        savltree::TYPE_CODE => match method.method_id() {
            savltree::DIGEST_METHOD_ID => self::savltree::DIGEST_EVAL_FN,
            savltree::UPDATE_DIGEST_METHOD_ID => self::savltree::UPDATE_DIGEST_EVAL_FN,
            savltree::ENABLED_OPERATIONS_METHOD_ID => self::savltree::ENABLED_OPERATIONS_EVAL_FN,
            savltree::KEY_LENGTH_METHOD_ID => self::savltree::KEY_LENGTH_EVAL_FN,
            savltree::VALUE_LENGTH_OPT_METHOD_ID => self::savltree::VALUE_LENGTH_OPT_EVAL_FN,
            savltree::IS_INSERT_ALLOWED_METHOD_ID => self::savltree::IS_INSERT_ALLOWED_EVAL_FN,
            savltree::IS_UPDATE_ALLOWED_METHOD_ID => self::savltree::IS_UPDATE_ALLOWED_EVAL_FN,
            savltree::IS_REMOVE_ALLOWED_METHOD_ID => self::savltree::IS_REMOVE_ALLOWED_EVAL_FN,
            savltree::UPDATE_OPERATIONS_METHOD_ID => self::savltree::UPDATE_OPERATIONS_EVAL_FN,
            savltree::GET_METHOD_ID => self::savltree::GET_EVAL_FN,
            savltree::GET_MANY_METHOD_ID => self::savltree::GET_MANY_EVAL_FN,
            savltree::INSERT_METHOD_ID => self::savltree::INSERT_EVAL_FN,
            savltree::CONTAINS_METHOD_ID => self::savltree::CONTAINS_EVAL_FN,
            savltree::REMOVE_METHOD_ID => self::savltree::REMOVE_EVAL_FN,
            savltree::UPDATE_METHOD_ID => self::savltree::UPDATE_EVAL_FN,
            savltree::INSERT_OR_UPDATE_METHOD_ID => self::savltree::INSERT_OR_UPDATE_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SAvlTree: {:?}",
                    method_id
                )))
            }
        },
        scontext::TYPE_CODE => match method.method_id() {
            scontext::DATA_INPUTS_PROPERTY_METHOD_ID => self::scontext::DATA_INPUTS_EVAL_FN,
            scontext::SELF_BOX_INDEX_PROPERTY_METHOD_ID => self::scontext::SELF_BOX_INDEX_EVAL_FN,
            scontext::HEADERS_PROPERTY_METHOD_ID => self::scontext::HEADERS_EVAL_FN,
            scontext::PRE_HEADER_PROPERTY_METHOD_ID => self::scontext::PRE_HEADER_EVAL_FN,
            scontext::LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_ID => {
                self::scontext::LAST_BLOCK_UTXO_ROOT_HASH_EVAL_FN
            }
            scontext::MINER_PUBKEY_PROPERTY_METHOD_ID => self::scontext::MINER_PUBKEY_EVAL_FN,
            scontext::GET_VAR_FROM_INPUT_METHOD_ID => self::scontext::GET_VAR_FROM_INPUT_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SContext: {:?}",
                    method_id
                )))
            }
        },
        sbox::TYPE_CODE => match method.method_id() {
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
        scoll::TYPE_CODE => match method.method_id() {
            scoll::INDEX_OF_METHOD_ID => self::scoll::INDEX_OF_EVAL_FN,
            scoll::FLATMAP_METHOD_ID => self::scoll::flatmap_eval,
            scoll::ZIP_METHOD_ID => self::scoll::ZIP_EVAL_FN,
            scoll::INDICES_METHOD_ID => self::scoll::INDICES_EVAL_FN,
            scoll::PATCH_METHOD_ID => self::scoll::PATCH_EVAL_FN,
            scoll::UPDATED_METHOD_ID => self::scoll::UPDATED_EVAL_FN,
            scoll::UPDATE_MANY_METHOD_ID => self::scoll::UPDATE_MANY_EVAL_FN,
            scoll::REVERSE_METHOD_ID => self::scoll::REVERSE_EVAL_FN,
            scoll::STARTS_WITH_METHOD_ID => self::scoll::STARTS_WITH_EVAL_FN,
            scoll::ENDS_WITH_METHOD_ID => self::scoll::ENDS_WITH_EVAL_FN,
            scoll::GET_METHOD_ID => self::scoll::GET_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SCollection: {:?}",
                    method_id
                )))
            }
        },
        sgroup_elem::TYPE_CODE => match method.method_id() {
            sgroup_elem::GET_ENCODED_METHOD_ID => self::sgroup_elem::GET_ENCODED_EVAL_FN,
            sgroup_elem::NEGATE_METHOD_ID => self::sgroup_elem::NEGATE_EVAL_FN,
            sgroup_elem::EXPONENTIATE_METHOD_ID => self::sgroup_elem::EXPONENTIATE_EVAL_FN,
            sgroup_elem::MULTIPLY_METHOD_ID => self::sgroup_elem::MULTIPLY_EVAL_FN,
            sgroup_elem::EXPONENTIATE_UNSIGNED_METHOD_ID => {
                self::sgroup_elem::EXPONENTIATE_UNSIGNED_EVAL_FN
            }
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SGroupElement: {:?}",
                    method_id
                )))
            }
        },
        soption::TYPE_CODE => match method.method_id() {
            soption::MAP_METHOD_ID => self::soption::map_eval,
            soption::FILTER_METHOD_ID => self::soption::filter_eval,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: unknown method id in SOption: {:?}",
                    method_id
                )))
            }
        },
        sheader::TYPE_CODE => match method.method_id() {
            sheader::VERSION_METHOD_ID => self::sheader::VERSION_EVAL_FN,
            sheader::ID_METHOD_ID => self::sheader::ID_EVAL_FN,
            sheader::PARENT_ID_METHOD_ID => self::sheader::PARENT_ID_EVAL_FN,
            sheader::AD_PROOFS_ROOT_METHOD_ID => self::sheader::AD_PROOFS_ROOT_EVAL_FN,
            sheader::STATE_ROOT_METHOD_ID => self::sheader::STATE_ROOT_EVAL_FN,
            sheader::TRANSACTIONS_ROOT_METHOD_ID => self::sheader::TRANSACTION_ROOT_EVAL_FN,
            sheader::EXTENSION_ROOT_METHOD_ID => self::sheader::EXTENSION_ROOT_EVAL_FN,
            sheader::TIMESTAMP_METHOD_ID => self::sheader::TIMESTAMP_EVAL_FN,
            sheader::N_BITS_METHOD_ID => self::sheader::N_BITS_EVAL_FN,
            sheader::HEIGHT_METHOD_ID => self::sheader::HEIGHT_EVAL_FN,
            sheader::MINER_PK_METHOD_ID => self::sheader::MINER_PK_EVAL_FN,
            sheader::POW_ONETIME_PK_METHOD_ID => self::sheader::POW_ONETIME_PK_EVAL_FN,
            sheader::POW_DISTANCE_METHOD_ID => self::sheader::POW_DISTANCE_EVAL_FN,
            sheader::POW_NONCE_METHOD_ID => self::sheader::POW_NONCE_EVAL_FN,
            sheader::VOTES_METHOD_ID => self::sheader::VOTES_EVAL_FN,
            sheader::CHECK_POW_METHOD_ID => self::sheader::CHECK_POW_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: method {:?} with method id {:?} not found in SHeader",
                    method.name(),
                    method_id,
                )))
            }
        },
        spreheader::TYPE_CODE => match method.method_id() {
            spreheader::VERSION_METHOD_ID => self::spreheader::VERSION_EVAL_FN,
            spreheader::PARENT_ID_METHOD_ID => self::spreheader::PARENT_ID_EVAL_FN,
            spreheader::TIMESTAMP_METHOD_ID => self::spreheader::TIMESTAMP_EVAL_FN,
            spreheader::N_BITS_METHOD_ID => self::spreheader::N_BITS_EVAL_FN,
            spreheader::HEIGHT_METHOD_ID => self::spreheader::HEIGHT_EVAL_FN,
            spreheader::MINER_PK_METHOD_ID => self::spreheader::MINER_PK_EVAL_FN,
            spreheader::VOTES_METHOD_ID => self::spreheader::VOTES_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: method {:?} with method id {:?} not found in SPreHeader",
                    method.name(),
                    method_id,
                )))
            }
        },
        sglobal::TYPE_CODE => match method.method_id() {
            sglobal::GROUP_GENERATOR_METHOD_ID => self::sglobal::GROUP_GENERATOR_EVAL_FN,
            sglobal::XOR_METHOD_ID => self::sglobal::XOR_EVAL_FN,
            sglobal::FROM_BIGENDIAN_BYTES_METHOD_ID => {
                self::sglobal::SGLOBAL_FROM_BIGENDIAN_BYTES_EVAL_FN
            }
            sglobal::DESERIALIZE_METHOD_ID => self::sglobal::DESERIALIZE_EVAL_FN,
            sglobal::SERIALIZE_METHOD_ID => self::sglobal::SERIALIZE_EVAL_FN,
            sglobal::SOME_METHOD_ID => self::sglobal::SGLOBAL_SOME_EVAL_FN,
            sglobal::NONE_METHOD_ID => self::sglobal::SGLOBAL_NONE_EVAL_FN,
            sglobal::ENCODE_NBITS_METHOD_ID => self::sglobal::ENCODE_NBITS_EVAL_FN,
            sglobal::DECODE_NBITS_METHOD_ID => self::sglobal::DECODE_NBITS_EVAL_FN,
            sglobal::POW_HIT_METHOD_ID => self::sglobal::POW_HIT_EVAL_FN,
            method_id => {
                return Err(EvalError::NotFound(format!(
                    "Eval fn: method {:?} with method id {:?} not found in SGlobal",
                    method.name(),
                    method_id,
                )))
            }
        },
        snumeric::sbyte::TYPE_CODE
        | snumeric::sshort::TYPE_CODE
        | snumeric::sint::TYPE_CODE
        | snumeric::slong::TYPE_CODE
        | snumeric::sbigint::TYPE_CODE
        | snumeric::sunsignedbigint::TYPE_CODE => numeric_method_evalfn(method)?,
        type_id => {
            return Err(EvalError::NotFound(format!(
                "Eval fn: unknown type id {:?}",
                type_id
            )))
        }
    })
}

#[doc(hidden)]
#[allow(missing_docs)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::todo)]
#[cfg(feature = "arbitrary")]
pub mod test_util {

    use super::env::Env;
    use super::*;
    use ergotree_ir::mir::constant::TryExtractFrom;
    use ergotree_ir::mir::constant::TryExtractInto;
    use ergotree_ir::serialization::sigma_byte_reader::from_bytes;
    use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
    use ergotree_ir::serialization::SigmaSerializable;
    use sigma_test_util::force_any_val;

    thread_local! {
        static TEST_CTX: Context<'static> = force_any_val::<Context>();
    }

    pub fn eval_out_wo_ctx<T: TryExtractFrom<Value<'static>> + 'static>(expr: &Expr) -> T {
        TEST_CTX.with(|ctx| eval_out(expr, ctx))
    }

    pub fn eval_out<T: TryExtractFrom<Value<'static>> + 'static>(
        expr: &Expr,
        ctx: &Context<'static>,
    ) -> T {
        let mut env = Env::empty();
        expr.eval(&mut env, ctx)
            .unwrap()
            .to_static()
            .try_extract_into::<T>()
            .unwrap()
    }

    pub fn try_eval_out<'ctx, T: TryExtractFrom<Value<'static>> + 'static>(
        expr: &Expr,
        ctx: &'ctx Context<'ctx>,
    ) -> Result<T, EvalError> {
        let mut env = Env::empty();
        expr.eval(&mut env, ctx).and_then(|v| {
            v.to_static()
                .try_extract_into::<T>()
                .map_err(EvalError::TryExtractFrom)
        })
    }

    /// Eval expr, performing deserialize node substitution before evaluation
    pub fn try_eval_with_deserialize<'ctx, T: TryExtractFrom<Value<'static>> + 'static>(
        expr: &Expr,
        ctx: &'ctx Context<'ctx>,
    ) -> Result<T, EvalError> {
        let expr = expr.clone().substitute_deserialize(ctx)?;
        try_eval_out(&expr, ctx)
    }

    // Evaluate with activated version (set block version to version + 1)
    pub fn try_eval_out_with_version<'ctx, T: TryExtractFrom<Value<'static>> + 'static>(
        expr: &Expr,
        ctx: &'ctx Context<'ctx>,
        tree_version: u8,
        activated_version: u8,
    ) -> Result<T, EvalError> {
        let mut ctx = ctx.clone();
        ctx.pre_header.version = activated_version + 1;
        ctx.tree_version.set(tree_version.into());
        // roundtrip expr to test methodcall versioning
        from_bytes(&expr.sigma_serialize_bytes()?)
            .with_tree_version(ctx.tree_version(), Expr::sigma_parse)?;
        let mut env = Env::empty();
        expr.eval(&mut env, &ctx).and_then(|v| {
            v.to_static()
                .try_extract_into::<T>()
                .map_err(EvalError::TryExtractFrom)
        })
    }

    pub fn try_eval_out_wo_ctx<T: TryExtractFrom<Value<'static>> + 'static>(
        expr: &Expr,
    ) -> Result<T, EvalError> {
        TEST_CTX.with(|ctx| try_eval_out(expr, ctx))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod test {
    use ergotree_ir::{
        chain::context::Context,
        ergo_tree::ErgoTree,
        mir::{
            bin_op::{BinOp, BinOpKind, RelationOp},
            block::BlockValue,
            expr::Expr,
            val_def::ValDef,
            val_use::ValUse,
        },
        sigma_protocol::sigma_boolean::SigmaBoolean,
        types::stype::SType,
    };
    use expect_test::expect;
    use sigma_test_util::force_any_val;

    use crate::eval::reduce_to_crypto;

    #[test]
    fn diag_on_reduced_to_false() {
        let bin_op: Expr = BinOp {
            kind: BinOpKind::Relation(RelationOp::Eq),
            left: Box::new(
                ValUse {
                    val_id: 1.into(),
                    tpe: SType::SInt,
                }
                .into(),
            ),
            right: Box::new(0i32.into()),
        }
        .into();
        let block: ErgoTree = Expr::BlockValue(
            BlockValue {
                items: vec![ValDef {
                    id: 1.into(),
                    tpe_args: vec![],
                    rhs: Box::new(Expr::Const(1i32.into())),
                }
                .into()],
                result: Box::new(bin_op),
            }
            .into(),
        )
        .try_into()
        .unwrap();
        let ctx = force_any_val::<Context>();
        let res = reduce_to_crypto(&block, &ctx).unwrap();
        assert!(res.sigma_prop == SigmaBoolean::TrivialProp(false));
        expect![[r#"
            Pretty printed expr:
            {
              val v1 = 1
              v1 == 0
            }

            Env:
            v1: 1
        "#]]
        .assert_eq(&res.diag.to_string());
    }
}
