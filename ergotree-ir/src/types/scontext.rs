#![allow(missing_docs)]

use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use lazy_static::lazy_static;

pub const TYPE_ID: TypeCode = TypeCode::SCONTEXT;

static S_CONTEXT_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Context",
};

pub const DATA_INPUTS_PROPERTY_METHOD_ID: MethodId = MethodId(1);
lazy_static! {
    static ref DATA_INPUTS_PROPERTY_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: DATA_INPUTS_PROPERTY_METHOD_ID,
        name: "dataInputs",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SContext],
            t_range: Box::new(SType::SColl(Box::new(SType::SBox))),
            tpe_params: vec![],
        }),
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
