import { expect } from 'chai';

import {TxInputs, TxDataInputs, SecretKey, ErgoBoxCandidate, Contract, TxOutputs, ErgoStateContext} from '../pkg/ergo_wallet_wasm';

const sigma_rust = import('../pkg/ergo_wallet_wasm');

it('new signed transaction', async () => {
  const {
    Address,
    new_signed_transaction,
  } = await sigma_rust;

  const recipient = Address.from_testnet_str('test');
  const tx_inputs = TxInputs.from_boxes([]);
  const tx_data_inputs = TxDataInputs.from_boxes([]);
  const send_change_to = Address.from_testnet_str('');
  const sk = SecretKey.parse('');

  let outbox = new ErgoBoxCandidate(1, 0, Contract.pay_to_address(recipient));
  let tx_outputs = TxOutputs.from_boxes([outbox]);
  let dummy_ctx = ErgoStateContext.dummy();
  expect(() => new_signed_transaction(dummy_ctx, tx_inputs, tx_data_inputs, tx_outputs, send_change_to, sk)).to.throw("Not yet implemented");
});

