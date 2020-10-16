//! ErgoBoxCandidate builder

use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

use crate::ast::Constant;
use crate::chain::token::Token;
use crate::chain::token::TokenAmount;
use crate::chain::token::TokenId;
use crate::serialization::SigmaSerializable;
use crate::ErgoTree;

use super::box_value::BoxValue;
use super::register::NonMandatoryRegisterId;
use super::register::NonMandatoryRegisters;
use super::register::NonMandatoryRegistersError;
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

    /// NonMandatoryRegisters error
    #[error("NonMandatoryRegisters error: {0}")]
    NonMandatoryRegistersError(#[from] NonMandatoryRegistersError),
}

/// ErgoBoxCandidate builder
#[derive(Debug, Clone)]
pub struct ErgoBoxCandidateBuilder {
    min_value_per_byte: u32,
    value: BoxValue,
    ergo_tree: ErgoTree,
    tokens: Vec<Token>,
    additional_registers: HashMap<NonMandatoryRegisterId, Constant>,
    creation_height: u32,
}

impl ErgoBoxCandidateBuilder {
    fn build_registers(&self) -> Result<NonMandatoryRegisters, ErgoBoxCandidateBuilderError> {
        Ok(NonMandatoryRegisters::new(
            self.additional_registers.clone(),
        )?)
    }

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
            additional_registers: HashMap::new(),
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
    pub fn calc_box_size_bytes(&self) -> Result<usize, ErgoBoxCandidateBuilderError> {
        let regs = self.build_registers()?;
        let b = ErgoBoxCandidate {
            value: self.value,
            ergo_tree: self.ergo_tree.clone(),
            tokens: self.tokens.clone(),
            additional_registers: regs,
            creation_height: self.creation_height,
        };
        Ok(b.sigma_serialise_bytes().len())
    }

    /// Calculate minimal box value for the current box serialized size(in bytes)
    pub fn calc_min_box_value(&self) -> Result<BoxValue, ErgoBoxCandidateBuilderError> {
        let box_size_bytes = self.calc_box_size_bytes()?;
        Ok(
            BoxValue::try_from(box_size_bytes as i64 * BoxValue::MIN_VALUE_PER_BOX_BYTE as i64)
                .unwrap(),
        )
    }

    /// Set register with a given id (R4-R9) to the given value
    pub fn set_register_value(&mut self, register_id: NonMandatoryRegisterId, value: Constant) {
        self.additional_registers.insert(register_id, value);
    }

    /// Returns register value for the given register id (R4-R9), or None if the register is empty
    pub fn register_value(&self, register_id: &NonMandatoryRegisterId) -> Option<&Constant> {
        self.additional_registers.get(register_id)
    }

    /// Delete register value(make register empty) for the given register id (R4-R9)
    pub fn delete_register_value(&mut self, register_id: &NonMandatoryRegisterId) {
        self.additional_registers.remove(register_id);
    }

    /// Mint token, as defined in https://github.com/ergoplatform/eips/blob/master/eip-0004.md
    /// `token` - token id(box id of the first input box in transaction) and token amount,
    /// `token_name` - token name (will be encoded in R4),
    /// `token_desc` - token description (will be encoded in R5),
    /// `num_decimals` - number of decimals (will be encoded in R6)
    pub fn mint_token(
        &mut self,
        token: Token,
        token_name: String,
        token_desc: String,
        num_decimals: usize,
    ) {
        self.tokens.push(token);
        // TODO: encode minted token info to registers
    }

    /// Add given token id and token amount
    pub fn add_token(&mut self, token_id: TokenId, amount: TokenAmount) {
        self.tokens.push(Token { token_id, amount });
    }

    /// Build the box candidate
    pub fn build(self) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
        // TODO: according to EIP4 if token is minted in this box there should be no other tokens
        let regs = self.build_registers()?;
        let b = ErgoBoxCandidate {
            value: self.value,
            ergo_tree: self.ergo_tree,
            tokens: self.tokens,
            additional_registers: regs,
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

    use NonMandatoryRegisterId::*;

    use crate::test_util::force_any_val;

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
        let new_value = BoxValue::SAFE_USER_MIN.checked_mul_u32(10).unwrap();
        let mut builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
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
        let estimated_box_size = builder.calc_box_size_bytes().unwrap();
        let b = builder.build().unwrap();
        assert_eq!(b.sigma_serialise_bytes().len(), estimated_box_size);
    }

    #[test]
    fn test_calc_min_box_value() {
        let builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
        assert!(builder.calc_min_box_value().unwrap() > BoxValue::MIN);
    }

    #[test]
    fn test_build_fail_box_value_too_low() {
        let builder = ErgoBoxCandidateBuilder::new(BoxValue::MIN, force_any_val::<ErgoTree>(), 1);
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_set_get_register_value() {
        let reg_value: Constant = 1i32.into();
        let mut builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
        assert!(builder.register_value(&R4).is_none());
        builder.set_register_value(R4, reg_value.clone());
        assert_eq!(builder.register_value(&R4).unwrap(), &reg_value);
        let b = builder.build().unwrap();
        assert_eq!(b.additional_registers.get(R4).unwrap(), &reg_value);
    }

    #[test]
    fn test_delete_register_value() {
        let reg_value: Constant = 1i32.into();
        let mut builder =
            ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, force_any_val::<ErgoTree>(), 1);
        builder.set_register_value(R4, reg_value);
        builder.delete_register_value(&R4);
        assert!(builder.register_value(&R4).is_none());
        let b = builder.build().unwrap();
        assert!(b.additional_registers.get(R4).is_none());
    }
}
