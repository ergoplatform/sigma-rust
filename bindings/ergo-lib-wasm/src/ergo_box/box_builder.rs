//! ErgoBoxCandidate builder
use ergo_lib::chain;
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
            chain::contract::Contract::from(contract.clone()).ergo_tree(),
            creation_height,
        ))
    }

    /// Set minimal value (per byte of the serialized box size)
    pub fn set_min_box_value_per_byte(self, new_min_value_per_byte: u32) -> Self {
        ErgoBoxCandidateBuilder(self.0.set_min_box_value_per_byte(new_min_value_per_byte))
    }

    /// Get minimal value (per byte of the serialized box size)
    pub fn min_box_value_per_byte(&self) -> u32 {
        self.0.min_box_value_per_byte()
    }

    /// Set new box value
    pub fn set_value(self, new_value: BoxValue) -> Self {
        ErgoBoxCandidateBuilder(self.0.set_value(new_value.into()))
    }

    /// Get box value
    pub fn value(&self) -> BoxValue {
        (*self.0.value()).into()
    }

    /// Calculate serialized box size(in bytes)
    pub fn calc_box_size_bytes(&self) -> usize {
        self.0.calc_box_size_bytes()
    }

    /// Calculate minimal box value for the current box serialized size(in bytes)
    pub fn calc_min_box_value(&self) -> BoxValue {
        self.0.calc_min_box_value().into()
    }

    /// Build the box candidate
    pub fn build(self) -> Result<ErgoBoxCandidate, JsValue> {
        self.0
            .build()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(ErgoBoxCandidate)
    }
}
