import { expect, assert } from 'chai';

import {
  Address, Wallet, ErgoBox, ErgoBoxCandidate, Contract,
  ErgoBoxes, ErgoBoxCandidates,
  ErgoStateContext, TxBuilder, BoxValue, UnsignedTransaction, BoxSelector, SecretKey, TxId,
} from '../pkg/ergo_wallet_lib_wasm';

it('TxBuilder test', async () => {
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = ErgoBoxes.from_boxes_json([
    {
      "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
      "value": 67500000000,
      "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
      "assets": [],
      "creationHeight": 284761,
      "additionalRegisters": {},
      "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
      "index": 1
    }
  ]);
  const contract = Contract.pay_to_address(recipient);
  const outbox = new ErgoBoxCandidate(BoxValue.MIN(), 0, contract);
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = BoxValue.MIN();
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.MIN();
  const tx_builder = TxBuilder.new(BoxSelector.SelectAll, unspent_boxes, tx_outputs, 0, fee, change_address, min_change_value);
  const tx = tx_builder.build();
  assert(tx != null);
});


it('sign transaction', async () => {
  const sk = SecretKey.random_dlog();
  // simulate existing box guarded by the sk key
  const input_contract = Contract.pay_to_address(sk.get_address());
  const input_box = new ErgoBox(BoxValue.from_u32(1000000000), 0, input_contract, TxId.zero(), 0);
  // create a transaction that spends the "simulated" box
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = new ErgoBoxes(input_box);
  const contract = Contract.pay_to_address(recipient);
  const outbox = new ErgoBoxCandidate(BoxValue.MIN(), 0, contract);
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = BoxValue.MIN();
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.MIN();
  const tx_builder = TxBuilder.new(BoxSelector.SelectAll, unspent_boxes, tx_outputs, 0, fee, change_address, min_change_value);
  const tx = tx_builder.build();
  const tx_data_inputs = ErgoBoxes.from_boxes_json([]);
  const dummy_ctx = ErgoStateContext.dummy();
  const wallet = Wallet.from_secret(sk);
  const signed_tx = wallet.sign_transaction(dummy_ctx, tx, unspent_boxes, tx_data_inputs);
  assert(signed_tx != null);
});
