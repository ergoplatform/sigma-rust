//! Blockchain parameters. This module defines adjustable blockchain parameters that can be voted on by miners
use std::collections::HashMap;

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
// TODO: soft-fork parameter
/// A parameter that can be adjusted by voting
pub enum Parameter {
    /// Storage fee factor (per byte per storage period)
    StorageFeeFactor = 1,
    ///Minimum monetary value of a box
    MinValuePerByte = 2,
    ///Maximum block size
    MaxBlockSize = 3,
    ///Maximum cumulative computational cost of a block
    MaxBlockCost = 4,
    ///Token access cost
    TokenAccessCost = 5,
    /// Cost per one transaction input
    InputCost = 6,
    /// Cost per one data input
    DataInputCost = 7,
    /// Cost per one transaction output
    OutputCost = 8,
    /// Current block version
    BlockVersion = 123,
}

/// System parameters which can be adjusted via soft-fork
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameters {
    parameters_table: HashMap<Parameter, i32>,
}

impl Parameters {
    /// Get current block version
    pub fn block_version(&self) -> i32 {
        self.parameters_table[&Parameter::BlockVersion]
    }
    /// Cost of storing 1 byte per Storage Period of block chain
    pub fn storage_fee_factor(&self) -> i32 {
        self.parameters_table[&Parameter::StorageFeeFactor]
    }
    /// Minimum value per byte an output must have to not be considered dust
    pub fn min_value_per_byte(&self) -> i32 {
        self.parameters_table[&Parameter::MinValuePerByte]
    }
    /// Maximum size of transactions size in a block
    pub fn max_block_size(&self) -> i32 {
        self.parameters_table[&Parameter::MaxBlockSize]
    }
    /// Maximum total computation cost in a block
    pub fn max_block_cost(&self) -> i32 {
        self.parameters_table[&Parameter::MaxBlockCost]
    }
    /// Cost of accessing a single token
    pub fn token_access_cost(&self) -> i32 {
        self.parameters_table[&Parameter::TokenAccessCost]
    }
    /// Validation cost per one transaction input
    pub fn input_cost(&self) -> i32 {
        self.parameters_table[&Parameter::InputCost]
    }
    /// Validation cost per data input
    pub fn data_input_cost(&self) -> i32 {
        self.parameters_table[&Parameter::DataInputCost]
    }
    /// Validation cost per one output
    pub fn output_cost(&self) -> i32 {
        self.parameters_table[&Parameter::OutputCost]
    }
}

impl std::default::Default for Parameters {
    /// Default blockchain parameters
    // Taken from https://github.com/ergoplatform/ergo/blob/master/ergo-core/src/main/scala/org/ergoplatform/settings/Parameters.scala#L291
    fn default() -> Self {
        let mut parameters_table = HashMap::new();
        parameters_table.insert(Parameter::StorageFeeFactor, 1250000);
        parameters_table.insert(Parameter::MinValuePerByte, 30 * 12);
        parameters_table.insert(Parameter::TokenAccessCost, 100);
        parameters_table.insert(Parameter::InputCost, 2000);
        parameters_table.insert(Parameter::DataInputCost, 100);
        parameters_table.insert(Parameter::OutputCost, 100);
        parameters_table.insert(Parameter::MaxBlockSize, 512 * 1024);
        parameters_table.insert(Parameter::BlockVersion, 1);
        Self { parameters_table }
    }
}
