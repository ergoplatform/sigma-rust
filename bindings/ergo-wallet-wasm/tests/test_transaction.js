import { expect } from 'chai';

import {
  Address, Wallet, UnspentBoxes, TxDataInputs, ErgoBoxCandidate, Contract,
  TxOutputs, ErgoStateContext
} from '../pkg/ergo_wallet_wasm';

it('new signed transaction', async () => {

  const recipient = Address.from_testnet_str('test');
  const unspent_boxes = UnspentBoxes.from_boxes([]);
  const tx_data_inputs = TxDataInputs.from_boxes([]);
  const send_change_to = Address.from_testnet_str('');

  let outbox = new ErgoBoxCandidate(1, 0, Contract.pay_to_address(recipient));
  let tx_outputs = TxOutputs.from_boxes([outbox]);
  let dummy_ctx = ErgoStateContext.dummy();
  let wallet = Wallet.from_mnemonic("", "");
  expect(() => wallet.new_signed_transaction(dummy_ctx,
    unspent_boxes,
    tx_data_inputs,
    tx_outputs,
    send_change_to,
    1,
    1)).to.throw("Not yet implemented");
});

