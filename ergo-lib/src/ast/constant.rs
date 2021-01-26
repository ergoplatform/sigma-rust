//! Constant(Literal) IR node

use crate::chain::ergo_box::BoxValue;
use crate::chain::ergo_box::ErgoBox;
use crate::chain::{Base16DecodedBytes, Base16EncodedBytes};
use crate::types::stype::LiftIntoSType;
use crate::types::stype::SType;
use crate::{
    serialization::{SerializationError, SigmaSerializable},
    sigma_protocol::{dlog_group::EcPoint, sigma_boolean::SigmaProp},
};
use impl_trait_for_tuples::impl_for_tuples;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;

mod constant_placeholder;

pub(crate) use constant_placeholder::*;

use super::value::CollKind;
use super::value::CollPrim;
use super::value::StoredNonPrimitive;
use super::value::Value;

use thiserror::Error;

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
    pub v: Value,
}

impl From<Constant> for Base16EncodedBytes {
    fn from(v: Constant) -> Base16EncodedBytes {
        Base16EncodedBytes::new(&v.sigma_serialize_bytes())
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

impl From<ErgoBox> for Constant {
    fn from(b: ErgoBox) -> Self {
        Constant {
            tpe: SType::SBox,
            v: b.into(),
        }
    }
}

impl From<BoxValue> for Constant {
    fn from(v: BoxValue) -> Self {
        v.as_i64().into()
    }
}

impl From<Vec<u8>> for Constant {
    fn from(v: Vec<u8>) -> Self {
        Constant {
            tpe: SType::SColl(Box::new(SType::SByte)),
            v: Value::Coll(CollKind::Primitive(CollPrim::CollByte(
                v.into_iter().map(|b| b as i8).collect(),
            ))),
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

impl<T: LiftIntoSType + StoredNonPrimitive + Into<Value>> From<Vec<T>> for Constant {
    fn from(v: Vec<T>) -> Self {
        Constant {
            tpe: Vec::<T>::stype(),
            v: v.into(),
        }
    }
}

impl<T: LiftIntoSType + Into<Value>> From<Option<T>> for Constant {
    fn from(opt: Option<T>) -> Self {
        Constant {
            tpe: SType::SOption(Box::new(T::stype())),
            v: opt.into(),
        }
    }
}

#[impl_for_tuples(2, 4)]
impl Into<Constant> for Tuple {
    fn into(self) -> Constant {
        let constants: Vec<Constant> = [for_tuples!(  #( Tuple.into() ),* )].to_vec();
        let (types, values): (Vec<SType>, Vec<Value>) =
            constants.into_iter().map(|c| (c.tpe, c.v)).unzip();
        Constant {
            tpe: SType::STuple(types.try_into().unwrap()),
            v: Value::Tup(values.try_into().unwrap()),
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

impl<T: TryExtractFrom<Value>> TryExtractFrom<Constant> for T {
    fn try_extract_from(cv: Constant) -> Result<Self, TryExtractFromError> {
        T::try_extract_from(cv.v)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use crate::types::stuple::STuple;

    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    fn primitive_type_value() -> BoxedStrategy<Constant> {
        prop_oneof![
            any::<bool>().prop_map_into(),
            any::<i8>().prop_map_into(),
            any::<i16>().prop_map_into(),
            any::<i32>().prop_map_into(),
            any::<i64>().prop_map_into(),
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
            v: Value::Coll(if c.tpe == SType::SByte {
                let mut values: Vec<i8> = Vec::with_capacity(length);
                let byte: i8 = c.v.try_extract_into().unwrap();
                for _ in 0..length {
                    values.push(byte);
                }
                CollKind::Primitive(CollPrim::CollByte(values))
            } else {
                let mut values: Vec<Value> = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(c.v.clone());
                }
                CollKind::NonPrimitive {
                    elem_tpe: c.tpe,
                    v: values,
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
            // SType::SBigInt => {}
            SType::SGroupElement => any::<EcPoint>().prop_map_into().boxed(),
            SType::SSigmaProp => any::<SigmaProp>().prop_map_into().boxed(),
            // SType::SBox => {}
            // SType::SAvlTree => {}
            // SType::SOption(tpe) =>
            SType::SColl(elem_tpe) => match *elem_tpe {
                SType::SByte => vec(any::<u8>(), 0..400).prop_map_into().boxed(),
                SType::SBoolean => vec(any::<bool>(), 0..400).prop_map_into().boxed(),
                _ => todo!(),
            },
            // SType::STuple(_) => {}
            _ => todo!(),
        }
    }

    impl Default for SType {
        fn default() -> Self {
            SType::SAny
        }
    }

    impl Arbitrary for Constant {
        type Parameters = SType;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(tpe: Self::Parameters) -> Self::Strategy {
            match tpe {
                SType::SAny => {
                    prop_oneof![primitive_type_value().prop_recursive(3, 16, 8, |elem| {
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
                                v: Value::Tup(
                                    constants
                                        .into_iter()
                                        .map(|c| c.v)
                                        .collect::<Vec<Value>>()
                                        .try_into()
                                        .unwrap()
                                )
                            }),
                        ]
                    })]
                    .boxed()
                }
                _ => const_with_type(tpe),
            }
        }
    }

    fn test_constant_roundtrip<T>(v: T)
    where
        T: TryExtractInto<T> + TryExtractFrom<Value> + Into<Constant> + fmt::Debug + Eq + Clone,
    {
        let constant: Constant = v.clone().into();
        let v_extracted: T = constant.try_extract_into::<T>().unwrap();
        assert_eq!(v, v_extracted);
    }

    proptest! {

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
