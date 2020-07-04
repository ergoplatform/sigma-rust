#[cfg(feature = "with-serde")]
use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use crate::{chain::ErgoBox, sigma_protocol::SigmaProp, types::SType};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "with-serde")]
use sigma_ser::serializer::{SerializationError, SigmaSerializable};
#[cfg(feature = "with-serde")]
use std::convert::TryFrom;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Collection for primitive values (i.e byte array)
pub enum CollPrim {
    /// Collection of bytes
    CollByte(Vec<i8>),
}

/// Collection elements
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CollElems {
    /// Collection elements stored as a vector of primitive types
    Primitive(CollPrim),
    /// Collection elements stored as a vector of ConstantVals
    NonPrimitive(Vec<ConstantVal>),
}

/// Constant value
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConstantVal {
    /// Boolean
    Boolean(bool),
    /// Byte
    Byte(i8),
    /// Short
    Short(i16),
    /// Int
    Int(i32),
    /// Long
    Long(i64),
    /// Big integer
    BigInt,
    /// GroupElement
    GroupElement,
    /// Sigma property
    SigmaProp(Box<SigmaProp>),
    /// Box
    CBox(Box<ErgoBox>),
    /// AVL tree
    AvlTree,
    /// Collection of values of the same type
    Coll {
        /// Collection element type
        elem_tpe: SType,
        /// Collection elements
        v: CollElems,
    },
    /// Tuple (arbitrary type values)
    Tup(Vec<ConstantVal>),
}

impl ConstantVal {
    /// Create Sigma property constant
    pub fn sigma_prop(prop: SigmaProp) -> ConstantVal {
        ConstantVal::SigmaProp(Box::new(prop))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "with-serde",
    serde(into = "Base16EncodedBytes", try_from = "Base16DecodedBytes")
)]
/// Constant
pub struct Constant {
    /// Constant type
    pub tpe: SType,
    /// Constant value
    pub v: ConstantVal,
}

#[cfg(feature = "with-serde")]
impl Into<Base16EncodedBytes> for Constant {
    fn into(self) -> Base16EncodedBytes {
        Base16EncodedBytes::new(&self.sigma_serialise_bytes())
    }
}

#[cfg(feature = "with-serde")]
impl TryFrom<Base16DecodedBytes> for Constant {
    type Error = SerializationError;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        Constant::sigma_parse_bytes(bytes.0)
    }
}

impl Constant {
    /// Create bool value constant
    pub fn bool(v: bool) -> Constant {
        Constant {
            tpe: SType::SBoolean,
            v: ConstantVal::Boolean(v),
        }
    }

    /// Create short value constant
    pub fn short(v: i16) -> Constant {
        Constant {
            tpe: SType::SShort,
            v: ConstantVal::Short(v),
        }
    }

    /// Create int value constant
    pub fn int(v: i32) -> Constant {
        Constant {
            tpe: SType::SInt,
            v: ConstantVal::Int(v),
        }
    }

    /// Create long value constant
    pub fn long(v: i64) -> Constant {
        Constant {
            tpe: SType::SLong,
            v: ConstantVal::Long(v),
        }
    }

    /// Create Sigma property constant
    pub fn sigma_prop(prop: SigmaProp) -> Constant {
        Constant {
            tpe: SType::SSigmaProp,
            v: ConstantVal::sigma_prop(prop),
        }
    }
}

// TODO: remove Constant::int, long, etc.

/// Conversion to SType
pub trait LiftIntoSType {
    /// get SType
    fn stype() -> SType;
}

impl Into<ConstantVal> for i8 {
    fn into(self) -> ConstantVal {
        ConstantVal::Byte(self)
    }
}

impl Into<ConstantVal> for i32 {
    fn into(self) -> ConstantVal {
        ConstantVal::Int(self)
    }
}

impl Into<Constant> for i8 {
    fn into(self) -> Constant {
        Constant {
            tpe: i8::stype(),
            v: self.into(),
        }
    }
}

impl Into<Constant> for i32 {
    fn into(self) -> Constant {
        Constant {
            tpe: i32::stype(),
            v: self.into(),
        }
    }
}

impl<T: LiftIntoSType> LiftIntoSType for Vec<T> {
    fn stype() -> SType {
        SType::SColl(Box::new(T::stype()))
    }
}

impl LiftIntoSType for i32 {
    fn stype() -> SType {
        SType::SInt
    }
}

impl LiftIntoSType for i8 {
    fn stype() -> SType {
        SType::SByte
    }
}

/// Marker trait to select types for which CollElems::NonPrimitive is used to store elements as Vec<ConstantVal>
pub trait StoredNonPrimitive {}

impl StoredNonPrimitive for i32 {}

impl<T: LiftIntoSType + StoredNonPrimitive + Into<ConstantVal>> Into<ConstantVal> for Vec<T> {
    fn into(self) -> ConstantVal {
        ConstantVal::Coll {
            elem_tpe: T::stype(),
            v: CollElems::NonPrimitive(self.into_iter().map(|i| i.into()).collect()),
        }
    }
}

impl Into<Constant> for Vec<i8> {
    fn into(self) -> Constant {
        Constant {
            tpe: SType::SColl(Box::new(SType::SByte)),
            v: ConstantVal::Coll {
                elem_tpe: SType::SByte,
                v: CollElems::Primitive(CollPrim::CollByte(self)),
            },
        }
    }
}

impl<T: LiftIntoSType + StoredNonPrimitive + Into<ConstantVal>> Into<Constant> for Vec<T> {
    fn into(self) -> Constant {
        Constant {
            tpe: Self::stype(),
            v: self.into(),
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
                any::<bool>().prop_map(Constant::bool),
                any::<i8>().prop_map_into(),
                any::<i16>().prop_map(Constant::short),
                any::<i32>().prop_map_into(),
                any::<i64>().prop_map(Constant::long),
                (vec(any::<i8>(), 0..100)).prop_map_into(),
                (vec(any::<i32>(), 0..100)).prop_map_into(),
            ]
            .boxed()
        }
    }
}
