use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stuple::STuple;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use super::stype_param::STypeVar;
use lazy_static::lazy_static;

/// SBox type id
pub const TYPE_ID: TypeCode = TypeCode::SBOX;
/// Box.value property
pub const VALUE_METHOD_ID: MethodId = MethodId(1);
/// Box.Rx property
pub const GET_REG_METHOD_ID: MethodId = MethodId(7);
/// Box.tokens property
pub const TOKENS_METHOD_ID: MethodId = MethodId(8);

static S_BOX_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Box",
};

lazy_static! {
    /// Box object type companion
    pub static ref S_BOX_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_BOX_TYPE_COMPANION_HEAD,
        vec![
            &GET_REG_METHOD_DESC,
            &VALUE_METHOD_DESC,
            &TOKENS_METHOD_DESC
        ]
    );
}

lazy_static! {
    static ref VALUE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: VALUE_METHOD_ID,
        name: "value",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SLong),
            tpe_params: vec![],
        }),
    };
    /// Box.value
    pub static ref VALUE_METHOD: SMethod = SMethod::new(&S_BOX_TYPE_COMPANION, &VALUE_METHOD_DESC,);
}

lazy_static! {
    static ref GET_REG_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_REG_METHOD_ID,
        name: "getReg",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SBox, SType::SByte],
            t_range: Box::new(SType::SOption(Box::new(SType::STypeVar(STypeVar::T)))),
            tpe_params: vec![],
        }),
    };
    /// Box.getReg
    pub static ref GET_REG_METHOD: SMethod =
        SMethod::new(&S_BOX_TYPE_COMPANION, &GET_REG_METHOD_DESC,);
}

lazy_static! {
    static ref TOKENS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TOKENS_METHOD_ID,
        name: "tokens",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SColl(Box::new(SType::STuple(STuple::pair(
                SType::SColl(Box::new(SType::SByte)),
                SType::SLong
            ))))),
            tpe_params: vec![],
        }),
    };
    /// Box.tokens
    pub static ref TOKENS_METHOD: SMethod =
        SMethod::new(&S_BOX_TYPE_COMPANION, &TOKENS_METHOD_DESC,);
}
