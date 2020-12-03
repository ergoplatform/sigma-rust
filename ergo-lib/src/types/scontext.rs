use super::sfunc::SFunc;
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
    type_id: TypeId(108),
    type_name: "Context",
};

lazy_static! {
    static ref DATA_INPUTS_METHOD_RAW: SMethodDesc = SMethodDesc {
        method_id: MethodId(1),
        name: "dataInputs",
        tpe: SType::SFunc(Box::new(SFunc {
            t_dom: vec![SType::SContext(SContext())],
            t_range: SType::SColl(Box::new(SType::SBox)),
            tpe_params: vec![],
        })),
    };
}

lazy_static! {
    pub static ref S_CONTEXT_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_CONTEXT_TYPE_COMPANION_HEAD,
        vec![&DATA_INPUTS_METHOD_RAW]
    );
}

lazy_static! {
    pub static ref DATA_INPUTS_METHOD: SMethod =
        SMethod::new(&S_CONTEXT_TYPE_COMPANION, &DATA_INPUTS_METHOD_RAW,);
}
