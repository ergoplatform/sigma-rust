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

pub(crate) static VALUE_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(8)?;
    Ok(Value::Long(
        obj.try_extract_into::<Ref<'_, ErgoBox>>()?.value.as_i64(),
    ))
};

pub(crate) static GET_REG_EVAL_FN: EvalFn = |mc, _env, ctx, obj, args| {
    ctx.add_jit_cost(50)?;
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

pub(crate) static TOKENS_EVAL_FN: EvalFn = |_mc, _env, ctx, obj, _args| {
    ctx.add_jit_cost(15)?;
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
    use ergotree_ir::ergo_tree::ErgoTreeVersion;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::method_call::MethodCall;
    use ergotree_ir::mir::property_call::PropertyCall;
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
