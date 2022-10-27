import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

// enable when ergo-lib-wasm is built with "compiler" feature enabled
//
// it("Contract compiles from ErgoScript", async () => {
//   let contract = ergo_wasm.Contract.compile("HEIGHT");
//   assert(contract != null);
// });
