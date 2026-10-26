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
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V0
    };
    /// Box.value
    pub static ref VALUE_METHOD: SMethod = SMethod::new(STypeCompanion::Box, VALUE_METHOD_DESC.clone(),);
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
        explicit_type_args: vec![STypeVar::t()].into_boxed_slice(),
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
        explicit_type_args: alloc::boxed::Box::default(),
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
