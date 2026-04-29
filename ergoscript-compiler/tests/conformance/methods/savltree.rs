//! `SAvlTreeMethods` — method-call lowering tests for `AvlTree`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (case object `SAvlTreeMethods`).
//!
//! S67 closure: 4 V0 property accessors + 1 V6 method (`insertOrUpdate`).
//! Combined with the 11 already wired (digest/enabledOperations/keyLength
//! /contains/get/getMany/insert/update/remove/updateDigest/updateOperations),
//! all 16 unique SAvlTreeMethods entries are now wired.

use ergotree_ir::types::stype::SType;

use crate::compile_ok;

// ---------------------------------------------------------------------------
// V0 — property accessors (no args)
// ---------------------------------------------------------------------------

#[test]
fn property_value_length_opt() {
    // AvlTree.valueLengthOpt: Option[Int]
    let expr = compile_ok(r#"{ SELF.R4[AvlTree].get.valueLengthOpt }"#);
    assert_eq!(expr.tpe(), SType::SOption(SType::SInt.into()));
}

#[test]
fn property_is_insert_allowed() {
    // AvlTree.isInsertAllowed: Boolean
    let expr = compile_ok(r#"{ SELF.R4[AvlTree].get.isInsertAllowed }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
}

#[test]
fn property_is_update_allowed() {
    let expr = compile_ok(r#"{ SELF.R4[AvlTree].get.isUpdateAllowed }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
}

#[test]
fn property_is_remove_allowed() {
    let expr = compile_ok(r#"{ SELF.R4[AvlTree].get.isRemoveAllowed }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
}

// ---------------------------------------------------------------------------
// V6 — argful method
// ---------------------------------------------------------------------------

#[test]
fn method_insert_or_update_v6() {
    // AvlTree.insertOrUpdate(entries: Coll[(Coll[Byte], Coll[Byte])], proof: Coll[Byte])
    //   → Option[AvlTree]
    let src = r#"{
        val tree = SELF.R4[AvlTree].get
        val entries = SELF.R5[Coll[(Coll[Byte], Coll[Byte])]].get
        val proof = SELF.R6[Coll[Byte]].get
        tree.insertOrUpdate(entries, proof)
    }"#;
    let expr = compile_ok(src);
    assert_eq!(expr.tpe(), SType::SOption(SType::SAvlTree.into()));
}
