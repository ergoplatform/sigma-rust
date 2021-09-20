use crate::serialization::types::TypeCode;

use super::smethod::MethodId;
use super::smethod::SMethodDesc;
use super::stype::SType;
use crate::types::stype::SType::{SByte, SColl};
use lazy_static::lazy_static;

/// SPreHeader type code
pub const TYPE_CODE: TypeCode = TypeCode::SPRE_HEADER;
/// SPreHeader type name
pub static TYPE_NAME: &str = "PreHeader";

lazy_static! {
    /// Pre-header method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![
            &VERSION_METHOD,
            &PARENT_ID_METHOD,
            &TIMESTAMP_METHOD,
            &N_BITS_METHOD,
            &HEIGHT_METHOD,
            &MINER_PK_METHOD,
            &VOTES_METHOD,
        ]
    ;
}

lazy_static! {
    static ref VERSION_METHOD: SMethodDesc = property("version", SByte, MethodId(1));
}

lazy_static! {
    static ref PARENT_ID_METHOD: SMethodDesc =
        property("parentId", SColl(SByte.into()), MethodId(2));
}

lazy_static! {
    static ref TIMESTAMP_METHOD: SMethodDesc = property("timestamp", SType::SLong, MethodId(3));
}

lazy_static! {
    static ref N_BITS_METHOD: SMethodDesc = property("nBits", SType::SLong, MethodId(4));
}

lazy_static! {
    static ref HEIGHT_METHOD: SMethodDesc = property("height", SType::SInt, MethodId(5));
}

lazy_static! {
    static ref MINER_PK_METHOD: SMethodDesc =
        property("minerPk", SType::SGroupElement, MethodId(6));
}

lazy_static! {
    static ref VOTES_METHOD: SMethodDesc = property("votes", SColl(SByte.into()), MethodId(7));
}

fn property(name: &'static str, res_tpe: SType, id: MethodId) -> SMethodDesc {
    SMethodDesc::property(SType::SPreHeader, name, res_tpe, id)
}
