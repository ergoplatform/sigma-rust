//! Test suite for the Web and headless browsers.

extern crate sigma_tree_wasm;
extern crate wasm_bindgen_test;
use sigma_tree_wasm::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_signed_p2pk_transaction() {
    let tx_inputs = TxInputs::from_boxes(Box::new([]));
    let send_change_to = Address::from_testnet_str("").expect("failed");
    let sk = PrivateKey::from_str("");
    let recipient = Address::from_testnet_str("").expect("failed");

    let outbox = ErgoBoxCandidate::new(1, 0, Contract::pay_2pk(recipient));
    let out_boxes = [outbox.to_json().expect("failed")];
    let tx_outputs = TxOutputs::from_boxes(Box::new(out_boxes));
    let res = new_signed_transaction(tx_inputs, tx_outputs, send_change_to, sk);
    assert!(res.is_err());
}
