import { expect, assert } from "chai";
import { generate_block_headers } from '../tests/utils';

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

it("Browser: P2PK from base16 ergo tree", async () => {
  // ProveDlog in ErgoTree root
  let tree_bytes_base16_str =
    "0008cd0327e65711a59378c59359c3e1d0f7abe906479eccb76094e50fe79d743ccc15e6";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  let addr = ergo_wasm.Address.recreate_from_ergo_tree(tree);
  assert(addr != null);
});

it("Browser: P2S from base16 ergo tree", async () => {
  // Non ProveDlog in ErgoTree root
  let tree_bytes_base16_str =
    "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  let addr = ergo_wasm.Address.recreate_from_ergo_tree(tree);
  assert(addr != null);
});

it('node REST API get_nipopow_proof_by_header_id endpoint', async () => {
  let node_conf = new ergo_wasm.NodeConf("213.239.193.208:9053");
  assert(node_conf != null);
  const block_headers = generate_block_headers();
  const header_id = block_headers.get(0).id();
  let res = await ergo_wasm.get_nipopow_proof_by_header_id(node_conf, 3, 4, header_id);
  assert(res != null);
  assert(node_conf != null);
});