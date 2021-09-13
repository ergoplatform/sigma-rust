//! Constant(Literal) IR node

use crate::bigint256::BigInt256;
use crate::ir_ergo_box::IrBoxId;
use crate::mir::value::CollKind;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::{ProveDhTuple, ProveDlog};
use crate::sigma_protocol::{dlog_group::EcPoint, sigma_boolean::SigmaProp};
use crate::types::stuple::TupleItems;
use crate::types::stype::LiftIntoSType;
use crate::types::stype::SType;
use impl_trait_for_tuples::impl_for_tuples;
use std::convert::TryFrom;
use std::convert::TryInto;

mod constant_placeholder;

pub use constant_placeholder::*;

use super::value::NativeColl;
use super::value::StoreWrapped;

use thiserror::Error;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Constant
pub struct Constant {
    /// Constant type
    pub tpe: SType,
    /// Constant value
    pub v: Literal,
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Possible values for `Constant`
pub enum Literal {
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
    /// Big integer
    BigInt(BigInt256),
    /// Sigma property
    SigmaProp(Box<SigmaProp>),
    /// GroupElement
    GroupElement(Box<EcPoint>),
    /// Ergo box ID
    CBox(IrBoxId),
    /// Collection
    Coll(CollKind<Literal>),
    /// Option type
    Opt(Box<Option<Literal>>),
    /// Tuple (arbitrary type values)
    Tup(TupleItems<Literal>),
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

impl From<EcPoint> for Literal {
    fn from(v: EcPoint) -> Literal {
        Literal::GroupElement(Box::new(v))
    }
}

impl From<IrBoxId> for Literal {
    fn from(b: IrBoxId) -> Self {
        Literal::CBox(b)
    }
}

impl From<Vec<u8>> for Literal {
    fn from(v: Vec<u8>) -> Self {
        Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(
            v.into_iter().map(|b| b as i8).collect(),
        )))
    }
}

impl From<Vec<i8>> for Literal {
    fn from(v: Vec<i8>) -> Literal {
        Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(v)))
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
        Literal::Opt(Box::new(opt.map(|e| e.into())))
    }
}
impl From<bool> for Constant {
    fn from(v: bool) -> Constant {
        Constant {
            tpe: bool::stype(),
            v: v.into(),
        }
    }
}

impl From<i8> for Constant {
    fn from(v: i8) -> Constant {
        Constant {
            tpe: i8::stype(),
            v: v.into(),
        }
    }
}

impl From<i16> for Constant {
    fn from(v: i16) -> Constant {
        Constant {
            tpe: i16::stype(),
            v: v.into(),
        }
    }
}

impl From<i32> for Constant {
    fn from(v: i32) -> Constant {
        Constant {
            tpe: i32::stype(),
            v: v.into(),
        }
    }
}

impl From<i64> for Constant {
    fn from(v: i64) -> Constant {
        Constant {
            tpe: i64::stype(),
            v: v.into(),
        }
    }
}

impl From<SigmaProp> for Constant {
    fn from(v: SigmaProp) -> Constant {
        Constant {
            tpe: SType::SSigmaProp,
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

impl From<IrBoxId> for Constant {
    fn from(b: IrBoxId) -> Self {
        Constant {
            tpe: SType::SBox,
            v: b.into(),
        }
    }
}

impl From<Vec<u8>> for Constant {
    fn from(v: Vec<u8>) -> Self {
        Constant {
            tpe: SType::SColl(Box::new(SType::SByte)),
            v: v.into(),
        }
    }
}

impl From<Vec<i8>> for Constant {
    fn from(v: Vec<i8>) -> Constant {
        Constant {
            tpe: SType::SColl(Box::new(SType::SByte)),
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
            tpe: SType::SOption(Box::new(T::stype())),
            v: Literal::Opt(Box::new(opt.map(|e| e.into().v))),
        }
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

impl TryExtractFrom<Literal> for IrBoxId {
    fn try_extract_from(c: Literal) -> Result<Self, TryExtractFromError> {
        match c {
            Literal::CBox(b) => Ok(b),
            _ => Err(TryExtractFromError(format!(
                "expected IrErgoBox, found {:?}",
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

impl TryExtractFrom<Literal> for Vec<i8> {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::Coll(v) => match v {
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

impl TryExtractFrom<Literal> for Vec<u8> {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        use crate::util::FromVecI8;
        Vec::<i8>::try_extract_from(v).map(Vec::<u8>::from_vec_i8)
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
                std::any::type_name::<Self>(),
                v
            ))),
        }
    }
}

impl<T: TryExtractFrom<Literal>> TryExtractFrom<Literal> for Option<T> {
    fn try_extract_from(v: Literal) -> Result<Self, TryExtractFromError> {
        match v {
            Literal::Opt(opt) => opt.map(T::try_extract_from).transpose(),
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

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
#[allow(clippy::todo)]
/// Arbitrary impl
pub(crate) mod arbitrary {
    use std::convert::TryFrom;

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
            tpe: SType::SColl(Box::new(c.tpe.clone())),
            v: Literal::Coll(if c.tpe == SType::SByte {
                let mut values: Vec<i8> = Vec::with_capacity(length);
                let byte: i8 = c.v.try_extract_into().unwrap();
                for _ in 0..length {
                    values.push(byte);
                }
                CollKind::NativeColl(NativeColl::CollByte(values))
            } else {
                let mut values: Vec<Literal> = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(c.v.clone());
                }
                CollKind::WrappedColl {
                    elem_tpe: c.tpe,
                    items: values,
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
            // SType::SBox => {}
            // SType::SAvlTree => {}
            // SType::SOption(tpe) =>
            SType::SColl(elem_tpe) => match *elem_tpe {
                SType::SBoolean => vec(any::<bool>(), 0..400).prop_map_into().boxed(),
                SType::SByte => vec(any::<u8>(), 0..400).prop_map_into().boxed(),
                SType::SShort => vec(any::<i16>(), 0..400).prop_map_into().boxed(),
                SType::SInt => vec(any::<i32>(), 0..400).prop_map_into().boxed(),
                SType::SLong => vec(any::<i64>(), 0..400).prop_map_into().boxed(),
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
#[allow(clippy::panic)]
pub mod tests {
    use super::*;
    use core::fmt;
    use proptest::prelude::*;

    fn test_constant_roundtrip<T>(v: T)
    where
        T: TryExtractInto<T> + TryExtractFrom<Literal> + Into<Constant> + fmt::Debug + Eq + Clone,
    {
        let constant: Constant = v.clone().into();
        let v_extracted: T = constant.try_extract_into::<T>().unwrap();
        assert_eq!(v, v_extracted);
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
        fn vec_i8_roundtrip(v in any::<Vec<i8>>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_u8_roundtrip(v in any::<Vec<u8>>()) {
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
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_bigint_roundtrip(raw in any::<Vec<i64>>()) {
            let v: Vec<BigInt256> = raw.into_iter().map(BigInt256::from).collect();
            test_constant_roundtrip(v);
        }

        #[test]
        fn vec_option_bigint_roundtrip(raw in any::<Vec<i64>>()) {
            let v: Vec<Option<BigInt256>> = raw.into_iter().map(|i| Some(BigInt256::from(i))).collect();
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
            test_constant_roundtrip(v);
        }

        #[test]
        fn option_nested_tuple_type_roundtrip(v in any::<Option<(i64, bool)>>()) {
            test_constant_roundtrip(v);
        }


        #[test]
        fn tuple_primitive_types_roundtrip(v in any::<(i64, bool)>()) {
            test_constant_roundtrip(v);
        }

        #[test]
        fn tuple_nested_types_roundtrip(v in any::<(Option<i64>, Vec<SigmaProp>)>()) {
            test_constant_roundtrip(v);
        }

    }
}
