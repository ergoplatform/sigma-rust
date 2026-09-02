use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::types::TypeCode;

use super::sfunc::SFunc;
use super::smethod::MethodId;
use super::smethod::SMethodDesc;
use super::stype::SType;
use crate::types::smethod::SMethod;
use crate::types::stype_companion::STypeCompanion;
use crate::types::stype_param::STypeVar;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;

/// SGlobal type code
pub const TYPE_CODE: TypeCode = TypeCode::SGLOBAL;
/// SGlobal type name
pub static TYPE_NAME: &str = "Global";

/// groupGenerator property
pub const GROUP_GENERATOR_METHOD_ID: MethodId = MethodId(1);
/// "xor" predefined function
pub const XOR_METHOD_ID: MethodId = MethodId(2);
/// serialize function added in v6.0
pub const SERIALIZE_METHOD_ID: MethodId = MethodId(3);
/// "fromBigEndianBytes" predefined function
pub const DESERIALIZE_METHOD_ID: MethodId = MethodId(4);
/// "fromBigEndianBytes" predefined function
pub const FROM_BIGENDIAN_BYTES_METHOD_ID: MethodId = MethodId(5);
/// encodeNBits method id (v6.0)
pub const ENCODE_NBITS_METHOD_ID: MethodId = MethodId(6);
/// decodeNBits method id (v6.0)
pub const DECODE_NBITS_METHOD_ID: MethodId = MethodId(7);
/// Global.powHit function
pub const POW_HIT_METHOD_ID: MethodId = MethodId(8);
/// "some" property
pub const SOME_METHOD_ID: MethodId = MethodId(9);
/// "none" property
pub const NONE_METHOD_ID: MethodId = MethodId(10);

lazy_static! {
    /// Global method descriptors
    pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> =
        vec![GROUP_GENERATOR_METHOD_DESC.clone(), XOR_METHOD_DESC.clone(), SERIALIZE_METHOD_DESC.clone(), DESERIALIZE_METHOD_DESC.clone(), FROM_BIGENDIAN_BYTES_METHOD_DESC.clone(), ENCODE_NBITS_METHOD_DESC.clone(), DECODE_NBITS_METHOD_DESC.clone(), NONE_METHOD_DESC.clone(), SOME_METHOD_DESC.clone(), POW_HIT_METHOD_DESC.clone()];
}

lazy_static! {
    static ref GROUP_GENERATOR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: GROUP_GENERATOR_METHOD_ID,
        name: "groupGenerator",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal],
            t_range: SType::SGroupElement.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
     /// GLOBAL.GroupGenerator
    pub static ref GROUP_GENERATOR_METHOD: SMethod = SMethod::new(STypeCompanion::Global, GROUP_GENERATOR_METHOD_DESC.clone(),);

}

lazy_static! {
    static ref XOR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: XOR_METHOD_ID,
        name: "xor",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                SType::SColl(SType::SByte.into()),
                SType::SColl(SType::SByte.into()),
            ],
            t_range: SType::SColl(SType::SByte.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V0
    };
     /// GLOBAL.xor
    pub static ref XOR_METHOD: SMethod = SMethod::new(STypeCompanion::Global, XOR_METHOD_DESC.clone(),);

}

lazy_static! {
    static ref FROM_BIGENDIAN_BYTES_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: FROM_BIGENDIAN_BYTES_METHOD_ID,
        name: "fromBigEndianBytes",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal, SType::SColl(SType::SByte.into())],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.fromBigEndianBytes
    pub static ref FROM_BIGENDIAN_BYTES_METHOD: SMethod = SMethod::new(STypeCompanion::Global, FROM_BIGENDIAN_BYTES_METHOD_DESC.clone(),);

    static ref ENCODE_NBITS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: ENCODE_NBITS_METHOD_ID,
        name: "encodeNBits",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                SType::SBigInt,
            ],
            t_range: SType::SLong.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.encodeNBits
    pub static ref ENCODE_NBITS_METHOD: SMethod = SMethod::new(STypeCompanion::Global, ENCODE_NBITS_METHOD_DESC.clone());

    static ref DECODE_NBITS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: DECODE_NBITS_METHOD_ID,
        name: "decodeNBits",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                SType::SLong
            ],
            t_range: SType::SBigInt.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.decodeNBits
    pub static ref DECODE_NBITS_METHOD: SMethod = SMethod::new(STypeCompanion::Global, DECODE_NBITS_METHOD_DESC.clone());

    static ref DESERIALIZE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: DESERIALIZE_METHOD_ID,
        name: "deserialize",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                SType::SColl(SType::SByte.into())
            ],
            t_range: Box::new(STypeVar::t().into()),
            tpe_params: vec![],
        },
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.deserialize
    pub static ref DESERIALIZE_METHOD: SMethod = SMethod::new(STypeCompanion::Global, DESERIALIZE_METHOD_DESC.clone(),);

    static ref SERIALIZE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: SERIALIZE_METHOD_ID,
        name: "serialize",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                STypeVar::t().into()
            ],
            t_range: SType::SColl(SType::SByte.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.serialize
    pub static ref SERIALIZE_METHOD: SMethod = SMethod::new(STypeCompanion::Global, SERIALIZE_METHOD_DESC.clone(),);

    static ref POW_HIT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: POW_HIT_METHOD_ID,
        name: "powHit",
        tpe: SFunc {
            t_dom: vec![
                SType::SGlobal,
                SType::SInt,
                SType::SColl(SType::SByte.into()),
                SType::SColl(SType::SByte.into()),
                SType::SColl(SType::SByte.into()),
                SType::SInt,
            ],
            // powHit computes the Autolykos-2 PoW hit value (a big integer), not a
            // boolean. Scala `SGlobalMethods.powHit` returns `SUnsignedBigInt`
            // (methods.scala) and `powHit_eval` yields an `UnsignedBigInt`; the
            // interpreter `POW_HIT_EVAL_FN` already produces `Value::UnsignedBigInt`.
            // Mis-typing this as `SBoolean` made `Coll[..].map(powHit).exists(UnsignedBigInt => Boolean)`
            // fail parse-time type-checking, wedging testnet block 28,474.
            t_range: SType::SUnsignedBigInt.into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// Global.powHit
    pub static ref POW_HIT_METHOD: SMethod = SMethod::new(STypeCompanion::Global, POW_HIT_METHOD_DESC.clone());
}

lazy_static! {
    static ref SOME_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: SOME_METHOD_ID,
        name: "some",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal, SType::STypeVar(STypeVar::t())],
            t_range:SType::SOption(SType::STypeVar(STypeVar::t()).into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.some
    pub static ref SOME_METHOD : SMethod = SMethod::new(STypeCompanion::Global, SOME_METHOD_DESC.clone(),);
}

lazy_static! {
    static ref NONE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: NONE_METHOD_ID,
        name: "none",
        tpe: SFunc {
            t_dom: vec![SType::SGlobal],
            t_range:SType::SOption(SType::STypeVar(STypeVar::t()).into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: vec![STypeVar::t()],
        min_version: ErgoTreeVersion::V3
    };
    /// GLOBAL.none
    pub static ref NONE_METHOD : SMethod = SMethod::new(STypeCompanion::Global, NONE_METHOD_DESC.clone(),);
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod test {
    use proptest::prelude::*;

    use crate::{
        bigint256::BigInt256,
        ergo_tree::ErgoTreeVersion,
        mir::{constant::Constant, expr::Expr, method_call::MethodCall},
        serialization::roundtrip_new_feature,
        types::{sglobal::POW_HIT_METHOD, stype::SType, stype_param::STypeVar},
    };

    use super::{DECODE_NBITS_METHOD, DESERIALIZE_METHOD, ENCODE_NBITS_METHOD};
    use crate::ergo_tree::ErgoTree;
    use crate::serialization::SigmaSerializable;
    proptest! {
       #[test]
       fn test_deserialize_method_roundtrip(v in any::<SType>()) {
           let type_args = core::iter::once((STypeVar::t(), v)).collect();
           let mc = MethodCall::with_type_args(
               Expr::Global,
               DESERIALIZE_METHOD.clone(),
               vec![vec![0i8].into()],
               type_args,
           ).unwrap();
           roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
       }
       #[test]
       fn pow_hit_roundtrip(k in any::<i32>(), msg in any::<Vec<u8>>(), nonce in any::<Vec<u8>>(), h in any::<Vec<u8>>(), big_n: u32) {
           let mc = MethodCall::new(Expr::Global, POW_HIT_METHOD.clone(), vec![Constant::from(k).into(), Constant::from(msg).into(), Constant::from(nonce).into(), Constant::from(h).into(), Constant::from(big_n as i32).into()]).unwrap();
           roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
       }
    }

    #[test]
    fn encode_nbits_method_roundtrip() {
        let mc = MethodCall::new(
            Expr::Global,
            ENCODE_NBITS_METHOD.clone(),
            vec![BigInt256::from(1i64).into()],
        )
        .unwrap();
        roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
    }

    #[test]
    fn decode_nbits_method_roundtrip() {
        let mc =
            MethodCall::new(Expr::Global, DECODE_NBITS_METHOD.clone(), vec![1i64.into()]).unwrap();
        roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
    }

    /// Regression: `Global.powHit` returns `SUnsignedBigInt` (the Autolykos-2 hit
    /// value), not `SBoolean`. Mirrors Scala `SGlobalMethods.powHit` and the
    /// interpreter's `POW_HIT_EVAL_FN` (which yields `Value::UnsignedBigInt`).
    #[test]
    fn powhit_returns_unsigned_bigint() {
        assert_eq!(
            POW_HIT_METHOD.tpe().t_range.as_ref(),
            &SType::SUnsignedBigInt
        );
    }

    /// Real-block regression: testnet block 28,474 tx[4] input[1], spending box
    /// `1d746ebe5da0a0df46de9c34c60c5ed642b07fbff7c4cbf46dc93b1cd4a95166`. An
    /// Autolykos-2 verify of the form
    /// `coll.map(x => Global.powHit(..)).exists((u: UnsignedBigInt) => u > ..)`.
    /// With `powHit` mis-typed `SBoolean`, the mapped collection resolved to
    /// `Coll[Boolean]` and `Exists::new` rejected the `UnsignedBigInt => Boolean`
    /// predicate ("Invalid condition tpe"), wedging the node off testnet here.
    /// The JVM (6.0.3) accepts this canonical block. The header carries the size
    /// flag, so the body type error surfaces at `proposition()`, not parse.
    #[test]
    fn powhit_coll_hof_tree_parses() {
        let tree_hex = "1bb8042204000200040004100400041004000402040404060408040a040c040e041004120400040804b803040204400442040804c003040004080440040804ee020400040806207fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffed04a09c010402d807d601b2a4730000d602e4c672010464d6038301027301d604e5e3000e7203d605e4dc640a7202027204e5e3010e7203d606d901060e7cb4720673027303d607b2a5730400d1edaeaddad901080ed801d60ab472087305b172088cb0830a047306730773087309730a730b730c730d730e730f860283000e7310d9010b4c4c1ad804d60d8c720b01d60e8c720d02d60f9a9a720e73117312d6109a9a9a720f731373149c7eb2720a720f000473158602b38c720d0183010eb4720a720e7210721001017205d901080ed801d60adad9010a0ed801d60cdad9010c0edc6a04dd01b4720c731673176801720a8602db680d720cb47a7edb6809720c0573187319017208dc6a08dd05731adad9010b0ecbb4720b731b731c0172088c720a018c720a02dad9010b0edc6a05dd01b4720b731d731e04017208d90108099172089ddad9010a0edad9010c099ddb060e731f720c01db060e7eda720601720a060172057e7320099683060193c17207c1720193db6401e4dc640e72020283010e7204e5e3020e7203db6401e4c67207046493e4c67207050499e4c672010504732193e4c67207060699e4c6720106067eda72060172050693e4c672070705e4c67201070593e4c672070805e4c672010805";
        let bytes = base16::decode(tree_hex).unwrap();
        let tree = ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        tree.proposition().unwrap();
    }
}
