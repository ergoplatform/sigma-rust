//! `SCollectionMethods` — method-call lowering tests for `Coll[T]`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (object `SCollection`).
//!
//! V0-tier methods compile end-to-end (`compile_tree`); V6-tier methods only
//! reach raw IR (`compile_ok`) since the default ErgoTree header is V0.

use ergotree_ir::types::stuple::STuple;
use ergotree_ir::types::stype::SType;

use crate::{compile_ok, compile_tree};

// ---------------------------------------------------------------------------
// V0 — args methods
// ---------------------------------------------------------------------------

#[test]
fn method_zip() {
    // Coll[Int].zip(Coll[Long]) → Coll[(Int, Long)]
    let expr = compile_tree_expr(r#"{ Coll(1, 2, 3).zip(Coll(10L, 20L, 30L)) }"#);
    let pair = STuple::pair(SType::SInt, SType::SLong);
    assert_eq!(expr.tpe(), SType::SColl(SType::STuple(pair).into()));
}

#[test]
fn method_patch() {
    // Coll[Int].patch(from: Int, patch: Coll[Int], replaced: Int) → Coll[Int]
    compile_tree(r#"{ sigmaProp(Coll(1, 2, 3).patch(1, Coll(7, 8), 1).size > 0) }"#);
}

#[test]
fn method_updated() {
    // Coll[Int].updated(idx: Int, elem: Int) → Coll[Int]
    compile_tree(r#"{ sigmaProp(Coll(1, 2, 3).updated(0, 99).size > 0) }"#);
}

#[test]
fn method_update_many() {
    // Coll[Int].updateMany(indices: Coll[Int], values: Coll[Int]) → Coll[Int]
    compile_tree(r#"{ sigmaProp(Coll(1, 2, 3).updateMany(Coll(0, 2), Coll(7, 9)).size > 0) }"#);
}

#[test]
fn method_index_of() {
    // Coll[Int].indexOf(elem: Int, from: Int) → Int
    compile_tree(r#"{ sigmaProp(Coll(1, 2, 3).indexOf(2, 0) >= 0) }"#);
}

#[test]
fn method_indices() {
    // Coll[T].indices → Coll[Int] (property-style, no parens).
    compile_tree(r#"{ sigmaProp(Coll(10L, 20L, 30L).indices.size > 0) }"#);
}

// ---------------------------------------------------------------------------
// V6 — must use compile_ok (raw IR); compile() emits V0 ErgoTree which
// rejects v6 ops at serialization
// ---------------------------------------------------------------------------

#[test]
fn method_reverse_v6() {
    // Coll[T].reverse → Coll[T] (V6, property-style).
    let expr = compile_ok(r#"{ Coll(1, 2, 3).reverse }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SInt.into()));
}

#[test]
fn method_starts_with_v6() {
    // Coll[T].startsWith(prefix: Coll[T]) → Boolean (V6).
    let expr = compile_ok(r#"{ Coll(1, 2, 3).startsWith(Coll(1, 2)) }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
}

#[test]
fn method_ends_with_v6() {
    // Coll[T].endsWith(suffix: Coll[T]) → Boolean (V6).
    let expr = compile_ok(r#"{ Coll(1, 2, 3).endsWith(Coll(2, 3)) }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
}

#[test]
fn method_get_v6() {
    // Coll[T].get(idx: Int) → Option[T] (V6, distinct from V0 `apply` which
    // throws on out-of-bounds).
    let expr = compile_ok(r#"{ Coll(1, 2, 3).get(0) }"#);
    assert_eq!(expr.tpe(), SType::SOption(SType::SInt.into()));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Like `compile_tree`, but returns the raw IR so the test can assert on the
/// result type. Used for V0 methods where we want both end-to-end
/// serialization parity AND a type assertion.
fn compile_tree_expr(src: &str) -> ergotree_ir::mir::expr::Expr {
    // Reach the typed IR via `compile_ok` (no serialization), then separately
    // assert the full pipeline succeeds via `compile_tree`.
    let expr = compile_ok(src);
    compile_tree(src);
    expr
}
