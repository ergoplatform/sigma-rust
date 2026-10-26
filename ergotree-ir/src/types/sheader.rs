#![allow(missing_docs)]

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::types::TypeCode;
use crate::types::sfunc::SFunc;
use crate::types::stype_companion::STypeCompanion;

use super::smethod::{MethodId, SMethod, SMethodDesc};
use super::stype::SType::{self, SByte, SColl};
use super::stype_companion::STypeCompanion::Header;

/// SHeader type code
pub const TYPE_CODE: TypeCode = TypeCode::SHEADER;
/// SHeader type name
pub static TYPE_NAME: &str = "Header";
/// `Header.id`
pub const ID_METHOD_ID: MethodId = MethodId(1);
/// `Header.version`
pub const VERSION_METHOD_ID: MethodId = MethodId(2);
/// `Header.parentId`
pub const PARENT_ID_METHOD_ID: MethodId = MethodId(3);
/// `Header.AdProofsRoot`
pub const AD_PROOFS_ROOT_METHOD_ID: MethodId = MethodId(4);
/// `Header.stateRoot`
pub const STATE_ROOT_METHOD_ID: MethodId = MethodId(5);
/// `Header.transactionsRoot`
pub const TRANSACTIONS_ROOT_METHOD_ID: MethodId = MethodId(6);
/// `Header.timestamp`
pub const TIMESTAMP_METHOD_ID: MethodId = MethodId(7);
/// `Header.nBits`
pub const N_BITS_METHOD_ID: MethodId = MethodId(8);
/// `Header.height`
pub const HEIGHT_METHOD_ID: MethodId = MethodId(9);
/// `Header.extensionRoot`
pub const EXTENSION_ROOT_METHOD_ID: MethodId = MethodId(10);
/// `Header.minerPk`
pub const MINER_PK_METHOD_ID: MethodId = MethodId(11);
/// `Header.powOnetimePk`
pub const POW_ONETIME_PK_METHOD_ID: MethodId = MethodId(12);
/// `Header.powNonce`
pub const POW_NONCE_METHOD_ID: MethodId = MethodId(13);
/// `Header.powDistance`
pub const POW_DISTANCE_METHOD_ID: MethodId = MethodId(14);
/// `Header.votes`
pub const VOTES_METHOD_ID: MethodId = MethodId(15);
/// `Header.checkPow`
pub const CHECK_POW_METHOD_ID: MethodId = MethodId(16);

lazy_static! {
    /// Header method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![
            ID_PROPERTY_METHOD_DESC.clone(),
            VERSION_PROPERTY_METHOD_DESC.clone(),
            PARENT_ID_PROPERTY_METHOD_DESC.clone(),
            AD_PROOFS_ROOT_PROPERTY_METHOD_DESC.clone(),
            STATE_ROOT_PROPERTY_METHOD_DESC.clone(),
            TRANSACTIONS_ROOT_PROPERTY_METHOD_DESC.clone(),
            TIMESTAMP_PROPERTY_METHOD_DESC.clone(),
            N_BITS_PROPERTY_METHOD_DESC.clone(),
            HEIGHT_PROPERTY_METHOD_DESC.clone(),
            EXTENSION_ROOT_PROPERTY_METHOD_DESC.clone(),
            MINER_PK_PROPERTY_METHOD_DESC.clone(),
            POW_ONETIME_PK_PROPERTY_METHOD_DESC.clone(),
            POW_NONCE_PROPERTY_METHOD_DESC.clone(),
            POW_DISTANCE_PROPERTY_METHOD_DESC.clone(),
            VOTES_PROPERTY_METHOD_DESC.clone(),
            CHECK_POW_METHOD_DESC.clone(),
        ]
    ;
}

lazy_static! {
    // Declaring public `SHeader` properties method signatures type

    pub static ref ID_PROPERTY: SMethod = SMethod::new(Header, ID_PROPERTY_METHOD_DESC.clone(),);
    pub static ref VERSION_PROPERTY: SMethod =
        SMethod::new(Header, VERSION_PROPERTY_METHOD_DESC.clone(),);
    pub static ref PARENT_ID_PROPERTY: SMethod =
        SMethod::new(Header, PARENT_ID_PROPERTY_METHOD_DESC.clone(),);
    pub static ref AD_PROOFS_ROOT_PROPERTY: SMethod =
        SMethod::new(Header, AD_PROOFS_ROOT_PROPERTY_METHOD_DESC.clone(),);
    pub static ref STATE_ROOT_PROPERTY: SMethod =
        SMethod::new(Header, STATE_ROOT_PROPERTY_METHOD_DESC.clone(),);
    pub static ref TRANSACTIONS_ROOT_PROPERTY: SMethod =
        SMethod::new(Header, TRANSACTIONS_ROOT_PROPERTY_METHOD_DESC.clone(),);
    pub static ref TIMESTAMP_PROPERTY: SMethod =
        SMethod::new(Header, TIMESTAMP_PROPERTY_METHOD_DESC.clone(),);
    pub static ref N_BITS_PROPERTY: SMethod =
        SMethod::new(Header, N_BITS_PROPERTY_METHOD_DESC.clone(),);
    pub static ref HEIGHT_PROPERTY: SMethod =
        SMethod::new(Header, HEIGHT_PROPERTY_METHOD_DESC.clone(),);
    pub static ref EXTENSION_ROOT_PROPERTY: SMethod =
        SMethod::new(Header, EXTENSION_ROOT_PROPERTY_METHOD_DESC.clone(),);
    pub static ref MINER_PK_PROPERTY: SMethod =
        SMethod::new(Header, MINER_PK_PROPERTY_METHOD_DESC.clone(),);
    pub static ref POW_ONETIME_PK_PROPERTY: SMethod =
        SMethod::new(Header, POW_ONETIME_PK_PROPERTY_METHOD_DESC.clone(),);
    pub static ref POW_NONCE_PROPERTY: SMethod =
        SMethod::new(Header, POW_NONCE_PROPERTY_METHOD_DESC.clone(),);
    pub static ref POW_DISTANCE_PROPERTY: SMethod =
        SMethod::new(Header, POW_DISTANCE_PROPERTY_METHOD_DESC.clone(),);
    pub static ref VOTES_PROPERTY: SMethod =
        SMethod::new(Header, VOTES_PROPERTY_METHOD_DESC.clone(),);

    // Declaring private `SHeader` properties methods descriptors

    static ref ID_PROPERTY_METHOD_DESC: SMethodDesc =
        property("id", SColl(SByte.into()), ID_METHOD_ID);
    static ref VERSION_PROPERTY_METHOD_DESC: SMethodDesc =
        property("version", SByte, VERSION_METHOD_ID);
    static ref PARENT_ID_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "parentId",
        SColl(SByte.into()),
        PARENT_ID_METHOD_ID
    );
    static ref AD_PROOFS_ROOT_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "ADProofsRoot",
        SColl(SByte.into()),
        AD_PROOFS_ROOT_METHOD_ID
    );
    static ref STATE_ROOT_PROPERTY_METHOD_DESC: SMethodDesc =
        property("stateRoot", SType::SAvlTree, STATE_ROOT_METHOD_ID);
    static ref TRANSACTIONS_ROOT_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "transactionsRoot",
        SColl(SByte.into()),
        TRANSACTIONS_ROOT_METHOD_ID
    );
    static ref TIMESTAMP_PROPERTY_METHOD_DESC: SMethodDesc =
        property("timestamp", SType::SLong, TIMESTAMP_METHOD_ID);
    static ref N_BITS_PROPERTY_METHOD_DESC: SMethodDesc =
        property("nBits", SType::SLong, N_BITS_METHOD_ID);
    static ref HEIGHT_PROPERTY_METHOD_DESC: SMethodDesc =
        property("height", SType::SInt, HEIGHT_METHOD_ID);
    static ref EXTENSION_ROOT_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "extensionRoot",
        SColl(SByte.into()),
        EXTENSION_ROOT_METHOD_ID
    );
    static ref MINER_PK_PROPERTY_METHOD_DESC: SMethodDesc =
        property("minerPk", SType::SGroupElement, MINER_PK_METHOD_ID);
    static ref POW_ONETIME_PK_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "powOnetimePk",
        SType::SGroupElement,
        POW_ONETIME_PK_METHOD_ID
    );
    static ref POW_NONCE_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "powNonce",
        SColl(SByte.into()),
        POW_NONCE_METHOD_ID
    );
    static ref POW_DISTANCE_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "powDistance",
        SType::SBigInt,
        POW_DISTANCE_METHOD_ID
    );
    static ref VOTES_PROPERTY_METHOD_DESC: SMethodDesc =
        property("votes", SColl(SByte.into()), VOTES_METHOD_ID);
    static ref CHECK_POW_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: CHECK_POW_METHOD_ID,
        name: "checkPow",
        tpe: SFunc {
            t_dom: vec![SType::SHeader],
            t_range: Box::new(SType::SBoolean),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    /// Header.checkPow
    pub static ref CHECK_POW_METHOD: SMethod = SMethod::new(STypeCompanion::Header, CHECK_POW_METHOD_DESC.clone());
}

fn property(name: &'static str, res_tpe: SType, id: MethodId) -> SMethodDesc {
    SMethodDesc::property(SType::SHeader, name, res_tpe, id)
}
