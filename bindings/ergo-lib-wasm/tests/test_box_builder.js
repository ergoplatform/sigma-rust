import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
let recipient, contract;
beforeEach(async () => {
  ergo_wasm = await ergo;
  recipient = ergo_wasm.Address.from_testnet_str(
    "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN"
  );
  contract = ergo_wasm.Contract.pay_to_address(recipient);
});

it("ErgoBoxCandidateBuilder test", async () => {
  const b = new ergo_wasm.ErgoBoxCandidateBuilder(
    ergo_wasm.BoxValue.from_i64(ergo_wasm.I64.from_str("10000000")),
    contract,
    0
  ).build();
  assert(b != null);
});

it("ErgoBoxCandidateBuilder set register value test", async () => {
  let builder = new ergo_wasm.ErgoBoxCandidateBuilder(
    ergo_wasm.BoxValue.from_i64(ergo_wasm.I64.from_str("10000000")),
    contract,
    0
  );
  assert(builder.register_value(ergo_wasm.NonMandatoryRegisterId.R4) == null);
  const c = ergo_wasm.Constant.from_i32(1);
  builder.set_register_value(ergo_wasm.NonMandatoryRegisterId.R4, c);
  assert(
    builder.register_value(ergo_wasm.NonMandatoryRegisterId.R4).to_i32() == c.to_i32()
  );
  const b = builder.build();
  assert((b.register_value(ergo_wasm.NonMandatoryRegisterId.R4).to_i32 = c.to_i32));
});

it("ErgoBoxCandidateBuilder delete register value test", async () => {
  let builder = new ergo_wasm.ErgoBoxCandidateBuilder(
    ergo_wasm.BoxValue.from_i64(ergo_wasm.I64.from_str("10000000")),
    contract,
    0
  );
  const c = ergo_wasm.Constant.from_i32(1);
  builder.set_register_value(ergo_wasm.NonMandatoryRegisterId.R4, c);
  assert(
    builder.register_value(ergo_wasm.NonMandatoryRegisterId.R4).to_i32() == c.to_i32()
  );
  builder.delete_register_value(ergo_wasm.NonMandatoryRegisterId.R4);
  assert(builder.register_value(ergo_wasm.NonMandatoryRegisterId.R4) == null);
  const b = builder.build();
  assert(b.register_value(ergo_wasm.NonMandatoryRegisterId.R4) == null);
});
