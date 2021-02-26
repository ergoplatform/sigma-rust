use crate::eval::EvalError;
use crate::ir_ergo_box::IrBoxId;
use crate::mir::constant::TryExtractInto;
use crate::mir::value::Value;

use super::sfunc::SFunc;
use super::smethod::EvalFn;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stuple::STuple;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use super::stype_companion::TypeId;
use super::stype_param::STypeVar;
use lazy_static::lazy_static;

static S_BOX_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TypeId(99),
    type_name: "Box",
};

lazy_static! {
    pub static ref S_BOX_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_BOX_TYPE_COMPANION_HEAD,
        vec![
            &GET_REG_METHOD_DESC,
            &VALUE_METHOD_DESC,
            &TOKENS_METHOD_DESC
        ]
    );
}

static VALUE_EVAL_FN: EvalFn = |ctx, obj, _args| {
    Ok(Value::Long(
        obj.try_extract_into::<IrBoxId>()?
            .get_box(&ctx.box_arena)?
            .value(),
    ))
};

lazy_static! {
    static ref VALUE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MethodId(1),
        name: "value",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SLong),
            tpe_params: vec![],
        }),
        eval_fn: VALUE_EVAL_FN,
    };
    pub static ref VALUE_METHOD: SMethod = SMethod::new(&S_BOX_TYPE_COMPANION, &VALUE_METHOD_DESC,);
}

static GET_REG_EVAL_FN: EvalFn = |ctx, obj, args| {
    Ok(Value::Opt(Box::new(
        obj.try_extract_into::<IrBoxId>()?
            .get_box(&ctx.box_arena)?
            .get_register(
                args.get(0)
                    .cloned()
                    .ok_or_else(|| EvalError::NotFound("register index is missing".to_string()))?
                    .try_extract_into::<i8>()?,
            )
            .map(|c| c.v.clone()),
    )))
};

lazy_static! {
    static ref GET_REG_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MethodId(7),
        name: "getReg",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SBox, SType::SByte],
            t_range: Box::new(SType::SOption(Box::new(SType::STypeVar(STypeVar::T)))),
            tpe_params: vec![],
        }),
        eval_fn: GET_REG_EVAL_FN,
    };
    pub static ref GET_REG_METHOD: SMethod =
        SMethod::new(&S_BOX_TYPE_COMPANION, &GET_REG_METHOD_DESC,);
}

static TOKENS_EVAL_FN: EvalFn = |ctx, obj, _args| {
    let res: Value = obj
        .try_extract_into::<IrBoxId>()?
        .get_box(&ctx.box_arena)?
        .tokens()
        .into();
    Ok(res)
};

lazy_static! {
    static ref TOKENS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MethodId(8),
        name: "tokens",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SColl(Box::new(SType::STuple(STuple::pair(
                SType::SColl(Box::new(SType::SByte)),
                SType::SLong
            ))))),
            tpe_params: vec![],
        }),
        eval_fn: TOKENS_EVAL_FN,
    };
    pub static ref TOKENS_METHOD: SMethod =
        SMethod::new(&S_BOX_TYPE_COMPANION, &TOKENS_METHOD_DESC,);
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::rc::Rc;

    use crate::eval::context::Context;
    use crate::eval::tests::eval_out;
    use crate::mir::expr::Expr;
    use crate::mir::global_vars::GlobalVars;
    use crate::mir::property_call::PropertyCall;
    use crate::test_util::force_any_val;

    use super::*;

    #[test]
    fn eval_box_value() {
        let expr: Expr = PropertyCall {
            obj: Box::new(GlobalVars::SelfBox.into()),
            method: VALUE_METHOD.clone(),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<i64>(&expr, ctx.clone()),
            ctx.self_box.get_box(&ctx.box_arena).unwrap().value()
        );
    }

    #[test]
    fn eval_box_tokens() {
        let expr: Expr = PropertyCall {
            obj: Box::new(GlobalVars::SelfBox.into()),
            method: TOKENS_METHOD.clone(),
        }
        .into();
        let ctx = Rc::new(force_any_val::<Context>());
        assert_eq!(
            eval_out::<Vec<(Vec<i8>, i64)>>(&expr, ctx.clone()),
            ctx.self_box.get_box(&ctx.box_arena).unwrap().tokens()
        );
    }
}
