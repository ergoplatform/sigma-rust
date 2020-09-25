//! ErgoBoxCandidate builder
use sigma_tree::chain;
use wasm_bindgen::prelude::*;

use crate::contract::Contract;

use super::BoxValue;
use super::ErgoBoxCandidate;

/// ErgoBoxCandidate builder
#[wasm_bindgen]
pub struct ErgoBoxCandidateBuilder(chain::ergo_box::box_builder::ErgoBoxCandidateBuilder);

#[wasm_bindgen]
impl ErgoBoxCandidateBuilder {
    /// Create builder with required box parameters:
    /// `value` - amount of money associated with the box
    /// `contract` - guarding contract([`Contract`]), which should be evaluated to true in order
    /// to open(spend) this box
    /// `creation_height` - height when a transaction containing the box is created.
    /// It should not exceed height of the block, containing the transaction with this box.
    #[wasm_bindgen(constructor)]
    pub fn new(value: &BoxValue, contract: &Contract, creation_height: u32) -> Self {
        ErgoBoxCandidateBuilder(chain::ergo_box::box_builder::ErgoBoxCandidateBuilder::new(
            value.clone().into(),
            chain::contract::Contract::from(contract.clone()).get_ergo_tree(),
            creation_height,
        ))
    }

    /// Build the box candidate using default(`BoxValue::MIN_VALUE_PER_BOX_BYTE`) value for box value checks
    pub fn build(self) -> Result<ErgoBoxCandidate, JsValue> {
        self.build_with_custom_min_value_per_byte(
            chain::ergo_box::box_value::BoxValue::MIN_VALUE_PER_BOX_BYTE,
        )
    }

    /// Build the box candidate using provided `min_value_per_byte` for box value checks
    pub fn build_with_custom_min_value_per_byte(
        self,
        min_value_per_byte: u32,
    ) -> Result<ErgoBoxCandidate, JsValue> {
        self.0
            .build_with_custom_min_value_per_byte(min_value_per_byte)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(ErgoBoxCandidate)
    }
}
