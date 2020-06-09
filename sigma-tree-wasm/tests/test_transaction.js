import { expect } from 'chai';

import {TxInputs, SecretKey, ErgoBoxCandidate, Contract, TxOutputs} from '../pkg/sigma_tree_wasm';

const sigma_rust = import('../pkg/sigma_tree_wasm');

it('new transaction', async () => {
  const {
    Address,
    new_signed_transaction,
  } = await sigma_rust;

  const recipient = Address.from_testnet_str('test');
  const tx_inputs = TxInputs.from_boxes([]);
  const send_change_to = Address.from_testnet_str('');
  const sk = SecretKey.parse('');

  let outbox = new ErgoBoxCandidate(1, 0, Contract.pay_to_address(recipient));
  let tx_outputs = TxOutputs.from_boxes([outbox]);
  expect(() => new_signed_transaction(tx_inputs, tx_outputs, send_change_to, sk)).to.throw();
});

