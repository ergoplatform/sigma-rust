use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::CollKind;
use ergotree_ir::mir::value::Value;
use ergotree_ir::types::stype::SType;

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
        let input_v = self.input.eval(env, ctx)?;
        let index_v = self.index.eval(env, ctx)?;
        let normalized_input_vals: &CollKind<Value<'ctx>> = match &input_v {
            Value::Coll(coll) => Ok(coll),
            _ => Err(EvalError::UnexpectedValue(format!(
                "ByIndex: expected input to be Value::Coll, got: {0:?}",
                input_v
            ))),
        }?;
        // a statically Byte or Short index parses only in trees before v3, where sigmastate
        // upcasts it to Int (a runtime Byte, Short or Int); otherwise the index must be an Int
        let small = matches!(self.index.post_eval_tpe(), SType::SByte | SType::SShort);
        let index = match index_v {
            Value::Byte(i) if small => i as i32,
            Value::Short(i) if small => i as i32,
            ref v => v.clone().try_extract_into::<i32>()?,
        };
        match self.default.as_ref() {
            Some(default) => {
                let mut default_v = || default.eval(env, ctx);
                let val = normalized_input_vals.get_val(index as usize);
                if ctx.tree_version() >= ErgoTreeVersion::V3 {
                    val.map(Ok).unwrap_or_else(default_v)
                } else {
                    Ok(val.unwrap_or(default_v()?))
                }
            }
            None => normalized_input_vals
                .get_val(index as usize)
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

    // v0..v2 trees: a Byte or Short index is used as an Int, as sigmastate 6.0.7 does
    #[test]
    fn byte_and_short_index_in_v0_to_v2_tree() {
        use crate::eval::reduce_to_crypto;
        use ergotree_ir::ergo_tree::ErgoTree;
        use ergotree_ir::serialization::SigmaSerializable;
        use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
        let mut ctx = force_any_val::<Context>();
        ctx.pre_header.version = 4; // activated script version 3
        ctx.height = 1;
        for hex in [
            "00d193b2100202040200000402",     // v0 Coll(1, 2)(0.toByte) == 1
            "090cd193b2100202040200000402",   // v1
            "0a0cd193b2100202040200000402",   // v2
            "00d193b2100202040302000404",     // v0 Coll(1, 2)(1.toShort) == 2
            "00d193b2100202047d040002000402", // index Downcast(0, Byte)
            "10031002020402000402d193b273007301007302", // segregated, ph Byte index
            "00d193b21002020495010002000400000402", // static Byte index, runtime Int 0
        ] {
            let tree = ErgoTree::sigma_parse_bytes(&base16::decode(hex).unwrap()).unwrap();
            let res = reduce_to_crypto(&tree, &ctx).map(|r| r.sigma_prop);
            assert_eq!(res, Ok(SigmaBoolean::TrivialProp(true)), "{hex}");
        }
        // static Int index with a runtime Byte: sigmastate throws ClassCastException
        for hex in [
            "00d193b21002020495010004000200000402", // v0 Coll(1, 2)(If(false, 0, 0.toByte)) == 1
            "0b11d193b21002020495010004000200000402", // v3
        ] {
            let tree = ErgoTree::sigma_parse_bytes(&base16::decode(hex).unwrap()).unwrap();
            let res = reduce_to_crypto(&tree, &ctx).map(|r| r.sigma_prop);
            assert_ne!(res, Ok(SigmaBoolean::TrivialProp(true)), "{hex}");
        }
    }
}
