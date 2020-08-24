//! Test suite for the Web and headless browsers.

extern crate ergo_wallet_lib_wasm;
extern crate wasm_bindgen_test;
use ergo_wallet_lib_wasm::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_signed_p2pk_transaction() {
    let tx_inputs = UnspentBoxes::from_boxes(Box::new([]));
    let tx_data_inputs = TxDataInputs::from_boxes(Box::new([]));
    let send_change_to =
        Address::from_testnet_str("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
            .expect("failed");
    let recipient =
        Address::from_testnet_str("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
            .expect("failed");

    let contract = Contract::pay_to_address(recipient).expect("failed");

    let outbox = ErgoBoxCandidate::new(1, 0, contract);
    let tx_outputs = TxOutputCandidates::new(outbox);
    let dummy_ctx = ErgoStateContext::dummy();
    let wallet = Wallet::from_mnemonic("", "");
    let res = wallet.new_signed_transaction(
        dummy_ctx,
        tx_inputs,
        tx_data_inputs,
        tx_outputs,
        send_change_to,
        1,
        1,
    );
    assert!(res.is_err());
}

#[wasm_bindgen_test]
fn test_random() {
    let sk1 = SecretKey::random_dlog();
    let sk2 = SecretKey::random_dlog();
    assert_ne!(sk1, sk2);
}
