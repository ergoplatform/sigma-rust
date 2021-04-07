use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use lazy_static::lazy_static;

/// SGlobal type id
pub const TYPE_ID: TypeCode = TypeCode::SGLOBAL;

/// groupGenerator property
pub const GROUP_GENERATOR_METHOD_ID: MethodId = MethodId(1);
/// "xor" predefined function
pub const XOR_METHOD_ID: MethodId = MethodId(2);

static S_GLOBAL_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Global",
};

lazy_static! {
    /// Box object type companion
    pub static ref S_GLOBAL_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_GLOBAL_TYPE_COMPANION_HEAD,
        vec![
            &GROUP_GENERATOR_METHOD_DESC,
            &XOR_METHOD_DESC,
        ]
    );
}

lazy_static! {
    static ref GROUP_GENERATOR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GROUP_GENERATOR_METHOD_ID,
        name: "groupGenerator",
        tpe: SFunc {
            t_dom: vec![],
            t_range: SType::SGroupElement.into(),
            tpe_params: vec![],
        },
    };
}

lazy_static! {
    static ref XOR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: XOR_METHOD_ID,
        name: "xor",
        tpe: SFunc {
            t_dom: vec![
                SType::SColl(SType::SByte.into()),
                SType::SColl(SType::SByte.into())
            ],
            t_range: SType::SColl(SType::SByte.into()).into(),
            tpe_params: vec![],
        },
    };
}
