import { expect, assert } from "chai";

// import { Address, ErgoTree } from "../pkg/ergo_lib_wasm";

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
