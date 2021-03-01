import { expect, assert } from 'chai';

import {
  Contract,
  ErgoTree
} from '../pkg/ergo_lib_wasm';

it('Contract compiles from ErgoScript', async () => {
  let contract = Contract.compile("HEIGHT");
  assert(contract!= null);
});

