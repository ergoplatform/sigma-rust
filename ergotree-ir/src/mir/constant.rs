//! Constant(Literal) IR node

use crate::base16_str::Base16Str;
use crate::bigint256::BigInt256;
use crate::chain::ergo_box::ErgoBox;
use crate::chain::token::TokenId;
use crate::mir::value::CollKind;
use crate::reference::Ref;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializationError;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::SigmaProp;
use crate::sigma_protocol::sigma_boolean::{ProveDhTuple, ProveDlog};
use crate::types::stuple::STuple;
use crate::types::stuple::TupleItems;
use crate::types::stype::LiftIntoSType;
use crate::types::stype::SType;
use crate::unsignedbigint256::UnsignedBigInt;
use alloc::boxed::Box;

use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::fmt::Formatter;
use ergo_chain_types::ADDigest;
use ergo_chain_types::Base16DecodedBytes;
use ergo_chain_types::Digest32;
use ergo_chain_types::EcPoint;
use ergo_chain_types::Header;
use impl_trait_for_tuples::impl_for_tuples;
use sigma_util::AsVecI8;
use sigma_util::AsVecU8;

mod constant_placeholder;

pub use constant_placeholder::*;

use super::avl_tree_data::AvlTreeData;
use super::avl_tree_data::AvlTreeFlags;
use super::value::NativeColl;
use super::value::StoreWrapped;
use super::value::Value;

use thiserror::Error;

#[derive(PartialEq, Eq, Clone)]
/// Constant
pub struct Constant {
    /// Constant type
    pub tpe: SType,
    /// Constant value
    pub v: Literal,
}

#[derive(PartialEq, Eq, Clone)]
/// Possible values for `Constant`
pub enum Literal {
    /// Unit
    Unit,
    /// Boolean
    Boolean(bool),
    /// i8
    Byte(i8),
    /// Short
    Short(i16),
    /// Int
    Int(i32),
    /// Long
    Long(i64),
    /// String type
    String(Arc<str>),
    /// Big integer
    BigInt(BigInt256),
    /// Unsigned 256 BigInteger type
    UnsignedBigInt(UnsignedBigInt),
    /// Sigma property
    SigmaProp(Box<SigmaProp>),
    /// GroupElement
    GroupElement(Arc<EcPoint>),
    /// AVL tree
    AvlTree(Box<AvlTreeData>),
    /// Block Header type
    Header(Box<Header>),
    /// Ergo box
    CBox(Ref<'static, ErgoBox>),
    /// Collection
    Coll(CollKind<Literal>),
    /// Option type
    Opt(Option<Box<Literal>>),
    /// Tuple (arbitrary type values)
    Tup(TupleItems<Literal>),
}

impl Literal {
    fn tpe(&self) -> Result<SType, &'static str> {
        let tpe = match self {
            Literal::Unit => SType::SUnit,
            Literal::Boolean(_) => SType::SBoolean,
            Literal::Byte(_) => SType::SByte,
            Literal::Short(_) => SType::SShort,
            Literal::Int(_) => SType::SInt,
            Literal::Long(_) => SType::SLong,
            Literal::String(_) => SType::SString,
            Literal::BigInt(_) => SType::SBigInt,
            Literal::UnsignedBigInt(_) => SType::SUnsignedBigInt,
            Literal::SigmaProp(_) => SType::SSigmaProp,
            Literal::GroupElement(_) => SType::SGroupElement,
            Literal::AvlTree(_) => SType::SAvlTree,
            Literal::Header(_) => SType::SHeader,
            Literal::CBox(_) => SType::SBox,
            Literal::Coll(coll_kind) => SType::SColl(coll_kind.elem_tpe().clone().into()),
            Literal::Opt(Some(literal)) => SType::SOption(literal.tpe()?.into()),
            Literal::Opt(None) => return Err("Can not infer type of Option.None"),
            Literal::Tup(bounded_vec) => SType::STuple(STuple {
                items: bounded_vec.try_mapped_ref(Literal::tpe)?,
            }),
        };
        Ok(tpe)
    }
}

impl Constant {
    /// Create a new Constant::Coll from an iterator of constants.
    /// Returns an error if i is empty and tpe is not provided (type cannot be inferred), or the type for all values isn't the same
    pub fn coll_from_iter(
        i: impl IntoIterator<Item = Constant>,
        elem_tpe: Option<SType>,
    ) -> Result<Constant, TryExtractFromError> {
        let mut stype: Option<SType> = elem_tpe;
        let iter = i
            .into_iter()
            .map(|constant| {
                if let Some(ref stype) = stype {
                    if constant.tpe != *stype {
                        return Err(TryExtractFromError(format!(
                            "Constant.from_iter: expected tpe to be {:?}, found {:?}",
                            stype, constant.tpe
                        )));
                    }
                } else {
                    stype = Some(constant.tpe);
                }
                Ok(constant.v)
            })
            .collect::<Result<Arc<[_]>, TryExtractFromError>>()?;
        match stype {
            None => Err(TryExtractFromError(
                "Constant.from_iter does not support empty iterators".into(),
            )),
            Some(SType::SByte) => Ok(Constant {
                tpe: SType::SColl(SType::SByte.into()),
                v: Literal::Coll(CollKind::from_collection(SType::SByte, iter)?),
            }),
            Some(tpe) => Ok(Constant {
                tpe: SType::SColl(tpe.clone().into()),
                v: Literal::Coll(CollKind::from_collection(tpe, iter)?),
            }),
        }
    }
}

impl core::fmt::Debug for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        format!("{:?}: {:?}", self.v, self.tpe).fmt(f)
    }
}

impl core::fmt::Display for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.v.fmt(f)
    }
}

impl core::fmt::Debug for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(i8_bytes))) => {
                base16::encode_lower(&i8_bytes.as_vec_u8()).fmt(f)
            }
            Literal::Coll(CollKind::WrappedColl { elem_tpe: _, items }) => items.fmt(f),
            Literal::Opt(boxed_opt) => boxed_opt.fmt(f),
            Literal::Tup(items) => items.fmt(f),
            Literal::Unit => ().fmt(f),
            Literal::Boolean(v) => v.fmt(f),
            Literal::Byte(v) => v.fmt(f),
            Literal::Short(v) => v.fmt(f),
            Literal::Int(v) => v.fmt(f),
            Literal::Long(v) => v.fmt(f),
            Literal::BigInt(v) => v.fmt(f),
            Literal::SigmaProp(v) => v.fmt(f),
            Literal::GroupElement(v) => v.fmt(f),
            Literal::UnsignedBigInt(v) => v.fmt(f),
            Literal::AvlTree(v) => v.fmt(f),
            Literal::Header(v) => v.fmt(f),
            Literal::CBox(v) => v.fmt(f),
            Literal::String(v) => v.fmt(f),
        }
    }
}

impl core::fmt::Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(i8_bytes))) => {
                write!(f, "Coll[Byte](")?;
                for (i, b) in i8_bytes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", b)?;
                }
                write!(f, ")")
            }
            Literal::Coll(CollKind::WrappedColl { elem_tpe, items }) => {
                write!(f, "Coll[{}](", elem_tpe)?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.fmt(f)?;
                }
                write!(f, ")")
            }
            Literal::Opt(boxed_opt) => {
                if let Some(v) = boxed_opt {
                    write!(f, "Some(")?;
                    v.fmt(f)?;
                    write!(f, ")")
                } else {
                    write!(f, "None")
                }
            }
            Literal::Tup(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.fmt(f)?;
                }
                write!(f, ")")
            }
            Literal::Unit => write!(f, "()"),
            Literal::Boolean(v) => v.fmt(f),
            Literal::Byte(v) => v.fmt(f),
            Literal::Short(v) => v.fmt(f),
            Literal::Int(v) => v.fmt(f),
            Literal::Long(v) => write!(f, "{}L", v),
            Literal::BigInt(v) => v.fmt(f),
            Literal::SigmaProp(v) => v.fmt(f),
            Literal::UnsignedBigInt(v) => v.fmt(f),
            Literal::GroupElement(v) => v.fmt(f),
            Literal::AvlTree(v) => write!(f, "AvlTree({:?})", v),
            Literal::Header(v) => write!(f, "Header({:?})", v),
            Literal::CBox(v) => write!(f, "ErgoBox({:?})", v),
            Literal::String(v) => write!(f, "String({v})"),
        }
    }
}

impl From<()> for Literal {
    fn from(_: ()) -> Literal {
        Literal::Unit
    }
}

impl From<bool> for Literal {
    fn from(v: bool) -> Literal {
        Literal::Boolean(v)
    }
}

impl From<i8> for Literal {
    fn from(v: i8) -> Literal {
        Literal::Byte(v)
    }
}

impl From<i16> for Literal {
    fn from(v: i16) -> Literal {
        Literal::Short(v)
    }
}

impl From<i32> for Literal {
    fn from(v: i32) -> Literal {
        Literal::Int(v)
    }
}

impl From<i64> for Literal {
    fn from(v: i64) -> Literal {
        Literal::Long(v)
    }
}

impl From<BigInt256> for Literal {
    fn from(v: BigInt256) -> Literal {
        Literal::BigInt(v)
    }
}

impl From<SigmaProp> for Literal {
    fn from(v: SigmaProp) -> Literal {
        Literal::SigmaProp(Box::new(v))
    }
}

impl From<UnsignedBigInt> for Literal {
    fn from(v: UnsignedBigInt) -> Literal {
        Literal::UnsignedBigInt(v)
    }
}

impl From<EcPoint> for Literal {
    fn from(v: EcPoint) -> Literal {
        Literal::GroupElement(Arc::new(v))
    }
}

impl From<Ref<'_, EcPoint>> for Literal {
    fn from(value: Ref<'_, EcPoint>) -> Self {
        Literal::GroupElement(value.to_arc())
    }
}

impl From<Ref<'static, ErgoBox>> for Literal {
    fn from(b: Ref<'static, ErgoBox>) -> Self {
        Literal::CBox(b)
    }
}

impl From<ErgoBox> for Literal {
    fn from(b: ErgoBox) -> Self {
        Literal::CBox(Arc::new(b).into())
    }
}

impl From<Vec<u8>> for Literal {
    fn from(v: Vec<u8>) -> Self {
        Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(
            v.as_vec_i8().into(),
        ))) // TODO: optimize
    }
}

impl From<Digest32> for Literal {
    fn from(v: Digest32) -> Self {
        let bytes: Vec<u8> = v.into();
        Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(
            bytes.as_vec_i8().into(), // TODO: optimize
        )))
    }
}

impl From<TokenId> for Literal {
    fn from(v: TokenId) -> Self {
        Digest32::from(v).into()
    }
}

impl From<Vec<i8>> for Literal {
    fn from(v: Vec<i8>) -> Self {
        Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(v.into()))) // TODO
    }
}

impl<T: LiftIntoSType + StoreWrapped + Into<Literal>> From<Vec<T>> for Literal {
    fn from(v: Vec<T>) -> Self {
        Literal::Coll(CollKind::WrappedColl {
            elem_tpe: T::stype(),
            items: v.into_iter().map(|i| i.into()).collect(),
        })
    }
}

impl<T: LiftIntoSType + Into<Literal>> From<Option<T>> for Literal {
    fn from(opt: Option<T>) -> Self {
        Literal::Opt(opt.map(|e| e.into()).map(Box::new))
    }
}

impl<'ctx> TryFrom<Value<'ctx>> for Literal {
    type Error = String;
    #[allow(clippy::unwrap_used)]
    fn try_from(value: Value<'ctx>) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(b) => Ok(Literal::from(b)),
            Value::Byte(b) => Ok(Literal::from(b)),
            Value::Short(s) => Ok(Literal::from(s)),
            Value::Int(i) => Ok(Literal::from(i)),
            Value::Long(l) => Ok(Literal::from(l)),
            Value::BigInt(b) => Ok(Literal::from(b)),
            Value::Unit => Ok(Literal::Unit),
            Value::SigmaProp(s) => Ok(Literal::from(*s)),
            Value::UnsignedBigInt(b) => Ok(Literal::from(b)),
            Value::GroupElement(e) => Ok(Literal::from(e)),
            Value::CBox(i) => Ok(Literal::from(i.to_static())),
            Value::Coll(coll) => {
                let v = match coll {
                    CollKind::NativeColl(n) => Literal::Coll(CollKind::NativeColl(n)),
                    CollKind::WrappedColl { elem_tpe, items } => {
                        let new_items = items
                            .iter()
                            .map(|v| Literal::try_from(v.clone()))
                            .collect::<Result<Arc<[_]>, String>>()?;

                        Literal::Coll(CollKind::WrappedColl {
                            elem_tpe: elem_tpe.clone(),
                            items: new_items,
                        })
                    }
                };
                Ok(v)
            }
            Value::Opt(lit) => match lit {
                Some(v) => {
                    let v = Literal::try_from((*v).clone())?;
                    Ok(Literal::Opt(Some(Box::new(v))))
                }
                None => Ok(Literal::Opt(None)),
            },
            Value::Tup(t) => {
                if let Ok(t) = t.try_mapped::<_, _, String>(Literal::try_from) {
                    Ok(Literal::Tup(t))
                } else {
                    Err("Can't convert Value:Tup element".into())
                }
            }
            Value::AvlTree(a) => Ok(Literal::from(*a)),
            Value::String(s) => Ok(Literal::from(s)),
            Value::Header(h) => Ok(Literal::from(*h)),
            Value::Context => Err("Cannot convert Value::Context into Literal".into()),
            Value::PreHeader(_) => Err("Cannot convert Value::PreHeader(_) into Literal".into()),
            Value::Global => Err("Cannot convert Value::Global into Literal".into()),
            Value::Lambda(_) => Err("Cannot convert Value::Lambda(_) into Literal".into()),
        }
    }
}

impl TryFrom<Literal> for Constant {
    type Error = &'static str;

    fn try_from(v: Literal) -> Result<Self, Self::Error> {
        Ok(Self { tpe: v.tpe()?, v })
    }
}

impl<'ctx> TryFrom<Value<'ctx>> for Constant {
    type Error = String;
    #[allow(clippy::unwrap_used)]
    fn try_from(value: Value<'ctx>) -> Result<Self, Self::Error> {
        Literal::try_from(value)?.try_into().map_err(Into::into)
    }
}

impl From<()> for Constant {
    fn from(_: ()) -> Self {
        Constant {
            tpe: SType::SUnit,
            v: Literal::Unit,
        }
    }
}

impl From<bool> for Constant {
    fn from(v: bool) -> Self {
        Constant {
            tpe: bool::stype(),
            v: v.into(),
        }
    }
}

impl From<i8> for Constant {
    fn from(v: i8) -> Self {
        Constant {
            tpe: i8::stype(),
            v: v.into(),
        }
    }
}

impl From<i16> for Constant {
    fn from(v: i16) -> Self {
        Constant {
            tpe: i16::stype(),
            v: v.into(),
        }
    }
}

impl From<i32> for Constant {
    fn from(v: i32) -> Self {
        Constant {
            tpe: i32::stype(),
            v: v.into(),
        }
    }
}

impl From<i64> for Constant {
    fn from(v: i64) -> Self {
        Constant {
            tpe: i64::stype(),
            v: v.into(),
        }
    }
}

impl From<SigmaProp> for Constant {
    fn from(v: SigmaProp) -> Self {
        Constant {
            tpe: SType::SSigmaProp,
            v: v.into(),
        }
    }
}

impl From<UnsignedBigInt> for Constant {
    fn from(v: UnsignedBigInt) -> Self {
        Constant {
            tpe: SType::SUnsignedBigInt,
            v: v.into(),
        }
    }
}

impl From<EcPoint> for Constant {
    fn from(v: EcPoint) -> Constant {
        Constant {
            tpe: SType::SGroupElement,
            v: v.into(),
        }
    }
}

impl From<Ref<'_, EcPoint>> for Constant {
    fn from(v: Ref<'_, EcPoint>) -> Self {
        Constant {
            tpe: SType::SGroupElement,
            v: v.into(),
        }
    }
}

impl From<ErgoBox> for Constant {
    fn from(b: ErgoBox) -> Self {
        Constant {
            tpe: SType::SBox,
            v: Literal::CBox(Arc::new(b).into()),
        }
    }
}

impl From<Ref<'static, ErgoBox>> for Constant {
    fn from(b: Ref<'static, ErgoBox>) -> Self {
        Constant {
            tpe: SType::SBox,
            v: Literal::CBox(b),
        }
    }
}

impl From<Vec<u8>> for Constant {
    fn from(v: Vec<u8>) -> Self {
        Constant {
            tpe: SType::SColl(Arc::new(SType::SByte)),
            v: v.into(),
        }
    }
}

impl From<Digest32> for Constant {
    fn from(v: Digest32) -> Self {
        Constant {
            tpe: SType::SColl(Arc::new(SType::SByte)),
            v: v.into(),
        }
    }
}

impl From<TokenId> for Constant {
    fn from(v: TokenId) -> Self {
        Digest32::from(v).into()
    }
}

impl From<Vec<i8>> for Constant {
    fn from(v: Vec<i8>) -> Self {
        Constant {
            tpe: SType::SColl(Arc::new(SType::SByte)),
            v: v.into(),
        }
    }
}

impl<T: LiftIntoSType + StoreWrapped + Into<Constant>> From<Vec<T>> for Constant {
    fn from(v: Vec<T>) -> Self {
        Constant {
            tpe: Vec::<T>::stype(),
            v: Literal::Coll(CollKind::WrappedColl {
                elem_tpe: T::stype(),
                items: v.into_iter().map(|i| i.into().v).collect(),
            }),
        }
    }
}

impl<T: LiftIntoSType + Into<Constant>> From<Option<T>> for Constant {
    fn from(opt: Option<T>) -> Self {
        Constant {
            tpe: SType::SOption(Arc::new(T::stype())),
            v: Literal::Opt(opt.map(|e| e.into().v).map(Box::new)),
        }
    }
}

impl From<ProveDlog> for Literal {
    fn from(v: ProveDlog) -> Self {
        Literal::from(SigmaProp::from(SigmaBoolean::from(
            SigmaProofOfKnowledgeTree::from(v),
        )))
    }
}

impl From<ProveDhTuple> for Literal {
    fn from(dht: ProveDhTuple) -> Self {
        Literal::from(SigmaProp::from(SigmaBoolean::from(
            SigmaProofOfKnowledgeTree::from(dht),
        )))
    }
}

impl From<SigmaBoolean> for Literal {
    fn from(sb: SigmaBoolean) -> Self {
        Literal::from(SigmaProp::from(sb))
    }
}

impl From<AvlTreeData> for Literal {
    fn from(a: AvlTreeData) -> Self {
        Literal::AvlTree(Box::new(a))
    }
}

impl From<Header> for Literal {
    fn from(h: Header) -> Self {
        Literal::Header(h.into())
    }
}

impl From<AvlTreeFlags> for Literal {
    fn from(a: AvlTreeFlags) -> Self {
        Literal::Byte(a.serialize() as i8)
    }
}

impl From<Arc<str>> for Literal {
    fn from(s: Arc<str>) -> Self {
        Literal::String(s)
    }
}

impl From<ProveDlog> for Constant {
    fn from(v: ProveDlog) -> Self {
        Constant::from(SigmaProp::from(SigmaBoolean::from(
            SigmaProofOfKnowledgeTree::from(v),
        )))
    }
}

impl From<ProveDhTuple> for Constant {
    fn from(dht: ProveDhTuple) -> Self {
        Constant::from(SigmaProp::from(SigmaBoolean::from(
            SigmaProofOfKnowledgeTree::from(dht),
        )))
    }
}

impl From<SigmaBoolean> for Constant {
    fn from(sb: SigmaBoolean) -> Self {
        Constant::from(SigmaProp::from(sb))
    }
}

impl From<BigInt256> for Constant {
    fn from(b: BigInt256) -> Self {
        Constant {
            tpe: SType::SBigInt,
            v: Literal::BigInt(b),
        }
    }
}

impl From<AvlTreeData> for Constant {
    fn from(a: AvlTreeData) -> Self {
        Constant {
            tpe: SType::SAvlTree,
            v: Literal::AvlTree(Box::new(a)),
        }
    }
}

impl From<Header> for Constant {
    fn from(h: Header) -> Self {
        Constant {
            tpe: SType::SHeader,
            v: Literal::Header(h.into()),
        }
    }
}

impl From<AvlTreeFlags> for Constant {
    fn from(a: AvlTreeFlags) -> Self {
        Constant {
            tpe: SType::SByte,
            v: Literal::Byte(a.serialize() as i8),
        }
    }
}

impl From<ADDigest> for Constant {
    fn from(a: ADDigest) -> Self {
        Constant {
            tpe: SType::SColl(Arc::new(SType::SByte)),
            v: Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(
                a.0.iter().map(|&i| i as i8).collect(),
            ))),
        }
    }
}

impl From<Arc<str>> for Constant {
    fn from(s: Arc<str>) -> Self {
        Constant {
            tpe: SType::SString,
            v: Literal::String(s),
        }
    }
}

impl From<String> for Constant {
    fn from(s: String) -> Self {
        Constant {
            tpe: SType::SString,
            v: Literal::String(s.into()),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::from_over_into)]
#[impl_for_tuples(2, 4)]
impl Into<Constant> for Tuple {
    fn into(self) -> Constant {
        let constants: Vec<Constant> = [for_tuples!(  #( Tuple.into() ),* )].to_vec();
        let (types, values): (Vec<SType>, Vec<Literal>) =
            constants.into_iter().map(|c| (c.tpe, c.v)).unzip();
        Constant {
            tpe: SType::STuple(types.try_into().unwrap()),
            v: Literal::Tup(values.try_into().unwrap()),
        }
    }
}

/// Extract value wrapped in a type
pub trait TryExtractInto<F> {
    /// Extract value of the given type from any type (e.g. ['Constant'], [`super::value::Value`])
    /// on which [`TryExtractFrom`] is implemented
    fn try_extract_into<T: TryExtractFrom<F>>(self) -> Result<T, TryExtractFromError>;
}

impl<F> TryExtractInto<F> for F {
    fn try_extract_into<T: TryExtractFrom<F>>(self) -> Result<T, TryExtractFromError> {
        T::try_extract_from(self)
    }
}

/// Underlying type is different from requested value type
#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("Failed TryExtractFrom: {0}")]
pub struct TryExtractFromError(pub String);

/// Extract underlying value if type matches
pub trait TryExtractFrom<T>: Sized {
    /// Extract the value or return an error if type does not match
    fn try_extract_from(v: T) -> Result<Self, TryExtractFromError>;
}

impl<T: TryExtractFrom<Literal>> TryExtractFrom<Constant> for T {
    fn try_extract_from(cv: Constant) -> Result<Self, TryExtractFromError> {
        T::try_extract_from(cv.v)
    }
}

impl TryExtractFrom<Literal> for () {
    fn try_extract_from(cv: Literal) -> Result<(), TryExtractFromError> {
        match cv {
            Literal::Unit => Ok(()),
            _ => Err(TryExtractFromError(format!(
                "expected Unit, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for bool {
    fn try_extract_from(cv: Literal) -> Result<bool, TryExtractFromError> {
        match cv {
            Literal::Boolean(v) => Ok(v),
            _ => Err(TryExtractFromError(format!(
                "expected bool, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for i8 {
    fn try_extract_from(cv: Literal) -> Result<i8, TryExtractFromError> {
        match cv {
            Literal::Byte(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i8, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Literal> for i16 {
    fn try_extract_from(cv: Literal) -> Result<i16, TryExtractFromError> {
        match cv {
            Literal::Short(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i16, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Literal> for i32 {
    fn try_extract_from(cv: Literal) -> Result<i32, TryExtractFromError> {
        match cv {
            Literal::Int(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i32, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Literal> for i64 {
    fn try_extract_from(cv: Literal) -> Result<i64, TryExtractFromError> {
        match cv {
            Literal::Long(v) => Ok(v),
            _ => Err(TryExtractFromError(format!("expected i64, found {:?}", cv))),
        }
    }
}

impl TryExtractFrom<Literal> for EcPoint {
    fn try_extract_from(cv: Literal) -> Result<EcPoint, TryExtractFromError> {
        match cv {
            Literal::GroupElement(v) => Ok(*v),
            _ => Err(TryExtractFromError(format!(
                "expected EcPoint, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for SigmaProp {
    fn try_extract_from(cv: Literal) -> Result<SigmaProp, TryExtractFromError> {
        match cv {
            Literal::SigmaProp(v) => Ok(*v),
            _ => Err(TryExtractFromError(format!(
                "expected SigmaProp, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for UnsignedBigInt {
    fn try_extract_from(cv: Literal) -> Result<UnsignedBigInt, TryExtractFromError> {
        match cv {
            Literal::UnsignedBigInt(v) => Ok(v),
            _ => Err(TryExtractFromError(format!(
                "expected UnsignedBigInt, found {:?}",
                cv
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for Ref<'static, ErgoBox> {
    fn try_extract_from(c: Literal) -> Result<Self, TryExtractFromError> {
        match c {
            Literal::CBox(b) => Ok(b),
            _ => Err(TryExtractFromError(format!(
                "expected ErgoBox, found {:?}",
                c
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for ErgoBox {
    fn try_extract_from(c: Literal) -> Result<Self, TryExtractFromError> {
        match c {
            Literal::CBox(b) => Ok((*b).clone()),
            _ => Err(TryExtractFromError(format!(
                "expected ErgoBox, found {:?}",
                c
            ))),
        }
    }
}

impl<T: TryExtractFrom<Literal> + StoreWrapped> TryExtractFrom<Literal> for Vec<T> {
    fn try_extract_from(c: Literal) -> Result<Self, TryExtractFromError> {
        match c {
            Literal::Coll(coll) => match coll {
                CollKind::WrappedColl {
                    elem_tpe: _,
                    items: v,
                } => v.iter().cloned().map(T::try_extract_from).collect(),
                _ => Err(TryExtractFromError(format!(
                    "expected {:?}, found {:?}",
                    core::any::type_name::<Self>(),
                    coll
                ))),
            },
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                core::any::type_name::<Self>(),
                c
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for Vec<i8> {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::Coll(v) => match v {
                CollKind::NativeColl(NativeColl::CollByte(bs)) => Ok(bs.iter().copied().collect()), // TODO: optimize
                _ => Err(TryExtractFromError(format!(
                    "expected {:?}, found {:?}",
                    core::any::type_name::<Self>(),
                    v
                ))),
            },
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                core::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for Vec<u8> {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        use sigma_util::FromVecI8;
        Vec::<i8>::try_extract_from(v).map(Vec::<u8>::from_vec_i8)
    }
}

impl TryExtractFrom<Literal> for Digest32 {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        use sigma_util::FromVecI8;
        let bytes = Vec::<i8>::try_extract_from(v).map(Vec::<u8>::from_vec_i8)?;
        Digest32::try_from(bytes).map_err(|e| {
            TryExtractFromError(format!("failed to extract Digest32 with error: {:?}", e))
        })
    }
}

impl TryExtractFrom<Literal> for TokenId {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        Digest32::try_extract_from(v).map(Into::into)
    }
}

impl TryExtractFrom<Literal> for Literal {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        Ok(v)
    }
}

impl TryExtractFrom<Literal> for BigInt256 {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::BigInt(bi) => Ok(bi),
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                core::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for String {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::String(s) => Ok(String::from(&*s)),
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                core::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl TryExtractFrom<Literal> for AvlTreeData {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::AvlTree(a) => Ok(*a),
            _ => Err(TryExtractFromError(format!(
                "expected {:?}, found {:?}",
                core::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl<T: TryExtractFrom<Literal>> TryExtractFrom<Literal> for Option<T> {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::Opt(opt) => opt.map(|boxed| *boxed).map(T::try_extract_from).transpose(),
            _ => Err(TryExtractFromError(format!(
                "expected Option, found {:?}",
                v
            ))),
        }
    }
}

#[impl_for_tuples(2, 4)]
impl TryExtractFrom<Literal> for Tuple {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::Tup(items) => {
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

impl TryFrom<Literal> for ProveDlog {
    type Error = TryExtractFromError;
    fn try_from(cv: Literal) -> Result<Self, Self::Error> {
        match cv {
            Literal::SigmaProp(sp) => match sp.value() {
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

impl Base16Str for &Constant {
    fn base16_str(&self) -> Result<String, SigmaSerializationError> {
        self.sigma_serialize_bytes()
            .map(|bytes| base16::encode_lower(&bytes))
    }
}

impl Base16Str for Constant {
    fn base16_str(&self) -> Result<String, SigmaSerializationError> {
        self.sigma_serialize_bytes()
            .map(|bytes| base16::encode_lower(&bytes))
    }
}

impl TryFrom<Base16DecodedBytes> for Constant {
    type Error = SigmaParsingError;

    fn try_from(value: Base16DecodedBytes) -> Result<Self, Self::Error> {
        Constant::sigma_parse_bytes(&value.0)
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
#[allow(clippy::todo)]
/// Arbitrary impl
pub(crate) mod arbitrary {
    use core::convert::TryFrom;

    use super::*;
    use crate::mir::value::CollKind;
    use crate::types::stuple::STuple;
    use proptest::collection::vec;
    use proptest::prelude::*;

    extern crate derive_more;
    use derive_more::From;
    use derive_more::TryInto;

    fn primitive_type_value() -> BoxedStrategy<Constant> {
        prop_oneof![
            any::<bool>().prop_map_into(),
            any::<i8>().prop_map_into(),
            any::<i16>().prop_map_into(),
            any::<i32>().prop_map_into(),
            any::<i64>().prop_map_into(),
            any::<i64>().prop_map(|v| BigInt256::from(v).into()),
            any::<EcPoint>().prop_map_into(),
            any::<SigmaProp>().prop_map_into(),
            // although it's not strictly a primitive type, byte array is widely used as one
            vec(any::<i8>(), 0..100).prop_map_into(),
        ]
        .boxed()
    }

    fn coll_from_constant(c: Constant, length: usize) -> Constant {
        Constant {
            tpe: SType::SColl(Arc::new(c.tpe.clone())),
            v: Literal::Coll(if c.tpe == SType::SByte {
                let mut values: Vec<i8> = Vec::with_capacity(length);
                let byte: i8 = c.v.try_extract_into().unwrap();
                for _ in 0..length {
                    values.push(byte);
                }
                CollKind::NativeColl(NativeColl::CollByte(values.into())) // TODO: optimize
            } else {
                let mut values: Vec<Literal> = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(c.v.clone());
                }
                CollKind::WrappedColl {
                    elem_tpe: c.tpe,
                    items: values.into(),
                }
            }),
        }
    }

    fn const_with_type(tpe: SType) -> BoxedStrategy<Constant> {
        match tpe {
            SType::SAny => any::<Constant>(),
            SType::SBoolean => any::<bool>().prop_map_into().boxed(),
            SType::SByte => any::<i8>().prop_map_into().boxed(),
            SType::SShort => any::<i16>().prop_map_into().boxed(),
            SType::SInt => any::<i32>().prop_map_into().boxed(),
            SType::SLong => any::<i64>().prop_map_into().boxed(),
            SType::SBigInt => any::<i64>().prop_map(|v| BigInt256::from(v).into()).boxed(),
            SType::SGroupElement => any::<EcPoint>().prop_map_into().boxed(),
            SType::SSigmaProp => any::<SigmaProp>().prop_map_into().boxed(),
            SType::SBox => any::<ErgoBox>().prop_map_into().boxed(),
            SType::SAvlTree => any::<AvlTreeData>().prop_map_into().boxed(),
            // SType::SOption(tpe) =>
            SType::SOption(tpe) => match *tpe {
                SType::SBoolean => any::<Option<bool>>().prop_map_into().boxed(),
                SType::SByte => any::<Option<i8>>().prop_map_into().boxed(),
                SType::SShort => any::<Option<i16>>().prop_map_into().boxed(),
                SType::SInt => any::<Option<i32>>().prop_map_into().boxed(),
                SType::SLong => any::<Option<i64>>().prop_map_into().boxed(),
                _ => todo!(),
            },
            SType::SColl(elem_tpe) => match *elem_tpe {
                SType::SBoolean => vec(any::<bool>(), 0..400).prop_map_into().boxed(),
                SType::SByte => vec(any::<u8>(), 0..400).prop_map_into().boxed(),
                SType::SShort => vec(any::<i16>(), 0..400).prop_map_into().boxed(),
                SType::SInt => vec(any::<i32>(), 0..400).prop_map_into().boxed(),
                SType::SLong => vec(any::<i64>(), 0..400).prop_map_into().boxed(),
                SType::SSigmaProp => vec(any::<SigmaProp>(), 0..3).prop_map_into().boxed(),
                _ => todo!(),
            },
            // SType::STuple(_) => {}
            _ => todo!("{0:?} not yet implemented", tpe),
        }
    }

    impl Default for ArbConstantParams {
        fn default() -> Self {
            ArbConstantParams::AnyWithDepth(1)
        }
    }

    /// Parameters for arbitrary Constant generation
    #[derive(PartialEq, Eq, Debug, Clone, From, TryInto)]
    pub enum ArbConstantParams {
        /// Constant of any type with a structrure of a given depth
        AnyWithDepth(u8),
        /// Constant of a given type
        Exact(SType),
    }

    impl Arbitrary for Constant {
        type Parameters = ArbConstantParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            match args {
                ArbConstantParams::AnyWithDepth(depth) => {
                    prop_oneof![primitive_type_value().prop_recursive(
                        depth as u32,
                        16,
                        8,
                        |elem| {
                            prop_oneof![
                                // Coll[_]
                                elem.clone().prop_map(|c| coll_from_constant(c, 0)),
                                elem.clone().prop_map(|c| coll_from_constant(c, 1)),
                                elem.clone().prop_map(|c| coll_from_constant(c, 2)),
                                elem.clone().prop_map(|c| coll_from_constant(c, 10)),
                                // no Option[_] since it cannot be serialized (for now)
                                // // Some(v)
                                // elem.clone().prop_map(|c| Constant {
                                //     tpe: SType::SOption(Box::new(c.tpe)),
                                //     v: Value::Opt(Box::new(Some(c.v)))
                                // }),
                                // // None
                                // elem.prop_map(|c| Constant {
                                //     tpe: SType::SOption(Box::new(c.tpe)),
                                //     v: Value::Opt(Box::new(None))
                                // })

                                // Tuple
                                vec(elem, 2..=4).prop_map(|constants| Constant {
                                    tpe: SType::STuple(
                                        STuple::try_from(
                                            constants
                                                .clone()
                                                .into_iter()
                                                .map(|c| c.tpe)
                                                .collect::<Vec<SType>>()
                                        )
                                        .unwrap()
                                    ),
                                    v: Literal::Tup(
                                        constants
                                            .into_iter()
                                            .map(|c| c.v)
                                            .collect::<Vec<Literal>>()
                                            .try_into()
                                            .unwrap()
                                    )
                                }),
                            ]
                        }
                    )]
                    .boxed()
                }
                ArbConstantParams::Exact(tpe) => const_with_type(tpe),
            }
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use crate::{
        ergo_tree::ErgoTreeVersion,
        serialization::{data::MAX_STRING_LEN, roundtrip_new_feature},
    };

    use super::*;
    use core::fmt;
    use proptest::prelude::*;

    fn test_constant_roundtrip<T>(v: T)
    where
        T: TryExtractInto<T>
            + TryExtractFrom<Literal>
            + Into<Constant>
            + fmt::Debug
            + Eq
            + Clone
            + 'static,
    {
        let constant: Constant = v.clone().into();
        let v_extracted: T = constant.try_extract_into::<T>().unwrap();
        assert_eq!(v, v_extracted);
    }

    #[test]
    fn unit_roundtrip() {
        test_constant_roundtrip(());
    }

    // test that invalid strings don't error but are instead parsed lossily to match reference impl
    #[test]
    fn parse_invalid_string() {
        let mut bytes = Constant::from(".".to_string())
            .sigma_serialize_bytes()
            .unwrap();
        *bytes.last_mut().unwrap() = 0xf0;
        assert_eq!(
            Constant::sigma_parse_bytes(&bytes).unwrap().v,
            Literal::String("�".into())
        );
    }

    #[test]
    fn string_max_len_boundary_roundtrip() {
        let s = "a".repeat(MAX_STRING_LEN as usize);
        let constant = Constant::from(s.clone());
        let bytes = constant.sigma_serialize_bytes().unwrap();
        let parsed = Constant::sigma_parse_bytes(&bytes).unwrap();
        assert_eq!(parsed.v, Literal::String(s.into()));
    }

    #[test]
    fn string_len_over_max_rejected_on_parse() {
        // serialized: [SSTRING typecode = 102, VLQ u32 len = 4097 = 0x81 0x20, one payload byte]
        let bytes: Vec<u8> = vec![102, 0x81, 0x20, b'a'];
        assert!(matches!(
            Constant::sigma_parse_bytes(&bytes),
            Err(SigmaParsingError::ValueOutOfBounds(_))
        ));
    }

    #[test]
    fn string_len_huge_declared_rejected_on_parse() {
        // serialized: [SSTRING typecode = 102, VLQ u32 len = u32::MAX = 0xFF 0xFF 0xFF 0xFF 0x0F,
        // one payload byte]; must be rejected before any allocation (pre-fix code pre-allocated
        // the declared length, ~4GiB, which aborts on 32-bit/WASM targets)
        let bytes: Vec<u8> = vec![102, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, b'a'];
        assert!(matches!(
            Constant::sigma_parse_bytes(&bytes),
            Err(SigmaParsingError::ValueOutOfBounds(_))
        ));
    }

    #[test]
    fn string_len_over_max_rejected_on_serialize() {
        let constant = Constant::from("a".repeat(MAX_STRING_LEN as usize + 1));
        assert!(matches!(
            constant.sigma_serialize_bytes(),
            Err(SigmaSerializationError::NotSupported(_))
        ));
    }

    #[test]
    fn constant_from_iter() {
        let iter1 = [
            Constant::from(1i32),
            Constant::from(2i32),
            Constant::from(3i32),
        ];
        assert_eq!(
            Constant::coll_from_iter(iter1, None).unwrap(),
            Constant {
                tpe: SType::SColl(SType::SInt.into()),
                v: Literal::Coll(CollKind::WrappedColl {
                    elem_tpe: SType::SInt,
                    items: [
                        Literal::from(1i32),
                        Literal::from(2i32),
                        Literal::from(3i32)
                    ]
                    .into()
                })
            }
        );
        assert!(Constant::coll_from_iter([], None).is_err());
        assert!(Constant::coll_from_iter([], Some(SType::SBoolean)).is_ok());
        // tpe mismatch
        assert!(Constant::coll_from_iter(
            [
                Constant::from(1i32),
                Constant::from(2i64),
                Constant::from(3i64)
            ],
            None
        )
        .is_err());
    }

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(8))]

        #[test]
        fn bool_roundtrip(v in any::<bool>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn i8_roundtrip(v in any::<i8>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn i16_roundtrip(v in any::<i16>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn i32_roundtrip(v in any::<i32>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn i64_roundtrip(v in any::<i64>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn bigint_roundtrip(raw in any::<i64>()) {
            let v = BigInt256::from(raw);
            test_constant_roundtrip(v);
        }

        #[test]
        fn group_element_roundtrip(v in any::<EcPoint>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn sigma_prop_roundtrip(v in any::<SigmaProp>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn unsigned_bigint_roundtrip(v in any::<UnsignedBigInt>()) {
             test_constant_roundtrip(v);
        }

        #[test]
        fn vec_i8_roundtrip(v in any::<Vec<i8>>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_u8_roundtrip(v in any::<Vec<u8>>()) {
            // eprintln!("{:?}", Constant::from(v.clone()));
            test_constant_roundtrip(v);
        }

        #[test]
        fn token_id_roundtrip(v in any::<TokenId>()) {
            // eprintln!("{:?}", Constant::from(v.clone()));
            test_constant_roundtrip(v);
        }

        #[test]
        fn digest32_roundtrip(v in any::<Digest32>()) {
            test_constant_roundtrip(v);
        }


        #[test]
        fn vec_i16_roundtrip(v in any::<Vec<i16>>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_i32_roundtrip(v in any::<Vec<i32>>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_i64_roundtrip(v in any::<Vec<i64>>()) {
            // eprintln!("{:?}", Constant::from(v.clone()));
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_bigint_roundtrip(raw in any::<Vec<i64>>()) {
            let v: Vec<BigInt256> = raw.into_iter().map(BigInt256::from).collect();
            // eprintln!("{:?}", Constant::from(v.clone()));
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_option_bigint_roundtrip(raw in any::<Vec<i64>>()) {
            let v: Vec<Option<BigInt256>> = raw.into_iter().map(|i| Some(BigInt256::from(i))).collect();
            // eprintln!("{:?}", Constant::from(v.clone()));
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_sigmaprop_roundtrip(v in any::<Vec<SigmaProp>>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn option_primitive_type_roundtrip(v in any::<Option<i64>>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn option_nested_vector_type_roundtrip(v in any::<Option<Vec<(i64, bool)>>>()) {
            // eprintln!("{:?}", Constant::from(v.clone()));
            test_constant_roundtrip(v);
        }

        #[test]
        fn option_nested_tuple_type_roundtrip(v in any::<Option<(i64, bool)>>()) {
            test_constant_roundtrip(v);
        }


        #[test]
        fn tuple_primitive_types_roundtrip(v in any::<(i64, bool)>()) {
            // let constant: Constant = v.into();
            // eprintln!("{:?}", constant);
            test_constant_roundtrip(v);
        }

        #[test]
        fn tuple_nested_types_roundtrip(v in any::<(Option<i64>, Vec<SigmaProp>)>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn string_roundtrip(v in any::<String>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn header_ser_roundtrip(h in any::<Header>()) {
            roundtrip_new_feature(&Constant::from(h), ErgoTreeVersion::V3);
        }

    }
}
