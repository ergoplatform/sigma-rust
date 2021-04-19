#![allow(missing_docs)]

use crate::serialization::types::TypeCode;

use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype::SType::{SAvlTree, SBox, SByte, SColl, SHeader, SInt, SPreHeader};
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use lazy_static::lazy_static;

pub const TYPE_ID: TypeCode = TypeCode::SCONTEXT;

static S_CONTEXT_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Context",
};

lazy_static! {
    pub static ref S_CONTEXT_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_CONTEXT_TYPE_COMPANION_HEAD,
        vec![
            &DATA_INPUTS_PROPERTY_METHOD_DESC,
            &HEADERS_PROPERTY_METHOD_DESC,
            &PRE_HEADERS_PROPERTY_METHOD_DESC,
            &INPUTS_PROPERTY_METHOD_DESC,
            &OUTPUTS_PROPERTY_METHOD_DESC,
            &HEIGHT_PROPERTY_METHOD_DESC,
            &SELF_PROPERTY_METHOD_DESC,
            &SELF_BOX_INDEX_PROPERTY_METHOD_DESC,
            &LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_DESC,
            &MINER_PUBKEY_PROPERTY_METHOD_DESC,
        ]
    );
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
        &S_CONTEXT_TYPE_COMPANION,
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
        &S_CONTEXT_TYPE_COMPANION,
        HEADERS_PROPERTY_METHOD_DESC.clone()
    );
}

pub const PRE_HEADERS_PROPERTY_METHOD_ID: MethodId = MethodId(3);
lazy_static! {
    static ref PRE_HEADERS_PROPERTY_METHOD_DESC: SMethodDesc =
        property("preHeader", SPreHeader, PRE_HEADERS_PROPERTY_METHOD_ID);
}
lazy_static! {
    pub static ref PRE_HEADERS_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
        PRE_HEADERS_PROPERTY_METHOD_DESC.clone()
    );
}

pub const INPUTS_PROPERTY_METHOD_ID: MethodId = MethodId(4);
lazy_static! {
    static ref INPUTS_PROPERTY_METHOD_DESC: SMethodDesc =
        property("INPUTS", SColl(SBox.into()), INPUTS_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref INPUTS_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
        INPUTS_PROPERTY_METHOD_DESC.clone()
    );
}

pub const OUTPUTS_PROPERTY_METHOD_ID: MethodId = MethodId(5);
lazy_static! {
    static ref OUTPUTS_PROPERTY_METHOD_DESC: SMethodDesc =
        property("OUTPUTS", SColl(SBox.into()), OUTPUTS_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref OUTPUTS_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
        OUTPUTS_PROPERTY_METHOD_DESC.clone()
    );
}

pub const HEIGHT_PROPERTY_METHOD_ID: MethodId = MethodId(6);
lazy_static! {
    static ref HEIGHT_PROPERTY_METHOD_DESC: SMethodDesc =
        property("HEIGHT", SInt, HEIGHT_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref HEIGHT_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
        HEIGHT_PROPERTY_METHOD_DESC.clone()
    );
}

pub const SELF_PROPERTY_METHOD_ID: MethodId = MethodId(7);
lazy_static! {
    static ref SELF_PROPERTY_METHOD_DESC: SMethodDesc =
        property("SELF", SBox, SELF_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref SELF_PROPERTY: SMethod =
        SMethod::new(&S_CONTEXT_TYPE_COMPANION, SELF_PROPERTY_METHOD_DESC.clone());
}

pub const SELF_BOX_INDEX_PROPERTY_METHOD_ID: MethodId = MethodId(8);
lazy_static! {
    static ref SELF_BOX_INDEX_PROPERTY_METHOD_DESC: SMethodDesc =
        property("selfBoxIndex", SInt, SELF_BOX_INDEX_PROPERTY_METHOD_ID);
}
lazy_static! {
    static ref SELF_BOX_INDEX_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
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
    static ref LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
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
    static ref MINER_PUBKEY_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
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
            SMethod::from_ids(TYPE_ID, DATA_INPUTS_PROPERTY_METHOD_ID).map(|e| e.name())
                == Ok("dataInputs")
        );
    }
}
