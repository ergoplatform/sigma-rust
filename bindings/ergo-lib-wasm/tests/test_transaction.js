import { expect, assert } from 'chai';

import {
  Address, Wallet, ErgoBox, ErgoBoxCandidateBuilder, Contract,
  ErgoBoxes, ErgoBoxCandidates,
  ErgoStateContext, TxBuilder, BoxValue, BoxSelector, I64, SecretKey, SecretKeys, TxId, DataInputs,
  SimpleBoxSelector, Tokens, Token, TokenAmount, TokenId, BlockHeaders, PreHeader,
} from '../pkg/ergo_lib_wasm';

const block_headers = BlockHeaders.from_json([{
      "extensionId": "d16f25b14457186df4c5f6355579cc769261ce1aebc8209949ca6feadbac5a3f",
      "difficulty": "626412390187008",
      "votes": "040000",
      "timestamp": 1618929697400,
      "size": 221,
      "stateRoot": "8ad868627ea4f7de6e2a2fe3f98fafe57f914e0f2ef3331c006def36c697f92713",
      "height": 471746,
      "nBits": 117586360,
      "version": 2,
      "id": "4caa17e62fe66ba7bd69597afdc996ae35b1ff12e0ba90c22ff288a4de10e91b",
      "adProofsRoot": "d882aaf42e0a95eb95fcce5c3705adf758e591532f733efe790ac3c404730c39",
      "transactionsRoot": "63eaa9aff76a1de3d71c81e4b2d92e8d97ae572a8e9ab9e66599ed0912dd2f8b",
      "extensionHash": "3f91f3c680beb26615fdec251aee3f81aaf5a02740806c167c0f3c929471df44",
      "powSolutions": {
        "pk": "02b3a06d6eaa8671431ba1db4dd427a77f75a5c2acbd71bfb725d38adc2b55f669",
        "w": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        "n": "5939ecfee6b0d7f4",
        "d": 0
      },
      "adProofsId": "86eaa41f328bee598e33e52c9e515952ad3b7874102f762847f17318a776a7ae",
      "transactionsId": "ac80245714f25aa2fafe5494ad02a26d46e7955b8f5709f3659f1b9440797b3e",
      "parentId": "6481752bace5fa5acba5d5ef7124d48826664742d46c974c98a2d60ace229a34"
}]);

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
  const tx_data_inputs = ErgoBoxes.from_boxes_json([]);
  const pre_header = PreHeader.from_block_header(block_headers.get(0));
  const ctx = new ErgoStateContext(pre_header);
  const sks = new SecretKeys();
  sks.add(sk);
  const wallet = Wallet.from_secrets(sks);
  const signed_tx = wallet.sign_transaction(ctx, tx, unspent_boxes, tx_data_inputs);
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
