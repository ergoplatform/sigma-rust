use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use lazy_static::lazy_static;

/// SGroupElement type id
pub const TYPE_ID: TypeCode = TypeCode::SGROUP_ELEMENT;
/// GroupElement.negate
pub const NEGATE_METHOD_ID: MethodId = MethodId(5);

static S_GROUP_ELEM_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "GroupElement",
};

lazy_static! {
    /// GroupElement object type companion
    pub static ref S_GROUP_ELEM_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_GROUP_ELEM_TYPE_COMPANION_HEAD,
        vec![
            &NEGATE_METHOD_DESC
        ]
    );
}

lazy_static! {
    static ref NEGATE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: NEGATE_METHOD_ID,
        name: "negate",
        tpe: SFunc {
            t_dom: vec![SType::SGroupElement],
            t_range: Box::new(SType::SGroupElement),
            tpe_params: vec![],
        },
    };
    /// GroupElement.negate
    pub static ref NEGATE_METHOD: SMethod = SMethod::new(&S_GROUP_ELEM_TYPE_COMPANION, NEGATE_METHOD_DESC.clone(),);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_ID, NEGATE_METHOD_ID).map(|e| e.name()) == Ok("negate"));
    }
}
