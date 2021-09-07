use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_param::STypeVar;
use crate::types::smethod::MethodId;
use crate::types::stype_companion::STypeCompanionHead;
use lazy_static::lazy_static;

/// type id
pub const TYPE_ID: TypeCode = TypeCode::OPTION;
/// Option.map
pub const MAP_METHOD_ID: MethodId = MethodId(7);
/// Option.filter
pub const FILTER_METHOD_ID: MethodId = MethodId(8);

static S_OPTION_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Option",
};

lazy_static! {
    /// Option object type companion
    pub static ref S_OPTION_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_OPTION_TYPE_COMPANION_HEAD,
        vec![
            &MAP_METHOD_DESC,
            &FILTER_METHOD_DESC,
        ]
    );
}

lazy_static! {
    static ref MAP_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: MAP_METHOD_ID,
        name: "map",
        tpe: SFunc::new(
            vec![
                SType::SOption(SType::STypeVar(STypeVar::iv()).into()),
                SFunc::new(
                    vec![STypeVar::iv().into()],
                    STypeVar::ov().into(),
                ).into()
                ],
            SType::SOption(SType::STypeVar(STypeVar::ov()).into()),
        ),
    };
    /// Option.map
    pub static ref MAP_METHOD: SMethod = SMethod::new(&S_OPTION_TYPE_COMPANION, MAP_METHOD_DESC.clone());
}

lazy_static! {
    static ref FILTER_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: FILTER_METHOD_ID,
        name: "filter",
        tpe: SFunc::new(
            vec![
                SType::SOption(SType::STypeVar(STypeVar::iv()).into()),
                SFunc::new(
                    vec![STypeVar::iv().into()],
                    SType::SBoolean,
                ).into()
                ],
            SType::SOption(SType::STypeVar(STypeVar::iv()).into()),
        ),
    };
    /// Option.map
    pub static ref FILTER_METHOD: SMethod = SMethod::new(&S_OPTION_TYPE_COMPANION, FILTER_METHOD_DESC.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_ID, MAP_METHOD_ID).map(|e| e.name()) == Ok("map"));
        assert!(SMethod::from_ids(TYPE_ID, FILTER_METHOD_ID).map(|e| e.name()) == Ok("filter"));
    }
}
