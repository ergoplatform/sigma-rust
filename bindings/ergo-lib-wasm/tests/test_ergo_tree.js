import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

it("constants_len", async () => {
  let tree_bytes_base16_str =
    "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree != null);
  assert(tree.constants_len() == 2);
});

it("get_constant", async () => {
  let tree_bytes_base16_str =
    "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree != null);
  assert(tree.constants_len() == 2);
  assert(tree.get_constant(0) != null);
  assert(tree.get_constant(1) != null);
});

it("get_constant, out of bounds", async () => {
  let tree_bytes_base16_str =
    "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree != null);
  assert(tree.constants_len() == 2);
  assert(tree.get_constant(3) == null);
});

it("with_constant", async () => {
  let tree_bytes_base16_str =
    "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree.constants_len() == 2);
  let constant = ergo_wasm.Constant.from_i32(99);
  let new_tree = tree.with_constant(0, constant);
  assert(new_tree != null);
  assert(new_tree.get_constant(0).to_i32() == 99);
});

it("with_constant, out of bounds", async () => {
  let tree_bytes_base16_str =
    "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree.constants_len() == 2);
  let constant = ergo_wasm.Constant.from_i32(99);
  expect(function () {
    tree.with_constant(3, constant);
  }).throw();
});

it("with_constant, type mismatch", async () => {
  let tree_bytes_base16_str =
    "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
  let tree = ergo_wasm.ErgoTree.from_base16_bytes(tree_bytes_base16_str);
  assert(tree.constants_len() == 2);
  let constant = ergo_wasm.Constant.from_i64(ergo_wasm.I64.from_str("324234"));
  expect(function () {
    tree.with_constant(0, constant);
  }).throw();
});
