import { expect, assert } from 'chai';

import {
  Address,
  ErgoTree
} from '../pkg/ergo_lib_wasm';

it('new_p2pk from base16 ergo tree', async () => {
  let tree_bytes_base16_str = '0008cd0327e65711a59378c59359c3e1d0f7abe906479eccb76094e50fe79d743ccc15e6';
  let tree = ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  let addr = Address.new_p2pk(tree);
  assert(addr != null);
});
