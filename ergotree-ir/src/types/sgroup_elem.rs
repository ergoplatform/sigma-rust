use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::types::TypeCode;
use crate::types::stype_companion::STypeCompanion;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

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
/// GroupElement.exponentiate
pub const EXPONENTIATE_METHOD_ID: MethodId = MethodId(3);
/// GroupElement.multiply
pub const MULTIPLY_METHOD_ID: MethodId = MethodId(4);
/// GroupElement.negate
pub const NEGATE_METHOD_ID: MethodId = MethodId(5);
/// GroupElement.exponentiate
pub const EXPONENTIATE_UNSIGNED_METHOD_ID: MethodId = MethodId(6);

lazy_static! {
    /// GroupElement method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![
            GET_ENCODED_METHOD_DESC.clone(),
            NEGATE_METHOD_DESC.clone(),
            // expUnsigned (v6/V3): defined + evaluable, but previously absent from
            // this registry, so `from_ids` rejected it (UnknownMethodId) and any
            // ErgoTree using it failed to deserialize — while the JVM parses + evals
            // it. (exponentiate/multiply are intentionally absent: they serialize as
            // dedicated `Exponentiate`/`MultiplyGroup` ops, not method calls.)
            EXPONENTIATE_UNSIGNED_METHOD_DESC.clone(),
        ]
    ;
}

lazy_static! {
    static ref GET_ENCODED_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_ENCODED_METHOD_ID,
        name: "getEncoded",
        tpe: SFunc::new(
            vec![SType::SGroupElement],
            SType::SColl(Arc::new(SType::SByte)),
        ),
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// GroupElement.geEncoded
    pub static ref GET_ENCODED_METHOD: SMethod = SMethod::new(STypeCompanion::GroupElem, GET_ENCODED_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref EXPONENTIATE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: EXPONENTIATE_METHOD_ID,
        name: "exponentiate",
        tpe: SFunc::new(
            vec![SType::SGroupElement, SType::SBigInt],
            SType::SGroupElement,
        ),
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// GroupElement.exponentiate
    pub static ref EXPONENTIATE_METHOD: SMethod = SMethod::new(STypeCompanion::GroupElem, EXPONENTIATE_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref MULTIPLY_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MULTIPLY_METHOD_ID,
        name: "multiply",
        tpe: SFunc::new(
            vec![SType::SGroupElement, SType::SGroupElement],
            SType::SGroupElement,
        ),
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// GroupElement.multiply
    pub static ref MULTIPLY_METHOD: SMethod = SMethod::new(STypeCompanion::GroupElem, MULTIPLY_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref NEGATE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: NEGATE_METHOD_ID,
        name: "negate",
        tpe: SFunc::new(
            vec![SType::SGroupElement],
            SType::SGroupElement,
        ),
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// GroupElement.negate
    pub static ref NEGATE_METHOD: SMethod = SMethod::new(STypeCompanion::GroupElem, NEGATE_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref EXPONENTIATE_UNSIGNED_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: EXPONENTIATE_UNSIGNED_METHOD_ID,
        name: "exponentiate",
        tpe: SFunc::new(
            vec![SType::SGroupElement, SType::SUnsignedBigInt],
            SType::SGroupElement,
        ),
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// GroupElement.exponentiate
    pub static ref EXPONENTIATE_UNSIGNED_METHOD: SMethod = SMethod::new(STypeCompanion::GroupElem, EXPONENTIATE_UNSIGNED_METHOD_DESC.clone(),);
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
        // Regression: expUnsigned (method 6, v6/V3) must be resolvable by the
        // deserializer. It was defined + evaluable but missing from METHOD_DESC,
        // so `from_ids` returned UnknownMethodId and any tree using it failed to
        // parse (the JVM parses + evals it — a consensus divergence).
        assert!(
            SMethod::from_ids(TYPE_CODE, EXPONENTIATE_UNSIGNED_METHOD_ID).map(|e| e.method_id())
                == Ok(EXPONENTIATE_UNSIGNED_METHOD_ID)
        );
    }
}
