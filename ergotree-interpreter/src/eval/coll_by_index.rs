use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;

use crate::eval::env::Env;
use crate::eval::Context;
use crate::eval::EvalError;
use crate::eval::Evaluable;

impl Evaluable for ByIndex {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(30)?; // ByIndex = Fixed(30)
        let input_v = self.input.eval(env, ctx)?;
        let index_v = self.index.eval(env, ctx)?;
        let normalized_input_vals: &CollKind<Value<'ctx>> = match &input_v {
            Value::Coll(coll) => Ok(coll),
            _ => Err(EvalError::UnexpectedValue(format!(
                "ByIndex: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        match self.default.as_ref() {
            Some(default) => {
                let mut default_v = || default.eval(env, ctx);
                let val =
                    normalized_input_vals.get_val(index_v.try_extract_into::<i32>()? as usize);
                if ctx.tree_version() >= ErgoTreeVersion::V3 {
                    val.map(Ok).unwrap_or_else(default_v)
                } else {
                    Ok(val.unwrap_or(default_v()?))
                }
            }
            None => normalized_input_vals
                .get_val(index_v.clone().try_extract_into::<i32>()? as usize)
                .ok_or_else(|| {
                    EvalError::Misc(format!(
                        "ByIndex: index {0:?} out of bounds for collection size {1:?}",
                        index_v,
                        normalized_input_vals.len()
                    ))
                }),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::mir::bin_op::ArithOp;
    use ergotree_ir::mir::bin_op::BinOp;
    use ergotree_ir::mir::bin_op::BinOpKind;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::reference::Ref;
    use sigma_test_util::force_any_val;

    use super::*;
    use crate::eval::test_util::eval_out;
    use crate::eval::test_util::eval_out_wo_ctx;
    use crate::eval::test_util::try_eval_out_with_version;
    use ergotree_ir::chain::context::Context;

    #[test]
    fn eval() {
        let expr: Expr = ByIndex::new(GlobalVars::Outputs.into(), Expr::Const(0i32.into()), None)
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Ref<'_, ErgoBox>>(&expr, &ctx).box_id(),
            ctx.outputs.first().unwrap().box_id()
        );
    }

    #[test]
    fn eval_with_default() {
        let expr: Expr = ByIndex::new(
            Expr::Const(vec![1i64, 2i64].into()),
            Expr::Const(3i32.into()),
            Some(Box::new(Expr::Const(5i64.into()))),
        )
        .unwrap()
        .into();
        assert_eq!(eval_out_wo_ctx::<i64>(&expr), 5);
    }

    #[test]
    fn eval_lazy() {
        let expr: Expr = ByIndex::new(
            Expr::Const(vec![1i64, 2i64].into()),
            Expr::Const(1i32.into()),
            Some(Box::new(Expr::BinOp(
                BinOp {
                    kind: BinOpKind::Arith(ArithOp::Divide),
                    left: Box::new(Constant::from(1i64).into()),
                    right: Box::new(Constant::from(0i64).into()),
                }
                .into(),
            ))),
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        for tree_version in 0u8..ErgoTreeVersion::V3.into() {
            assert!(try_eval_out_with_version::<Value>(&expr, &ctx, tree_version, 3).is_err());
        }
        for tree_version in ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()
        {
            assert_eq!(
                try_eval_out_with_version::<i64>(&expr, &ctx, tree_version, 3).unwrap(),
                2i64
            );
        }
    }
}
