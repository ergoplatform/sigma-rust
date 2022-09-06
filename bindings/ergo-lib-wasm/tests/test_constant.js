import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

it("decode Constant i32", async () => {
  let enc_v = "048ce5d4e505";
  let c = ergo_wasm.Constant.decode_from_base16(enc_v);
  let c_value = c.to_i32();
  expect(c_value).equal(777689414);
});

it("roundtrip Constant i32", async () => {
  let value = 999999999;
  let c = ergo_wasm.Constant.from_i32(value);
  let encoded = c.encode_to_base16();
  let decoded_c = ergo_wasm.Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_i32();
  expect(decoded_c_value).equal(value);
});

it("roundtrip i64 via to_i64", async () => {
  let value_str = "9223372036854775807"; // i64 max value
  let c = ergo_wasm.Constant.from_i64(ergo_wasm.I64.from_str(value_str));
  let encoded = c.encode_to_base16();
  let decoded_c = ergo_wasm.Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_i64();
  let decoded_c_value_str = decoded_c_value.to_str();
  expect(decoded_c_value_str).equal(value_str);
});

it("roundtrip Constant byte array", async () => {
  let value = new Uint8Array([1, 1, 2, 255]);
  let c = ergo_wasm.Constant.from_byte_array(value);
  let encoded = c.encode_to_base16();
  let decoded_c = ergo_wasm.Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_byte_array();
  expect(decoded_c_value.toString()).equal(value.toString());
});

it("roundtrip Constant array of i32", async () => {
  let value = [2147483647, 1, 2]; // i32 max value
  let c = ergo_wasm.Constant.from_i32_array(value);
  let value_decoded = c.to_i32_array();
  expect(value_decoded.toString()).equal(value.toString());
});

it("roundtrip Constant array of i64", async () => {
  let value_str = ["9223372036854775807", "1", "2"]; // i64 max value
  let c = ergo_wasm.Constant.from_i64_str_array(value_str);
  let encoded = c.encode_to_base16();
  let decoded_c = ergo_wasm.Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_i64_str_array();
  expect(decoded_c_value.toString()).equal(value_str.toString());
});

it("Constant from EcPoint bytes", async () => {
  let base16_bytes_str = `02d6b2141c21e4f337e9b065a031a6269fb5a49253094fc6243d38662eb765db00`;
  let c = ergo_wasm.Constant.from_ecpoint_bytes(
    Uint8Array.from(Buffer.from(base16_bytes_str, "hex"))
  );
  expect(c != null);
});

it("roundtrip array of byte arrays", async () => {
  let bytes1 = new Uint8Array([1, 1, 2, 255]);
  let bytes2 = new Uint8Array([5, 6, 7, 255]);
  let concat = [bytes1, bytes2];
  let c = ergo_wasm.Constant.from_coll_coll_byte(concat);
  let decoded_c_value = c.to_coll_coll_byte();
  expect(decoded_c_value.toString()).equal(concat.toString());
});

it("roundtrip tuple of byte arrays", async () => {
  let bytes1 = new Uint8Array([1, 1, 2, 255]);
  let bytes2 = new Uint8Array([5, 6, 7, 255]);
  let c = ergo_wasm.Constant.from_tuple_coll_bytes(bytes1, bytes2);
  let encoded = c.encode_to_base16();
  let decoded_c = ergo_wasm.Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_tuple_coll_bytes();
  expect(decoded_c_value.toString()).equal([bytes1, bytes2].toString());
});

it("roundtrip tuple of i64", async () => {
  let value_str1 = "9223372036854775807"; // i64 max value
  let value_str2 = "29428734987293874";
  let c = ergo_wasm.Constant.from_tuple_i64(
    ergo_wasm.I64.from_str(value_str1),
    ergo_wasm.I64.from_str(value_str2)
  );
  let encoded = c.encode_to_base16();
  let decoded_c = ergo_wasm.Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_tuple_i64();
  expect(decoded_c_value.toString()).equal([value_str1, value_str2].toString());
});

it("roundtrip ErgoBox", async () => {
  const boxes = ergo_wasm.ErgoBoxes.from_boxes_json([
    {
      boxId: "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
      value: 67500000000,
      ergoTree:
        "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
      assets: [],
      creationHeight: 284761,
      additionalRegisters: {},
      transactionId:
        "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
      index: 1,
    },
  ]);
  let box = boxes.get(0);
  let c = ergo_wasm.Constant.from_ergo_box(box);
  let encoded = c.encode_to_base16();
  let decoded_c = ergo_wasm.Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_ergo_box();
  assert(decoded_c_value != null);
  expect(decoded_c_value.to_json().toString()).equal(box.to_json().toString());
});

it("roundtrip Coll[Coll[Byte]]", async () => {
  let bytes1 = new Uint8Array([1, 2, 3]);
  let bytes2 = new Uint8Array([3, 2, 1]);
  let value = [bytes1, bytes2];
  let c_js = ergo_wasm.Constant.from_js(value);
  expect(c_js != null);
  expect(c_js.dbg_tpe()).equal("SColl(SColl(SByte))");
  assert.deepEqual(c_js.to_js(), value);
});

it("roundtrip Coll[(Coll[Byte], Coll[Byte])]", async () => {
  let bytes1 = new Uint8Array([1, 2, 3]);
  let bytes2 = new Uint8Array([3, 2, 1]);
  let value = [ergo_wasm.array_as_tuple([bytes1, bytes2]), ergo_wasm.array_as_tuple([bytes2, bytes1])];
  let expected_value = [[bytes1, bytes2], [bytes2, bytes1]];
  let c_js = ergo_wasm.Constant.from_js(value);
  expect(c_js != null);
  expect(c_js.dbg_tpe()).equal("SColl(STuple([SColl(SByte), SColl(SByte)]))");
  // console.log(c_js.dbg_inner());
  assert.deepEqual(c_js.to_js(), expected_value);
});

it("roundtrip EIP-24 R7 monster type", async () => {
  let bytes1 = new Uint8Array([1, 2, 3]);
  let bytes2 = new Uint8Array([4, 5, 6]);
  let value = ergo_wasm.array_as_tuple([
    [ergo_wasm.array_as_tuple([bytes1, bytes2])],
    ergo_wasm.array_as_tuple([
      [ergo_wasm.array_as_tuple([bytes1, ergo_wasm.array_as_tuple([10, 11])])],
      [ergo_wasm.array_as_tuple([bytes2, ergo_wasm.array_as_tuple([12, 13])])]
    ])
  ]);
  let expected_value = [
    [[bytes1, bytes2]],
    [
      [[bytes1, [10, 11]]],
      [[bytes2, [12, 13]]]
    ]
  ];
  let c_js = ergo_wasm.Constant.from_js(value);
  expect(c_js != null);
  expect(c_js.dbg_tpe()).equal("STuple([SColl(STuple([SColl(SByte), SColl(SByte)])), STuple([SColl(STuple([SColl(SByte), STuple([SInt, SInt])])), SColl(STuple([SColl(SByte), STuple([SInt, SInt])]))])])");
  // console.log(c_js.dbg_inner());
  assert.deepEqual(c_js.to_js(), expected_value);
});

it("roundtrip i64", async () => {
  let value_str = "9223372036854775807"; // i64 max value
  let c_js = ergo_wasm.Constant.from_js(value_str);
  let decoded_c_value = c_js.to_js();
  expect(c_js.dbg_tpe()).equal("SLong");
  expect(decoded_c_value).equal(value_str);
});

it("roundtrip BigInt", async () => {
  let bigint = BigInt(92233720368547758071111111111111111111111111n);
  let c_js = ergo_wasm.Constant.from_js(bigint);
  expect(c_js.dbg_tpe()).equal("SBigInt");
  let decoded_c_value = c_js.to_js();
  expect(decoded_c_value).equal(bigint);
  assert.deepEqual(c_js.to_js(), bigint);
});

it("too big BigInt fail", async () => {
  let bigint = BigInt(92233720368547758071111111111111111111111111111111111111111111111111111111111111111111111111n);
  expect(function () {ergo_wasm.Constant.from_js(bigint);}).to.throw();
});
