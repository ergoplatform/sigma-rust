//! `SContextMethods` — context-property lowering tests.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (case object `SContextMethods`).
//!
//! S67: wired `LastBlockUtxoRootHash` (PropertyCall) and `minerPubKey`
//! (both bare and `CONTEXT.minerPubKey` → GlobalVars::MinerPubKey).
//! `dataInputs`/`selfBoxIndex`/`HEIGHT`/`preHeader`/`headers` were already
//! wired in earlier slices, plus the bare globals (HEIGHT/SELF/INPUTS/OUTPUTS
//! /groupGenerator/CONTEXT) via `binder.rs`.

use ergotree_ir::types::stype::SType;

use crate::compile_ok;
use crate::compile_tree;

#[test]
fn property_last_block_utxo_root_hash() {
    // CONTEXT.LastBlockUtxoRootHash: AvlTree
    let expr = compile_ok(r#"{ CONTEXT.LastBlockUtxoRootHash }"#);
    assert_eq!(expr.tpe(), SType::SAvlTree);
}

#[test]
fn property_last_block_utxo_root_hash_v0_pipeline() {
    compile_tree(r#"{ sigmaProp(CONTEXT.LastBlockUtxoRootHash.digest.size > 0) }"#);
}

#[test]
fn property_miner_pub_key_bare() {
    // bare `minerPubKey` → GlobalVars::MinerPubKey: Coll[Byte]
    let expr = compile_ok(r#"{ minerPubKey }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn property_miner_pub_key_via_context() {
    // CONTEXT.minerPubKey lowers to the same GlobalVars node as bare access.
    let expr = compile_ok(r#"{ CONTEXT.minerPubKey }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn property_miner_pub_key_v0_pipeline() {
    compile_tree(r#"{ sigmaProp(minerPubKey.size > 0) }"#);
}
