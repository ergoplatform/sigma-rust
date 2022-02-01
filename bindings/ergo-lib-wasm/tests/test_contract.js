import { expect, assert } from "chai";

// import {
//   Contract,
//   ErgoTree
// } from '../pkg/ergo_lib_wasm';

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

it("Contract compiles from ErgoScript", async () => {
  let contract = ergo_wasm.Contract.compile("HEIGHT");
  assert(contract != null);
});
