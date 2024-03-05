#![allow(missing_docs)]

use crate::serialization::types::TypeCode;
use crate::types::stype_companion::STypeCompanion;

use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype::SType::{SAvlTree, SBox, SByte, SColl, SHeader, SInt, SPreHeader};
use lazy_static::lazy_static;

/// SContext type code
pub const TYPE_CODE: TypeCode = TypeCode::SCONTEXT;
/// SContext type name
pub static TYPE_NAME: &str = "Context";

lazy_static! {
    /// Context method descriptors
    pub(crate) static ref METHOD_DESC: Vec<&'static SMethodDesc> = vec![
        &DATA_INPUTS_PROPERTY_METHOD_DESC,
        &HEADERS_PROPERTY_METHOD_DESC,
        &PRE_HEADER_PROPERTY_METHOD_DESC,
        &INPUTS_PROPERTY_METHOD_DESC,
        &OUTPUTS_PROPERTY_METHOD_DESC,
        &HEIGHT_PROPERTY_METHOD_DESC,
        &SELF_PROPERTY_METHOD_DESC,
        &SELF_BOX_INDEX_PROPERTY_METHOD_DESC,
        &LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_DESC,
        &MINER_PUBKEY_PROPERTY_METHOD_DESC,
    ];
}

// ---- Methods ----

pub const DATA_INPUTS_PROPERTY_METHOD_ID: MethodId = MethodId(1);
lazy_static! {
    static ref DATA_INPUTS_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "dataInputs",
        SColl(SBox.into()),
        DATA_INPUTS_PROPERTY_METHOD_ID
    );
}
lazy_static! {
    pub static ref DATA_INPUTS_PROPERTY: SMethod = SMethod::new(
        STypeCompanion::Context,
        DATA_INPUTS_PROPERTY_METHOD_DESC.clone(),
    );
}

pub const HEADERS_PROPERTY_METHOD_ID: MethodId = MethodId(2);
lazy_static! {
    static ref HEADERS_PROPERTY_METHOD_DESC: SMethodDesc =
        property("headers", SColl(SHeader.into()), HEADERS_PROPERTY_METHOD_ID);
}
lazy_static! {
    pub static ref HEADERS_PROPERTY: SMethod = SMethod::new(
        STypeCompanion::Context,
        HEADERS_PROPERTY_METHOD_DESC.clone()
    );
}

pub const PRE_HEADER_PROPERTY_METHOD_ID: MethodId = MethodId(3);
lazy_static! {
    static ref PRE_HEADER_PROPERTY_METHOD_DESC: SMethodDesc =
        property("preHeader", SPreHeader, PRE_HEADER_PROPERTY_METHOD_ID);
}
lazy_static! {
    pub static ref PRE_HEADER_PROPERTY: SMethod = SMethod::new(
        STypeCompanion::Context,
        PRE_HEADER_PROPERTY_METHOD_DESC.clone()
    );
}

pub const INPUTS_PROPERTY_METHOD_ID: MethodId = MethodId(4);
lazy_static! {
    static ref INPUTS_PROPERTY_METHOD_DESC: SMethodDesc =
        property("INPUTS", SColl(SBox.into()), INPUTS_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref INPUTS_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::Context, INPUTS_PROPERTY_METHOD_DESC.clone());
}

pub const OUTPUTS_PROPERTY_METHOD_ID: MethodId = MethodId(5);
lazy_static! {
    static ref OUTPUTS_PROPERTY_METHOD_DESC: SMethodDesc =
        property("OUTPUTS", SColl(SBox.into()), OUTPUTS_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref OUTPUTS_PROPERTY: SMethod = SMethod::new(
        STypeCompanion::Context,
        OUTPUTS_PROPERTY_METHOD_DESC.clone()
    );
}

pub const HEIGHT_PROPERTY_METHOD_ID: MethodId = MethodId(6);
lazy_static! {
    static ref HEIGHT_PROPERTY_METHOD_DESC: SMethodDesc =
        property("HEIGHT", SInt, HEIGHT_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref HEIGHT_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::Context, HEIGHT_PROPERTY_METHOD_DESC.clone());
}

pub const SELF_PROPERTY_METHOD_ID: MethodId = MethodId(7);
lazy_static! {
    static ref SELF_PROPERTY_METHOD_DESC: SMethodDesc =
        property("SELF", SBox, SELF_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref SELF_PROPERTY: SMethod =
        SMethod::new(STypeCompanion::Context, SELF_PROPERTY_METHOD_DESC.clone());
}

pub const SELF_BOX_INDEX_PROPERTY_METHOD_ID: MethodId = MethodId(8);
lazy_static! {
    static ref SELF_BOX_INDEX_PROPERTY_METHOD_DESC: SMethodDesc =
        property("selfBoxIndex", SInt, SELF_BOX_INDEX_PROPERTY_METHOD_ID);
}
lazy_static! {
    pub static ref SELF_BOX_INDEX_PROPERTY: SMethod = SMethod::new(
        STypeCompanion::Context,
        SELF_BOX_INDEX_PROPERTY_METHOD_DESC.clone()
    );
}

pub const LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_ID: MethodId = MethodId(9);
lazy_static! {
    static ref LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "LastBlockUtxoRootHash",
        SAvlTree,
        LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_ID
    );
}
lazy_static! {
    pub static ref LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY: SMethod = SMethod::new(
        STypeCompanion::Context,
        LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_DESC.clone()
    );
}

pub const MINER_PUBKEY_PROPERTY_METHOD_ID: MethodId = MethodId(10);
lazy_static! {
    static ref MINER_PUBKEY_PROPERTY_METHOD_DESC: SMethodDesc = property(
        "minerPubKey",
        SColl(SByte.into()),
        MINER_PUBKEY_PROPERTY_METHOD_ID
    );
}
lazy_static! {
    pub static ref MINER_PUBKEY_PROPERTY: SMethod = SMethod::new(
        STypeCompanion::Context,
        MINER_PUBKEY_PROPERTY_METHOD_DESC.clone()
    );
}

fn property(name: &'static str, res_tpe: SType, id: MethodId) -> SMethodDesc {
    SMethodDesc::property(SType::SContext, name, res_tpe, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(
            SMethod::from_ids(TYPE_CODE, DATA_INPUTS_PROPERTY_METHOD_ID).map(|e| e.name())
                == Ok("dataInputs")
        );
    }
}
