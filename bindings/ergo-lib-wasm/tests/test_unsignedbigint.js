import { expect, assert } from "chai";

import * as ergo from "..";
let ergo_wasm;
beforeEach(async () => {
  ergo_wasm = await ergo;
});

for (const [name, value] of [
  ["negative Number", -1],
  ["NaN", NaN],
  ["positive infinity", Infinity],
  ["negative infinity", -Infinity],
  ["fractional Number", 1.5],
  ["unsafe Number", Number.MAX_SAFE_INTEGER + 1],
]) {
  it(`UnsignedBigInt rejects ${name}`, () => {
    expect(() => new ergo_wasm.UnsignedBigInt(value)).to.throw(Error);
  });
}

for (const value of [0, Number.MAX_SAFE_INTEGER]) {
  it(`UnsignedBigInt accepts safe Number ${value} exactly`, () => {
    const actual = new ergo_wasm.UnsignedBigInt(value);
    const expected = new ergo_wasm.UnsignedBigInt(BigInt(value));
    try {
      assert(actual.eq(expected));
    } finally {
      actual.free();
      expected.free();
    }
  });
}

for (const [name, value] of [
  ["above u64", BigInt(1) << BigInt(64)],
  ["maximum u256", (BigInt(1) << BigInt(256)) - BigInt(1)],
]) {
  it(`UnsignedBigInt accepts BigInt ${name} exactly`, () => {
    const actual = new ergo_wasm.UnsignedBigInt(value);
    const expected = ergo_wasm.UnsignedBigInt.from_str_radix(value.toString(), 10);
    try {
      assert(actual.eq(expected));
    } finally {
      actual.free();
      expected.free();
    }
  });
}

for (const [name, value] of [
  ["negative BigInt", -BigInt(1)],
  ["BigInt above u256", BigInt(1) << BigInt(256)],
]) {
  it(`UnsignedBigInt rejects ${name}`, () => {
    expect(() => new ergo_wasm.UnsignedBigInt(value)).to.throw(Error);
  });
}

it("unsignedbigint tests", async () => {
  const bigint = new ergo_wasm.UnsignedBigInt(5)
  const bigint2 = new ergo_wasm.UnsignedBigInt(BigInt(5))
  assert(bigint.eq(bigint2), "should be equal")
  assert(bigint.add(new ergo_wasm.UnsignedBigInt(2)).eq(new ergo_wasm.UnsignedBigInt(7)))
  const modulus = new ergo_wasm.UnsignedBigInt(7)
  assert(bigint.mod_mul(bigint.mod_inv(modulus), modulus).eq(new ergo_wasm.UnsignedBigInt(1)))
});

for (const [method, a, b] of [
  ["mod_sub", 5, 3],
  ["mod_sub", 3, 3],
  ["mod_sub", 1, 3],
  ["mod_inv", 5, 0],
]) {
  it(`UnsignedBigInt ${method}(${a}, ${b}) rejects zero modulus with an ordinary Error`, () => {
    const value = new ergo_wasm.UnsignedBigInt(a);
    const other = new ergo_wasm.UnsignedBigInt(b);
    const zero = new ergo_wasm.UnsignedBigInt(0);
    try {
      const operation = method === "mod_inv"
        ? () => value.mod_inv(zero)
        : () => value.mod_sub(other, zero);
      expect(operation).to.throw(Error).with.property("name", "Error");
    } finally {
      value.free();
      other.free();
      zero.free();
    }
  });
}
