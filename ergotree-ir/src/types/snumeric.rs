use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{
    ergo_tree::ErgoTreeVersion,
    types::{sfunc::SFunc, stype::SType, stype_param::STypeVar},
};

use super::smethod::{MethodId, SMethodDesc};

/// toByte MethodId
pub const TO_BYTE_METHOD_ID: MethodId = MethodId(1);
/// toShort MethodId
pub const TO_SHORT_METHOD_ID: MethodId = MethodId(2);
/// toInt MethodId
pub const TO_INT_METHOD_ID: MethodId = MethodId(3);
/// toLong MethodId
pub const TO_LONG_METHOD_ID: MethodId = MethodId(4);
/// toBigInt MethodId
pub const TO_BIGINT_METHOD_ID: MethodId = MethodId(5);
/// toBytes MethodId
pub const TO_BYTES_METHOD_ID: MethodId = MethodId(6);
/// toBits MethodId
pub const TO_BITS_METHOD_ID: MethodId = MethodId(7);
/// bitwiseInverse MethodId
pub const BITWISE_INVERSE_METHOD_ID: MethodId = MethodId(8);
/// bitwiseOr MethodId
pub const BITWISE_OR_METHOD_ID: MethodId = MethodId(9);
/// bitwiseAnd MethodId
pub const BITWISE_AND_METHOD_ID: MethodId = MethodId(10);
/// bitwiseXor MethodId
pub const BITWISE_XOR_METHOD_ID: MethodId = MethodId(11);
/// shiftLeft MethodId
pub const SHIFT_LEFT_METHOD_ID: MethodId = MethodId(12);
/// shiftRight MethodId
pub const SHIFT_RIGHT_METHOD_ID: MethodId = MethodId(13);

// The following methods (toByte, toShort, to...) do not seem to have any implementation in upstream (only methodcall is defined). Since they are used in v6 they are defined here so the methodcalls can be deserialized
lazy_static! {
    static ref TO_BYTE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BYTE_METHOD_ID,
        name: "toByte",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SByte.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_SHORT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_SHORT_METHOD_ID,
        name: "toShort",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SShort.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_INT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_INT_METHOD_ID,
        name: "toInt",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SInt.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_LONG_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_LONG_METHOD_ID,
        name: "toLong",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SLong.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_BIGINT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BIGINT_METHOD_ID,
        name: "toBigInt",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SBigInt.into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_BYTES_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BYTES_METHOD_ID,
        name: "toBytes",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SColl(SType::SByte.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref TO_BITS_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: TO_BITS_METHOD_ID,
        name: "toBits",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::SColl(SType::SBoolean.into()).into(),
            tpe_params: vec![],
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref BITWISE_INVERSE_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: BITWISE_INVERSE_METHOD_ID,
        name: "bitwiseInverse",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t())],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref BITWISE_OR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: BITWISE_OR_METHOD_ID,
        name: "bitwiseOr",
        tpe: SFunc {
            t_dom: vec![
                SType::STypeVar(STypeVar::t()),
                SType::STypeVar(STypeVar::t())
            ],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref BITWISE_AND_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: BITWISE_AND_METHOD_ID,
        name: "bitwiseAnd",
        tpe: SFunc {
            t_dom: vec![
                SType::STypeVar(STypeVar::t()),
                SType::STypeVar(STypeVar::t())
            ],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref BITWISE_XOR_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: BITWISE_XOR_METHOD_ID,
        name: "bitwiseXor",
        tpe: SFunc {
            t_dom: vec![
                SType::STypeVar(STypeVar::t()),
                SType::STypeVar(STypeVar::t())
            ],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref SHIFT_LEFT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: SHIFT_LEFT_METHOD_ID,
        name: "shiftLeft",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t()), SType::SInt],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref SHIFT_RIGHT_METHOD_DESC: SMethodDesc = SMethodDesc {
        method_id: SHIFT_RIGHT_METHOD_ID,
        name: "shiftRight",
        tpe: SFunc {
            t_dom: vec![SType::STypeVar(STypeVar::t()), SType::SInt],
            t_range: SType::STypeVar(STypeVar::t()).into(),
            tpe_params: vec![]
        },
        explicit_type_args: alloc::boxed::Box::default(),
        min_version: ErgoTreeVersion::V3
    };
    static ref METHOD_DESC: [&'static SMethodDesc; 12] = [
        &TO_BYTE_METHOD_DESC,
        &TO_SHORT_METHOD_DESC,
        &TO_INT_METHOD_DESC,
        &TO_LONG_METHOD_DESC,
        &TO_BYTES_METHOD_DESC,
        &TO_BITS_METHOD_DESC,
        &BITWISE_INVERSE_METHOD_DESC,
        &BITWISE_OR_METHOD_DESC,
        &BITWISE_AND_METHOD_DESC,
        &BITWISE_XOR_METHOD_DESC,
        &SHIFT_LEFT_METHOD_DESC,
        &SHIFT_RIGHT_METHOD_DESC,
    ];
}

fn specialize_method(method: &SMethodDesc, tpe: SType) -> SMethodDesc {
    SMethodDesc {
        tpe: method
            .tpe
            .with_subst(&[(STypeVar::t(), tpe)].into_iter().collect()),
        ..method.clone()
    }
}

/// SByte type and methods
pub mod sbyte {
    use super::*;
    use crate::{
        serialization::types::TypeCode,
        types::{smethod::SMethod, stype_companion::STypeCompanion},
    };

    /// Byte TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SBYTE;
    /// Byte name
    pub const TYPE_NAME: &str = "Byte";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SByte))
            .collect();
        /// Byte methods
        pub static ref METHODS: Vec<SMethod> = METHOD_DESC
            .iter()
            .map(|desc| SMethod::new(STypeCompanion::SByte, desc.clone()))
            .collect();
    }
}

/// SShort type and methods
pub mod sshort {
    use super::*;
    use crate::{
        serialization::types::TypeCode,
        types::{smethod::SMethod, stype_companion::STypeCompanion},
    };

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SSHORT;
    /// Short Type name
    pub const TYPE_NAME: &str = "Short";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SShort))
            .collect();
        /// Short methods
        pub static ref METHODS: Vec<SMethod> = METHOD_DESC
            .iter()
            .map(|desc| SMethod::new(STypeCompanion::SShort, desc.clone()))
            .collect();
    }
}

/// SInt type and methods
pub mod sint {
    use super::*;
    use crate::{
        serialization::types::TypeCode,
        types::{smethod::SMethod, stype_companion::STypeCompanion},
    };

    /// Int TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SINT;
    /// Int Type name
    pub const TYPE_NAME: &str = "Int";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SInt))
            .collect();
        /// Int methods
        pub static ref METHODS: Vec<SMethod> = METHOD_DESC
            .iter()
            .map(|desc| SMethod::new(STypeCompanion::SInt, desc.clone()))
            .collect();
    }
}

/// SLong type and methods
pub mod slong {
    use super::*;
    use crate::{
        serialization::types::TypeCode,
        types::{smethod::SMethod, stype_companion::STypeCompanion},
    };

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SLONG;
    /// Short Type name
    pub const TYPE_NAME: &str = "Long";

    lazy_static! {
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SLong))
            .collect();
        /// Long methods
        pub static ref METHODS: Vec<SMethod> = METHOD_DESC
            .iter()
            .map(|desc| SMethod::new(STypeCompanion::SLong, desc.clone()))
            .collect();
    }
}

/// SBigInt type and methods
pub mod sbigint {
    use super::*;
    use crate::{
        serialization::types::TypeCode,
        types::{smethod::SMethod, stype_companion::STypeCompanion},
    };

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SBIGINT;
    /// Short Type name
    pub const TYPE_NAME: &str = "BigInt";

    /// BigInt.toUnsigned method id
    pub const TO_UNSIGNED_METHOD_ID: MethodId = MethodId(14);
    /// BigInt.toUnsignedMod method id
    pub const TO_UNSIGNED_MOD_METHOD_ID: MethodId = MethodId(15);

    lazy_static! {
        /// toUnsigned method descriptor
        pub static ref TO_UNSIGNED_METHOD_DESC: SMethodDesc = SMethodDesc {
            method_id: TO_UNSIGNED_METHOD_ID,
            name: "toUnsigned",
            tpe: SFunc {
                t_dom: vec![SType::SBigInt],
                t_range: SType::SUnsignedBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3
        };
        /// toUnsignedMod method descriptor
        pub static ref TO_UNSIGNED_MOD_METHOD_DESC: SMethodDesc = SMethodDesc {
            method_id: TO_UNSIGNED_MOD_METHOD_ID,
            name: "toUnsignedMod",
            tpe: SFunc {
                t_dom: vec![SType::SBigInt, SType::SUnsignedBigInt],
                t_range: SType::SUnsignedBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3
        };
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SBigInt))
            .chain([
                TO_UNSIGNED_METHOD_DESC.clone(),
                TO_UNSIGNED_MOD_METHOD_DESC.clone()
            ])
            .collect();
        /// BigInt methods
        pub static ref METHODS: Vec<SMethod> = METHOD_DESC
            .iter()
            .map(|desc| SMethod::new(STypeCompanion::SBigInt, desc.clone()))
            .collect();
    }
}

/// SUnsignedBigInt type and methods
pub mod sunsignedbigint {
    use super::*;
    use crate::{
        serialization::types::TypeCode,
        types::{smethod::SMethod, stype_companion::STypeCompanion},
    };

    /// Short TypeCode
    pub const TYPE_CODE: TypeCode = TypeCode::SUNSIGNEDBIGINT;
    /// Short Type name
    pub const TYPE_NAME: &str = "UnsignedBigInt";

    /// UnsignedBigInt.modInverse method id
    pub const MOD_INVERSE_METHOD_ID: MethodId = MethodId(14);
    /// UnsignedBigInt.plusMod method id
    pub const PLUS_MOD_METHOD_ID: MethodId = MethodId(15);
    /// UnsignedBigInt.subtractMod method id
    pub const SUBTRACT_MOD_METHOD_ID: MethodId = MethodId(16);
    /// UnsignedBigInt.multiplyMod method id
    pub const MULTIPLY_MOD_METHOD_ID: MethodId = MethodId(17);
    /// UnsignedBigInt.mod method id
    pub const MOD_METHOD_ID: MethodId = MethodId(18);
    /// UnsignedBigInt.toSigned method id
    pub const TO_SIGNED_METHOD_ID: MethodId = MethodId(19);

    lazy_static! {
        /// UnsignedBigInt.modInverse method
        pub static ref MOD_INVERSE_METHOD_DESC: SMethodDesc = SMethodDesc {
            name: "modInverse",
            method_id: MOD_INVERSE_METHOD_ID,
            tpe: SFunc {
                t_dom: vec![SType::SUnsignedBigInt, SType::SUnsignedBigInt],
                t_range: SType::SUnsignedBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3,
        };
        /// UnsignedBigInt.plusMod method
        pub static ref PLUS_MOD_METHOD_DESC: SMethodDesc = SMethodDesc {
            name: "plusMod",
            method_id: PLUS_MOD_METHOD_ID,
            tpe: SFunc {
                t_dom: vec![SType::SUnsignedBigInt, SType::SUnsignedBigInt, SType::SUnsignedBigInt],
                t_range: SType::SUnsignedBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3,
        };
        /// UnsignedBigInt.subtractMod method
        pub static ref SUBTRACT_MOD_METHOD_DESC: SMethodDesc = SMethodDesc {
            name: "subtractMod",
            method_id: SUBTRACT_MOD_METHOD_ID,
            tpe: SFunc {
                t_dom: vec![SType::SUnsignedBigInt, SType::SUnsignedBigInt, SType::SUnsignedBigInt],
                t_range: SType::SUnsignedBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3,
        };
        /// UnsignedBigInt.multiplyMod method
        pub static ref MULTIPLY_MOD_METHOD_DESC: SMethodDesc = SMethodDesc {
            name: "multiplyMod",
            method_id: MULTIPLY_MOD_METHOD_ID,
            tpe: SFunc {
                t_dom: vec![SType::SUnsignedBigInt, SType::SUnsignedBigInt, SType::SUnsignedBigInt],
                t_range: SType::SUnsignedBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3,
        };
        /// UnsignedBigInt.mod method
        pub static ref MOD_METHOD_DESC: SMethodDesc = SMethodDesc {
            name: "mod",
            method_id: MOD_METHOD_ID,
            tpe: SFunc {
                t_dom: vec![SType::SUnsignedBigInt, SType::SUnsignedBigInt],
                t_range: SType::SUnsignedBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3,
        };
        /// UnsignedBigInt.mod method
        pub static ref TO_SIGNED_METHOD_DESC: SMethodDesc = SMethodDesc {
            name: "toSigned",
            method_id: TO_SIGNED_METHOD_ID,
            tpe: SFunc {
                t_dom: vec![SType::SUnsignedBigInt],
                t_range: SType::SBigInt.into(),
                tpe_params: vec![],
            },
            explicit_type_args: alloc::boxed::Box::default(),
            min_version: ErgoTreeVersion::V3,
        };
        pub(crate) static ref METHOD_DESC: Vec<SMethodDesc> = super::METHOD_DESC
            .into_iter()
            .map(|method| specialize_method(method, SType::SUnsignedBigInt))
            .chain([MOD_INVERSE_METHOD_DESC.clone(), PLUS_MOD_METHOD_DESC.clone(), SUBTRACT_MOD_METHOD_DESC.clone(), MULTIPLY_MOD_METHOD_DESC.clone(), MOD_METHOD_DESC.clone(), TO_SIGNED_METHOD_DESC.clone()])
            .collect();
        /// UnsignedBigInt methods
        pub static ref METHODS: Vec<SMethod> = METHOD_DESC
            .iter()
            .map(|desc| SMethod::new(STypeCompanion::SUnsignedBigInt, desc.clone()))
            .collect();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::unreachable)]
mod test {
    use crate::{
        bigint256::BigInt256,
        ergo_tree::ErgoTreeVersion,
        mir::{constant::Constant, expr::Expr, method_call::MethodCall},
        serialization::roundtrip_new_feature,
        types::{stype::SType, stype_companion::STypeCompanion},
        unsignedbigint256::UnsignedBigInt,
    };

    use super::sbigint::{TO_UNSIGNED_METHOD_DESC, TO_UNSIGNED_MOD_METHOD_DESC};

    #[test]
    fn byte_method_roundtrips() {
        super::sbyte::METHODS.iter().cloned().for_each(|method| {
            let args = match &*method.method_raw.tpe.t_dom {
                [SType::SByte] => vec![],
                [SType::SByte, SType::SByte] => vec![Constant::from(1i8).into()],
                [SType::SByte, SType::SInt] => vec![Constant::from(1i32).into()],
                _ => unreachable!(),
            };
            let mc = MethodCall::new(Constant::from(1i8).into(), method, args).unwrap();
            roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
        });
    }
    #[test]
    fn short_method_roundtrips() {
        super::sshort::METHODS.iter().cloned().for_each(|method| {
            let args = match &*method.method_raw.tpe.t_dom {
                [SType::SShort] => vec![],
                [SType::SShort, SType::SShort] => vec![Constant::from(1i16).into()],
                [SType::SShort, SType::SInt] => vec![Constant::from(1i32).into()],
                _ => unreachable!(),
            };
            let mc = MethodCall::new(Constant::from(1i16).into(), method, args).unwrap();
            roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
        });
    }
    #[test]
    fn int_method_roundtrips() {
        super::sint::METHODS.iter().cloned().for_each(|method| {
            let args = match &*method.method_raw.tpe.t_dom {
                [SType::SInt] => vec![],
                [SType::SInt, SType::SInt] => vec![Constant::from(1i32).into()],
                _ => unreachable!(),
            };
            let mc = MethodCall::new(Constant::from(1i32).into(), method, args).unwrap();
            roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
        });
    }
    #[test]
    fn long_method_roundtrips() {
        super::slong::METHODS.iter().cloned().for_each(|method| {
            let args = match &*method.method_raw.tpe.t_dom {
                [SType::SLong] => vec![],
                [SType::SLong, SType::SLong] => vec![Constant::from(1i64).into()],
                [SType::SLong, SType::SInt] => vec![Constant::from(1i32).into()],
                _ => unreachable!(),
            };
            let mc = MethodCall::new(Constant::from(1i64).into(), method, args).unwrap();
            roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
        });
    }
    #[test]
    fn bigint_method_roundtrips() {
        super::sbigint::METHODS
            .iter()
            .take(super::METHOD_DESC.len())
            .cloned()
            .for_each(|method| {
                let args = match &*method.method_raw.tpe.t_dom {
                    [SType::SBigInt] => vec![],
                    [SType::SBigInt, SType::SBigInt] => {
                        vec![Constant::from(BigInt256::from(1)).into()]
                    }
                    [SType::SBigInt, SType::SInt] => vec![Constant::from(1i32).into()],
                    _ => unreachable!(),
                };
                let mc = MethodCall::new(Constant::from(BigInt256::from(1)).into(), method, args)
                    .unwrap();
                roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
            });
        let to_unsigned_mc = MethodCall::new(
            Constant::from(BigInt256::from(1)).into(),
            TO_UNSIGNED_METHOD_DESC.as_method(STypeCompanion::SBigInt),
            vec![],
        )
        .unwrap();
        roundtrip_new_feature(&to_unsigned_mc, ErgoTreeVersion::V3);
        let to_unsigned_mod_mc = MethodCall::new(
            Constant::from(BigInt256::from(1)).into(),
            TO_UNSIGNED_MOD_METHOD_DESC.as_method(STypeCompanion::SBigInt),
            vec![Constant::from(UnsignedBigInt::from(1u32)).into()],
        )
        .unwrap();
        roundtrip_new_feature(&to_unsigned_mod_mc, ErgoTreeVersion::V3);
    }
    #[test]
    fn unsigned_bigint_method_roundtrips() {
        super::sunsignedbigint::METHODS
            .iter()
            .take(super::METHOD_DESC.len())
            .cloned()
            .for_each(|method| {
                let args = match &*method.method_raw.tpe.t_dom {
                    [SType::SUnsignedBigInt] => vec![],
                    [SType::SUnsignedBigInt, SType::SUnsignedBigInt] => {
                        vec![Constant::from(UnsignedBigInt::from(1u32)).into()]
                    }
                    [SType::SUnsignedBigInt, SType::SInt] => vec![Constant::from(1i32).into()],
                    _ => unreachable!(),
                };
                let mc = MethodCall::new(
                    Constant::from(UnsignedBigInt::from(1u32)).into(),
                    method,
                    args,
                )
                .unwrap();
                roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
            });
        super::sunsignedbigint::METHOD_DESC
            .iter()
            .skip(super::METHOD_DESC.len())
            .map(|m| m.as_method(STypeCompanion::SUnsignedBigInt))
            .for_each(|method| {
                let args: Vec<Expr> = vec![
                    Constant::from(UnsignedBigInt::from(1u32)).into();
                    method.method_raw.tpe.t_dom.len() - 1
                ];
                let mc = MethodCall::new(
                    Constant::from(UnsignedBigInt::from(1u32)).into(),
                    method,
                    args,
                )
                .unwrap();
                roundtrip_new_feature(&mc, ErgoTreeVersion::V3);
            });
    }
}
