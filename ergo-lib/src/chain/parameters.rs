//! Blockchain parameters. This module defines adjustable blockchain parameters that can be voted on by miners
use hashbrown::HashMap;

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
/// A parameter that can be adjusted by voting.
///
/// Note: `SoftForkDisablingRules` (id 124) is intentionally not represented here.
/// Its on-chain value is a variable-length `ErgoValidationSettingsUpdate` byte
/// vector, which is incompatible with the current `HashMap<Parameter, i32>`
/// storage type. Supporting it will require a follow-up change to the storage
/// representation.
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
    /// Number of sub-blocks per block, on average. Introduced by the 6.0
    /// soft-fork (block version 4). Mirrors JVM
    /// `Parameters.SubblocksPerBlockIncrease`. Auto-inserted by
    /// `Parameters.update` whenever `BlockVersion == 4`.
    SubblocksPerBlock = 9,
    /// Number of soft-fork votes collected during the current voting period.
    /// Tracked in `parameters_table` while a soft-fork vote is in progress;
    /// removed at activation or expiration. Mirrors JVM
    /// `Parameters.SoftForkVotesCollected`.
    SoftForkVotesCollected = 121,
    /// Height at which the current soft-fork voting period began.
    /// Tracked in `parameters_table` while a soft-fork vote is in progress;
    /// removed at activation or expiration. Mirrors JVM
    /// `Parameters.SoftForkStartingHeight`.
    SoftForkStartingHeight = 122,
    /// Current block version
    BlockVersion = 123,
}

/// System parameters which can be adjusted via soft-fork
#[cfg_attr(feature = "json", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "json",
    serde(try_from = "crate::chain::json::parameters::ParametersJson")
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameters {
    /// table of adjustable system parameters
    pub parameters_table: HashMap<Parameter, i32>,
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

    /// Number of sub-blocks per block. Returns `None` pre-6.0 (block version
    /// less than 4) or when the parameter is otherwise absent from the table.
    pub fn sub_blocks_per_block(&self) -> Option<i32> {
        self.parameters_table
            .get(&Parameter::SubblocksPerBlock)
            .copied()
    }

    /// Number of soft-fork votes collected, if a vote is currently in progress.
    ///
    /// Returns `None` when no soft-fork vote is in progress; the entry is only
    /// present in `parameters_table` during an active voting period.
    pub fn soft_fork_votes_collected(&self) -> Option<i32> {
        self.parameters_table
            .get(&Parameter::SoftForkVotesCollected)
            .copied()
    }

    /// Height at which the current soft-fork voting period began, if any.
    ///
    /// Returns `None` when no soft-fork vote is in progress; the entry is only
    /// present in `parameters_table` during an active voting period.
    pub fn soft_fork_starting_height(&self) -> Option<i32> {
        self.parameters_table
            .get(&Parameter::SoftForkStartingHeight)
            .copied()
    }

    /// Create new parameters from provided blockchain parameters
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_version: i32,
        storage_fee_factor: i32,
        min_value_per_byte: i32,
        max_block_size: i32,
        max_block_cost: i32,
        token_access_cost: i32,
        input_cost: i32,
        data_input_cost: i32,
        output_cost: i32,
    ) -> Self {
        let mut parameters_table = HashMap::new();
        parameters_table.insert(Parameter::BlockVersion, block_version);
        parameters_table.insert(Parameter::StorageFeeFactor, storage_fee_factor);
        parameters_table.insert(Parameter::MinValuePerByte, min_value_per_byte);
        parameters_table.insert(Parameter::MaxBlockSize, max_block_size);
        parameters_table.insert(Parameter::MaxBlockCost, max_block_cost);
        parameters_table.insert(Parameter::TokenAccessCost, token_access_cost);
        parameters_table.insert(Parameter::InputCost, input_cost);
        parameters_table.insert(Parameter::DataInputCost, data_input_cost);
        parameters_table.insert(Parameter::OutputCost, output_cost);
        Self { parameters_table }
    }
}

impl Default for Parameters {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_does_not_contain_soft_fork_variants() {
        let params = Parameters::default();
        assert!(!params
            .parameters_table
            .contains_key(&Parameter::SoftForkVotesCollected));
        assert!(!params
            .parameters_table
            .contains_key(&Parameter::SoftForkStartingHeight));
    }

    #[test]
    fn default_soft_fork_votes_collected_is_none() {
        assert_eq!(Parameters::default().soft_fork_votes_collected(), None);
    }

    #[test]
    fn default_soft_fork_starting_height_is_none() {
        assert_eq!(Parameters::default().soft_fork_starting_height(), None);
    }

    #[test]
    fn soft_fork_votes_collected_returns_inserted_value() {
        let mut params = Parameters::default();
        params
            .parameters_table
            .insert(Parameter::SoftForkVotesCollected, 42);
        assert_eq!(params.soft_fork_votes_collected(), Some(42));
    }

    #[test]
    fn soft_fork_starting_height_returns_inserted_value() {
        let mut params = Parameters::default();
        params
            .parameters_table
            .insert(Parameter::SoftForkStartingHeight, 1000);
        assert_eq!(params.soft_fork_starting_height(), Some(1000));
    }

    #[test]
    fn sub_blocks_per_block_default_none() {
        let p = Parameters::default();
        assert_eq!(p.sub_blocks_per_block(), None);
    }

    #[test]
    fn sub_blocks_per_block_set_and_get() {
        let mut p = Parameters::default();
        p.parameters_table.insert(Parameter::SubblocksPerBlock, 30);
        assert_eq!(p.sub_blocks_per_block(), Some(30));
    }
}
