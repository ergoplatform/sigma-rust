import { expect } from 'chai';

import {TxInputs, PrivateKey, ErgoBoxCandidate, Contract, TxOutputs} from '../pkg/sigma_tree_wasm';

const sigma_rust = import('../pkg/sigma_tree_wasm');

it('new transaction', async () => {
  const {
    Address,
    new_signed_transaction,
  } = await sigma_rust;

  const recipient = Address.from_str('');
  const tx_inputs = TxInputs.from_boxes([]);
  const send_change_to = Address.from_str('');
  const sk = PrivateKey.from_str('');

  let outbox = ErgoBoxCandidate.new(1, 0, Contract.pay_2pk(recipient));
  let tx_outputs = TxOutputs.from_boxes([outbox]);
  let signed_tx = new_signed_transaction(tx_inputs, tx_outputs, send_change_to, sk);

  expect(signed_tx).to.not.be.null;
});

