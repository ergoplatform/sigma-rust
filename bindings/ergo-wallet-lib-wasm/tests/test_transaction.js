import { expect } from 'chai';

import {
  Address, Wallet, ErgoBoxCandidate, Contract,
  ErgoBoxes, ErgoBoxCandidates,
  ErgoStateContext, TxBuilder, BoxValue, UnsignedTransaction, BoxSelector, SecretKey
} from '../pkg/ergo_wallet_lib_wasm';

it('TxBuilder test', async () => {
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = ErgoBoxes.from_boxes([
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
  let contract = Contract.pay_to_address(recipient);
  let outbox = new ErgoBoxCandidate(BoxValue.from_u32(1), 0, contract);
  let tx_outputs = new ErgoBoxCandidates(outbox);
  let fee = BoxValue.from_u32(1);
  let tx_builder = TxBuilder.new(unspent_boxes, tx_outputs, 0, fee);
  let tx = tx_builder.build();
});


it('sign transaction', async () => {
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = ErgoBoxes.from_boxes([
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
  let contract = Contract.pay_to_address(recipient);
  let outbox = new ErgoBoxCandidate(BoxValue.from_u32(1), 0, contract);
  let tx_outputs = new ErgoBoxCandidates(outbox);
  let fee = BoxValue.from_u32(1);
  let tx_builder = TxBuilder.new(unspent_boxes, tx_outputs, 0, fee);
  let tx = tx_builder.build();
  console.error(unspent_boxes);
  const tx_data_inputs = ErgoBoxes.from_boxes([]);
  let dummy_ctx = ErgoStateContext.dummy();
  let wallet = Wallet.from_secret(SecretKey.random_dlog());
  expect(() => wallet.sign_transaction(dummy_ctx, tx, unspent_boxes, tx_data_inputs)).to.throw("Not yet implemented");
});
