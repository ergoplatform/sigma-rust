use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use crate::{
    chain::ergo_box::ErgoBox,
    serialization::{op_code::OpCode, SerializationError, SigmaSerializable},
    sigma_protocol::{dlog_group::EcPoint, sigma_boolean::SigmaProp},
    types::{LiftIntoSType, SType},
};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
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
    GroupElement(Box<EcPoint>),
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
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
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

impl<T: Into<SigmaProp>> From<T> for ConstantVal {
    fn from(t: T) -> Self {
        ConstantVal::SigmaProp(Box::new(t.into()))
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

impl Into<ConstantVal> for EcPoint {
    fn into(self) -> ConstantVal {
        ConstantVal::GroupElement(Box::new(self))
    }
}

impl Into<Constant> for EcPoint {
    fn into(self) -> Constant {
        Constant {
            tpe: SType::SGroupElement,
            v: self.into(),
        }
    }
}

/// Underlying type is different from requested value type
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TryExtractFromError(String);

/// Extract underlying value if type matches
pub trait TryExtractFrom<T>: Sized {
    /// Extract the value or return an error if type does not match
    fn try_extract_from(c: T) -> Result<Self, TryExtractFromError>;
}

impl TryExtractFrom<ConstantVal> for bool {
    fn try_extract_from(cv: ConstantVal) -> Result<bool, TryExtractFromError> {
        match cv {
            ConstantVal::Boolean(v) => Ok(v),
            _ => Err(TryExtractFromError(format!(
                "expected bool, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<ConstantVal> for i8 {
    fn try_extract_from(cv: ConstantVal) -> Result<i8, TryExtractFromError> {
        match cv {
            ConstantVal::Byte(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i8, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<ConstantVal> for i16 {
    fn try_extract_from(cv: ConstantVal) -> Result<i16, TryExtractFromError> {
        match cv {
            ConstantVal::Short(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i16, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<ConstantVal> for i32 {
    fn try_extract_from(cv: ConstantVal) -> Result<i32, TryExtractFromError> {
        match cv {
            ConstantVal::Int(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i32, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<ConstantVal> for i64 {
    fn try_extract_from(cv: ConstantVal) -> Result<i64, TryExtractFromError> {
        match cv {
            ConstantVal::Long(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i64, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<ConstantVal> for EcPoint {
    fn try_extract_from(cv: ConstantVal) -> Result<EcPoint, TryExtractFromError> {
        match cv {
            ConstantVal::GroupElement(v) => Ok(*v),
            _ => Err(TryExtractFromError(format!(
                "expected EcPoint, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<ConstantVal> for SigmaProp {
    fn try_extract_from(cv: ConstantVal) -> Result<SigmaProp, TryExtractFromError> {
        match cv {
            ConstantVal::SigmaProp(v) => Ok(*v),
            _ => Err(TryExtractFromError(format!(
                "expected SigmaProp, found {:?}",
                cv
            ))),
        }
    }
}

impl<T: TryExtractFrom<ConstantVal>> TryExtractFrom<Constant> for T {
    fn try_extract_from(cv: Constant) -> Result<Self, TryExtractFromError> {
        T::try_extract_from(cv.v)
    }
}

impl<T: TryExtractFrom<ConstantVal> + StoredNonPrimitive + LiftIntoSType> TryExtractFrom<Constant>
    for Vec<T>
{
    fn try_extract_from(c: Constant) -> Result<Self, TryExtractFromError> {
        match c.v {
            ConstantVal::Coll(ConstantColl::NonPrimitive { elem_tpe: _, v }) => {
                v.into_iter().map(T::try_extract_from).collect()
            }
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                std::any::type_name::<Self>(),
                c.v
            ))),
        }
    }
}

impl TryExtractFrom<Constant> for Vec<i8> {
    fn try_extract_from(c: Constant) -> Result<Self, TryExtractFromError> {
        match c.v {
            ConstantVal::Coll(ConstantColl::Primitive(CollPrim::CollByte(bs))) => Ok(bs),
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                std::any::type_name::<Self>(),
                c.v
            ))),
        }
    }
}

impl TryExtractFrom<Constant> for Vec<u8> {
    fn try_extract_from(cv: Constant) -> Result<Self, TryExtractFromError> {
        use crate::util::FromVecI8;
        Vec::<i8>::try_extract_from(cv).map(Vec::<u8>::from_vec_i8)
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

impl Into<Constant> for Vec<u8> {
    fn into(self) -> Constant {
        Constant {
            tpe: SType::SColl(Box::new(SType::SByte)),
            v: ConstantVal::Coll(ConstantColl::Primitive(CollPrim::CollByte(
                self.into_iter().map(|b| b as i8).collect(),
            ))),
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

/// Placeholder for a constant in ErgoTree.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConstantPlaceholder {
    /// Zero based index in ErgoTree.constants array.
    pub id: u32,
    /// Type of the constant value
    pub tpe: SType,
}

impl ConstantPlaceholder {
    /// OpCode value
    pub const OP_CODE: OpCode = OpCode::CONSTANT_PLACEHOLDER;

    /// OpCode for the serialization
    pub fn op_code(&self) -> OpCode {
        OpCode::CONSTANT_PLACEHOLDER
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
                any::<EcPoint>().prop_map_into(),
                any::<SigmaProp>().prop_map_into(),
                (vec(any::<i8>(), 0..100)).prop_map_into(),
                (vec(any::<i16>(), 0..100)).prop_map_into(),
                (vec(any::<i32>(), 0..100)).prop_map_into(),
                (vec(any::<i64>(), 0..100)).prop_map_into(),
            ]
            .boxed()
        }
    }

    proptest! {

        #[test]
        fn test_try_extract_from(c in any::<Constant>()) {
            // let c = force_any_val::<Constant>();
            match c.clone().tpe {
                SType::SBoolean => {
                    let _ = bool::try_extract_from(c).unwrap();
                }
                SType::SByte => {
                    let _ = i8::try_extract_from(c).unwrap();
                }
                SType::SShort => {
                    let _ = i16::try_extract_from(c).unwrap();
                }
                SType::SInt => {
                    let _ = i32::try_extract_from(c).unwrap();
                }
                SType::SLong => {
                    let _ = i64::try_extract_from(c).unwrap();
                }
                SType::SGroupElement => {
                    let _ = EcPoint::try_extract_from(c).unwrap();
                }
                SType::SSigmaProp => {
                    let _ = SigmaProp::try_extract_from(c).unwrap();
                }
                SType::SColl(elem_type) => {
                    match *elem_type {
                        SType::SByte => { let _ = Vec::<i8>::try_extract_from(c).unwrap(); }
                        SType::SShort => { let _ = Vec::<i16>::try_extract_from(c).unwrap(); }
                        SType::SInt => { let _ = Vec::<i32>::try_extract_from(c).unwrap(); }
                        SType::SLong => { let _ = Vec::<i64>::try_extract_from(c).unwrap(); }
                        _ => todo!()
                    }
                }
                _ => todo!(),
            };
        }
    }
}
