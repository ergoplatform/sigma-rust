use crate::eval::EvalError;

use ergotree_ir::ir_ergo_box::IrBoxId;
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::mir::value::Value;

use super::EvalFn;

pub static VALUE_EVAL_FN: EvalFn = |ctx, obj, _args| {
    Ok(Value::Long(
        obj.try_extract_into::<IrBoxId>()?
            .get_box(&ctx.box_arena)?
            .value(),
    ))
};

pub static GET_REG_EVAL_FN: EvalFn = |ctx, obj, args| {
    Ok(Value::Opt(Box::new(
        obj.try_extract_into::<IrBoxId>()?
            .get_box(&ctx.box_arena)?
            .get_register(
                args.get(0)
                    .cloned()
                    .ok_or_else(|| EvalError::NotFound("register index is missing".to_string()))?
                    .try_extract_into::<i8>()?,
            )
            .map(|c| c.v),
    )))
};

pub static TOKENS_EVAL_FN: EvalFn = |ctx, obj, _args| {
    let res: Value = obj
        .try_extract_into::<IrBoxId>()?
        .get_box(&ctx.box_arena)?
        .tokens_raw()
        .into();
    Ok(res)
};

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::global_vars::GlobalVars;
    use ergotree_ir::mir::property_call::PropertyCall;
    use ergotree_ir::types::sbox;
    use sigma_test_util::force_any_val;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use std::rc::Rc;

    #[test]
    fn eval_box_value() {
        let expr: Expr = PropertyCall::new(GlobalVars::SelfBox.into(), sbox::VALUE_METHOD.clone())
            .unwrap()
            .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i64>(&expr, ctx.clone()),
            ctx.self_box.get_box(&ctx.box_arena).unwrap().value()
        );
    }

    #[test]
    fn eval_box_tokens() {
        let expr: Expr = PropertyCall::new(GlobalVars::SelfBox.into(), sbox::TOKENS_METHOD.clone())
            .unwrap()
            .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<(Vec<i8>, i64)>>(&expr, ctx.clone()),
            ctx.self_box.get_box(&ctx.box_arena).unwrap().tokens_raw()
        );
    }
}
