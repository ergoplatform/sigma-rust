//! Evaluation of ErgoTree expressions

use alloc::string::ToString;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::value::Value;
use ergotree_ir::source_span::Spanned;

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
        //ctx.cost_accum.add_cost_of(self)?;
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
            Expr::ConstPlaceholder(cp) => {
                let constant = ctx
                    .constants
                    .and_then(|cs| cs.get(cp.id as usize))
                    .ok_or_else(|| {
                        EvalError::UnexpectedExpr(format!(
                            "ConstPlaceholder({}): constant not found",
                            cp.id
                        ))
                    })?;
                Ok(Value::from(constant.v.clone()))
            }
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

impl<T: Evaluable> Evaluable for Spanned<T> {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        self.expr.eval(env, ctx)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::constant::ConstantPlaceholder;
    use ergotree_ir::types::stype::SType;
    use sigma_test_util::force_any_val;

    #[test]
    fn eval_const_placeholder_with_constants_in_ctx() {
        let expr = Expr::ConstPlaceholder(ConstantPlaceholder {
            id: 0,
            tpe: SType::SInt,
        });
        let constants = vec![Constant::from(42i32)];
        let base_ctx = force_any_val::<Context>();
        let ctx = base_ctx.with_constants(&constants);
        let mut env = Env::empty();
        let result = expr.eval(&mut env, &ctx).unwrap();
        assert_eq!(result, Value::from(42i32));
    }

    #[test]
    fn eval_const_placeholder_without_constants_errors() {
        let expr = Expr::ConstPlaceholder(ConstantPlaceholder {
            id: 0,
            tpe: SType::SInt,
        });
        let ctx = force_any_val::<Context>();
        let mut env = Env::empty();
        assert!(expr.eval(&mut env, &ctx).is_err());
    }
}
