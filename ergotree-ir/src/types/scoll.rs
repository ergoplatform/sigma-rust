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
        ]
    );
}

lazy_static! {
    static ref INDEX_OF_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: INDEX_OF_METHOD_ID,
        name: "indexOf",
        tpe: SType::SFunc(SFunc {
            t_dom: vec![SType::SColl(SType::STypeVar(STypeVar::T).into()), SType::SInt, SType::SInt],
            t_range: SType::SInt.into(),
            tpe_params: vec![],
        }),
    };
    /// Box.value
    pub static ref INDEX_OF_METHOD: SMethod = SMethod::new(&S_COLL_TYPE_COMPANION, &INDEX_OF_METHOD_DESC);
}
