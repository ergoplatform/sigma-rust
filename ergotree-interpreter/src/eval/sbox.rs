use crate::eval::EvalError;

use alloc::boxed::Box;
use alloc::string::ToString;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;
use ergotree_ir::reference::Ref;
use ergotree_ir::types::stype::SType;

use super::EvalFn;

pub(crate) static VALUE_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    Ok(Value::Long(
        obj.try_extract_into::<Ref<'_, ErgoBox>>()?.value.as_i64(),
    ))
};

pub(crate) static GET_REG_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    if ctx.tree_version() < ErgoTreeVersion::V3 {
        return Err(EvalError::ScriptVersionError {
            required_version: ErgoTreeVersion::V3,
            activated_version: ctx.tree_version(),
        });
    }
    #[allow(clippy::unwrap_used)]
    let reg_id: i8 = args
        .first()
        .cloned()
        .ok_or_else(|| EvalError::NotFound("register index is missing".to_string()))?
        .try_extract_into::<i32>()?
        .try_into()
        .map_err(|e| {
            EvalError::RegisterIdOutOfBounds(format!("register index is out of bounds: {:?} ", e))
        })?;
    let reg_id = reg_id.try_into().map_err(|e| {
        EvalError::RegisterIdOutOfBounds(format!(
            "register index {reg_id} is out of bounds: {:?} ",
            e
        ))
    })?;

    let reg_val_opt = obj
        .try_extract_into::<Ref<'_, ErgoBox>>()?
        .get_register(reg_id)
        .map_err(|e| {
            EvalError::NotFound(format!(
                "Error getting the register id {reg_id} with error {e:?}"
            ))
        })?;
    // Return type of getReg[T] is always Option[T]
    #[allow(clippy::unreachable)]
    let SType::SOption(expected_type) = &*mc.tpe().t_range
    else {
        unreachable!()
    };
    match reg_val_opt {
        Some(constant) if constant.tpe == **expected_type => {
            Ok(Value::Opt(Some(Box::new(constant.v.into()))))
        }
        Some(constant) => Err(EvalError::UnexpectedValue(format!(
            "Expected register {reg_id} to be of type {}, got {}",
            expected_type, constant.tpe
        ))),
        None => Ok(Value::Opt(None)),
    }
};

pub(crate) static TOKENS_EVAL_FN: EvalFn = |_mc, _env, _ctx, obj, _args| {
    let res: Value = obj
        .try_extract_into::<Ref<'_, ErgoBox>>()?
        .tokens_raw()
        .into();
    Ok(res)
};

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use alloc::boxed::Box;

    use ergotree_ir::chain::ergo_box::ErgoBox;
    use ergotree_ir::ergo_tree::ErgoTreeVersion;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::extract_amount::ExtractAmount;
    use ergotree_ir::mir::extract_reg_as::ExtractRegisterAs;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::option_get::OptionGet;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
    use ergotree_ir::serialization::SigmaSerializable;
    use ergotree_ir::types::sbox;
    use ergotree_ir::types::stype::SType;
    use ergotree_ir::types::stype_param::STypeVar;
    use sigma_test_util::force_any_val;

    use crate::eval::test_util::{eval_out, try_eval_out_with_version};
    use crate::eval::EvalError;
    use ergotree_ir::chain::context::Context;

    #[test]
    fn eval_box_value() {
        let expr: Expr = PropertyCall::new(GlobalVars::SelfBox.into(), sbox::VALUE_METHOD.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(eval_out::<i64>(&expr, &ctx), ctx.self_box.value.as_i64());
    }

    // JVM-blessed vectors (santa-eval `Box.signed_view_u64`): box value and token
    // amounts are unbounded u64 on the wire (reference impl reads `getULong()` with
    // no range check); values in `[2^63, 2^64)` hydrate and surface as their signed
    // (negative) view at eval, like the JVM's Long.
    #[test]
    fn eval_box_signed_view_u64() {
        fn ctx_with_self_box(bytes_hex: &str) -> Context<'static> {
            let b = ErgoBox::sigma_parse_bytes(&base16::decode(bytes_hex).unwrap()).unwrap();
            let ctx = force_any_val::<Context>();
            Context {
                self_box: Box::leak(Box::new(b)),
                ..ctx
            }
        }
        let value_expr: Expr = ExtractAmount {
            input: Box::new(GlobalVars::SelfBox.into()),
        }
        .into();
        let r0_expr: Expr = OptionGet::try_build(
            ExtractRegisterAs::new(
                GlobalVars::SelfBox.into(),
                0,
                SType::SOption(SType::SLong.into()),
            )
            .unwrap()
            .into(),
        )
        .unwrap()
        .into();
        let tokens_expr: Expr =
            PropertyCall::new(GlobalVars::SelfBox.into(), sbox::TOKENS_METHOD.clone())
                .unwrap()
                .into();

        // box value = 2^63 → SELF.value / SELF.R0[Long].get = Long(-2^63)
        let ctx = ctx_with_self_box("808080808080808080010008cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(eval_out::<i64>(&value_expr, &ctx), i64::MIN);
        assert_eq!(eval_out::<i64>(&r0_expr, &ctx), i64::MIN);

        // box value = u64::MAX → Long(-1)
        let ctx = ctx_with_self_box("ffffffffffffffffff010008cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798000000000000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(eval_out::<i64>(&value_expr, &ctx), -1i64);
        assert_eq!(eval_out::<i64>(&r0_expr, &ctx), -1i64);

        // token amount = 2^63 → SELF.tokens(0)._2 = Long(-2^63)
        let ctx = ctx_with_self_box("c0843d0008cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798000107070707070707070707070707070707070707070707070707070707070707078080808080808080800100000000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(
            eval_out::<Vec<(Vec<i8>, i64)>>(&tokens_expr, &ctx)[0].1,
            i64::MIN
        );

        // token amount = u64::MAX → SELF.tokens(0)._2 = Long(-1)
        let ctx = ctx_with_self_box("c0843d0008cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179800010707070707070707070707070707070707070707070707070707070707070707ffffffffffffffffff0100000000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(
            eval_out::<Vec<(Vec<i8>, i64)>>(&tokens_expr, &ctx)[0].1,
            -1i64
        );
    }

    #[test]
    fn eval_box_tokens() {
        let expr: Expr = PropertyCall::new(GlobalVars::SelfBox.into(), sbox::TOKENS_METHOD.clone())
            .unwrap()
            .into();
        let ctx = force_any_val::<Context>();
        assert_eq!(
            eval_out::<Vec<(Vec<i8>, i64)>>(&expr, &ctx),
            ctx.self_box.tokens_raw()
        );
    }

    #[test]
    fn eval_reg_out() {
        let type_args = std::iter::once((STypeVar::t(), SType::SLong)).collect();
        let expr: Expr = MethodCall::with_type_args(
            GlobalVars::SelfBox.into(),
            sbox::GET_REG_METHOD.clone().with_concrete_types(&type_args),
            vec![Constant::from(0i32).into()],
            type_args,
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        (0..ErgoTreeVersion::V3.into()).for_each(|version| {
            assert!(try_eval_out_with_version::<i64>(&expr, &ctx, version, version).is_err())
        });
        (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
            |version| {
                assert_eq!(
                    try_eval_out_with_version::<Option<i64>>(&expr, &ctx, version, version)
                        .unwrap()
                        .unwrap(),
                    ctx.self_box.value.as_i64()
                )
            },
        );
    }

    // Attempt to extract SigmaProp from register of type SLong
    #[test]
    fn eval_reg_out_wrong_type() {
        let type_args = std::iter::once((STypeVar::t(), SType::SSigmaProp)).collect();
        let expr: Expr = MethodCall::with_type_args(
            GlobalVars::SelfBox.into(),
            sbox::GET_REG_METHOD.clone().with_concrete_types(&type_args),
            vec![Constant::from(0i32).into()],
            type_args,
        )
        .unwrap()
        .into();
        let ctx = force_any_val::<Context>();
        (0..ErgoTreeVersion::V3.into()).for_each(|version| {
            let res = try_eval_out_with_version::<Option<i64>>(&expr, &ctx, version, version);
            match res {
                Err(EvalError::Spanned(err))
                    if matches!(
                        *err.error,
                        EvalError::ScriptVersionError {
                            required_version: ErgoTreeVersion::V3,
                            activated_version: _
                        }
                    ) => {}
                _ => panic!("Expected script version error"),
            }
        });
        (ErgoTreeVersion::V3.into()..=ErgoTreeVersion::MAX_SCRIPT_VERSION.into()).for_each(
            |version| {
                assert!(
                    try_eval_out_with_version::<Option<i64>>(&expr, &ctx, version, version)
                        .is_err()
                )
            },
        );
    }
}
