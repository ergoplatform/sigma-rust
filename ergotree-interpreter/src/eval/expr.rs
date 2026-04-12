//! Evaluation of ErgoTree expressions

use alloc::string::ToString;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::value::Value;
use ergotree_ir::source_span::Spanned;
use ergotree_ir::types::stype::SType;

use super::cost_accum::add_fixed_cost;
use super::costs;
use super::error::ExtResultEvalError;
use super::Context;
use super::Env;
use super::EvalError;
use super::Evaluable;

impl Evaluable for Expr {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        // Charge cost for each expression node before evaluation
        charge_expr_cost(self, ctx)?;
        let res = match self {
            Expr::Const(c) => Ok(Value::from(c.v.clone())),
            Expr::SubstConstants(op) => op.expr().eval(env, ctx),
            Expr::ByteArrayToLong(op) => op.expr().eval(env, ctx),
            Expr::ByteArrayToBigInt(op) => op.expr().eval(env, ctx),
            Expr::LongToByteArray(op) => op.eval(env, ctx),
            Expr::CalcBlake2b256(op) => op.eval(env, ctx),
            Expr::CalcSha256(op) => op.eval(env, ctx),
            Expr::Fold(op) => op.eval(env, ctx),
            Expr::ExtractRegisterAs(op) => op.expr().eval(env, ctx),
            Expr::GlobalVars(op) => op.eval(env, ctx),
            Expr::MethodCall(op) => op.expr().eval(env, ctx),
            Expr::PropertyCall(op) => op.expr().eval(env, ctx),
            Expr::BinOp(op) => op.expr().eval(env, ctx),
            Expr::Global => Ok(Value::Global),
            Expr::Context => Ok(Value::Context),
            Expr::OptionGet(v) => v.expr().eval(env, ctx),
            Expr::Apply(op) => op.eval(env, ctx),
            Expr::FuncValue(op) => op.eval(env, ctx),
            Expr::ValUse(op) => op.eval(env, ctx),
            Expr::BlockValue(op) => op.expr().eval(env, ctx),
            Expr::SelectField(op) => op.eval(env, ctx),
            Expr::ExtractAmount(op) => op.eval(env, ctx),
            Expr::ConstPlaceholder(cp) => match &cp.resolved {
                Some(c) => Ok(Value::from(c.v.clone())),
                None => Err(EvalError::UnexpectedExpr(
                    "ConstPlaceholder without resolved value".to_string(),
                )),
            },
            Expr::Collection(op) => op.eval(env, ctx),
            Expr::ValDef(_) => Err(EvalError::UnexpectedExpr(
                ("ValDef should be evaluated in BlockValue").to_string(),
            )),
            Expr::And(op) => op.eval(env, ctx),
            Expr::Or(op) => op.eval(env, ctx),
            Expr::Xor(op) => op.eval(env, ctx),
            Expr::Atleast(op) => op.eval(env, ctx),
            Expr::LogicalNot(op) => op.eval(env, ctx),
            Expr::Map(op) => op.eval(env, ctx),
            Expr::Filter(op) => op.eval(env, ctx),
            Expr::BoolToSigmaProp(op) => op.eval(env, ctx),
            Expr::Upcast(op) => op.eval(env, ctx),
            Expr::Downcast(op) => op.eval(env, ctx),
            Expr::If(op) => op.eval(env, ctx),
            Expr::Append(op) => op.expr().eval(env, ctx),
            Expr::ByIndex(op) => op.expr().eval(env, ctx),
            Expr::ExtractScriptBytes(op) => op.eval(env, ctx),
            Expr::SizeOf(op) => op.eval(env, ctx),
            Expr::Slice(op) => op.eval(env, ctx),
            Expr::CreateProveDlog(op) => op.eval(env, ctx),
            Expr::CreateProveDhTuple(op) => op.eval(env, ctx),
            Expr::ExtractCreationInfo(op) => op.eval(env, ctx),
            Expr::Exists(op) => op.eval(env, ctx),
            Expr::ExtractId(op) => op.eval(env, ctx),
            Expr::SigmaPropBytes(op) => op.eval(env, ctx),
            Expr::OptionIsDefined(op) => op.expr().eval(env, ctx),
            Expr::OptionGetOrElse(op) => op.expr().eval(env, ctx),
            Expr::Negation(op) => op.expr().eval(env, ctx),
            Expr::BitInversion(op) => op.eval(env, ctx),
            Expr::ForAll(op) => op.eval(env, ctx),
            Expr::Tuple(op) => op.eval(env, ctx),
            Expr::DecodePoint(op) => op.eval(env, ctx),
            Expr::SigmaAnd(op) => op.eval(env, ctx),
            Expr::SigmaOr(op) => op.eval(env, ctx),
            Expr::DeserializeRegister(_) => Err(EvalError::UnexpectedExpr(
                "DeserializeRegister cannot be evaluated".to_string(),
            )),
            Expr::DeserializeContext(_) => Err(EvalError::UnexpectedExpr(
                "DeserializeContext cannot be evaluated".to_string(),
            )),
            Expr::GetVar(op) => op.eval(env, ctx),
            Expr::MultiplyGroup(op) => op.eval(env, ctx),
            Expr::Exponentiate(op) => op.eval(env, ctx),
            Expr::XorOf(op) => op.eval(env, ctx),
            Expr::ExtractBytes(op) => op.eval(env, ctx),
            Expr::ExtractBytesWithNoRef(op) => op.eval(env, ctx),
            Expr::TreeLookup(op) => op.eval(env, ctx),
            Expr::CreateAvlTree(op) => op.eval(env, ctx),
        };
        res.enrich_err(self.span(), env)
    }
}

/// Charge the fixed cost for an expression node.
/// Per-item costs for collection ops are charged inside their specific implementations.
fn charge_expr_cost(expr: &Expr, ctx: &Context) -> Result<(), EvalError> {
    use ergotree_ir::mir::bin_op::BinOpKind;
    match expr {
        Expr::Const(_) => add_fixed_cost(ctx, costs::CONST_COST)?,
        Expr::ConstPlaceholder(_) => add_fixed_cost(ctx, costs::CONST_PLACEHOLDER_COST)?,
        Expr::ValUse(_) => add_fixed_cost(ctx, costs::VAL_USE_COST)?,
        Expr::FuncValue(_) => add_fixed_cost(ctx, costs::FUNC_VALUE_COST)?,
        Expr::Apply(_) => add_fixed_cost(ctx, costs::APPLY_COST)?,
        Expr::MethodCall(_) => add_fixed_cost(ctx, costs::METHOD_CALL_COST)?,
        Expr::PropertyCall(_) => add_fixed_cost(ctx, costs::PROPERTY_CALL_COST)?,
        Expr::Tuple(_) => add_fixed_cost(ctx, costs::TUPLE_COST)?,
        Expr::Collection(_) => add_fixed_cost(ctx, costs::CONCRETE_COLLECTION_COST)?,
        Expr::SelectField(_) => add_fixed_cost(ctx, costs::SELECT_FIELD_COST)?,

        // Global variables
        Expr::GlobalVars(gv) => match gv {
            GlobalVars::Height => add_fixed_cost(ctx, costs::HEIGHT_COST)?,
            GlobalVars::Inputs => add_fixed_cost(ctx, costs::INPUTS_COST)?,
            GlobalVars::Outputs => add_fixed_cost(ctx, costs::OUTPUTS_COST)?,
            GlobalVars::SelfBox => add_fixed_cost(ctx, costs::SELF_COST)?,
            GlobalVars::GroupGenerator => add_fixed_cost(ctx, costs::SGLOBAL_GROUP_GENERATOR_COST)?,
            GlobalVars::MinerPubKey => add_fixed_cost(ctx, costs::SCONTEXT_MINER_PUBKEY_COST)?,
        },
        Expr::Context => add_fixed_cost(ctx, costs::CONTEXT_COST)?,
        Expr::Global => add_fixed_cost(ctx, costs::GLOBAL_COST)?,

        // Box extraction
        Expr::ExtractAmount(_) => add_fixed_cost(ctx, costs::EXTRACT_AMOUNT_COST)?,
        Expr::ExtractScriptBytes(_) => add_fixed_cost(ctx, costs::EXTRACT_SCRIPT_BYTES_COST)?,
        Expr::ExtractBytes(_) => add_fixed_cost(ctx, costs::EXTRACT_BYTES_COST)?,
        Expr::ExtractBytesWithNoRef(_) => {
            add_fixed_cost(ctx, costs::EXTRACT_BYTES_WITH_NO_REF_COST)?
        }
        Expr::ExtractId(_) => add_fixed_cost(ctx, costs::EXTRACT_ID_COST)?,
        Expr::ExtractRegisterAs(_) => add_fixed_cost(ctx, costs::EXTRACT_REGISTER_AS_COST)?,
        Expr::ExtractCreationInfo(_) => add_fixed_cost(ctx, costs::EXTRACT_CREATION_INFO_COST)?,

        // Option operations
        Expr::GetVar(_) => add_fixed_cost(ctx, costs::GET_VAR_COST)?,
        Expr::OptionGet(_) => add_fixed_cost(ctx, costs::OPTION_GET_COST)?,
        Expr::OptionGetOrElse(_) => add_fixed_cost(ctx, costs::OPTION_GET_OR_ELSE_COST)?,
        Expr::OptionIsDefined(_) => add_fixed_cost(ctx, costs::OPTION_IS_DEFINED_COST)?,

        // Sigma protocol
        Expr::BoolToSigmaProp(_) => add_fixed_cost(ctx, costs::BOOL_TO_SIGMA_PROP_COST)?,
        Expr::CreateProveDlog(_) => add_fixed_cost(ctx, costs::CREATE_PROVE_DLOG_COST)?,
        Expr::CreateProveDhTuple(_) => add_fixed_cost(ctx, costs::CREATE_PROVE_DH_TUPLE_COST)?,

        // Collection ops with fixed cost
        Expr::ByIndex(_) => add_fixed_cost(ctx, costs::BY_INDEX_COST)?,
        Expr::SizeOf(_) => add_fixed_cost(ctx, costs::SIZE_OF_COST)?,

        // Numeric conversions
        Expr::LongToByteArray(_) => add_fixed_cost(ctx, costs::LONG_TO_BYTE_ARRAY_COST)?,
        Expr::ByteArrayToLong(_) => add_fixed_cost(ctx, costs::BYTE_ARRAY_TO_LONG_COST)?,
        Expr::ByteArrayToBigInt(_) => add_fixed_cost(ctx, costs::BYTE_ARRAY_TO_BIGINT_COST)?,
        Expr::Negation(_) => add_fixed_cost(ctx, costs::NEGATION_COST)?,
        Expr::BitInversion(_) => add_fixed_cost(ctx, costs::BIT_INVERSION_COST)?,
        Expr::LogicalNot(_) => add_fixed_cost(ctx, costs::LOGICAL_NOT_COST)?,

        // Cryptographic
        Expr::DecodePoint(_) => add_fixed_cost(ctx, costs::DECODE_POINT_COST)?,
        Expr::Exponentiate(_) => add_fixed_cost(ctx, costs::EXPONENTIATE_COST)?,
        Expr::MultiplyGroup(_) => add_fixed_cost(ctx, costs::MULTIPLY_GROUP_COST)?,

        // Control flow
        Expr::If(_) => add_fixed_cost(ctx, costs::IF_COST)?,

        // BinOp - cost depends on operation kind
        Expr::BinOp(spanned) => match spanned.expr.kind {
            // Arith and Relation costs are type-dependent; charged inside bin_op.rs
            BinOpKind::Arith(_) | BinOpKind::Relation(_) => {}
            BinOpKind::Logical(lop) => match lop {
                ergotree_ir::mir::bin_op::LogicalOp::And => {
                    add_fixed_cost(ctx, costs::BIN_AND_COST)?
                }
                ergotree_ir::mir::bin_op::LogicalOp::Or => add_fixed_cost(ctx, costs::BIN_OR_COST)?,
                ergotree_ir::mir::bin_op::LogicalOp::Xor => {
                    add_fixed_cost(ctx, costs::BIN_XOR_COST)?
                }
            },
            BinOpKind::Bit(_) => add_fixed_cost(ctx, costs::BIT_OP_COST)?,
        },

        // Upcast/Downcast — type-based cost (BigInt targets cost more)
        Expr::Upcast(op) => {
            let jit_cost = if matches!(op.tpe, SType::SBigInt | SType::SUnsignedBigInt) {
                costs::NUMERIC_CAST_COST.bigint_cost
            } else {
                costs::NUMERIC_CAST_COST.default_cost
            };
            add_fixed_cost(ctx, costs::FixedCost(jit_cost))?
        }
        Expr::Downcast(op) => {
            let jit_cost = if matches!(op.tpe, SType::SBigInt | SType::SUnsignedBigInt) {
                costs::NUMERIC_CAST_COST.bigint_cost
            } else {
                costs::NUMERIC_CAST_COST.default_cost
            };
            add_fixed_cost(ctx, costs::FixedCost(jit_cost))?
        }

        // Tree operations
        // TreeLookup uses DynamicCost: charged inside tree_lookup.rs after extracting proof/digest
        Expr::TreeLookup(_) => {}
        Expr::CreateAvlTree(_) => add_fixed_cost(ctx, costs::CREATE_AVL_TREE_COST)?,

        // Per-item cost operations: full cost (base + per-item) charged inside
        // their respective eval implementations where n_items is known
        Expr::Map(_)
        | Expr::Filter(_)
        | Expr::Exists(_)
        | Expr::ForAll(_)
        | Expr::Fold(_)
        | Expr::Append(_)
        | Expr::Slice(_)
        | Expr::And(_)
        | Expr::Or(_)
        | Expr::Xor(_)
        | Expr::XorOf(_)
        | Expr::Atleast(_)
        | Expr::CalcBlake2b256(_)
        | Expr::CalcSha256(_)
        | Expr::SubstConstants(_)
        | Expr::SigmaAnd(_)
        | Expr::SigmaOr(_)
        | Expr::SigmaPropBytes(_)
        | Expr::BlockValue(_) => {}

        // ValDef / Deserialize - no additional cost
        Expr::ValDef(_) | Expr::DeserializeRegister(_) | Expr::DeserializeContext(_) => {}
    }
    Ok(())
}

impl<T: Evaluable> Evaluable for Spanned<T> {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        self.expr.eval(env, ctx)
    }
}
