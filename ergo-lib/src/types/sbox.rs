use std::convert::TryInto;

use crate::ast::constant::TryExtractInto;
use crate::chain::ergo_box::ErgoBox;
use crate::eval::EvalError;

use super::sfunc::SFunc;
use super::smethod::EvalFn;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
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

static GET_REG_EVAL_FN: EvalFn = |obj, args| {
    Ok(obj
        .try_extract_into::<ErgoBox>()?
        .get_register(
            args.get(0)
                .cloned()
                .ok_or_else(|| EvalError::NotFound("register index is missing".to_string()))?
                .try_extract_into::<i8>()?
                .try_into()?,
        )
        // TODO: add register id to the error
        .ok_or_else(|| EvalError::NotFound("no value in register".to_string()))?
        .v)
    // TODO: return Opt
};

lazy_static! {
    static ref GET_REG_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MethodId(7),
        name: "getReg",
        tpe: SType::SFunc(Box::new(SFunc {
            t_dom: vec![SType::SBox, SType::SByte],
            t_range: SType::SOption(Box::new(SType::STypeVar(STypeVar::T))),
            tpe_params: vec![],
        })),
        eval_fn: GET_REG_EVAL_FN,
    };
}

lazy_static! {
    pub static ref S_BOX_TYPE_COMPANION: STypeCompanion =
        STypeCompanion::new(&S_BOX_TYPE_COMPANION_HEAD, vec![&GET_REG_METHOD_DESC]);
}

lazy_static! {
    pub static ref GET_REG_METHOD: SMethod =
        SMethod::new(&S_BOX_TYPE_COMPANION, &GET_REG_METHOD_DESC,);
}
