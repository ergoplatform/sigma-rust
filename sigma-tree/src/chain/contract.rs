//! Ergo contract

use super::address::Address;
use crate::{serialization::SerializationError, ErgoTree};

/// High-level wrapper for ErgoTree
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Contract {
    ergo_tree: ErgoTree,
}

impl Contract {
    /// create new contract from ErgoTree
    pub fn new(ergo_tree: ErgoTree) -> Contract {
        Contract { ergo_tree }
    }

    /// create new contract that allow spending for a given Address
    pub fn pay_to_address(address: Address) -> Result<Contract, SerializationError> {
        Ok(Contract::new(address.script()?))
    }

    /// get ErgoTree for this contract
    pub fn get_ergo_tree(&self) -> ErgoTree {
        self.ergo_tree.clone()
    }
}
