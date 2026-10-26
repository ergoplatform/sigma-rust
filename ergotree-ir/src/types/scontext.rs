#![allow(missing_docs)]

use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::types::TypeCode;
use crate::types::sfunc::SFunc;
use crate::types::stype_companion::STypeCompanion;
use crate::types::stype_param::STypeVar;

use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype::SType::{SAvlTree, SBox, SByte, SColl, SHeader, SInt, SPreHeader};
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;

/// SContext type code
pub const TYPE_CODE: TypeCode = TypeCode::SCONTEXT;
/// SContext type name
pub static TYPE_NAME: &str = "Context";

lazy_static! {
    /// Context method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = vec![
        DATA_INPUTS_PROPERTY_METHOD_DESC.clone(),
        HEADERS_PROPERTY_METHOD_DESC.clone(),
        PRE_HEADER_PROPERTY_METHOD_DESC.clone(),
        INPUTS_PROPERTY_METHOD_DESC.clone(),
        OUTPUTS_PROPERTY_METHOD_DESC.clone(),
        HEIGHT_PROPERTY_METHOD_DESC.clone(),
        SELF_PROPERTY_METHOD_DESC.clone(),
        SELF_BOX_INDEX_PROPERTY_METHOD_DESC.clone(),
        LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY_METHOD_DESC.clone(),
        MINER_PUBKEY_PROPERTY_METHOD_DESC.clone(),
        GET_VAR_V5_METHOD_DESC.clone(),
        GET_VAR_FROM_INPUT_METHOD_DESC.clone()
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

pub const GET_VAR_V5_METHOD_ID: MethodId = MethodId(11);
lazy_static! {
    // This method has no implementation because of lack of explicit type args making it impossible to substitute output type
    // Missing methods are a soft-forkable error and can be tolerated, whereas the reference implementation rejects scripts containing this due to lack of type args
    pub static ref GET_VAR_V5_METHOD_DESC: SMethodDesc = SMethodDesc {
        name: "getVar",
        method_id: GET_VAR_V5_METHOD_ID,
        tpe: SFunc {
            t_dom: vec![SType::SContext, SType::SByte],
            t_range: SType::SOption(SType::STypeVar(STypeVar::t()).into()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
}

pub const GET_VAR_FROM_INPUT_METHOD_ID: MethodId = MethodId(12);
lazy_static! {
    pub static ref GET_VAR_FROM_INPUT_METHOD_DESC: SMethodDesc = SMethodDesc {
        name: "getVarFromInput",
        method_id: GET_VAR_FROM_INPUT_METHOD_ID,
        tpe: SFunc {
            t_dom: vec![SType::SContext, SType::SShort, SType::SByte],
            t_range: SType::SOption(SType::STypeVar(STypeVar::t()).into()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: vec![STypeVar::t()].into_boxed_slice(),
        min_version: ErgoTreeVersion::V3
    };
    pub static ref GET_VAR_FROM_INPUT_METHOD: SMethod = SMethod::new(
        STypeCompanion::Context,
        GET_VAR_FROM_INPUT_METHOD_DESC.clone()
    );
}

fn property(name: &'static str, res_tpe: SType, id: MethodId) -> SMethodDesc {
    SMethodDesc::property(SType::SContext, name, res_tpe, id)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{
        mir::{expr::Expr, method_call::MethodCall},
        serialization::roundtrip_new_feature,
    };

    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(
            SMethod::from_ids(TYPE_CODE, DATA_INPUTS_PROPERTY_METHOD_ID).map(|e| e.name())
                == Ok("dataInputs")
        );
    }

    #[test]
    fn roundtrip_get_var_from_input() {
        let mc = MethodCall::with_type_args(
            Expr::Context,
            GET_VAR_FROM_INPUT_METHOD.clone(),
            vec![0i16.into(), 0i8.into()],
            [(STypeVar::t(), SType::SInt)].into_iter().collect(),
        )
        .unwrap();
        roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
    }
}
