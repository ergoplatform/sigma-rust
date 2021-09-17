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

pub(crate) static TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Box",
};

lazy_static! {
    /// Box method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![
            &GET_REG_METHOD_DESC,
            &VALUE_METHOD_DESC,
            &TOKENS_METHOD_DESC
        ]
    ;
}

lazy_static! {
    static ref VALUE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: VALUE_METHOD_ID,
        name: "value",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SLong),
            tpe_params: vec![],
        },
    };
    /// Box.value
    pub static ref VALUE_METHOD: SMethod = SMethod::new(STypeCompanion::Box, VALUE_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref GET_REG_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_REG_METHOD_ID,
        name: "getReg",
        tpe: SFunc {
            t_dom: vec![SType::SBox, SType::SByte],
            t_range: SType::SOption(Box::new(STypeVar::t().into())).into(),
            tpe_params: vec![],
        },
    };
    /// Box.getReg
    pub static ref GET_REG_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, GET_REG_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref TOKENS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TOKENS_METHOD_ID,
        name: "tokens",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: SType::SColl(Box::new(
                    STuple::pair(
                        SType::SColl(SType::SByte.into()),
                        SType::SLong
                    ).into())).into(),
            tpe_params: vec![],
        },
    };
    /// Box.tokens
    pub static ref TOKENS_METHOD: SMethod =
        SMethod::new( STypeCompanion::Box,TOKENS_METHOD_DESC.clone(),);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_ID, VALUE_METHOD_ID).map(|e| e.name()) == Ok("value"));
        assert!(SMethod::from_ids(TYPE_ID, GET_REG_METHOD_ID).map(|e| e.name()) == Ok("getReg"));
        assert!(SMethod::from_ids(TYPE_ID, TOKENS_METHOD_ID).map(|e| e.name()) == Ok("tokens"));
    }
}
