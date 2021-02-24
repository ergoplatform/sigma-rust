use crate::mir::value::CollKind;
use crate::mir::value::Value;

use super::sfunc::SFunc;
use super::smethod::EvalFn;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use super::stype_companion::TypeId;
use lazy_static::lazy_static;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SContext();

static S_CONTEXT_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TypeId(101),
    type_name: "Context",
};

static DATA_INPUTS_EVAL_FN: EvalFn = |ctx, _obj, _args| {
    // TODO: check that obj is Value::Context
    Ok(Value::Coll(CollKind::WrappedColl {
        items: ctx
            .data_inputs
            .clone()
            .into_iter()
            .map(|b| Value::CBox(b))
            .collect(),
        elem_tpe: SType::SBox,
    }))
};

lazy_static! {
    static ref DATA_INPUTS_PROPERTY_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MethodId(1),
        name: "dataInputs",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SContext(SContext())],
            t_range: Box::new(SType::SColl(Box::new(SType::SBox))),
            tpe_params: vec![],
        }),
        eval_fn: DATA_INPUTS_EVAL_FN,
    };
}

lazy_static! {
    pub static ref S_CONTEXT_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_CONTEXT_TYPE_COMPANION_HEAD,
        vec![&DATA_INPUTS_PROPERTY_METHOD_DESC]
    );
}

lazy_static! {
    pub static ref DATA_INPUTS_PROPERTY: SMethod =
        SMethod::new(&S_CONTEXT_TYPE_COMPANION, &DATA_INPUTS_PROPERTY_METHOD_DESC,);
}
