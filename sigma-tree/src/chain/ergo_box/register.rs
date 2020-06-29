//! Box registers

use crate::ast::Constant;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::serializer::SerializationError;
use std::{collections::HashMap, convert::TryFrom};
use thiserror::Error;

/// newtype for additional registers R4 - R9
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "with-serde", serde(into = "String", try_from = "String"))]
pub struct NonMandatoryRegisterId(u8);

impl NonMandatoryRegisterId {
    /// starting index for non-mandatory registers
    pub const START_INDEX: usize = 4;
    /// end index for non-mandatory registers
    pub const END_INDEX: usize = 9;

    /// max number of registers
    pub const NUM_REGS: usize = 6;

    /// register R4
    pub const R4: NonMandatoryRegisterId = NonMandatoryRegisterId(4);
    /// register R5
    pub const R5: NonMandatoryRegisterId = NonMandatoryRegisterId(5);
    /// register R6
    pub const R6: NonMandatoryRegisterId = NonMandatoryRegisterId(6);
    /// register R7
    pub const R7: NonMandatoryRegisterId = NonMandatoryRegisterId(7);
    /// register R8
    pub const R8: NonMandatoryRegisterId = NonMandatoryRegisterId(8);
    /// register R9
    pub const R9: NonMandatoryRegisterId = NonMandatoryRegisterId(9);

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
    /// `i` is expected to be in range 0..[`NUM_REGS`] , otherwise panic
    pub fn get_by_index(i: usize) -> NonMandatoryRegisterId {
        assert!(i < NonMandatoryRegisterId::NUM_REGS);
        NonMandatoryRegisterId::REG_IDS[i].clone()
    }
}

impl Into<String> for NonMandatoryRegisterId {
    fn into(self) -> String {
        format!("R{}", self.0)
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
                Ok(NonMandatoryRegisterId::get_by_index(
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
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "with-serde",
    serde(
        into = "HashMap<NonMandatoryRegisterId, Constant>",
        try_from = "HashMap<NonMandatoryRegisterId, Constant>"
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
    pub fn get(&self, reg_id: &NonMandatoryRegisterId) -> Option<&Constant> {
        self.0
            .get(reg_id.0 as usize - NonMandatoryRegisterId::START_INDEX)
    }

    /// Get ordered register values (first is R4, and so on, up to R9)
    pub fn get_ordered_values(&self) -> &Vec<Constant> {
        &self.0
    }
}

/// Possible errors when building NonMandatoryRegisters
#[derive(Error, Debug)]
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
            .map(|(i, c)| (NonMandatoryRegisterId::get_by_index(i), c))
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
                    None => Err(NonMandatoryRegistersError::NonDenselyPacked(reg_id.0)),
                })?;
            Ok(NonMandatoryRegisters(res))
        }
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
            vec(any::<Constant>(), 0..7)
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
                prop_assert_eq![regs.get(reg_id), hash_map.get(reg_id)];
                Ok(())
            })?;
        }
    }
}
