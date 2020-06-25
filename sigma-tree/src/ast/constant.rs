use crate::{chain::ErgoBox, sigma_protocol::SigmaProp, types::SType, Base16Bytes};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use std::convert::TryFrom;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CollPrim {
    CollBoolean(Vec<bool>),
    CollByte(Vec<i8>),
    CollShort(Vec<i16>),
    CollInt(Vec<i32>),
    CollLong(Vec<i64>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConstantVal {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    BigInt,
    GroupElement,
    SigmaProp(Box<SigmaProp>),
    CBox(Box<ErgoBox>),
    AvlTree,
    CollPrim(CollPrim),
    Coll(Vec<ConstantVal>),
    Tup(Vec<ConstantVal>),
}

impl ConstantVal {
    pub fn sigma_prop(prop: SigmaProp) -> ConstantVal {
        ConstantVal::SigmaProp(Box::new(prop))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "with-serde",
    serde(into = "Base16Bytes", try_from = "Base16Bytes")
)]
pub struct Constant {
    pub tpe: SType,
    pub v: ConstantVal,
}

impl Into<Base16Bytes> for Constant {
    fn into(self) -> Base16Bytes {
        Base16Bytes(self.sigma_serialise_bytes())
    }
}

impl TryFrom<Base16Bytes> for Constant {
    type Error = SerializationError;
    fn try_from(bytes: Base16Bytes) -> Result<Self, Self::Error> {
        Constant::sigma_parse_bytes(bytes.0)
    }
}

impl Constant {
    pub fn bool(v: bool) -> Constant {
        Constant {
            tpe: SType::SBoolean,
            v: ConstantVal::Boolean(v),
        }
    }

    pub fn byte(v: i8) -> Constant {
        Constant {
            tpe: SType::SByte,
            v: ConstantVal::Byte(v),
        }
    }

    pub fn short(v: i16) -> Constant {
        Constant {
            tpe: SType::SShort,
            v: ConstantVal::Short(v),
        }
    }

    pub fn int(v: i32) -> Constant {
        Constant {
            tpe: SType::SInt,
            v: ConstantVal::Int(v),
        }
    }

    pub fn long(v: i64) -> Constant {
        Constant {
            tpe: SType::SLong,
            v: ConstantVal::Long(v),
        }
    }

    pub fn byte_array(v: Vec<i8>) -> Constant {
        Constant {
            tpe: SType::SColl(Box::new(SType::SByte)),
            v: ConstantVal::CollPrim(CollPrim::CollByte(v)),
        }
    }

    pub fn sigma_prop(prop: SigmaProp) -> Constant {
        Constant {
            tpe: SType::SSigmaProp,
            v: ConstantVal::sigma_prop(prop),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Constant {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                any::<bool>().prop_map(|v| Constant::bool(v)),
                any::<i8>().prop_map(|v| Constant::byte(v)),
                any::<i16>().prop_map(|v| Constant::short(v)),
                any::<i32>().prop_map(|v| Constant::int(v)),
                any::<i64>().prop_map(|v| Constant::long(v)),
                (vec(any::<i8>(), 0..100)).prop_map(|v| Constant::byte_array(v)),
            ]
            .boxed()
        }
    }
}
