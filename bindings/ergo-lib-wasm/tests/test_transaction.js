import { expect, assert } from 'chai';

import { generate_block_headers } from './utils';

import {
  Address, Wallet, ErgoBox, ErgoBoxCandidateBuilder, Contract,
  ErgoBoxes, ErgoBoxCandidates,
  ErgoStateContext, TxBuilder, BoxValue, BoxSelector, I64, SecretKey, SecretKeys, TxId, DataInputs,
  SimpleBoxSelector, Tokens, Token, TokenAmount, TokenId, BlockHeaders, PreHeader, Transaction,
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
  const outbox_value = BoxValue.SAFE_USER_MIN();
  const outbox = new ErgoBoxCandidateBuilder(outbox_value, contract, 0).build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = TxBuilder.SUGGESTED_TX_FEE();
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const data_inputs = new DataInputs();
  const box_selector = new SimpleBoxSelector();
  const target_balance = BoxValue.from_i64(outbox_value.as_i64().checked_add(fee.as_i64()));
  const box_selection = box_selector.select(unspent_boxes, target_balance, new Tokens());
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  tx_builder.set_data_inputs(data_inputs);
  const tx = tx_builder.build();
  assert(tx != null);
});

it('sign transaction', async () => {
  const sk = SecretKey.random_dlog();
  // simulate existing box guarded by the sk key
  const input_contract = Contract.pay_to_address(sk.get_address());
  const input_box = new ErgoBox(BoxValue.from_i64(I64.from_str('1000000000')), 0, input_contract, TxId.zero(), 0, new Tokens());
  // create a transaction that spends the "simulated" box
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = new ErgoBoxes(input_box);
  const contract = Contract.pay_to_address(recipient);
  const outbox_value = BoxValue.SAFE_USER_MIN();
  const outbox = new ErgoBoxCandidateBuilder(outbox_value, contract, 0).build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = TxBuilder.SUGGESTED_TX_FEE();
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const box_selector = new SimpleBoxSelector();
  const target_balance = BoxValue.from_i64(outbox_value.as_i64().checked_add(fee.as_i64()));
  const box_selection = box_selector.select(unspent_boxes, target_balance, new Tokens());
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  const tx = tx_builder.build();
  // smoke test for to_js_eip12
  assert(tx.to_js_eip12() != null);
  const tx_data_inputs = ErgoBoxes.from_boxes_json([]);
  const block_headers = generate_block_headers();
  const pre_header = PreHeader.from_block_header(block_headers.get(0));
  const ctx = new ErgoStateContext(pre_header, block_headers);
  const sks = new SecretKeys();
  sks.add(sk);
  const wallet = Wallet.from_secrets(sks);
  const signed_tx = wallet.sign_transaction(ctx, tx, unspent_boxes, tx_data_inputs);
  assert(signed_tx != null);
  // smoke test for to_js_eip12
  assert(signed_tx.to_js_eip12() != null);
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
  const fee = TxBuilder.SUGGESTED_TX_FEE();
  const outbox_value = BoxValue.SAFE_USER_MIN();
  const target_balance = BoxValue.from_i64(outbox_value.as_i64().checked_add(fee.as_i64()));
  const box_selector = new SimpleBoxSelector();
  const box_selection = box_selector.select(unspent_boxes, target_balance, new Tokens());
  const contract = Contract.pay_to_address(recipient);
  const token_id = TokenId.from_box_id(box_selection.boxes().get(0).box_id());
  const box_builder = new ErgoBoxCandidateBuilder(outbox_value, contract, 0);
  const token = new Token(token_id, TokenAmount.from_i64(I64.from_str('1')));
  box_builder.mint_token(token, "TKN", "token desc", 2)
  const outbox = box_builder.build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const data_inputs = new DataInputs();
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  tx_builder.set_data_inputs(data_inputs);
  const tx = tx_builder.build();
  assert(tx != null);
});

it('TxBuilder burn token test', async () => {
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = ErgoBoxes.from_boxes_json([
    {
      "boxId": "0cf7b9e71961cc473242de389c8e594a4e5d630ddd2e4e590083fb0afb386341",
      "value": 11491500000,
      "ergoTree": "100f040005c801056404000e2019719268d230fd9093e4db0e2e42a07883ffe976e77c7419efc1bb218a05d4ba04000500043c040204c096b10204020101040205c096b1020400d805d601b2a5730000d602e4c6a70405d6039c9d720273017302d604b5db6501fed9010463ededed93e4c67204050ec5a7938cb2db6308720473030001730492e4c672040605997202720390e4c6720406059a72027203d605b17204ea02d1edededededed93cbc27201e4c6a7060e917205730593db63087201db6308a793e4c6720104059db072047306d9010641639a8c720601e4c68c72060206057e72050593e4c6720105049ae4c6a70504730792c1720199c1a77e9c9a720573087309058cb072048602730a730bd901063c400163d802d6088c720601d6098c72080186029a7209730ceded8c72080293c2b2a5720900d0cde4c68c720602040792c1b2a5720900730d02b2ad7204d9010663cde4c672060407730e00",
      "assets": [
        {
          "tokenId": "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac",
          "amount": 1
        }
      ],
      "creationHeight": 348198,
      "additionalRegisters": {
        "R4": "059acd9109",
        "R5": "04f2c02a",
        "R6": "0e20277c78751ff6f68d4dcd082eeea9506324911a875b6b9cd4d177d4fcab061327"
      },
      "transactionId": "5ed0e572a8c097b053965519a696f413f7be02754345e8ed650377e29a6dedb3",
      "index": 0
    }
  ]);
  const token_id = TokenId.from_str("19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac");
  const token = new Token(token_id, TokenAmount.from_i64(I64.from_str('1')));
  const box_selector = new SimpleBoxSelector();
  let tokens = new Tokens();
  tokens.add(token);
  const fee = TxBuilder.SUGGESTED_TX_FEE();
  const outbox_value = BoxValue.SAFE_USER_MIN();
  const target_balance = BoxValue.from_i64(outbox_value.as_i64().checked_add(fee.as_i64()));
  // select tokens from inputs
  const box_selection = box_selector.select(unspent_boxes, target_balance, tokens);
  const contract = Contract.pay_to_address(recipient);
  // but don't put selected tokens in the output box (burn them)
  const box_builder = new ErgoBoxCandidateBuilder(outbox_value, contract, 0);
  const outbox = box_builder.build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const data_inputs = new DataInputs();
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  tx_builder.set_data_inputs(data_inputs);
  const tx = tx_builder.build();
  assert(tx != null);
});

it('use signed tx outputs as inputs in a new tx', async () => {
  const sk = SecretKey.random_dlog();
  // simulate existing box guarded by the sk key
  const input_contract = Contract.pay_to_address(sk.get_address());
  const input_box = new ErgoBox(BoxValue.from_i64(I64.from_str('100000000000')), 0, input_contract, TxId.zero(), 0, new Tokens());
  // create a transaction that spends the "simulated" box
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = new ErgoBoxes(input_box);
  const contract = Contract.pay_to_address(recipient);
  const outbox_value = BoxValue.from_i64(I64.from_str('10000000000'));
  const outbox = new ErgoBoxCandidateBuilder(outbox_value, contract, 0).build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = TxBuilder.SUGGESTED_TX_FEE();
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const box_selector = new SimpleBoxSelector();
  const target_balance = BoxValue.from_i64(outbox_value.as_i64().checked_add(fee.as_i64()));
  const box_selection = box_selector.select(unspent_boxes, target_balance, new Tokens());
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  const tx = tx_builder.build();
  const tx_data_inputs = ErgoBoxes.from_boxes_json([]);
  const block_headers = generate_block_headers();
  const pre_header = PreHeader.from_block_header(block_headers.get(0));
  const ctx = new ErgoStateContext(pre_header, block_headers);
  const sks = new SecretKeys();
  sks.add(sk);
  const wallet = Wallet.from_secrets(sks);
  const signed_tx = wallet.sign_transaction(ctx, tx, unspent_boxes, tx_data_inputs);
  assert(signed_tx != null);
  // new tx
  const new_box_selection = box_selector.select(signed_tx.outputs(), BoxValue.SAFE_USER_MIN(), new Tokens());
  const new_tx_builder = TxBuilder.new(new_box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  const new_tx = tx_builder.build();
  assert(new_tx != null);
});

it('Transaction::from_unsigned_tx test', async () => {
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
  const outbox_value = BoxValue.SAFE_USER_MIN();
  const outbox = new ErgoBoxCandidateBuilder(outbox_value, contract, 0).build();
  const tx_outputs = new ErgoBoxCandidates(outbox);
  const fee = TxBuilder.SUGGESTED_TX_FEE();
  const change_address = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const min_change_value = BoxValue.SAFE_USER_MIN();
  const data_inputs = new DataInputs();
  const box_selector = new SimpleBoxSelector();
  const target_balance = BoxValue.from_i64(outbox_value.as_i64().checked_add(fee.as_i64()));
  const box_selection = box_selector.select(unspent_boxes, target_balance, new Tokens());
  const tx_builder = TxBuilder.new(box_selection, tx_outputs, 0, fee, change_address, min_change_value);
  tx_builder.set_data_inputs(data_inputs);
  const tx = tx_builder.build();
  assert(tx != null);
  const proof = new Uint8Array([1, 1, 2, 255]);
  const signed_tx = Transaction.from_unsigned_tx(tx, [proof]);
  assert(signed_tx != null);
  assert(signed_tx.inputs().get(0).spending_proof().proof().toString() == proof.toString());
});
