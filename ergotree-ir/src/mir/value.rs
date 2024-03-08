//! Ergo data type

use std::convert::TryInto;
use std::fmt::Formatter;
use std::sync::Arc;

use impl_trait_for_tuples::impl_for_tuples;
use sigma_util::AsVecI8;

use crate::bigint256::BigInt256;
use crate::chain::ergo_box::ErgoBox;
use crate::sigma_protocol::sigma_boolean::SigmaProp;
use crate::types::stuple::TupleItems;
use crate::types::stype::LiftIntoSType;
use crate::types::stype::SType;
use ergo_chain_types::{EcPoint, Header, PreHeader};

use super::avl_tree_data::AvlTreeData;
use super::constant::Literal;
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
pub enum CollKind<T> {
    /// Collection elements stored as a vector of Rust values
    NativeColl(NativeColl),
    /// Collection elements stored as a vector of Value's
    WrappedColl {
        /// Collection element type
        elem_tpe: SType,
        /// Collection elements
        items: Vec<T>,
    },
}

impl<T> CollKind<T>
where
    T: PartialEq + Eq + Clone,
    T: From<i8>,
    i8: TryExtractFrom<T>,
    Vec<i8>: TryExtractFrom<T>,
    Vec<T>: TryExtractFrom<T>,
{
    /// Build a collection from items, storing them as Rust types values when neccessary
    pub fn from_vec(elem_tpe: SType, items: Vec<T>) -> Result<CollKind<T>, TryExtractFromError> {
        match elem_tpe {
            SType::SByte => items
                .into_iter()
                .map(|v| v.try_extract_into::<i8>())
                .collect::<Result<Vec<_>, _>>()
                .map(|bytes| CollKind::NativeColl(NativeColl::CollByte(bytes))),
            _ => Ok(CollKind::WrappedColl { elem_tpe, items }),
        }
    }

    /// Build a collection from items where each is a collection as well, flattening the arrays
    /// This will convert a Coll[Coll[T]] to a Coll[T]
    pub fn from_vec_vec(
        elem_tpe: SType,
        items: Vec<T>,
    ) -> Result<CollKind<T>, TryExtractFromError> {
        match elem_tpe {
            SType::SColl(inner_type) if matches!(&*inner_type, SType::SByte) => items
                .into_iter()
                .map(|v| v.try_extract_into::<Vec<i8>>())
                .collect::<Result<Vec<_>, _>>()
                .map(|bytes| CollKind::NativeColl(NativeColl::CollByte(bytes.concat()))),
            SType::SColl(flat_type) => items
                .into_iter()
                .map(|v| v.try_extract_into::<Vec<T>>())
                .collect::<Result<Vec<_>, _>>()
                .map(|v| CollKind::WrappedColl {
                    elem_tpe: *flat_type,
                    items: v.concat(),
                }),
            _ => Err(TryExtractFromError(format!(
                "Expected Value::Coll, got: {:?}",
                elem_tpe
            ))),
        }
    }

    /// Collection element type
    pub fn elem_tpe(&self) -> &SType {
        match self {
            CollKind::NativeColl(ncoll) => match ncoll {
                NativeColl::CollByte(_) => &SType::SByte,
            },
            CollKind::WrappedColl { elem_tpe, .. } => elem_tpe,
        }
    }

    /// Return items, as vector of Values
    pub fn as_vec(&self) -> Vec<T> {
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
    /// Unit struct
    Unit,
    /// Big integer
    BigInt(BigInt256),
    /// GroupElement
    GroupElement(Box<EcPoint>),
    /// Sigma property
    SigmaProp(Box<SigmaProp>),
    /// Ergo box
    CBox(Arc<ErgoBox>),
    /// AVL tree
    AvlTree(Box<AvlTreeData>),
    /// Collection of values of the same type
    Coll(CollKind<Value>),
    /// Tuple (arbitrary type values)
    Tup(TupleItems<Value>),
    /// Transaction(and blockchain) context info
    Context,
    /// Block header
    Header(Box<Header>),
    /// Header with predictable data
    PreHeader(Box<PreHeader>),
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

impl From<Literal> for Value {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Byte(b) => Value::Byte(b),
            Literal::Short(s) => Value::Short(s),
            Literal::Int(i) => Value::Int(i),
            Literal::Long(l) => Value::Long(l),
            Literal::BigInt(b) => Value::BigInt(b),
            Literal::Unit => Value::Unit,
            Literal::SigmaProp(s) => Value::SigmaProp(s),
            Literal::GroupElement(e) => Value::GroupElement(e),
            Literal::CBox(b) => Value::CBox(b),
            Literal::Coll(coll) => {
                let converted_coll = match coll {
                    CollKind::NativeColl(n) => CollKind::NativeColl(n),
                    CollKind::WrappedColl { elem_tpe, items } => CollKind::WrappedColl {
                        elem_tpe,
                        items: items.into_iter().map(Value::from).collect(),
                    },
                };
                Value::Coll(converted_coll)
            }
            Literal::AvlTree(a) => Value::AvlTree(a),
            Literal::Opt(lit) => Value::Opt(Box::new(lit.into_iter().next().map(Value::from))),
            Literal::Tup(t) => Value::Tup(t.mapped(Value::from)),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Coll(CollKind::NativeColl(NativeColl::CollByte(i8_bytes))) => {
                write!(f, "Coll[Byte](")?;
                for (i, b) in i8_bytes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", b)?;
                }
                write!(f, ")")
            }
            Value::Coll(CollKind::WrappedColl { elem_tpe, items }) => {
                write!(f, "Coll[{}](", elem_tpe)?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.fmt(f)?;
                }
                write!(f, ")")
            }
            Value::Opt(boxed_opt) => {
                if let Some(v) = &**boxed_opt {
                    write!(f, "Some(")?;
                    v.fmt(f)?;
                    write!(f, ")")
                } else {
                    write!(f, "None")
                }
            }
            Value::Tup(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.fmt(f)?;
                }
                write!(f, ")")
            }
            Value::Unit => write!(f, "()"),
            Value::Boolean(v) => v.fmt(f),
            Value::Byte(v) => v.fmt(f),
            Value::Short(v) => v.fmt(f),
            Value::Int(v) => v.fmt(f),
            Value::Long(v) => write!(f, "{}L", v),
            Value::BigInt(v) => v.fmt(f),
            Value::SigmaProp(v) => v.fmt(f),
            Value::GroupElement(v) => v.fmt(f),
            Value::AvlTree(v) => write!(f, "AvlTree({:?})", v),
            Value::CBox(v) => write!(f, "ErgoBox({:?})", v),
            Value::Context => write!(f, "CONTEXT"),
            Value::Header(_) => write!(f, "HEADER"),
            Value::PreHeader(_) => write!(f, "PREHEADER"),
            Value::Global => write!(f, "GLOBAL"),
            Value::Lambda(v) => write!(f, "{v:?}"),
        }
    }
}

/// Marker trait to select types which stored as Vec of wrapped Value's
pub trait StoreWrapped {}

impl StoreWrapped for bool {}
impl StoreWrapped for i16 {}
impl StoreWrapped for i32 {}
impl StoreWrapped for i64 {}
impl StoreWrapped for BigInt256 {}
impl StoreWrapped for Header {}
impl StoreWrapped for Arc<ErgoBox> {}
impl StoreWrapped for EcPoint {}
impl StoreWrapped for SigmaProp {}
impl<T: StoreWrapped> StoreWrapped for Option<T> {}
impl<T> StoreWrapped for Vec<T> {}
impl StoreWrapped for Value {}
impl StoreWrapped for Literal {}

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

#[allow(clippy::from_over_into)]
#[allow(clippy::unwrap_used)]
#[impl_for_tuples(2, 4)]
impl Into<Value> for Tuple {
    fn into(self) -> Value {
        let v: Vec<Value> = [for_tuples!(  #( Tuple.into() ),* )].to_vec();
        Value::Tup(v.try_into().unwrap())
    }
}

impl From<Vec<Arc<ErgoBox>>> for Value {
    fn from(v: Vec<Arc<ErgoBox>>) -> Self {
        Value::Coll(CollKind::WrappedColl {
            elem_tpe: SType::SBox,
            items: v.into_iter().map(|i| i.into()).collect(),
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

impl TryExtractFrom<Value> for Arc<ErgoBox> {
    fn try_extract_from(c: Value) -> Result<Self, TryExtractFromError> {
        match c {
            Value::CBox(b) => Ok(b),
            _ => Err(TryExtractFromError(format!(
                "expected ErgoBox, found {:?}",
                c
            ))),
        }
    }
}

impl TryExtractFrom<Value> for Header {
    fn try_extract_from(c: Value) -> Result<Self, TryExtractFromError> {
        match c {
            Value::Header(h) => Ok(*h),
            _ => Err(TryExtractFromError(format!(
                "expected Header, found {:?}",
                c
            ))),
        }
    }
}

impl TryExtractFrom<Value> for PreHeader {
    fn try_extract_from(c: Value) -> Result<Self, TryExtractFromError> {
        match c {
            Value::PreHeader(ph) => Ok(*ph),
            _ => Err(TryExtractFromError(format!(
                "expected PreHeader, found {:?}",
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

impl<T: TryExtractFrom<Value> + StoreWrapped, const N: usize> TryExtractFrom<Value> for [T; N] {
    fn try_extract_from(c: Value) -> Result<Self, TryExtractFromError> {
        match c {
            Value::Coll(coll) => match coll {
                CollKind::WrappedColl {
                    elem_tpe: _,
                    items: v,
                } => {
                    let v = v
                        .into_iter()
                        .map(T::try_extract_from)
                        .collect::<Result<Vec<_>, _>>()?;
                    let len = v.len();
                    v.try_into().map_err(|_| TryExtractFromError(format!("can't convert vec of {:?} with length of {:?} to array with length of {:?}", std::any::type_name::<T>(), len, N)))
                }
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
        use sigma_util::FromVecI8;
        Vec::<i8>::try_extract_from(v).map(Vec::<u8>::from_vec_i8)
    }
}

impl TryExtractFrom<Value> for Value {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        Ok(v)
    }
}

impl TryExtractFrom<Value> for BigInt256 {
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

impl TryExtractFrom<Value> for AvlTreeData {
    fn try_extract_from(v: Value) -> Result<Self, TryExtractFromError> {
        match v {
            Value::AvlTree(a) => Ok(*a),
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                std::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl<T: TryExtractFrom<Value> + StoreWrapped> TryExtractFrom<Vec<Value>> for Vec<T> {
    fn try_extract_from(v: Vec<Value>) -> Result<Self, TryExtractFromError> {
        v.into_iter().map(|it| it.try_extract_into::<T>()).collect()
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
                "expected Option, found {:?}",
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
