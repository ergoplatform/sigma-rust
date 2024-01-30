use std::collections::HashMap;

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
// TODO: soft-fork parameter
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
    BlockVersion = 123,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameters {
    parameters_table: HashMap<Parameter, i32>,
}

impl Parameters {
    pub fn block_version(&self) -> i32 {
        self.parameters_table[&Parameter::BlockVersion]
    }
    pub fn storage_fee_factor(&self) -> i32 {
        self.parameters_table[&Parameter::StorageFeeFactor]
    }
    pub fn min_value_per_byte(&self) -> i32 {
        self.parameters_table[&Parameter::MinValuePerByte]
    }
    pub fn max_block_size(&self) -> i32 {
        self.parameters_table[&Parameter::MaxBlockSize]
    }
    pub fn max_block_cost(&self) -> i32 {
        self.parameters_table[&Parameter::MaxBlockCost]
    }
    pub fn token_access_cost(&self) -> i32 {
        self.parameters_table[&Parameter::TokenAccessCost]
    }
    pub fn input_cost(&self) -> i32 {
        self.parameters_table[&Parameter::InputCost]
    }
    pub fn data_input_cost(&self) -> i32 {
        self.parameters_table[&Parameter::DataInputCost]
    }
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
