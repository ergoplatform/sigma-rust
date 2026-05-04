//! `SNumericTypeMethods` — V6 numeric methods on `Byte`, `Short`, `Int`,
//! `Long`, `BigInt`, and `UnsignedBigInt`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (object `SNumericTypeMethods`) and `ergotree-ir/src/types/snumeric.rs` for
//! the per-type METHODS lists.
//!
//! All methods here are V3+ (v6.0). Tests use `compile_ok` since the default
//! `compile()` pipeline emits V0 ErgoTree which rejects v6 ops at
//! serialization time.
//!
//! Coverage: 8 shared methods × 6 numeric types + 2 BigInt extras +
//! 6 UnsignedBigInt extras = 56 method instantiations. Tests pick a
//! representative subset per method to keep the file readable.

use ergotree_ir::types::stype::SType;

use crate::compile_ok;

// ---- toBytes (T → Coll[Byte]) ----

#[test]
fn to_bytes_byte() {
    let expr = compile_ok(r#"{ (1.toByte).toBytes }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn to_bytes_int() {
    let expr = compile_ok(r#"{ (1).toBytes }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn to_bytes_long() {
    let expr = compile_ok(r#"{ 1L.toBytes }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn to_bytes_bigint() {
    let expr = compile_ok(r#"{ bigInt("100").toBytes }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn to_bytes_unsigned_bigint() {
    let expr = compile_ok(r#"{ unsignedBigInt("100").toBytes }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

// ---- Numeric cast coverage (toByte/toShort/toInt/toLong/toBigInt) ----
// SNumericTypeMethods exposes these uniformly on every numeric type, including
// identity casts. Coverage gaps in type_infer.rs left chained casts (e.g.
// `(x.toByte).toShort`) with tpe=None, which crashed MIR lowering.

#[test]
fn byte_to_short_chain() {
    let expr = compile_ok(
        r#"{ val r: Short = ((25.toByte).toShort) + (755.toShort); sigmaProp(r >= 0.toShort) }"#,
    );
    assert_eq!(expr.tpe(), SType::SSigmaProp);
}

#[test]
fn short_to_byte_chain() {
    let expr = compile_ok(
        r#"{ val r: Byte = ((100.toShort).toByte) + (1.toByte); sigmaProp(r >= 0.toByte) }"#,
    );
    assert_eq!(expr.tpe(), SType::SSigmaProp);
}

#[test]
fn int_to_int_identity() {
    let expr = compile_ok(r#"{ (25.toInt) + 1 }"#);
    assert_eq!(expr.tpe(), SType::SInt);
}

#[test]
fn long_to_long_identity() {
    let expr = compile_ok(r#"{ (25L.toLong) + 1L }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn byte_to_byte_identity() {
    let expr = compile_ok(r#"{ ((25.toByte).toByte) }"#);
    assert_eq!(expr.tpe(), SType::SByte);
}

#[test]
fn short_to_short_identity() {
    let expr = compile_ok(r#"{ ((25.toShort).toShort) }"#);
    assert_eq!(expr.tpe(), SType::SShort);
}

// ---- toBits (T → Coll[Boolean]) ----

#[test]
fn to_bits_int() {
    let expr = compile_ok(r#"{ (5).toBits }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SBoolean.into()));
}

#[test]
fn to_bits_long() {
    let expr = compile_ok(r#"{ 5L.toBits }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SBoolean.into()));
}

#[test]
fn to_bits_bigint() {
    let expr = compile_ok(r#"{ bigInt("5").toBits }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SBoolean.into()));
}

// ---- bitwiseInverse (T → T) ----

#[test]
fn bitwise_inverse_int() {
    let expr = compile_ok(r#"{ (5).bitwiseInverse }"#);
    assert_eq!(expr.tpe(), SType::SInt);
}

#[test]
fn bitwise_inverse_long() {
    let expr = compile_ok(r#"{ 5L.bitwiseInverse }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn bitwise_inverse_bigint() {
    let expr = compile_ok(r#"{ bigInt("5").bitwiseInverse }"#);
    assert_eq!(expr.tpe(), SType::SBigInt);
}

// ---- bitwiseOr / bitwiseAnd / bitwiseXor ((T, T) → T) ----

#[test]
fn bitwise_or_int() {
    let expr = compile_ok(r#"{ (5).bitwiseOr(3) }"#);
    assert_eq!(expr.tpe(), SType::SInt);
}

#[test]
fn bitwise_and_long() {
    let expr = compile_ok(r#"{ 5L.bitwiseAnd(3L) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn bitwise_xor_bigint() {
    let expr = compile_ok(r#"{ bigInt("5").bitwiseXor(bigInt("3")) }"#);
    assert_eq!(expr.tpe(), SType::SBigInt);
}

// ---- shiftLeft / shiftRight ((T, Int) → T) ----

#[test]
fn shift_left_int() {
    let expr = compile_ok(r#"{ (5).shiftLeft(2) }"#);
    assert_eq!(expr.tpe(), SType::SInt);
}

#[test]
fn shift_right_long() {
    let expr = compile_ok(r#"{ 20L.shiftRight(2) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn shift_left_bigint() {
    let expr = compile_ok(r#"{ bigInt("5").shiftLeft(2) }"#);
    assert_eq!(expr.tpe(), SType::SBigInt);
}

// ---- BigInt extras: toUnsigned, toUnsignedMod ----

#[test]
fn big_int_to_unsigned() {
    let expr = compile_ok(r#"{ bigInt("100").toUnsigned }"#);
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

#[test]
fn big_int_to_unsigned_mod() {
    let expr = compile_ok(r#"{ bigInt("100").toUnsignedMod(unsignedBigInt("7")) }"#);
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

// ---- UnsignedBigInt extras: modInverse / plusMod / subtractMod /
// multiplyMod / mod / toSigned ----

#[test]
fn unsigned_big_int_mod_inverse() {
    let expr = compile_ok(r#"{ unsignedBigInt("3").modInverse(unsignedBigInt("11")) }"#);
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

#[test]
fn unsigned_big_int_plus_mod() {
    let expr =
        compile_ok(r#"{ unsignedBigInt("3").plusMod(unsignedBigInt("4"), unsignedBigInt("11")) }"#);
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

#[test]
fn unsigned_big_int_subtract_mod() {
    let expr = compile_ok(
        r#"{ unsignedBigInt("10").subtractMod(unsignedBigInt("3"), unsignedBigInt("11")) }"#,
    );
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

#[test]
fn unsigned_big_int_multiply_mod() {
    let expr = compile_ok(
        r#"{ unsignedBigInt("3").multiplyMod(unsignedBigInt("4"), unsignedBigInt("11")) }"#,
    );
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

#[test]
fn unsigned_big_int_mod() {
    let expr = compile_ok(r#"{ unsignedBigInt("10").mod(unsignedBigInt("3")) }"#);
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

#[test]
fn unsigned_big_int_to_signed() {
    let expr = compile_ok(r#"{ unsignedBigInt("100").toSigned }"#);
    assert_eq!(expr.tpe(), SType::SBigInt);
}
