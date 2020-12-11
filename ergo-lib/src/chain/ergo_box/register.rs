//! Box registers

use crate::chain::json::ergo_box::ConstantHolder;
use crate::{ast::constant::Constant, serialization::SerializationError};
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

/// newtype for additional registers R4 - R9
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
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

impl Into<String> for NonMandatoryRegisterId {
    fn into(self) -> String {
        format!("R{}", self as u8)
    }
}

impl TryFrom<String> for NonMandatoryRegisterId {
    type Error = NonMandatoryRegisterIdParsingError;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        if str.len() == 2 && &str[..1] == "R" {
            let index = (&str[1..2])
                .parse::<usize>()
                .map_err(|_| NonMandatoryRegisterIdParsingError())?;
            if index >= NonMandatoryRegisterId::START_INDEX
                && index <= NonMandatoryRegisterId::END_INDEX
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

#[derive(Error, Debug)]
#[error("failed to parse register id")]
/// Error for failed parsing of the register id from string
pub struct NonMandatoryRegisterIdParsingError();

/// Stores non-mandatory registers for the box
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(
        into = "HashMap<NonMandatoryRegisterId, Constant>",
        try_from = "HashMap<NonMandatoryRegisterId, crate::chain::json::ergo_box::ConstantHolder>"
    )
)]
pub struct NonMandatoryRegisters(Vec<Constant>);

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
        NonMandatoryRegisters::try_from(regs)
    }

    /// Create new from ordered values (first element will be R4, and so on)
    pub fn from_ordered_values(
        values: Vec<Constant>,
    ) -> Result<NonMandatoryRegisters, NonMandatoryRegistersError> {
        if values.len() > NonMandatoryRegisters::MAX_SIZE {
            Err(NonMandatoryRegistersError::InvalidSize(values.len()))
        } else {
            Ok(NonMandatoryRegisters(values))
        }
    }

    /// Size of non-mandatory registers set
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return true if non-mandatory registers set is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get register value
    pub fn get(&self, reg_id: NonMandatoryRegisterId) -> Option<&Constant> {
        self.0
            .get(reg_id as usize - NonMandatoryRegisterId::START_INDEX)
    }

    /// Get ordered register values (first is R4, and so on, up to R9)
    pub fn get_ordered_values(&self) -> &Vec<Constant> {
        &self.0
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

impl Into<HashMap<NonMandatoryRegisterId, Constant>> for NonMandatoryRegisters {
    fn into(self) -> HashMap<NonMandatoryRegisterId, Constant> {
        self.0
            .into_iter()
            .enumerate()
            .map(|(i, c)| (NonMandatoryRegisterId::get_by_zero_index(i), c))
            .collect()
    }
}

impl TryFrom<HashMap<NonMandatoryRegisterId, Constant>> for NonMandatoryRegisters {
    type Error = NonMandatoryRegistersError;
    fn try_from(reg_map: HashMap<NonMandatoryRegisterId, Constant>) -> Result<Self, Self::Error> {
        let regs_num = reg_map.len();
        if regs_num > NonMandatoryRegisters::MAX_SIZE {
            Err(NonMandatoryRegistersError::InvalidSize(regs_num))
        } else {
            let mut res: Vec<Constant> = vec![];
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

impl TryFrom<HashMap<NonMandatoryRegisterId, ConstantHolder>> for NonMandatoryRegisters {
    type Error = NonMandatoryRegistersError;
    fn try_from(
        value: HashMap<NonMandatoryRegisterId, ConstantHolder>,
    ) -> Result<Self, Self::Error> {
        let cm: HashMap<NonMandatoryRegisterId, Constant> =
            value.into_iter().map(|(k, v)| (k, v.into())).collect();
        NonMandatoryRegisters::try_from(cm)
    }
}

impl From<NonMandatoryRegistersError> for SerializationError {
    fn from(error: NonMandatoryRegistersError) -> Self {
        SerializationError::Misc(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for NonMandatoryRegisters {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            vec(any::<Constant>(), 0..=NonMandatoryRegisterId::NUM_REGS)
                .prop_map(|constants| {
                    NonMandatoryRegisters::from_ordered_values(constants)
                        .expect("error building registers")
                })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn hash_map_roundtrip(regs in any::<NonMandatoryRegisters>()) {
            let hash_map: HashMap<NonMandatoryRegisterId, Constant> = regs.clone().into();
            let regs_from_map = NonMandatoryRegisters::try_from(hash_map);
            prop_assert![regs_from_map.is_ok()];
            prop_assert_eq![regs_from_map.unwrap(), regs];
        }

        #[test]
        fn get(regs in any::<NonMandatoryRegisters>()) {
            let hash_map: HashMap<NonMandatoryRegisterId, Constant> = regs.clone().into();
            hash_map.keys().try_for_each(|reg_id| {
                prop_assert_eq![regs.get(*reg_id), hash_map.get(reg_id)];
                Ok(())
            })?;
        }
    }

    #[test]
    fn test_empty() {
        assert!(NonMandatoryRegisters::empty().is_empty());
    }

    #[test]
    fn test_non_densely_packed_error() {
        let mut hash_map: HashMap<NonMandatoryRegisterId, Constant> = HashMap::new();
        hash_map.insert(NonMandatoryRegisterId::R4, 1i32.into());
        // gap, missing R5
        hash_map.insert(NonMandatoryRegisterId::R6, 1i32.into());
        assert!(NonMandatoryRegisters::try_from(hash_map).is_err());
    }
}
