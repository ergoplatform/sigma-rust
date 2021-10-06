#![allow(missing_docs)]

use crate::serialization::types::TypeCode;

use super::smethod::{MethodId, SMethod, SMethodDesc};
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use crate::types::stype::SType::{SByte, SColl};
use lazy_static::lazy_static;

/// SPreHeader type code
pub const TYPE_CODE: TypeCode = TypeCode::SPRE_HEADER;
/// SPreHeader type name
pub static TYPE_NAME: &str = "PreHeader";
/// `PreHeader.version`
pub const VERSION_METHOD_ID: MethodId = MethodId(1);
/// `PreHeader.parentId`
pub const PARENT_ID_METHOD_ID: MethodId = MethodId(2);
/// `PreHeader.timestamp`
pub const TIMESTAMP_METHOD_ID: MethodId = MethodId(3);
/// `PreHeader.nBits`
pub const N_BITS_METHOD_ID: MethodId = MethodId(4);
/// `PreHeader.height`
pub const HEIGHT_METHOD_ID: MethodId = MethodId(5);
/// `PreHeader.minerPk`
pub const MINER_PK_METHOD_ID: MethodId = MethodId(6);
/// `PreHeader.votes`
pub const VOTES_METHOD_ID: MethodId = MethodId(7);

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
    pub static ref VERSION_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::PreHeader, VERSION_METHOD.clone());
    static ref VERSION_METHOD: SMethodDesc = property("version", SByte, VERSION_METHOD_ID);
}

lazy_static! {
    pub static ref PARENT_ID_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::PreHeader, PARENT_ID_METHOD.clone());
    static ref PARENT_ID_METHOD: SMethodDesc =
        property("parentId", SColl(SByte.into()), PARENT_ID_METHOD_ID);
}

lazy_static! {
    pub static ref TIMESTAMP_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::PreHeader, TIMESTAMP_METHOD.clone());
    static ref TIMESTAMP_METHOD: SMethodDesc =
        property("timestamp", SType::SLong, TIMESTAMP_METHOD_ID);
}

lazy_static! {
    pub static ref N_BITS_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::PreHeader, N_BITS_METHOD.clone());
    static ref N_BITS_METHOD: SMethodDesc = property("nBits", SType::SLong, N_BITS_METHOD_ID);
}

lazy_static! {
    pub static ref HEIGHT_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::PreHeader, HEIGHT_METHOD.clone());
    static ref HEIGHT_METHOD: SMethodDesc = property("height", SType::SInt, HEIGHT_METHOD_ID);
}

lazy_static! {
    pub static ref MINER_PK_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::PreHeader, MINER_PK_METHOD.clone());
    static ref MINER_PK_METHOD: SMethodDesc =
        property("minerPk", SType::SGroupElement, MINER_PK_METHOD_ID);
}

lazy_static! {
    pub static ref VOTES_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::PreHeader, VOTES_METHOD.clone());
    static ref VOTES_METHOD: SMethodDesc = property("votes", SColl(SByte.into()), VOTES_METHOD_ID);
}

fn property(name: &'static str, res_tpe: SType, id: MethodId) -> SMethodDesc {
    SMethodDesc::property(SType::SPreHeader, name, res_tpe, id)
}
