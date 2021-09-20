use crate::serialization::types::TypeCode;
use crate::types::stype_companion::STypeCompanion;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use lazy_static::lazy_static;

/// SGroupElement type code
pub const TYPE_CODE: TypeCode = TypeCode::SGROUP_ELEMENT;
/// SGroupElement type name
pub static TYPE_NAME: &str = "GroupElement";
/// GroupElement.getEncoded
pub const GET_ENCODED_METHOD_ID: MethodId = MethodId(2);
/// GroupElement.negate
pub const NEGATE_METHOD_ID: MethodId = MethodId(5);

lazy_static! {
    /// GroupElement method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![
            &GET_ENCODED_METHOD_DESC,
            &NEGATE_METHOD_DESC
        ]
    ;
}

lazy_static! {
    static ref GET_ENCODED_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_ENCODED_METHOD_ID,
        name: "getEncoded",
        tpe: SFunc::new(
            vec![SType::SGroupElement],
            SType::SColl(Box::new(SType::SByte)),
        )
    };
    /// GroupElement.geEncoded
    pub static ref GET_ENCODED_METHOD: SMethod = SMethod::new(STypeCompanion::GroupElem, GET_ENCODED_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref NEGATE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: NEGATE_METHOD_ID,
        name: "negate",
        tpe: SFunc::new(
            vec![SType::SGroupElement],
            SType::SGroupElement,
        )
    };
    /// GroupElement.negate
    pub static ref NEGATE_METHOD: SMethod = SMethod::new(STypeCompanion::GroupElem, NEGATE_METHOD_DESC.clone(),);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(
            SMethod::from_ids(TYPE_CODE, GET_ENCODED_METHOD_ID).map(|e| e.name())
                == Ok("getEncoded")
        );
        assert!(SMethod::from_ids(TYPE_CODE, NEGATE_METHOD_ID).map(|e| e.name()) == Ok("negate"));
    }
}
