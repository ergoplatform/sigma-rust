import { expect } from 'chai';

import {
  Address, Wallet, ErgoBoxCandidate, Contract,
  ErgoBoxes, ErgoBoxCandidates,
  ErgoStateContext, TxBuilder, BoxValue, UnsignedTransaction
} from '../pkg/ergo_wallet_lib_wasm';

it('TxBuilder test', async () => {
  const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
  const unspent_boxes = ErgoBoxes.from_boxes([]);
  let contract = Contract.pay_to_address(recipient);
  let outbox = new ErgoBoxCandidate(BoxValue.from_u32(1), 0, contract);
  let tx_outputs = new ErgoBoxCandidates(outbox);
  let fee = BoxValue.from_u32(1);
  expect(() => TxBuilder.new(unspent_boxes, tx_outputs, 0, fee)).to.throw("Not yet implemented");
});


it('sign transaction', async () => {
  const unspent_boxes = ErgoBoxes.from_boxes([]);
  const tx_data_inputs = ErgoBoxes.from_boxes([]);
  let dummy_ctx = ErgoStateContext.dummy();
  let tx = UnsignedTransaction.dummy();
  let wallet = Wallet.from_mnemonic("", "");
  expect(() => wallet.sign_transaction(dummy_ctx, tx, unspent_boxes, tx_data_inputs)).to.throw("Not yet implemented");
});
