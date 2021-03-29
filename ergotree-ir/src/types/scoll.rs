use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use super::stype_param::STypeVar;
use lazy_static::lazy_static;

/// type id
pub const TYPE_ID: TypeCode = TypeCode::COLLECTION;
/// Coll.indexOf
pub const INDEX_OF_METHOD_ID: MethodId = MethodId(26);
/// Coll.flatmap
pub const FLATMAP_METHOD_ID: MethodId = MethodId(1);

static S_COLL_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Coll",
};

lazy_static! {
    /// Coll object type companion
    pub static ref S_COLL_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_COLL_TYPE_COMPANION_HEAD,
        vec![
            &INDEX_OF_METHOD_DESC,
            &FLATMAP_METHOD_DESC
        ]
    );
}

lazy_static! {
    static ref INDEX_OF_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: INDEX_OF_METHOD_ID,
        name: "indexOf",
        tpe: SFunc {
            t_dom: vec![SType::SColl(SType::STypeVar(STypeVar::t()).into()), SType::STypeVar(STypeVar::t()), SType::SInt],
            t_range: SType::SInt.into(),
            tpe_params: vec![],
        },
    };
    /// Coll.indexOf
    pub static ref INDEX_OF_METHOD: SMethod = SMethod::new(&S_COLL_TYPE_COMPANION, INDEX_OF_METHOD_DESC.clone());
}

lazy_static! {
    static ref FLATMAP_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: FLATMAP_METHOD_ID,
        name: "flatMap",
        tpe: SFunc {
            t_dom: vec![SType::SColl(SType::STypeVar(STypeVar::iv()).into()),
            SType::SFunc(
                SFunc::new(vec![SType::STypeVar(STypeVar::iv())], SType::STypeVar(STypeVar::ov()), vec![] ))
            ],
            t_range: SType::SColl(SType::STypeVar(STypeVar::ov()).into()).into(),
            tpe_params: vec![],
        },
    };
    /// Coll.flatMap
    pub static ref FLATMAP_METHOD: SMethod = SMethod::new(&S_COLL_TYPE_COMPANION, FLATMAP_METHOD_DESC.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_ID, INDEX_OF_METHOD_ID).name() == "indexOf");
        assert!(SMethod::from_ids(TYPE_ID, FLATMAP_METHOD_ID).name() == "flatMap");
    }
}
