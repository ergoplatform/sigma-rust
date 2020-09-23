//! Test suite for the Web and headless browsers.

extern crate wasm_bindgen_test;

use std::{convert::TryInto, rc::Rc};

use ergo_wallet_lib_wasm::{
    address::Address, box_coll::ErgoBoxCandidates, box_coll::ErgoBoxes, box_selector::BoxSelector,
    contract::Contract, ergo_box::BoxValue, ergo_box::ErgoBoxCandidate,
    ergo_state_ctx::ErgoStateContext, secret_key::SecretKey, tx_builder::TxBuilder, wallet::Wallet,
};
use sigma_tree::{
    ast::Expr, chain::ergo_box::register::NonMandatoryRegisters, chain::ergo_box::ErgoBox,
    chain::transaction::TxId, sigma_protocol::sigma_boolean::SigmaProp,
    sigma_protocol::DlogProverInput, ErgoTree,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_sign_transaction() {
    let pi = DlogProverInput::random();
    let sk = SecretKey::from(sigma_tree::wallet::secret_key::SecretKey::from(pi.clone()));
    let pk: SigmaProp = pi.public_image().into();
    let tree = ErgoTree::from(Rc::new(Expr::Const(pk.into())));

    let input_box = ErgoBox::new(
        1000000000u64.try_into().unwrap(),
        tree,
        vec![],
        NonMandatoryRegisters::empty(),
        0,
        TxId::zero(),
        0,
    );
    let input_box_json_str = serde_json::to_string(&input_box).unwrap();
    let box_json = JsValue::from_str(input_box_json_str.as_str());
    let tx_inputs = ErgoBoxes::from_boxes_json(Box::new([box_json])).unwrap();
    let recipient =
        Address::from_testnet_str("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
            .expect("failed");
    let contract = Contract::pay_to_address(&recipient).expect("failed");
    let outbox =
        ErgoBoxCandidate::new(&BoxValue::from_u32(10000000).unwrap(), 0, &contract).unwrap();
    let tx_outputs = ErgoBoxCandidates::new(&outbox);
    let fee = BoxValue::from_u32(1000000).unwrap();
    let change_address =
        Address::from_testnet_str("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
            .expect("failed");
    let min_change_value = BoxValue::MIN();
    let tx_builder = TxBuilder::new(
        BoxSelector::Simple,
        &tx_inputs,
        &tx_outputs,
        0,
        &fee,
        &change_address,
        &min_change_value,
    )
    .unwrap();
    let tx = tx_builder.build().unwrap();
    let wallet = Wallet::from_secret(&sk);
    let dummy_ctx = ErgoStateContext::dummy();
    let res = wallet.sign_transaction(
        &dummy_ctx,
        &tx,
        &tx_inputs,
        &ErgoBoxes::from_boxes_json(Box::new([])).unwrap(),
    );
    let _signed_tx = res.unwrap();
}

#[wasm_bindgen_test]
fn test_random() {
    let sk1 = SecretKey::random_dlog();
    let sk2 = SecretKey::random_dlog();
    assert_ne!(sk1, sk2);
}

#[wasm_bindgen_test]
fn test_tx_builder() {
    let box_json = JsValue::from_str(
        r#"{
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }"#,
    );
    let tx_inputs = ErgoBoxes::from_boxes_json(Box::new([box_json])).unwrap();
    let recipient =
        Address::from_testnet_str("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
            .expect("failed");
    let contract = Contract::pay_to_address(&recipient).expect("failed");
    let outbox =
        ErgoBoxCandidate::new(&BoxValue::from_u32(10000000).unwrap(), 0, &contract).unwrap();
    let tx_outputs = ErgoBoxCandidates::new(&outbox);
    let fee = BoxValue::from_u32(1000000).unwrap();
    let change_address =
        Address::from_testnet_str("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
            .expect("failed");
    let min_change_value = BoxValue::MIN();
    let tx_builder = TxBuilder::new(
        BoxSelector::Simple,
        &tx_inputs,
        &tx_outputs,
        0,
        &fee,
        &change_address,
        &min_change_value,
    )
    .unwrap();
    // assert!(tx_builder.is_ok());
    let _tx = tx_builder.build().unwrap();
}
