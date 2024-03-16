//! Blockchain parameters. This module defines adjustable blockchain parameters that can be voted on by miners

use ergo_lib::chain::parameters;
use wasm_bindgen::prelude::*;
extern crate derive_more;
use derive_more::{From, Into};

/// Blockchain parameters
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone, Eq, From, Into)]
pub struct Parameters(pub(crate) parameters::Parameters);

#[wasm_bindgen]
impl Parameters {
    /// Return default blockchain parameters that were set at genesis
    pub fn default_parameters() -> Parameters {
        parameters::Parameters::default().into()
    }
    /// Get current block version
    pub fn block_version(&self) -> i32 {
        self.0.block_version()
    }
    /// Cost of storing 1 byte per Storage Period of block chain
    pub fn storage_fee_factor(&self) -> i32 {
        self.0.storage_fee_factor()
    }
    /// Minimum value per byte an output must have to not be considered dust
    pub fn min_value_per_byte(&self) -> i32 {
        self.0.min_value_per_byte()
    }
    /// Maximum size of transactions size in a block
    pub fn max_block_size(&self) -> i32 {
        self.0.max_block_size()
    }
    /// Maximum total computation cost in a block
    pub fn max_block_cost(&self) -> i32 {
        self.0.max_block_cost()
    }
    /// Cost of accessing a single token
    pub fn token_access_cost(&self) -> i32 {
        self.0.token_access_cost()
    }
    /// Validation cost per one transaction input
    pub fn input_cost(&self) -> i32 {
        self.0.input_cost()
    }
    /// Validation cost per data input
    pub fn data_input_cost(&self) -> i32 {
        self.0.data_input_cost()
    }
    /// Validation cost per one output
    pub fn output_cost(&self) -> i32 {
        self.0.output_cost()
    }
}
