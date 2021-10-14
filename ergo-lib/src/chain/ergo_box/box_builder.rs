//! ErgoBoxCandidate builder

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::BoxTokens;
use ergotree_ir::chain::ergo_box::ErgoBoxCandidate;
use ergotree_ir::chain::ergo_box::NonMandatoryRegisterId;
use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
use ergotree_ir::chain::ergo_box::NonMandatoryRegistersError;
use ergotree_ir::chain::token::Token;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::serialization::{SigmaSerializable, SigmaSerializationError};
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

    /// When minting token no other tokens should be in the box (according to EIP4)
    #[error("When minting token no other tokens should be in the box (according to EIP4)")]
    ExclusiveMintedTokenError,

    /// When minting token R4, R5, R6 register are holding issued token info(according to EIP4) and cannot be used
    #[error("R4, R5, R6 are holding issuing token info and cannot be used(found {0:?} are used)")]
    MintedTokenRegisterOverwriteError(NonMandatoryRegisterId),

    /// Serialization error
    #[error("serialization error: {0}")]
    SerializationError(#[from] SigmaSerializationError),
    /// When creating a Box, it can either have no tokens, or 1-255 tokens
    #[error("Too many Tokens. The maximum number of Tokens in an Ergo Box is 255")]
    TooManyTokensError,
}

/// Minted token info (id, amount, name, desc)
#[derive(Debug, Clone)]
struct MintingToken {
    token: Token,
    name: String,
    desc: String,
    num_decimals: usize,
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
    minting_token: Option<MintingToken>,
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
            additional_registers: HashMap::new(),
            creation_height,
            minting_token: None,
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
        let b = self.build_box()?;
        Ok(b.sigma_serialize_bytes()?.len())
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

    /// Mint token, as defined in <https://github.com/ergoplatform/eips/blob/master/eip-0004.md>
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
        self.minting_token = Some(MintingToken {
            token,
            name: token_name,
            desc: token_desc,
            num_decimals,
        });
    }

    /// Add given token id and token amount
    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn build_box(&self) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
        let mut tokens = self.tokens.clone();
        let mut additional_registers = self.additional_registers.clone();
        if let Some(minting_token) = self.minting_token.clone() {
            // according to EIP4 if token is minted in this box there should be no other tokens
            // https://github.com/ergoplatform/eips/blob/master/eip-0004.md
            if !self.tokens.is_empty() {
                return Err(ErgoBoxCandidateBuilderError::ExclusiveMintedTokenError);
            }
            if additional_registers
                .get(&NonMandatoryRegisterId::R4)
                .is_some()
            {
                return Err(
                    ErgoBoxCandidateBuilderError::MintedTokenRegisterOverwriteError(
                        NonMandatoryRegisterId::R4,
                    ),
                );
            }
            if additional_registers
                .get(&NonMandatoryRegisterId::R5)
                .is_some()
            {
                return Err(
                    ErgoBoxCandidateBuilderError::MintedTokenRegisterOverwriteError(
                        NonMandatoryRegisterId::R5,
                    ),
                );
            }
            if additional_registers
                .get(&NonMandatoryRegisterId::R6)
                .is_some()
            {
                return Err(
                    ErgoBoxCandidateBuilderError::MintedTokenRegisterOverwriteError(
                        NonMandatoryRegisterId::R6,
                    ),
                );
            }
            tokens.push(minting_token.token);
            additional_registers.insert(
                NonMandatoryRegisterId::R4,
                minting_token.name.as_bytes().to_vec().into(),
            );
            additional_registers.insert(
                NonMandatoryRegisterId::R5,
                minting_token.desc.as_bytes().to_vec().into(),
            );
            additional_registers.insert(
                NonMandatoryRegisterId::R6,
                minting_token
                    .num_decimals
                    .to_string()
                    .as_bytes()
                    .to_vec()
                    .into(),
            );
        }
        let regs = NonMandatoryRegisters::new(additional_registers)?;
        let tokens = if tokens.is_empty() {
            None
        } else {
            Some(
                BoxTokens::from_vec(tokens)
                    .map_err(|_| ErgoBoxCandidateBuilderError::TooManyTokensError)?,
            )
        };
        let b = ErgoBoxCandidate {
            value: self.value,
            ergo_tree: self.ergo_tree.clone(),
            tokens,
            additional_registers: regs,
            creation_height: self.creation_height,
        };
        let box_size_bytes = b.sigma_serialize_bytes()?.len();
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

    /// Build the box candidate
    pub fn build(self) -> Result<ErgoBoxCandidate, ErgoBoxCandidateBuilderError> {
        self.build_box()
    }
}

#[cfg(test)]
mod tests {

    use ergotree_ir::base16_str::Base16Str;
    use ergotree_ir::chain::token::TokenId;
    use sigma_test_util::force_any_val;
    use NonMandatoryRegisterId::*;

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
        assert_eq!(b.sigma_serialize_bytes().unwrap().len(), estimated_box_size);
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

    #[test]
    fn test_mint_token() {
        let token_pair = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 1.try_into().unwrap(),
        };
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let token_name = "USD".to_string();
        let token_desc = "Nothing backed USD token".to_string();
        let token_num_dec = 2;
        let mut box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        box_builder.mint_token(token_pair.clone(), token_name, token_desc, token_num_dec);
        let out_box = box_builder.build().unwrap();
        assert_eq!(out_box.tokens.unwrap().get(0).unwrap(), &token_pair);
        // test registers are encoded according to https://github.com/ergoplatform/eips/blob/master/eip-0004.md
        assert_eq!(
            out_box
                .additional_registers
                .get(NonMandatoryRegisterId::R4)
                .unwrap()
                .base16_str()
                .unwrap(),
            "0e03555344",
            "invalid encoding of token name in R4"
        );
        assert_eq!(
            out_box
                .additional_registers
                .get(NonMandatoryRegisterId::R5)
                .unwrap()
                .base16_str()
                .unwrap(),
            "0e184e6f7468696e67206261636b65642055534420746f6b656e",
            "invalid encoding of token description in R5"
        );
        assert_eq!(
            out_box
                .additional_registers
                .get(NonMandatoryRegisterId::R6)
                .unwrap()
                .base16_str()
                .unwrap(),
            "0e0132",
            "invalid encoding of token number of decimals in R6"
        );
    }

    #[test]
    fn test_mint_token_exclusivity_rule() {
        let token_pair = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 1.try_into().unwrap(),
        };
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let token_name = "USD".to_string();
        let token_desc = "Nothing backed USD token".to_string();
        let token_num_dec = 2;
        let mut box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        box_builder.mint_token(token_pair, token_name, token_desc, token_num_dec);
        box_builder.add_token(Token {
            token_id: force_any_val::<TokenId>(),
            amount: 1.try_into().unwrap(),
        });
        assert!(box_builder.build().is_err());
    }

    #[test]
    fn test_mint_token_register_overwrite() {
        let token_pair = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 1.try_into().unwrap(),
        };
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let token_name = "USD".to_string();
        let token_desc = "Nothing backed USD token".to_string();
        let token_num_dec = 2;
        vec![R4, R5, R6].iter().for_each(|r_id| {
            let mut box_builder =
                ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
            box_builder.mint_token(
                token_pair.clone(),
                token_name.clone(),
                token_desc.clone(),
                token_num_dec,
            );
            box_builder.set_register_value(*r_id, force_any_val::<Constant>());
            assert!(box_builder.build().is_err());
        });
    }

    #[test]
    fn test_add_token() {
        let token = Token {
            token_id: force_any_val::<TokenId>(),
            amount: 1.try_into().unwrap(),
        };
        let out_box_value = BoxValue::SAFE_USER_MIN;
        let mut box_builder =
            ErgoBoxCandidateBuilder::new(out_box_value, force_any_val::<ErgoTree>(), 0);
        box_builder.add_token(token.clone());
        let out_box = box_builder.build().unwrap();
        assert_eq!(out_box.tokens.unwrap().first(), &token);
    }
}
