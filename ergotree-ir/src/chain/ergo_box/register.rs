//! Box registers

use crate::mir::constant::Constant;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use derive_more::From;
use ergo_chain_types::Base16EncodedBytes;
use std::convert::TryInto;
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

/// Box register id (0-9)
#[derive(PartialEq, Eq, Debug, Clone, Copy, From)]
pub enum RegisterId {
    /// Id for mandatory registers (0-3)
    MandatoryRegisterId(MandatoryRegisterId),
    /// Id for non-mandotory registers (4-9)
    NonMandatoryRegisterId(NonMandatoryRegisterId),
}

impl RegisterId {
    /// R0 register id (box.value)
    pub const R0: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R0);
    /// R1 register id (serialized ErgoTree)
    pub const R1: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R1);
    /// R2 register id (tokens)
    pub const R2: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R2);
    /// R2 register id (creationInfo)
    pub const R3: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R3);
}

/// Register id out of bounds error (not in 0-9 range)
#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("register id {0} is out of bounds (0 - 9)")]
pub struct RegisterIdOutOfBounds(pub i8);

impl TryFrom<i8> for RegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(RegisterIdOutOfBounds(value));
        }
        let v = value as usize;
        if v < NonMandatoryRegisterId::START_INDEX {
            Ok(RegisterId::MandatoryRegisterId(value.try_into()?))
        } else if v <= NonMandatoryRegisterId::END_INDEX {
            Ok(RegisterId::NonMandatoryRegisterId(value.try_into()?))
        } else {
            Err(RegisterIdOutOfBounds(value))
        }
    }
}

impl TryFrom<u8> for RegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        RegisterId::try_from(value as i8)
    }
}

/// newtype for additional registers R4 - R9
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(into = "String", try_from = "String"))]
#[repr(u8)]
pub enum NonMandatoryRegisterId {
    /// id for R4 register
    R4 = 4,
    /// id for R5 register
    R5 = 5,
    /// id for R6 register
    R6 = 6,
    /// id for R7 register
    R7 = 7,
    /// id for R8 register
    R8 = 8,
    /// id for R9 register
    R9 = 9,
}

impl NonMandatoryRegisterId {
    /// starting index for non-mandatory registers
    pub const START_INDEX: usize = 4;
    /// end index for non-mandatory registers
    pub const END_INDEX: usize = 9;

    /// max number of registers
    pub const NUM_REGS: usize = 6;

    /// all register ids
    pub const REG_IDS: [NonMandatoryRegisterId; NonMandatoryRegisterId::NUM_REGS] = [
        NonMandatoryRegisterId::R4,
        NonMandatoryRegisterId::R5,
        NonMandatoryRegisterId::R6,
        NonMandatoryRegisterId::R7,
        NonMandatoryRegisterId::R8,
        NonMandatoryRegisterId::R9,
    ];

    /// get register by it's index starting from 0
    /// `i` is expected to be in range 0..[`Self::NUM_REGS`] , otherwise panic
    pub fn get_by_zero_index(i: usize) -> NonMandatoryRegisterId {
        assert!(i < NonMandatoryRegisterId::NUM_REGS);
        NonMandatoryRegisterId::REG_IDS[i]
    }
}

impl From<NonMandatoryRegisterId> for String {
    fn from(v: NonMandatoryRegisterId) -> Self {
        format!("R{}", v as u8)
    }
}

impl TryFrom<String> for NonMandatoryRegisterId {
    type Error = NonMandatoryRegisterIdParsingError;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        if str.len() == 2 && &str[..1] == "R" {
            let index = str[1..2]
                .parse::<usize>()
                .map_err(|_| NonMandatoryRegisterIdParsingError())?;
            if (NonMandatoryRegisterId::START_INDEX..=NonMandatoryRegisterId::END_INDEX)
                .contains(&index)
            {
                Ok(NonMandatoryRegisterId::get_by_zero_index(
                    index - NonMandatoryRegisterId::START_INDEX,
                ))
            } else {
                Err(NonMandatoryRegisterIdParsingError())
            }
        } else {
            Err(NonMandatoryRegisterIdParsingError())
        }
    }
}

impl TryFrom<i8> for NonMandatoryRegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        let v_usize = value as usize;
        if (NonMandatoryRegisterId::START_INDEX..=NonMandatoryRegisterId::END_INDEX)
            .contains(&v_usize)
        {
            Ok(NonMandatoryRegisterId::get_by_zero_index(
                v_usize - NonMandatoryRegisterId::START_INDEX,
            ))
        } else {
            Err(RegisterIdOutOfBounds(value))
        }
    }
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("failed to parse register id")]
/// Error for failed parsing of the register id from string
pub struct NonMandatoryRegisterIdParsingError();

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

    /// Return true if register value is parsed as a Constant
    pub fn is_parseable(&self, reg_id: NonMandatoryRegisterId) -> Option<bool> {
        self.0.get(reg_id as usize).map(|v| v.is_parseable())
    }

    /// Get register value as a Constant (returns None, if there is no value for the given register id or if it's an unparseable)
    pub fn get_constant(&self, reg_id: NonMandatoryRegisterId) -> Option<&Constant> {
        self.0
            .get(reg_id as usize - NonMandatoryRegisterId::START_INDEX)
            .and_then(|rv| rv.as_option_constant())
    }

    /// Get register value as bytes (returns None, if there is no value for the given register id)
    pub fn get_bytes(&self, reg_id: NonMandatoryRegisterId) -> Option<Vec<u8>> {
        self.0
            .get(reg_id as usize - NonMandatoryRegisterId::START_INDEX)
            .map(|rv| rv.sigma_serialize_bytes())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, From)]
pub(crate) enum RegisterValue {
    Parsed(Constant),
    Unparseable(Vec<u8>),
}

impl RegisterValue {
    pub fn as_option_constant(&self) -> Option<&Constant> {
        match self {
            RegisterValue::Parsed(c) => Some(c),
            RegisterValue::Unparseable(_) => None,
        }
    }

    #[allow(clippy::unwrap_used)] // it could only fail on OOM, etc.
    fn sigma_serialize_bytes(&self) -> Vec<u8> {
        match self {
            RegisterValue::Parsed(c) => c.sigma_serialize_bytes().unwrap(),
            RegisterValue::Unparseable(bytes) => bytes.clone(),
        }
    }

    fn is_parseable(&self) -> bool {
        match self {
            RegisterValue::Parsed(_) => true,
            RegisterValue::Unparseable(_) => false,
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
        for reg_value in self.0.iter() {
            let bytes = reg_value.sigma_serialize_bytes();
            w.write_all(&bytes)?;
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let regs_num = r.get_u8()?;
        let mut additional_regs = Vec::with_capacity(regs_num as usize);
        for _ in 0..regs_num {
            let v = Constant::sigma_parse(r)?;
            additional_regs.push(v);
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

/// Register ids that every box have (box properties exposed as registers)
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MandatoryRegisterId {
    /// Monetary value, in Ergo tokens
    R0 = 0,
    /// Guarding script
    R1 = 1,
    /// Secondary tokens
    R2 = 2,
    /// Reference to transaction and output id where the box was created
    R3 = 3,
}

impl TryFrom<i8> for MandatoryRegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            v if v == MandatoryRegisterId::R0 as i8 => Ok(MandatoryRegisterId::R0),
            v if v == MandatoryRegisterId::R1 as i8 => Ok(MandatoryRegisterId::R1),
            v if v == MandatoryRegisterId::R2 as i8 => Ok(MandatoryRegisterId::R2),
            v if v == MandatoryRegisterId::R3 as i8 => Ok(MandatoryRegisterId::R3),
            _ => Err(RegisterIdOutOfBounds(value)),
        }
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
pub(crate) mod arbitrary {
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for NonMandatoryRegisters {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(
                prop_oneof![
                    any::<Constant>().prop_map(RegisterValue::Parsed),
                    vec(any::<u8>(), 0..100).prop_map(RegisterValue::Unparseable)
                ],
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
                prop_assert_eq![regs.get_constant(*reg_id), hash_map.get(reg_id).unwrap().as_option_constant()];
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
}
