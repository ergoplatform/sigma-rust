use crate::ast::Constant;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use sigma_ser::serializer::SerializationError;
use std::collections::HashMap;

/// newtype for additional registers R4 - R9
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct NonMandatoryRegisterId(u8);

impl NonMandatoryRegisterId {
    /// starting index for non-mandatory registers
    pub const START_INDEX: u8 = 4;
    /// end index for non-mandatory registers
    pub const END_INDEX: u8 = 9;

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

    const REG_IDS: [NonMandatoryRegisterId; 6] = [
        NonMandatoryRegisterId::R4,
        NonMandatoryRegisterId::R5,
        NonMandatoryRegisterId::R6,
        NonMandatoryRegisterId::R7,
        NonMandatoryRegisterId::R8,
        NonMandatoryRegisterId::R9,
    ];

    /// get register by it's index
    /// `i` is expected to be in range [`START_INDEX`] to [`END_INDEX`] , otherwise panic
    pub fn get_by_index(i: usize) -> NonMandatoryRegisterId {
        assert!(
            i >= NonMandatoryRegisterId::START_INDEX as usize
                && i <= NonMandatoryRegisterId::END_INDEX as usize
        );
        NonMandatoryRegisterId::REG_IDS[i - NonMandatoryRegisterId::START_INDEX as usize].clone()
    }
}

/// Stores non-mandatory registers for the box
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NonMandatoryRegisters(Vec<Constant>);

/// Possible errors when building NonMandatoryRegisters
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NonMandatoryRegistersError {
    /// Set of register has invalid size(maximum [`NonMandatoryRegisters::MAX_SIZE`])
    InvalidSize(usize),
    /// Set of non-mandatory indexes are not densely packed
    NonDenselyPacked(u8),
}

impl NonMandatoryRegistersError {
    /// get detailed error message
    pub fn error_msg(&self) -> String {
        match self {
            NonMandatoryRegistersError::InvalidSize(size) => format!(
                "invalid non-mandatory registers size {} (expected {})",
                size,
                NonMandatoryRegisters::MAX_SIZE
            ),
            NonMandatoryRegistersError::NonDenselyPacked(reg_id) => format!(
                "non-mandatory registers are not densely packed, {} is missing in range [{} .. {}]",
                reg_id,
                NonMandatoryRegisterId::START_INDEX,
                NonMandatoryRegisterId::END_INDEX
            ),
        }
    }
}

impl NonMandatoryRegisters {
    /// Maximum number of non-mandatory registers
    pub const MAX_SIZE: usize = 6;

    /// Empty non-mandatory registers
    pub fn empty() -> NonMandatoryRegisters {
        NonMandatoryRegisters(vec![])
    }

    /// Create new from map
    pub fn new(
        _regs: HashMap<NonMandatoryRegisterId, Box<Constant>>,
    ) -> Result<NonMandatoryRegisters, NonMandatoryRegistersError> {
        // return error if size is incorrect and/or there is a gap
        // we assume non-mandatory indexes are densely packed from startingNonMandatoryIndex
        // this convention allows to save 1 byte for each register
        todo!()
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
    pub fn len(&self) -> u8 {
        self.0.len() as u8
    }

    /// Return true if non-mandatory registers set is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get register value
    pub fn get(&self, _reg_id: &NonMandatoryRegisterId) -> Option<Box<Constant>> {
        todo!()
    }

    /// Get ordered register values (first is R4, and so on, up to R9)
    pub fn get_ordered_values(&self) -> Vec<Constant> {
        self.0.clone()
    }
}

impl From<NonMandatoryRegistersError> for SerializationError {
    fn from(error: NonMandatoryRegistersError) -> Self {
        SerializationError::Misc(error.error_msg())
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
}
