use crate::serialization::types::TypeCode;

use super::smethod::MethodId;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use crate::types::stype::SType::{SByte, SColl};
use lazy_static::lazy_static;

/// SBox type id
pub const TYPE_ID: TypeCode = TypeCode::SPRE_HEADER;

static S_PRE_HEADER_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "PreHeader",
};

lazy_static! {
    /// Box object type companion
    pub static ref S_PRE_HEADER_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_PRE_HEADER_TYPE_COMPANION_HEAD,
        vec![
            &VERSION_METHOD,
            &PARENT_ID_METHOD,
            &TIMESTAMP_METHOD,
            &N_BITS_METHOD,
            &HEIGHT_METHOD,
            &MINER_PK_METHOD,
            &VOTES_METHOD,
        ]
    );
}

lazy_static! {
    static ref VERSION_METHOD: SMethodDesc = SMethodDesc::property("version", SByte, MethodId(1));
}

lazy_static! {
    static ref PARENT_ID_METHOD: SMethodDesc =
        SMethodDesc::property("parentId", SColl(SByte.into()), MethodId(2));
}

lazy_static! {
    static ref TIMESTAMP_METHOD: SMethodDesc =
        SMethodDesc::property("timestamp", SType::SLong, MethodId(3));
}

lazy_static! {
    static ref N_BITS_METHOD: SMethodDesc =
        SMethodDesc::property("nBits", SType::SLong, MethodId(4));
}

lazy_static! {
    static ref HEIGHT_METHOD: SMethodDesc =
        SMethodDesc::property("height", SType::SInt, MethodId(5));
}

lazy_static! {
    static ref MINER_PK_METHOD: SMethodDesc =
        SMethodDesc::property("minerPk", SType::SGroupElement, MethodId(6));
}

lazy_static! {
    static ref VOTES_METHOD: SMethodDesc =
        SMethodDesc::property("votes", SColl(SByte.into()), MethodId(7));
}
