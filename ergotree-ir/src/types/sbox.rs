use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::types::TypeCode;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stuple::STuple;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_param::STypeVar;
use lazy_static::lazy_static;

/// SBox type code
pub const TYPE_CODE: TypeCode = TypeCode::SBOX;
/// SBox type name
pub static TYPE_NAME: &str = "Box";
/// Box.value property
pub const VALUE_METHOD_ID: MethodId = MethodId(1);
/// Box.propositionBytes property (JVM `PropositionBytesMethod`, op-form `ExtractScriptBytes`)
pub const PROPOSITION_BYTES_METHOD_ID: MethodId = MethodId(2);
/// Box.bytes property (JVM `BytesMethod`, op-form `ExtractBytes`)
pub const BYTES_METHOD_ID: MethodId = MethodId(3);
/// Box.bytesWithoutRef property (JVM `BytesWithoutRefMethod`, op-form `ExtractBytesWithNoRef`)
pub const BYTES_WITHOUT_REF_METHOD_ID: MethodId = MethodId(4);
/// Box.id property (JVM `IdMethod`, op-form `ExtractId`)
pub const ID_METHOD_ID: MethodId = MethodId(5);
/// Box.creationInfo property (JVM `creationInfoMethod`, op-form `ExtractCreationInfo`)
pub const CREATION_INFO_METHOD_ID: MethodId = MethodId(6);
/// Box.Rx property
pub const GET_REG_METHOD_ID: MethodId = MethodId(7);
/// Box.tokens property
pub const TOKENS_METHOD_ID: MethodId = MethodId(8);

lazy_static! {
    /// Box method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![
            GET_REG_METHOD_DESC.clone(),
            VALUE_METHOD_DESC.clone(),
            PROPOSITION_BYTES_METHOD_DESC.clone(),
            BYTES_METHOD_DESC.clone(),
            BYTES_WITHOUT_REF_METHOD_DESC.clone(),
            ID_METHOD_DESC.clone(),
            CREATION_INFO_METHOD_DESC.clone(),
            TOKENS_METHOD_DESC.clone()
        ]
    ;
}

lazy_static! {
    static ref VALUE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: VALUE_METHOD_ID,
        name: "value",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SLong),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.value
    pub static ref VALUE_METHOD: SMethod = SMethod::new(STypeCompanion::Box, VALUE_METHOD_DESC.clone(),);
}

// The byte-array accessor method-forms (propositionBytes / bytes / bytesWithoutRef / id)
// mirror the JVM `commonBoxMethods` entries (methods.scala, SBoxMethods): each is a zero-arg
// PropertyCall `SFunc(SBox, Coll[Byte])`, present from v5Methods with NO version gate, evaluated
// JVM-side via `MethodCall.eval`'s `invokeFixed` reflection over the op-form's `costKind`.
lazy_static! {
    static ref PROPOSITION_BYTES_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: PROPOSITION_BYTES_METHOD_ID,
        name: "propositionBytes",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SColl(SType::SByte.into())),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.propositionBytes
    pub static ref PROPOSITION_BYTES_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, PROPOSITION_BYTES_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref BYTES_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: BYTES_METHOD_ID,
        name: "bytes",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SColl(SType::SByte.into())),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.bytes
    pub static ref BYTES_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, BYTES_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref BYTES_WITHOUT_REF_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: BYTES_WITHOUT_REF_METHOD_ID,
        name: "bytesWithoutRef",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SColl(SType::SByte.into())),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.bytesWithoutRef
    pub static ref BYTES_WITHOUT_REF_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, BYTES_WITHOUT_REF_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref ID_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: ID_METHOD_ID,
        name: "id",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::SColl(SType::SByte.into())),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.id
    pub static ref ID_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, ID_METHOD_DESC.clone(),);
}

// creationInfo returns `(Int, Coll[Byte])` — the tx block height paired with the serialized
// transaction id followed by the box's output index (JVM `ExtractCreationInfo.OpType`).
lazy_static! {
    static ref CREATION_INFO_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: CREATION_INFO_METHOD_ID,
        name: "creationInfo",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: Box::new(SType::STuple(STuple::pair(
                SType::SInt,
                SType::SColl(SType::SByte.into())
            ))),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.creationInfo
    pub static ref CREATION_INFO_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, CREATION_INFO_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref GET_REG_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_REG_METHOD_ID,
        name: "getReg",
        tpe: SFunc {
            t_dom: vec![SType::SBox, SType::SInt],
            t_range: SType::SOption(Arc::new(STypeVar::t().into())).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.getReg
    pub static ref GET_REG_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, GET_REG_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref TOKENS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TOKENS_METHOD_ID,
        name: "tokens",
        tpe: SFunc {
            t_dom: vec![SType::SBox],
            t_range: SType::SColl(Arc::new(
                    STuple::pair(
                        SType::SColl(SType::SByte.into()),
                        SType::SLong
                    ).into())).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
    /// Box.tokens
    pub static ref TOKENS_METHOD: SMethod =
        SMethod::new( STypeCompanion::Box,TOKENS_METHOD_DESC.clone(),);
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::{
        mir::{constant::Constant, global_vars::GlobalVars, method_call::MethodCall},
        serialization::SigmaSerializable,
    };

    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_CODE, VALUE_METHOD_ID).map(|e| e.name()) == Ok("value"));
        assert!(
            SMethod::from_ids(TYPE_CODE, PROPOSITION_BYTES_METHOD_ID).map(|e| e.name())
                == Ok("propositionBytes")
        );
        assert!(SMethod::from_ids(TYPE_CODE, BYTES_METHOD_ID).map(|e| e.name()) == Ok("bytes"));
        assert!(
            SMethod::from_ids(TYPE_CODE, BYTES_WITHOUT_REF_METHOD_ID).map(|e| e.name())
                == Ok("bytesWithoutRef")
        );
        assert!(SMethod::from_ids(TYPE_CODE, ID_METHOD_ID).map(|e| e.name()) == Ok("id"));
        assert!(
            SMethod::from_ids(TYPE_CODE, CREATION_INFO_METHOD_ID).map(|e| e.name())
                == Ok("creationInfo")
        );
        assert!(SMethod::from_ids(TYPE_CODE, GET_REG_METHOD_ID).map(|e| e.name()) == Ok("getReg"));
        assert!(SMethod::from_ids(TYPE_CODE, TOKENS_METHOD_ID).map(|e| e.name()) == Ok("tokens"));
    }

    #[test]
    fn test_getreg_serialization_roundtrip() {
        let type_args = core::iter::once((STypeVar::t(), SType::SInt)).collect();
        let mc = MethodCall::with_type_args(
            GlobalVars::SelfBox.into(),
            GET_REG_METHOD.clone().with_concrete_types(&type_args),
            vec![Constant::from(4i32).into()],
            type_args,
        )
        .unwrap();
        assert_eq!(
            MethodCall::sigma_parse_bytes(&mc.sigma_serialize_bytes().unwrap()).unwrap(),
            mc
        );
    }
}
