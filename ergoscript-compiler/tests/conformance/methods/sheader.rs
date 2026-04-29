//! `SHeaderMethods` — property-access lowering tests for `Header`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (object `SHeaderMethods`).
//!
//! All accessors are V0 properties. `checkPow` is V3+ and must use `compile_ok`
//! since the default ErgoTree header is V0.
//!
//! `Header` values are obtained via `CONTEXT.headers(idx)` in user code.

use ergotree_ir::types::stype::SType;

use crate::{compile_ok, compile_tree};

#[test]
fn property_id() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).id }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).id.size > 0) }"#);
}

#[test]
fn property_version() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).version }"#);
    assert_eq!(expr.tpe(), SType::SByte);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).version > 0) }"#);
}

#[test]
fn property_parent_id() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).parentId }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).parentId.size > 0) }"#);
}

#[test]
fn property_ad_proofs_root() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).ADProofsRoot }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).ADProofsRoot.size > 0) }"#);
}

#[test]
fn property_state_root() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).stateRoot }"#);
    assert_eq!(expr.tpe(), SType::SAvlTree);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).stateRoot.digest.size > 0) }"#);
}

#[test]
fn property_transactions_root() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).transactionsRoot }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).transactionsRoot.size > 0) }"#);
}

#[test]
fn property_timestamp() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).timestamp }"#);
    assert_eq!(expr.tpe(), SType::SLong);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).timestamp > 0L) }"#);
}

#[test]
fn property_n_bits() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).nBits }"#);
    assert_eq!(expr.tpe(), SType::SLong);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).nBits > 0L) }"#);
}

#[test]
fn property_height() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).height }"#);
    assert_eq!(expr.tpe(), SType::SInt);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).height > 0) }"#);
}

#[test]
fn property_extension_root() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).extensionRoot }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).extensionRoot.size > 0) }"#);
}

#[test]
fn property_miner_pk() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).minerPk }"#);
    assert_eq!(expr.tpe(), SType::SGroupElement);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).minerPk.getEncoded.size > 0) }"#);
}

#[test]
fn property_pow_onetime_pk() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).powOnetimePk }"#);
    assert_eq!(expr.tpe(), SType::SGroupElement);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).powOnetimePk.getEncoded.size > 0) }"#);
}

#[test]
fn property_pow_nonce() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).powNonce }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).powNonce.size > 0) }"#);
}

#[test]
fn property_pow_distance() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).powDistance }"#);
    assert_eq!(expr.tpe(), SType::SBigInt);
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).powDistance > bigInt("0")) }"#);
}

#[test]
fn property_votes() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).votes }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
    compile_tree(r#"{ sigmaProp(CONTEXT.headers(0).votes.size > 0) }"#);
}

// V3+ — checkPow must use compile_ok (V0 ErgoTree rejects v6 ops).
#[test]
fn method_check_pow_v6() {
    let expr = compile_ok(r#"{ CONTEXT.headers(0).checkPow }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
}
