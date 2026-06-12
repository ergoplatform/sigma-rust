//! `SPreHeaderMethods` — property-access lowering tests for `PreHeader`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (object `SPreHeaderMethods`).
//!
//! All accessors are V0 properties. PreHeader values come from `CONTEXT.preHeader`.

use ergotree_ir::types::stype::SType;

use crate::{compile_ok, compile_tree};

#[test]
fn property_version() {
    let expr = compile_ok(r#"{ CONTEXT.preHeader.version }"#);
    assert_eq!(expr.tpe(), SType::SByte);
    compile_tree(r#"{ sigmaProp(CONTEXT.preHeader.version > 0) }"#);
}

#[test]
fn property_parent_id() {
    let expr = compile_ok(r#"{ CONTEXT.preHeader.parentId }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.preHeader.parentId.size > 0) }"#);
}

#[test]
fn property_timestamp() {
    let expr = compile_ok(r#"{ CONTEXT.preHeader.timestamp }"#);
    assert_eq!(expr.tpe(), SType::SLong);
    compile_tree(r#"{ sigmaProp(CONTEXT.preHeader.timestamp > 0L) }"#);
}

#[test]
fn property_n_bits() {
    let expr = compile_ok(r#"{ CONTEXT.preHeader.nBits }"#);
    assert_eq!(expr.tpe(), SType::SLong);
    compile_tree(r#"{ sigmaProp(CONTEXT.preHeader.nBits > 0L) }"#);
}

#[test]
fn property_height() {
    let expr = compile_ok(r#"{ CONTEXT.preHeader.height }"#);
    assert_eq!(expr.tpe(), SType::SInt);
    compile_tree(r#"{ sigmaProp(CONTEXT.preHeader.height > 0) }"#);
}

#[test]
fn property_miner_pk() {
    let expr = compile_ok(r#"{ CONTEXT.preHeader.minerPk }"#);
    assert_eq!(expr.tpe(), SType::SGroupElement);
    compile_tree(r#"{ sigmaProp(CONTEXT.preHeader.minerPk.getEncoded.size > 0) }"#);
}

#[test]
fn property_votes() {
    let expr = compile_ok(r#"{ CONTEXT.preHeader.votes }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.preHeader.votes.size > 0) }"#);
}
