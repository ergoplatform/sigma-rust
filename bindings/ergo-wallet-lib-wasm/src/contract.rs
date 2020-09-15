use sigma_tree::chain;
use wasm_bindgen::prelude::*;

use crate::address::Address;

/// Defines the contract(script) that will be guarding box contents
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Contract(chain::contract::Contract);

#[wasm_bindgen]
impl Contract {
    /// create new contract that allow spending of the guarded box by a given recipient ([`Address`])
    pub fn pay_to_address(recipient: &Address) -> Result<Contract, JsValue> {
        chain::contract::Contract::pay_to_address(recipient.clone().into())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(Contract)
    }
}

impl Into<chain::contract::Contract> for Contract {
    fn into(self) -> chain::contract::Contract {
        self.0
    }
}
