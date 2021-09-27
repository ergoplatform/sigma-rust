use crate::serialization::types::TypeCode;

use super::smethod::MethodId;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype_companion::STypeCompanionHead;
use crate::types::stype::SType::{SByte, SColl};
use lazy_static::lazy_static;

/// SBox type id
pub const TYPE_ID: TypeCode = TypeCode::SHEADER;

pub(crate) static TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Header",
};

lazy_static! {
    /// Header method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> =
        vec![
            &ID_METHOD_DESC,
            &VERSION_METHOD_DESC,
            &PARENT_ID_METHOD,
            &AD_PROOF_ROOT_METHOD,
            &STATE_ROOT_METHOD,
            &TRANSACTIONS_ROOT_METHOD,
            &TIMESTAMP_METHOD,
            &N_BITS_METHOD,
            &HEIGHT_METHOD,
            &EXTENSION_ROOT_METHOD,
            &MINER_PK_METHOD,
            &POW_ONETIME_PK_METHOD,
            &POW_NONCE_METHOD,
            &POW_DISTANCE_METHOD,
            &VOTES_METHOD,
        ]
    ;
}

lazy_static! {
    static ref ID_METHOD_DESC: SMethodDesc = property("id", SColl(SByte.into()), MethodId(1));
}

lazy_static! {
    static ref VERSION_METHOD_DESC: SMethodDesc = property("id", SByte, MethodId(2));
}

lazy_static! {
    static ref PARENT_ID_METHOD: SMethodDesc =
        property("parentId", SColl(SByte.into()), MethodId(3));
}

lazy_static! {
    static ref AD_PROOF_ROOT_METHOD: SMethodDesc =
        property("ADProofsRoot", SColl(SByte.into()), MethodId(4));
}

lazy_static! {
    static ref STATE_ROOT_METHOD: SMethodDesc = property("stateRoot", SType::SAvlTree, MethodId(5));
}

lazy_static! {
    static ref TRANSACTIONS_ROOT_METHOD: SMethodDesc =
        property("transactionsRoot", SColl(SByte.into()), MethodId(6));
}

lazy_static! {
    static ref TIMESTAMP_METHOD: SMethodDesc = property("timestamp", SType::SLong, MethodId(7));
}

lazy_static! {
    static ref N_BITS_METHOD: SMethodDesc = property("nBits", SType::SLong, MethodId(8));
}

lazy_static! {
    static ref HEIGHT_METHOD: SMethodDesc = property("height", SType::SInt, MethodId(9));
}

lazy_static! {
    static ref EXTENSION_ROOT_METHOD: SMethodDesc =
        property("extensionRoot", SColl(SByte.into()), MethodId(10));
}

lazy_static! {
    static ref MINER_PK_METHOD: SMethodDesc =
        property("minerPk", SType::SGroupElement, MethodId(11));
}

lazy_static! {
    static ref POW_ONETIME_PK_METHOD: SMethodDesc =
        property("powOnetimePk", SType::SGroupElement, MethodId(12));
}

lazy_static! {
    static ref POW_NONCE_METHOD: SMethodDesc =
        property("powNonce", SColl(SByte.into()), MethodId(13));
}

lazy_static! {
    static ref POW_DISTANCE_METHOD: SMethodDesc =
        property("powDistance", SType::SBigInt, MethodId(14));
}

lazy_static! {
    static ref VOTES_METHOD: SMethodDesc = property("votes", SColl(SByte.into()), MethodId(15));
}

fn property(name: &'static str, res_tpe: SType, id: MethodId) -> SMethodDesc {
    SMethodDesc::property(SType::SHeader, name, res_tpe, id)
}
