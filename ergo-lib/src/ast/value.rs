use crate::chain::ergo_box::ErgoBox;
use crate::sigma_protocol::dlog_group::EcPoint;
use crate::sigma_protocol::sigma_boolean::SigmaProp;
use crate::types::LiftIntoSType;
use crate::types::SType;

use super::constant::TryExtractFrom;
use super::constant::TryExtractFromError;

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
pub enum Coll {
    /// Collection elements stored as a vector of primitive types
    Primitive(CollPrim),
    /// Collection elements stored as a vector of ConstantVals
    NonPrimitive {
        /// Collection element type
        elem_tpe: SType,
        /// Collection elements
        v: Vec<Value>,
    },
}

impl Coll {
    /// Collection element type
    pub fn elem_tpe(&self) -> &SType {
        match self {
            cp @ Coll::Primitive(_) => cp.elem_tpe(),
            Coll::NonPrimitive { elem_tpe, .. } => elem_tpe,
        }
    }
}

/// Constant value
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Value {
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
    Coll(Coll),
    /// Tuple (arbitrary type values)
    Tup(Vec<Value>),
}

impl Value {
    /// Create Sigma property constant
    pub fn sigma_prop(prop: SigmaProp) -> Value {
        Value::SigmaProp(Box::new(prop))
    }
}

impl Into<Value> for bool {
    fn into(self) -> Value {
        Value::Boolean(self)
    }
}

impl Into<Value> for i8 {
    fn into(self) -> Value {
        Value::Byte(self)
    }
}

impl Into<Value> for i16 {
    fn into(self) -> Value {
        Value::Short(self)
    }
}

impl Into<Value> for i32 {
    fn into(self) -> Value {
        Value::Int(self)
    }
}

impl Into<Value> for i64 {
    fn into(self) -> Value {
        Value::Long(self)
    }
}

impl<T: Into<SigmaProp>> From<T> for Value {
    fn from(t: T) -> Self {
        Value::SigmaProp(Box::new(t.into()))
    }
}

impl Into<Value> for EcPoint {
    fn into(self) -> Value {
        Value::GroupElement(Box::new(self))
    }
}

/// Marker trait to select types for which CollElems::NonPrimitive is used to store elements as Vec<ConstantVal>
pub trait StoredNonPrimitive {}

impl StoredNonPrimitive for bool {}
impl StoredNonPrimitive for i16 {}
impl StoredNonPrimitive for i32 {}
impl StoredNonPrimitive for i64 {}

impl<T: LiftIntoSType + StoredNonPrimitive + Into<Value>> Into<Value> for Vec<T> {
    fn into(self) -> Value {
        Value::Coll(Coll::NonPrimitive {
            elem_tpe: T::stype(),
            v: self.into_iter().map(|i| i.into()).collect(),
        })
    }
}

impl TryExtractFrom<Value> for bool {
    fn try_extract_from(cv: Value) -> Result<bool, TryExtractFromError> {
        match cv {
            Value::Boolean(v) => Ok(v),
            _ => Err(TryExtractFromError(format!(
                "expected bool, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<Value> for i8 {
    fn try_extract_from(cv: Value) -> Result<i8, TryExtractFromError> {
        match cv {
            Value::Byte(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i8, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Value> for i16 {
    fn try_extract_from(cv: Value) -> Result<i16, TryExtractFromError> {
        match cv {
            Value::Short(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i16, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Value> for i32 {
    fn try_extract_from(cv: Value) -> Result<i32, TryExtractFromError> {
        match cv {
            Value::Int(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i32, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Value> for i64 {
    fn try_extract_from(cv: Value) -> Result<i64, TryExtractFromError> {
        match cv {
            Value::Long(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i64, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Value> for EcPoint {
    fn try_extract_from(cv: Value) -> Result<EcPoint, TryExtractFromError> {
        match cv {
            Value::GroupElement(v) => Ok(*v),
            _ => Err(TryExtractFromError(format!(
                "expected EcPoint, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<Value> for SigmaProp {
    fn try_extract_from(cv: Value) -> Result<SigmaProp, TryExtractFromError> {
        match cv {
            Value::SigmaProp(v) => Ok(*v),
            _ => Err(TryExtractFromError(format!(
                "expected SigmaProp, found {:?}",
                cv
            ))),
        }
    }
}
