//! ErgoBoxCandidate builder

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::chain::token::TokenAmount;
use crate::serialization::SigmaSerializable;
use crate::ErgoTree;

use super::box_value::BoxValue;
use super::register::NonMandatoryRegisters;
use super::ErgoBoxCandidate;
use thiserror::Error;

/// ErgoBoxCandidate builder errors
#[derive(Error, PartialEq, Eq, Clone, Debug)]
pub enum ErgoBoxCandidateBuilderError {
    /// Box value is too low
    #[error("Box value is too low, minimum value for box size of {box_size_bytes} bytes is: {min_box_value:?} nanoERGs")]
    BoxValueTooLow {
        /// minimum box value for that box size
        min_box_value: BoxValue,
        /// box size in bytes
        box_size_bytes: usize,
    },
}

/// ErgoBoxCandidate builder
pub struct ErgoBoxCandidateBuilder {
    min_value_per_byte: u32,
    value: BoxValue,
    ergo_tree: ErgoTree,
    tokens: Vec<TokenAmount>,
    additional_registers: NonMandatoryRegisters,
    creation_height: u32,
}

impl ErgoBoxCandidateBuilder {
    /// Create builder with required box parameters:
    /// `value` - box value in nanoErgs,
    /// `ergo_tree` - ErgoTree to guard this box,
    /// `current_height` - chain height that will be set in the built box
    pub fn new(value: BoxValue, ergo_tree: ErgoTree, creation_height: u32) -> Self {
        ErgoBoxCandidateBuilder {
            min_value_per_byte: BoxValue::MIN_VALUE_PER_BOX_BYTE,
            value,
            ergo_tree,
            tokens: vec![],
            additional_registers: NonMandatoryRegisters::empty(),
            creation_height,
        }
    }

    /// Set minimal value (per byte of the serialized box size)
    pub fn set_min_box_value_per_byte(&mut self, new_min_value_per_byte: u32) {
        self.min_value_per_byte = new_min_value_per_byte;
    }

    /// Get minimal value (per byte of the serialized box size)
    pub fn min_box_value_per_byte(&self) -> u32 {
        self.min_value_per_byte
    }

    /// Set new box value
    pub fn set_value(&mut self, new_value: BoxValue) {
        self.value = new_value;
    }

    /// Get box value
    pub fn value(&self) -> &BoxValue {
        &self.value
    }

    /// Calculate serialized box size(in bytes)
    pub fn calc_box_size_bytes(&self) -> usize {
        let b = ErgoBoxCandidate {
            value: self.value,
            ergo_tree: self.ergo_tree.clone(),
            tokens: self.tokens.clone(),
            additional_registers: self.additional_registers.clone(),
            creation_height: self.creation_height,
        };
        b.sigma_serialise_bytes().len()
    }

    /// Calculate minimal box value for the current box serialized size(in bytes)
    pub fn calc_min_box_value(&self) -> BoxValue {
        let box_size_bytes = self.calc_box_size_bytes();
        BoxValue::try_from(box_size_bytes as i64 * BoxValue::MIN_VALUE_PER_BOX_BYTE as i64).unwrap()
    }

    /// Build the box candidate
    pub fn build(self) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
        let b = ErgoBoxCandidate {
            value: self.value,
            ergo_tree: self.ergo_tree,
            tokens: self.tokens,
            additional_registers: self.additional_registers,
            creation_height: self.creation_height,
        };
        let box_size_bytes = b.sigma_serialise_bytes().len();
        let min_box_value: BoxValue = (box_size_bytes as i64 * self.min_value_per_byte as i64)
            .try_into()
            .unwrap();
        if self.value >= min_box_value {
            Ok(b)
        } else {
            Err(ErgoBoxCandidateBuilderError::BoxValueTooLow {
                min_box_value,
                box_size_bytes,
            })
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::tests::force_any_val;

    use super::*;

    #[test]
    fn test_build() {
        let tree = force_any_val::<ErgoTree>();
        let value = BoxValue::SAFE_USER_MIN;
        let builder = ErgoBoxCandidateBuilder::new(value, tree.clone(), 1);
        assert_eq!(builder.value(), &value);
        let b = builder.build().unwrap();
        assert_eq!(b.value, value);
        assert_eq!(b.ergo_tree, tree);
        assert_eq!(b.creation_height, 1);
    }

    #[test]
    fn test_set_value() {
        let mut builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
        let new_value = BoxValue::SAFE_USER_MIN.checked_mul_u32(10).unwrap();
        builder.set_value(new_value);
        assert_eq!(builder.value(), &new_value);
        let b = builder.build().unwrap();
        assert_eq!(b.value, new_value);
    }

    #[test]
    fn test_default_min_box_value() {
        let builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
        assert_eq!(
            builder.min_box_value_per_byte(),
            BoxValue::MIN_VALUE_PER_BOX_BYTE
        );
    }

    #[test]
    fn test_box_size_estimation() {
        let builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
        let estimated_box_size = builder.calc_box_size_bytes();
        let b = builder.build().unwrap();
        assert_eq!(b.sigma_serialise_bytes().len(), estimated_box_size);
    }

    #[test]
    fn test_calc_min_box_value() {
        let builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
        assert!(builder.calc_min_box_value() > BoxValue::MIN);
    }
}
