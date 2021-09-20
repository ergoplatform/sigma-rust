use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_param::STypeVar;
use crate::types::smethod::MethodId;
use lazy_static::lazy_static;

/// SOption type code
pub const TYPE_CODE: TypeCode = TypeCode::OPTION;
/// SOption type name
pub static TYPE_NAME: &str = "Option";
/// Option.map
pub const MAP_METHOD_ID: MethodId = MethodId(7);
/// Option.filter
pub const FILTER_METHOD_ID: MethodId = MethodId(8);

lazy_static! {
    /// Option method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![
            &MAP_METHOD_DESC,
            &FILTER_METHOD_DESC,
        ]
    ;
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
    pub static ref MAP_METHOD: SMethod = SMethod::new(
         STypeCompanion::Option,
         MAP_METHOD_DESC.clone());
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
    pub static ref FILTER_METHOD: SMethod = SMethod::new(
         STypeCompanion::Option,
         FILTER_METHOD_DESC.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_CODE, MAP_METHOD_ID).map(|e| e.name()) == Ok("map"));
        assert!(SMethod::from_ids(TYPE_CODE, FILTER_METHOD_ID).map(|e| e.name()) == Ok("filter"));
    }
}
