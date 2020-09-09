//! Test suite for the Web and headless browsers.

extern crate wasm_bindgen_test;

use ergo_wallet_lib_wasm::{
    address::Address, box_coll::ErgoBoxCandidates, box_coll::ErgoBoxes, box_selector::BoxSelector,
    contract::Contract, ergo_box::BoxValue, ergo_box::ErgoBoxCandidate,
    ergo_state_ctx::ErgoStateContext, secret_key::SecretKey, transaction::UnsignedTransaction,
    tx_builder::TxBuilder, wallet::Wallet,
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_sign_transaction() {
    let boxes_to_spend = ErgoBoxes::from_boxes(Box::new([]));
    let data_boxes = ErgoBoxes::from_boxes(Box::new([]));
    let dummy_ctx = ErgoStateContext::dummy();
    let tx = UnsignedTransaction::dummy();
    let wallet = Wallet::from_mnemonic("", "");
    let res = wallet.sign_transaction(dummy_ctx, tx, boxes_to_spend, data_boxes);
    assert!(res.is_err());
}

#[wasm_bindgen_test]
fn test_random() {
    let sk1 = SecretKey::random_dlog();
    let sk2 = SecretKey::random_dlog();
    assert_ne!(sk1, sk2);
}

#[wasm_bindgen_test]
fn test_tx_builder() {
    let tx_inputs = ErgoBoxes::from_boxes(Box::new([]));
    let recipient =
        Address::from_testnet_str("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
            .expect("failed");
    let contract = Contract::pay_to_address(recipient).expect("failed");
    let outbox = ErgoBoxCandidate::new(BoxValue::from_u32(1).unwrap(), 0, contract);
    let tx_outputs = ErgoBoxCandidates::new(outbox);
    let fee = BoxValue::from_u32(2).unwrap();
    let tx_builder = TxBuilder::new(BoxSelector::SelectAll, tx_inputs, tx_outputs, 0, fee);
    assert!(tx_builder.is_err());
}
