import { expect, assert } from 'chai';

import {
  Address, Wallet, ErgoBox, ErgoBoxCandidateBuilder, Contract,
  ErgoBoxes, ErgoBoxCandidates,
  ErgoStateContext, TxBuilder, BoxValue, BoxSelector, SecretKey, TxId, DataInputs, SimpleBoxSelector, Tokens, Token, TokenAmount, TokenId
} from '../pkg/ergo_lib_wasm';

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
  const outbox = new ErgoBoxCandidateBuilder(BoxValue.from_u32(10000000), contract, 0).build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = BoxValue.from_u32(1000000);
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const data_inputs = new DataInputs();
  const box_selector = new SimpleBoxSelector();
  const box_selection = box_selector.select(unspent_boxes, BoxValue.from_u32(11000000), new Tokens());
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  tx_builder.set_data_inputs(data_inputs);
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
  const outbox = new ErgoBoxCandidateBuilder(BoxValue.from_u32(10000000), contract, 0).build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = BoxValue.from_u32(1000000);
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const box_selector = new SimpleBoxSelector();
  const box_selection = box_selector.select(unspent_boxes, BoxValue.from_u32(11000000), new Tokens());
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  const tx = tx_builder.build();
  const tx_data_inputs = ErgoBoxes.from_boxes_json([]);
  const dummy_ctx = ErgoStateContext.dummy();
  const wallet = Wallet.from_secret(sk);
  const signed_tx = wallet.sign_transaction(dummy_ctx, tx, unspent_boxes, tx_data_inputs);
  assert(signed_tx != null);
});

it('TxBuilder mint token test', async () => {
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
  const box_selector = new SimpleBoxSelector();
  const box_selection = box_selector.select(unspent_boxes, BoxValue.from_u32(11000000), new Tokens());
  const contract = Contract.pay_to_address(recipient);
  const token_id = TokenId.from_box_id(box_selection.boxes().get(0).box_id());
  const box_builder = new ErgoBoxCandidateBuilder(BoxValue.from_u32(10000000), contract, 0);
  const token = new Token(token_id, TokenAmount.from_u32(1));
  box_builder.mint_token(token, "TKN", "token desc", 2)
  const outbox = box_builder.build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = BoxValue.from_u32(1000000);
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const data_inputs = new DataInputs();
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  tx_builder.set_data_inputs(data_inputs);
  const tx = tx_builder.build();
  assert(tx != null);
});
