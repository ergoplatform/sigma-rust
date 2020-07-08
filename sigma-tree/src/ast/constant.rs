use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use crate::{
    chain::ErgoBox,
    sigma_protocol::SigmaProp,
    types::{LiftIntoSType, SType},
};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::serializer::SerializationError;
use sigma_ser::serializer::SigmaSerializable;
use std::convert::TryFrom;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Collection for primitive values (i.e byte array)
pub enum CollPrim {
    /// Collection of bytes
    CollByte(Vec<i8>),
}

impl CollPrim {
    /// Collection element type
    pub fn elem_tpe(&self) -> &SType {
        match self {
            CollPrim::CollByte(_) => &SType::SByte,
        }
    }
}

/// Collection elements
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConstantColl {
    /// Collection elements stored as a vector of primitive types
    Primitive(CollPrim),
    /// Collection elements stored as a vector of ConstantVals
    NonPrimitive {
        /// Collection element type
        elem_tpe: SType,
        /// Collection elements
        v: Vec<ConstantVal>,
    },
}

impl ConstantColl {
    /// Collection element type
    pub fn elem_tpe(&self) -> &SType {
        match self {
            cp @ ConstantColl::Primitive(_) => cp.elem_tpe(),
            ConstantColl::NonPrimitive { elem_tpe, .. } => elem_tpe,
        }
    }
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
    Coll(ConstantColl),
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

impl Into<Base16EncodedBytes> for Constant {
    fn into(self) -> Base16EncodedBytes {
        Base16EncodedBytes::new(&self.sigma_serialise_bytes())
    }
}

impl TryFrom<Base16DecodedBytes> for Constant {
    type Error = SerializationError;
    fn try_from(bytes: Base16DecodedBytes) -> Result<Self, Self::Error> {
        Constant::sigma_parse_bytes(bytes.0)
    }
}

impl Constant {
    /// Serialized bytes encoded as Base16
    pub fn base16_str(&self) -> String {
        let base16_bytes: Base16EncodedBytes = self.clone().into();
        base16_bytes.into()
    }
}

impl Into<ConstantVal> for bool {
    fn into(self) -> ConstantVal {
        ConstantVal::Boolean(self)
    }
}

impl Into<Constant> for bool {
    fn into(self) -> Constant {
        Constant {
            tpe: bool::stype(),
            v: self.into(),
        }
    }
}

impl Into<ConstantVal> for i8 {
    fn into(self) -> ConstantVal {
        ConstantVal::Byte(self)
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

impl Into<ConstantVal> for i16 {
    fn into(self) -> ConstantVal {
        ConstantVal::Short(self)
    }
}

impl Into<Constant> for i16 {
    fn into(self) -> Constant {
        Constant {
            tpe: i16::stype(),
            v: self.into(),
        }
    }
}

impl Into<ConstantVal> for i32 {
    fn into(self) -> ConstantVal {
        ConstantVal::Int(self)
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

impl Into<ConstantVal> for i64 {
    fn into(self) -> ConstantVal {
        ConstantVal::Long(self)
    }
}

impl Into<Constant> for i64 {
    fn into(self) -> Constant {
        Constant {
            tpe: i64::stype(),
            v: self.into(),
        }
    }
}

impl Into<ConstantVal> for SigmaProp {
    fn into(self) -> ConstantVal {
        ConstantVal::SigmaProp(Box::new(self))
    }
}

impl Into<Constant> for SigmaProp {
    fn into(self) -> Constant {
        Constant {
            tpe: SType::SSigmaProp,
            v: self.into(),
        }
    }
}

/// Marker trait to select types for which CollElems::NonPrimitive is used to store elements as Vec<ConstantVal>
pub trait StoredNonPrimitive {}

impl StoredNonPrimitive for bool {}
impl StoredNonPrimitive for i16 {}
impl StoredNonPrimitive for i32 {}
impl StoredNonPrimitive for i64 {}

impl<T: LiftIntoSType + StoredNonPrimitive + Into<ConstantVal>> Into<ConstantVal> for Vec<T> {
    fn into(self) -> ConstantVal {
        ConstantVal::Coll(ConstantColl::NonPrimitive {
            elem_tpe: T::stype(),
            v: self.into_iter().map(|i| i.into()).collect(),
        })
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

impl Into<Constant> for Vec<i8> {
    fn into(self) -> Constant {
        Constant {
            tpe: SType::SColl(Box::new(SType::SByte)),
            v: ConstantVal::Coll(ConstantColl::Primitive(CollPrim::CollByte(self))),
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
                any::<bool>().prop_map_into(),
                any::<i8>().prop_map_into(),
                any::<i16>().prop_map_into(),
                any::<i32>().prop_map_into(),
                any::<i64>().prop_map_into(),
                (vec(any::<i8>(), 0..100)).prop_map_into(),
                (vec(any::<i16>(), 0..100)).prop_map_into(),
                (vec(any::<i32>(), 0..100)).prop_map_into(),
                (vec(any::<i64>(), 0..100)).prop_map_into(),
            ]
            .boxed()
        }
    }
}
