//! Box registers

use crate::mir::constant::Constant;
use crate::mir::expr::Expr;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializationError;
use crate::serialization::SigmaSerializeResult;
use ergo_chain_types::Base16EncodedBytes;
use std::convert::TryInto;
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

mod id;
pub use id::*;

mod value;
pub use value::*;

/// Stores non-mandatory registers for the box
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        into = "HashMap<NonMandatoryRegisterId, ergo_chain_types::Base16EncodedBytes>",
        try_from = "HashMap<NonMandatoryRegisterId, crate::chain::json::ergo_box::ConstantHolder>"
    )
)]
pub struct NonMandatoryRegisters(Vec<RegisterValue>);

impl NonMandatoryRegisters {
    /// Maximum number of non-mandatory registers
    pub const MAX_SIZE: usize = NonMandatoryRegisterId::NUM_REGS;

    /// Empty non-mandatory registers
    pub fn empty() -> NonMandatoryRegisters {
        NonMandatoryRegisters(vec![])
    }

    /// Create new from map
    pub fn new(
        regs: HashMap<NonMandatoryRegisterId, Constant>,
    ) -> Result<NonMandatoryRegisters, NonMandatoryRegistersError> {
        NonMandatoryRegisters::try_from(
            regs.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<HashMap<NonMandatoryRegisterId, RegisterValue>>(),
        )
    }

    /// Size of non-mandatory registers set
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return true if non-mandatory registers set is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get register value (returns None, if there is no value for the given register id)
    pub fn get(&self, reg_id: NonMandatoryRegisterId) -> Option<&RegisterValue> {
        self.0.get(reg_id as usize)
    }

    /// Get register value as a Constant
    /// returns None, if there is no value for the given register id or an error if it's an unparseable
    pub fn get_constant(
        &self,
        reg_id: NonMandatoryRegisterId,
    ) -> Result<Option<Constant>, RegisterValueError> {
        match self
            .0
            .get(reg_id as usize - NonMandatoryRegisterId::START_INDEX)
        {
            Some(rv) => match rv.as_constant() {
                Ok(c) => Ok(Some(c.clone())),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }
}

/// Create new from ordered values (first element will be R4, and so on)
impl TryFrom<Vec<RegisterValue>> for NonMandatoryRegisters {
    type Error = NonMandatoryRegistersError;

    fn try_from(values: Vec<RegisterValue>) -> Result<Self, Self::Error> {
        if values.len() > NonMandatoryRegisters::MAX_SIZE {
            Err(NonMandatoryRegistersError::InvalidSize(values.len()))
        } else {
            Ok(NonMandatoryRegisters(values))
        }
    }
}

impl TryFrom<Vec<Constant>> for NonMandatoryRegisters {
    type Error = NonMandatoryRegistersError;

    fn try_from(values: Vec<Constant>) -> Result<Self, Self::Error> {
        NonMandatoryRegisters::try_from(
            values
                .into_iter()
                .map(RegisterValue::Parsed)
                .collect::<Vec<RegisterValue>>(),
        )
    }
}

impl SigmaSerializable for NonMandatoryRegisters {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        let regs_num = self.len();
        w.put_u8(regs_num as u8)?;
        for (idx, reg_value) in self.0.iter().enumerate() {
            match reg_value {
                RegisterValue::Parsed(c) => c.sigma_serialize(w)?,
                RegisterValue::ParsedTupleExpr(t) => t.as_tuple().sigma_serialize(w)?,
                RegisterValue::Invalid { bytes, error_msg } => {
                    let bytes_str = base16::encode_lower(bytes);
                    return Err(SigmaSerializationError::NotSupported(format!("unparseable register value at {0:?} (parsing error: {error_msg}) cannot be serialized in the stream (writer), because it cannot be parsed later. Register value as base16-encoded bytes: {bytes_str}", NonMandatoryRegisterId::get_by_zero_index(idx))));
                }
            };
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let regs_num = r.get_u8()?;
        let mut additional_regs = Vec::with_capacity(regs_num as usize);
        // TODO: include register id in the error message
        for _ in 0..regs_num {
            let expr = Expr::sigma_parse(r)?;
            let reg_val = match expr {
                Expr::Const(c) => RegisterValue::Parsed(c),
                Expr::Tuple(t) => RegisterValue::ParsedTupleExpr(EvaluatedTuple::new(t)?),
                _ => {
                    return Err(RegisterValueError::UnexpectedRegisterValue(format!(
                        "invalid register value: {0:?} (expected Constant or Tuple)",
                        expr
                    ))
                    .into())
                }
            };
            additional_regs.push(reg_val);
        }
        Ok(additional_regs.try_into()?)
    }
}

/// Possible errors when building NonMandatoryRegisters
#[derive(Error, PartialEq, Eq, Clone, Debug)]
pub enum NonMandatoryRegistersError {
    /// Set of register has invalid size(maximum [`NonMandatoryRegisters::MAX_SIZE`])
    #[error("invalid non-mandatory registers size ({0})")]
    InvalidSize(usize),
    /// Set of non-mandatory indexes are not densely packed
    #[error("registers are not densely packed (register R{0} is missing)")]
    NonDenselyPacked(u8),
}

impl From<NonMandatoryRegisters>
    for HashMap<NonMandatoryRegisterId, ergo_chain_types::Base16EncodedBytes>
{
    fn from(v: NonMandatoryRegisters) -> Self {
        v.0.into_iter()
            .enumerate()
            .map(|(i, reg_value)| {
                (
                    NonMandatoryRegisterId::get_by_zero_index(i),
                    // no way of returning an error without writing custom JSON serializer
                    #[allow(clippy::unwrap_used)]
                    Base16EncodedBytes::new(&reg_value.sigma_serialize_bytes()),
                )
            })
            .collect()
    }
}

impl From<NonMandatoryRegisters> for HashMap<NonMandatoryRegisterId, RegisterValue> {
    fn from(v: NonMandatoryRegisters) -> Self {
        v.0.into_iter()
            .enumerate()
            .map(|(i, reg_val)| (NonMandatoryRegisterId::get_by_zero_index(i), reg_val))
            .collect()
    }
}

impl TryFrom<HashMap<NonMandatoryRegisterId, RegisterValue>> for NonMandatoryRegisters {
    type Error = NonMandatoryRegistersError;
    fn try_from(
        reg_map: HashMap<NonMandatoryRegisterId, RegisterValue>,
    ) -> Result<Self, Self::Error> {
        let regs_num = reg_map.len();
        if regs_num > NonMandatoryRegisters::MAX_SIZE {
            Err(NonMandatoryRegistersError::InvalidSize(regs_num))
        } else {
            let mut res: Vec<RegisterValue> = vec![];
            NonMandatoryRegisterId::REG_IDS
                .iter()
                .take(regs_num)
                .try_for_each(|reg_id| match reg_map.get(reg_id) {
                    Some(v) => Ok(res.push(v.clone())),
                    None => Err(NonMandatoryRegistersError::NonDenselyPacked(*reg_id as u8)),
                })?;
            Ok(NonMandatoryRegisters(res))
        }
    }
}

#[cfg(feature = "json")]
impl TryFrom<HashMap<NonMandatoryRegisterId, crate::chain::json::ergo_box::ConstantHolder>>
    for NonMandatoryRegisters
{
    type Error = NonMandatoryRegistersError;
    fn try_from(
        value: HashMap<NonMandatoryRegisterId, crate::chain::json::ergo_box::ConstantHolder>,
    ) -> Result<Self, Self::Error> {
        let cm: HashMap<NonMandatoryRegisterId, RegisterValue> =
            value.into_iter().map(|(k, v)| (k, v.into())).collect();
        NonMandatoryRegisters::try_from(cm)
    }
}

impl From<NonMandatoryRegistersError> for SigmaParsingError {
    fn from(error: NonMandatoryRegistersError) -> Self {
        SigmaParsingError::Misc(error.to_string())
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
pub(crate) mod arbitrary {
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    #[derive(Default)]
    pub struct ArbNonMandatoryRegistersParams {
        pub allow_unparseable: bool,
    }

    impl Arbitrary for NonMandatoryRegisters {
        type Parameters = ArbNonMandatoryRegistersParams;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            vec(
                if params.allow_unparseable {
                    prop_oneof![
                        any::<Constant>().prop_map(RegisterValue::Parsed),
                        vec(any::<u8>(), 0..100).prop_map({
                            |bytes| RegisterValue::Invalid {
                                bytes,
                                error_msg: "unparseable".to_string(),
                            }
                        })
                    ]
                    .boxed()
                } else {
                    any::<Constant>().prop_map(RegisterValue::Parsed).boxed()
                },
                0..=NonMandatoryRegisterId::NUM_REGS,
            )
            .prop_map(|reg_values| NonMandatoryRegisters::try_from(reg_values).unwrap())
            .boxed()
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn hash_map_roundtrip(regs in any::<NonMandatoryRegisters>()) {
            let hash_map: HashMap<NonMandatoryRegisterId, RegisterValue> = regs.clone().into();
            let regs_from_map = NonMandatoryRegisters::try_from(hash_map);
            prop_assert![regs_from_map.is_ok()];
            prop_assert_eq![regs_from_map.unwrap(), regs];
        }

        #[test]
        fn get(regs in any::<NonMandatoryRegisters>()) {
            let hash_map: HashMap<NonMandatoryRegisterId, RegisterValue> = regs.clone().into();
            hash_map.keys().try_for_each(|reg_id| {
                prop_assert_eq![&regs.get_constant(*reg_id).unwrap().unwrap(), hash_map.get(reg_id).unwrap().as_constant().unwrap()];
                Ok(())
            })?;
        }

        #[test]
        fn reg_id_from_byte(reg_id_byte in 0i8..NonMandatoryRegisterId::END_INDEX as i8) {
            assert!(RegisterId::try_from(reg_id_byte).is_ok());
        }

        #[test]
        fn ser_roundtrip(regs in any::<NonMandatoryRegisters>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&regs), regs];
        }
    }

    #[test]
    fn test_empty() {
        assert!(NonMandatoryRegisters::empty().is_empty());
    }

    #[test]
    fn test_non_densely_packed_error() {
        let mut hash_map: HashMap<NonMandatoryRegisterId, RegisterValue> = HashMap::new();
        let c: Constant = 1i32.into();
        hash_map.insert(NonMandatoryRegisterId::R4, c.clone().into());
        // gap, missing R5
        hash_map.insert(NonMandatoryRegisterId::R6, c.into());
        assert!(NonMandatoryRegisters::try_from(hash_map).is_err());
    }

    #[test]
    fn test_tuple_expr_i700() {
        // // see https://github.com/ergoplatform/sigma-rust/issues/700
        // //
        // // below is an expression and not a constant encoded.
        // // 0x86 is the op code for Tuple expression following by 0x02 (size), 0x0266
        // // (SByte type 0x02, value 0x66), 0x0263 (SByte type 0x02, value 0x63).
        // let expr_bytes_str = "860202660263";
        // let expr_bytes = base16::decode(expr_bytes_str).unwrap();
        // let try_parse_constant = Constant::sigma_parse_bytes(&expr_bytes);
        // dbg!(&try_parse_constant);
        // if let Err(e) = &try_parse_constant {
        //     println!("Failed to parse constant: {}", e);
        // }
        // assert!(try_parse_constant.is_err());
        // assert!(matches!(
        //     try_parse_constant,
        //     Err(SigmaParsingError::InvalidTypeCode(134))
        // ));
        // let try_parse_expr: Expr = Expr::sigma_parse_bytes(&expr_bytes).unwrap();
        // dbg!(&try_parse_expr);
        // assert!(matches!(try_parse_expr, Expr::Tuple(_)));
        //
        // // now let's construct a proper constant for (102, 99) byte tuple
        // let expected_c: Constant = Constant {
        //     tpe: SType::STuple(STuple::pair(SType::SByte, SType::SByte)),
        //     v: Tup([Literal::Byte(102), Literal::Byte(99)].into()),
        // };
        // let expected_str = expected_c.base16_str().unwrap();
        // dbg!(expected_c.sigma_serialize_bytes().unwrap());
        // assert_eq!(expected_str, "566663");
    }
}
