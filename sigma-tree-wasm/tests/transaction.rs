//! Test suite for the Web and headless browsers.

extern crate sigma_tree_wasm;
extern crate wasm_bindgen_test;
use sigma_tree_wasm::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_signed_p2pk_transaction() {
    let tx_inputs = TxInputs::from_boxes(Box::new([]));
    let recipient = Address::from_str("");
    let send_change_to = Address::from_str("");
    let sk = PrivateKey::from_str("");
    let res = signed_p2pk_transaction(tx_inputs, 1, recipient, send_change_to, sk);
    assert!(res.is_err());
}
