//! ErgoBoxCandidate builder

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
            value,
            ergo_tree,
            tokens: vec![],
            additional_registers: NonMandatoryRegisters::empty(),
            creation_height,
        }
    }

    /// Build the box candidate using default(`BoxValue::MIN_VALUE_PER_BOX_BYTE`) value for box value checks
    pub fn build(self) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
        self.build_with_custom_min_value_per_byte(BoxValue::MIN_VALUE_PER_BOX_BYTE)
    }

    /// Build the box candidate using provided `min_value_per_byte` for box value checks
    pub fn build_with_custom_min_value_per_byte(
        self,
        min_value_per_byte: u32,
    ) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
        let b = ErgoBoxCandidate {
            value: self.value,
            ergo_tree: self.ergo_tree,
            tokens: self.tokens,
            additional_registers: self.additional_registers,
            creation_height: self.creation_height,
        };
        let box_size_bytes = b.sigma_serialise_bytes().len();
        let min_box_value: BoxValue = (box_size_bytes as i64 * min_value_per_byte as i64)
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
