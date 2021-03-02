//! Contract, for easier ErgoTree generation
use ergo_lib::chain;
use ergo_lib::ergoscript_compiler::script_env::ScriptEnv;
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
        chain::contract::Contract::pay_to_address(&recipient.clone().into())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
            .map(Contract)
    }

    /// Compiles a contract from ErgoScript source code
    pub fn compile(source: &str) -> Result<Contract, JsValue> {
        chain::contract::Contract::compile(source, ScriptEnv::new())
            .map_err(|e| JsValue::from_str(e.pretty_desc(source).as_str()))
            .map(Contract)
    }
}

impl From<Contract> for chain::contract::Contract {
    fn from(c: Contract) -> Self {
        c.0
    }
}
