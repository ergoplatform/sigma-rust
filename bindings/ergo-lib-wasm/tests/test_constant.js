import { expect, assert } from 'chai';

import {
    Constant, I64
} from '../pkg/ergo_lib_wasm';

it('decode Constant i32', async () => {
  let enc_v = '048ce5d4e505';
  let c = Constant.decode_from_base16(enc_v);
  let c_value = c.to_i32();
  expect(c_value).equal(777689414);
});

it('roundtrip Constant i32', async () => {
  let value = 999999999;
  let c = Constant.from_i32(value);
  let encoded = c.encode_to_base16();
  let decoded_c = Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_i32();
  expect(decoded_c_value).equal(value);
});

it('roundtrip Constant i64', async () => {
  let value_str = '9223372036854775807'; // i64 max value
  let c = Constant.from_i64(I64.from_str(value_str));
  let encoded = c.encode_to_base16();
  let decoded_c = Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_i64();
  let decoded_c_value_str = decoded_c_value.to_str();
  expect(decoded_c_value_str).equal(value_str);
});

it('roundtrip Constant byte array', async () => {
  let value = new Uint8Array([1, 1, 2, 255]);
  let c = Constant.from_byte_array(value);
  let encoded = c.encode_to_base16();
  let decoded_c = Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_byte_array();
  expect(decoded_c_value.toString()).equal(value.toString());
});

it('roundtrip Constant array of i64', async () => {
  let value_str = ['9223372036854775807', '1', '2']; // i64 max value
  let c = Constant.from_i64_str_array(value_str);
  let encoded = c.encode_to_base16();
  let decoded_c = Constant.decode_from_base16(encoded);
  let decoded_c_value = decoded_c.to_i64_str_array();
  expect(decoded_c_value.toString()).equal(value_str.toString());
});

it('Constant from EcPoint bytes', async () => {
  let base16_bytes_str = `02d6b2141c21e4f337e9b065a031a6269fb5a49253094fc6243d38662eb765db00`;
  let c = Constant.from_ecpoint_bytes(Uint8Array.from(Buffer.from(base16_bytes_str, 'hex')));
  expect(c != null);
});
