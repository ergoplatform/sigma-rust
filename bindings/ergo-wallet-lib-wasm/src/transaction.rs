use sigma_tree::chain;
use wasm_bindgen::prelude::*;

/**
 * ErgoTransaction is an atomic state transition operation. It destroys Boxes from the state
 * and creates new ones. If transaction is spending boxes protected by some non-trivial scripts,
 * its inputs should also contain proof of spending correctness - context extension (user-defined
 * key-value map) and data inputs (links to existing boxes in the state) that may be used during
 * script reduction to crypto, signatures that satisfies the remaining cryptographic protection
 * of the script.
 * Transactions are not encrypted, so it is possible to browse and view every transaction ever
 * collected into a block.
 */
#[wasm_bindgen]
pub struct Transaction(chain::transaction::Transaction);

#[wasm_bindgen]
impl Transaction {
    /// JSON representation
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        JsValue::from_serde(&self.0).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

impl From<chain::transaction::Transaction> for Transaction {
    fn from(t: chain::transaction::Transaction) -> Self {
        Transaction(t)
    }
}

#[wasm_bindgen]
pub struct UnsignedTransaction(chain::transaction::unsigned::UnsignedTransaction);

#[wasm_bindgen]
impl UnsignedTransaction {
    #[wasm_bindgen]
    pub fn dummy() -> UnsignedTransaction {
        UnsignedTransaction(chain::transaction::unsigned::UnsignedTransaction::new(
            vec![],
            vec![],
            vec![],
        ))
    }
}

impl From<chain::transaction::unsigned::UnsignedTransaction> for UnsignedTransaction {
    fn from(t: chain::transaction::unsigned::UnsignedTransaction) -> Self {
        UnsignedTransaction(t)
    }
}

impl From<UnsignedTransaction> for chain::transaction::unsigned::UnsignedTransaction {
    fn from(t: UnsignedTransaction) -> Self {
        t.0
    }
}
