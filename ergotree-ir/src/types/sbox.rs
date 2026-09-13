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
/// Box.getReg[T](index) — the v6 dynamic register access method (JVM `getRegMethodV6`)
pub const GET_REG_METHOD_ID: MethodId = MethodId(19);
/// Box.getRegV5 — the legacy v5-era method id. Mirrors JVM `getRegMethodV5`: trees
/// carrying it deserialize at any version, but there is no eval for it (on the JVM
/// its reflective lookup fails), so a live occurrence errors at evaluation.
pub const GET_REG_V5_METHOD_ID: MethodId = MethodId(7);
/// Box.tokens property
pub const TOKENS_METHOD_ID: MethodId = MethodId(8);

lazy_static! {
    /// Box method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![
            GET_REG_METHOD_DESC.clone(),
            GET_REG_V5_METHOD_DESC.clone(),
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
        explicit_type_args: vec![],
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
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V3
    };
    /// Box.getReg
    pub static ref GET_REG_METHOD: SMethod =
        SMethod::new(STypeCompanion::Box, GET_REG_METHOD_DESC.clone(),);
}

lazy_static! {
    // Unlike getReg, getRegV5 carries no explicit type args on the wire
    // (JVM getRegMethodV5 has no `Seq(tT)`), so its serialized form ends
    // after the args. The unresolved T in t_range is fine: the node only
    // ever deserializes (dead-branch occurrences) and is rejected at eval.
    static ref GET_REG_V5_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GET_REG_V5_METHOD_ID,
        name: "getRegV5",
        tpe: SFunc {
            t_dom: vec![SType::SBox, SType::SInt],
            t_range: SType::SOption(Arc::new(STypeVar::t().into())).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
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
#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use crate::{
        mir::{constant::Constant, expr::Expr, global_vars::GlobalVars, method_call::MethodCall},
        serialization::{roundtrip_new_feature, SigmaSerializable},
    };

    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(SMethod::from_ids(TYPE_CODE, VALUE_METHOD_ID).map(|e| e.name()) == Ok("value"));
        assert!(SMethod::from_ids(TYPE_CODE, GET_REG_METHOD_ID).map(|e| e.name()) == Ok("getReg"));
        assert!(
            SMethod::from_ids(TYPE_CODE, GET_REG_V5_METHOD_ID).map(|e| e.name()) == Ok("getRegV5")
        );
        assert!(SMethod::from_ids(TYPE_CODE, TOKENS_METHOD_ID).map(|e| e.name()) == Ok("tokens"));
    }

    // getReg is v6-only (method id 19): rejected when parsing v0-v2 trees,
    // round-trips from v3 on — mirroring JVM SBoxMethods.v6Methods gating.
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
        roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
    }

    // getRegV5 (method id 7) deserializes at any version and carries NO trailing
    // explicit-type-arg byte — the wire shape JVM getRegMethodV5 produces. Byte-exact
    // roundtrip guards both directions. Bytes are the root expr of the blessed
    // adversarial vector `{ SELF.getRegV5(getVar[Int](1).get) }`:
    // dc=MethodCall 63=SBox 07=methodId a7=SELF 01=argc e4=OptionGet e3 01 04=getVar[Int](1)
    #[test]
    fn test_getregv5_parses_without_type_args() {
        let bytes: Vec<u8> = vec![0xdc, 0x63, 0x07, 0xa7, 0x01, 0xe4, 0xe3, 0x01, 0x04];
        let expr = Expr::sigma_parse_bytes(&bytes).unwrap();
        let Expr::MethodCall(mc) = &expr else {
            panic!("expected MethodCall, got {:?}", expr)
        };
        assert_eq!(mc.expr.method.method_id(), GET_REG_V5_METHOD_ID);
        assert_eq!(mc.expr.method.name(), "getRegV5");
        assert!(mc.expr.explicit_type_args.is_empty());
        assert_eq!(expr.sigma_serialize_bytes().unwrap(), bytes);
    }
}
