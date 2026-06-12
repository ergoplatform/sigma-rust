//! `SGlobalMethods` — additional Global SMethods user-facing as predefs.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (case object `SGlobalMethods`).
//!
//! S67: wired `some(value)` and `none[T]()` Option constructors. Both V3+,
//! reach raw IR only (`compile_ok`) — V0 trees reject v6 ops.
//! Existing SGlobalMethods coverage in `predef_funcs.rs`:
//! groupGenerator/xor/serialize/deserializeTo/fromBigEndianBytes/encodeNbits
//! /decodeNbits/powHit.

use ergotree_ir::types::stype::SType;

use crate::compile_ok;

#[test]
fn predef_some_int() {
    // some(42): Option[Int]
    let expr = compile_ok(r#"{ some(42) }"#);
    assert_eq!(expr.tpe(), SType::SOption(SType::SInt.into()));
}

#[test]
fn predef_some_long() {
    let expr = compile_ok(r#"{ some(7L) }"#);
    assert_eq!(expr.tpe(), SType::SOption(SType::SLong.into()));
}

#[test]
fn predef_none_long() {
    // none[Long](): Option[Long]
    let expr = compile_ok(r#"{ none[Long]() }"#);
    assert_eq!(expr.tpe(), SType::SOption(SType::SLong.into()));
}

#[test]
fn predef_none_coll_byte() {
    let expr = compile_ok(r#"{ none[Coll[Byte]]() }"#);
    assert_eq!(
        expr.tpe(),
        SType::SOption(SType::SColl(SType::SByte.into()).into())
    );
}
