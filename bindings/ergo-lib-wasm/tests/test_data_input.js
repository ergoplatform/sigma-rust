import { expect, assert } from "chai";

// import {
//   DataInput,
//   BoxId,
// } from '../pkg/ergo_lib_wasm';

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

it("from str", async () => {
  const box_id = ergo_wasm.BoxId.from_str(
    "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e"
  );
  assert(box_id != null);
  const di = new ergo_wasm.DataInput(box_id);
  assert(di != null);
});
