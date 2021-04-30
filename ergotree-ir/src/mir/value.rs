//! Ergo data type

use std::convert::TryFrom;
use std::convert::TryInto;

use impl_trait_for_tuples::impl_for_tuples;
use num_bigint::BigInt;

use crate::ir_ergo_box::IrBoxId;
use crate::sigma_protocol::dlog_group::EcPoint;
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::SigmaProp;
use crate::types::stuple::TupleItems;
use crate::types::stype::LiftIntoSType;
use crate::types::stype::SType;
use crate::util::AsVecI8;

use super::constant::TryExtractFrom;
use super::constant::TryExtractFromError;
use super::constant::TryExtractInto;
use super::expr::Expr;
use super::func_value::FuncArg;

extern crate derive_more;
use derive_more::From;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Collection for primitive values (i.e byte array)
pub enum NativeColl {
    /// Collection of bytes
    CollByte(Vec<i8>),
}

impl NativeColl {
    /// Collection element type
    pub fn elem_tpe(&self) -> &SType {
        match self {
            NativeColl::CollByte(_) => &SType::SByte,
        }
    }
}

/// Collection elements
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CollKind {
    /// Collection elements stored as a vector of Rust values
    NativeColl(NativeColl),
    /// Collection elements stored as a vector of Value's
    WrappedColl {
        /// Collection element type
        elem_tpe: SType,
        /// Collection elements
        items: Vec<Value>,
    },
}

impl CollKind {
    /// Build a collection from items, storing them as Rust types values when neccessary
    pub fn from_vec(elem_tpe: SType, items: Vec<Value>) -> Result<CollKind, TryExtractFromError> {
        match elem_tpe {
            SType::SByte => items
                .into_iter()
                .map(|v| v.try_extract_into::<i8>())
                .collect::<Result<Vec<_>, _>>()
                .map(|bytes| CollKind::NativeColl(NativeColl::CollByte(bytes))),
            _ => Ok(CollKind::WrappedColl { elem_tpe, items }),
        }
    }

    /// Build a collection from items where each is a collection as well, storing them as Rust types values when neccessary
    pub fn from_vec_vec(
        elem_tpe: SType,
        items: Vec<Value>,
    ) -> Result<CollKind, TryExtractFromError> {
        match elem_tpe {
            SType::SByte => items
                .into_iter()
                .map(|v| v.try_extract_into::<Vec<i8>>())
                .collect::<Result<Vec<_>, _>>()
                .map(|bytes| CollKind::NativeColl(NativeColl::CollByte(bytes.concat()))),
            _ => items
                .into_iter()
                .map(|v| v.try_extract_into::<Vec<Value>>())
                .collect::<Result<Vec<_>, _>>()
                .map(|v| CollKind::WrappedColl {
                    elem_tpe,
                    items: v.concat(),
                }),
        }
    }

    /// Collection element type
    pub fn elem_tpe(&self) -> &SType {
        match self {
            cp @ CollKind::NativeColl(_) => cp.elem_tpe(),
            CollKind::WrappedColl { elem_tpe, .. } => elem_tpe,
        }
    }

    /// Return items, as vector of Values
    pub fn as_vec(&self) -> Vec<Value> {
        match self {
            CollKind::NativeColl(NativeColl::CollByte(coll_byte)) => coll_byte
                .clone()
                .into_iter()
                .map(|byte| byte.into())
                .collect(),
            CollKind::WrappedColl {
                elem_tpe: _,
                items: v,
            } => v.clone(),
        }
    }
}

/// Lambda
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Lambda {
    /// Argument placeholders
    pub args: Vec<FuncArg>,
    /// Body
    pub body: Box<Expr>,
}

/// Runtime value
#[derive(PartialEq, Eq, Debug, Clone, From)]
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
    BigInt(BigInt),
    /// GroupElement
    GroupElement(Box<EcPoint>),
    /// Sigma property
    SigmaProp(Box<SigmaProp>),
    /// Box
    CBox(IrBoxId),
    /// AVL tree
    AvlTree,
    /// Collection of values of the same type
    Coll(CollKind),
    /// Tuple (arbitrary type values)
    Tup(TupleItems<Value>),
    /// Transaction(and blockchain) context info
    Context,
    /// Global which is used to define global methods
    Global,
    /// Optional value
    Opt(Box<Option<Value>>),
    /// lambda
    Lambda(Lambda),
}

impl Value {
    /// Create Sigma property constant
    pub fn sigma_prop(prop: SigmaProp) -> Value {
        Value::SigmaProp(Box::new(prop))
    }
}

impl<T: Into<SigmaProp>> From<T> for Value {
    fn from(t: T) -> Self {
        Value::SigmaProp(Box::new(t.into()))
    }
}

impl From<EcPoint> for Value {
    fn from(v: EcPoint) -> Self {
        Value::GroupElement(Box::new(v))
    }
}

impl From<Vec<i8>> for Value {
    fn from(v: Vec<i8>) -> Self {
        Value::Coll(CollKind::NativeColl(NativeColl::CollByte(v)))
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Coll(CollKind::NativeColl(NativeColl::CollByte(v.as_vec_i8())))
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(opt: Option<T>) -> Self {
        Value::Opt(Box::new(opt.map(|v| v.into())))
    }
}

/// Marker trait to select types which stored as Vec of wrapped Value's
pub trait StoreWrapped {}

impl StoreWrapped for bool {}
impl StoreWrapped for i16 {}
impl StoreWrapped for i32 {}
impl StoreWrapped for i64 {}
impl StoreWrapped for BigInt {}
impl StoreWrapped for IrBoxId {}
impl StoreWrapped for EcPoint {}
impl StoreWrapped for SigmaProp {}
impl<T: StoreWrapped> StoreWrapped for Option<T> {}
impl<T> StoreWrapped for Vec<T> {}
impl StoreWrapped for Value {}

#[impl_for_tuples(2, 4)]
impl StoreWrapped for Tuple {}

impl<T: LiftIntoSType + StoreWrapped + Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Coll(CollKind::WrappedColl {
            elem_tpe: T::stype(),
            items: v.into_iter().map(|i| i.into()).collect(),
        })
    }
}

#[allow(clippy::clippy::from_over_into)]
#[allow(clippy::unwrap_used)]
#[impl_for_tuples(2, 4)]
impl Into<Value> for Tuple {
    fn into(self) -> Value {
        let v: Vec<Value> = [for_tuples!(  #( Tuple.into() ),* )].to_vec();
        Value::Tup(v.try_into().unwrap())
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

impl TryExtractFrom<Value> for IrBoxId {
    fn try_extract_from(c: Value) -> Result<Self, TryExtractFromError> {
        match c {
            Value::CBox(b) => Ok(b),
            _ => Err(TryExtractFromError(format!(
                "expected IrErgoBox, found {:?}",
                c
            ))),
        }
    }
}

impl<T: TryExtractFrom<Value> + StoreWrapped> TryExtractFrom<Value> for Vec<T> {
    fn try_extract_from(c: Value) -> Result<Self, TryExtractFromError> {
        match c {
            Value::Coll(coll) => match coll {
                CollKind::WrappedColl {
                    elem_tpe: _,
                    items: v,
                } => v.into_iter().map(T::try_extract_from).collect(),
                _ => Err(TryExtractFromError(format!(
                    "expected {:?}, found {:?}",
                    std::any::type_name::<Self>(),
                    coll
                ))),
            },
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                std::any::type_name::<Self>(),
                c
            ))),
        }
    }
}

impl TryExtractFrom<Value> for Vec<i8> {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        match v {
            Value::Coll(v) => match v {
                CollKind::NativeColl(NativeColl::CollByte(bs)) => Ok(bs),
                _ => Err(TryExtractFromError(format!(
                    "expected {:?}, found {:?}",
                    std::any::type_name::<Self>(),
                    v
                ))),
            },
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                std::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl TryExtractFrom<Value> for Vec<u8> {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        use crate::util::FromVecI8;
        Vec::<i8>::try_extract_from(v).map(Vec::<u8>::from_vec_i8)
    }
}

impl TryExtractFrom<Value> for Value {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        Ok(v)
    }
}

impl TryExtractFrom<Value> for BigInt {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        match v {
            Value::BigInt(bi) => Ok(bi),
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                std::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl TryFrom<Value> for ProveDlog {
    type Error = TryExtractFromError;
    fn try_from(cv: Value) -> Result<Self, Self::Error> {
        match cv {
            Value::SigmaProp(sp) => match sp.value() {
                SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(
                    prove_dlog,
                )) => Ok(prove_dlog.clone()),
                _ => Err(TryExtractFromError(format!(
                    "expected ProveDlog, found {:?}",
                    sp
                ))),
            },
            _ => Err(TryExtractFromError(format!(
                "expected SigmaProp, found {:?}",
                cv
            ))),
        }
    }
}

// impl TryExtractFrom<Value> for Rc<Context> {
//     fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
//         match v {
//             Value::Context(ctx) => Ok(ctx),
//             _ => Err(TryExtractFromError(format!(
//                 "expected Context, found {:?}",
//                 v
//             ))),
//         }
//     }
// }

impl<T: TryExtractFrom<Value>> TryExtractFrom<Value> for Option<T> {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        match v {
            Value::Opt(opt) => opt.map(T::try_extract_from).transpose(),
            _ => Err(TryExtractFromError(format!(
                "expected Context, found {:?}",
                v
            ))),
        }
    }
}

#[impl_for_tuples(2, 4)]
impl TryExtractFrom<Value> for Tuple {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        match v {
            Value::Tup(items) => {
                let mut iter = items.iter();
                Ok(for_tuples!( ( #(
                                Tuple::try_extract_from(
                                    iter
                                        .next()
                                        .cloned()
                                        .ok_or_else(|| TryExtractFromError("not enough items in STuple".to_string()))?
                                )?
                                ),* ) ))
            }
            _ => Err(TryExtractFromError(format!(
                "expected Context, found {:?}",
                v
            ))),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn byte_u8_array_into() {
        let bytes = vec![1u8, 2u8, 3u8];
        let value: Value = bytes.into();
        assert!(matches!(
            value,
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(_)))
        ))
    }

    #[test]
    fn byte_i8_array_into() {
        let bytes = vec![1i8, 2i8, 3i8];
        let value: Value = bytes.into();
        assert!(matches!(
            value,
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(_)))
        ))
    }

    #[test]
    fn byte_from_vec_roundtrip() {
        let bytes = vec![1i8, 2i8, 3i8];
        let wrapped: Vec<Value> = bytes.into_iter().map(|b| b.into()).collect();
        let coll = CollKind::from_vec(SType::SByte, wrapped.clone()).unwrap();
        assert!(matches!(
            coll,
            CollKind::NativeColl(NativeColl::CollByte(_))
        ));
        let as_vec = coll.as_vec();
        assert_eq!(as_vec, wrapped);
    }

    #[test]
    fn wrapped_from_vec_roundtrip() {
        let longs = vec![1i64, 2i64, 3i64];
        let wrapped: Vec<Value> = longs.into_iter().map(|b| b.into()).collect();
        let coll = CollKind::from_vec(SType::SLong, wrapped.clone()).unwrap();
        assert!(matches!(
            coll,
            CollKind::WrappedColl {
                elem_tpe: SType::SLong,
                items: _,
            }
        ));
        let as_vec = coll.as_vec();
        assert_eq!(as_vec, wrapped);
    }
}
