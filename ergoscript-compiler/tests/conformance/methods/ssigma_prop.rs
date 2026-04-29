//! `SSigmaPropMethods` — method-call lowering tests for `SigmaProp`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (case object `SSigmaPropMethods`).
//!
//! S70: wired `isProven` after filling the matching IR gap.
//!
//! `isProven` had op-code 95 reserved at `op_code.rs:127` since the IR was
//! ported, but the Rust IR had no `SigmaPropIsProven` MIR struct, no
//! sigma_parse/serialize, and no eval — same situation as the BitShift
//! variants before S68. S70 adds the IR struct + serialization +
//! interpreter eval (returning `EvalError::Misc` to mirror Scala's
//! `costKind = notSupportedError` — Scala's graph-IR rewrites the node
//! away at AOT compile time), then wires the compiler arm.
//!
//! With this S70 work, SSigmaProp methods 2/2 covered (`propBytes`,
//! `isProven`).

use ergotree_ir::types::stype::SType;

use crate::compile_ok;
use crate::compile_tree;

#[test]
fn property_is_proven() {
    // SigmaProp.isProven: Boolean (V0+)
    let expr = compile_ok(r#"{ proveDlog(groupGenerator).isProven }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
}

#[test]
fn property_is_proven_compiles_v0() {
    // V0-tier method — full pipeline through ErgoTree should succeed.
    // The interpreter eval returns NotImplemented at runtime (matches Scala
    // testMissingCosting), but compile-time round-trip must work.
    compile_tree(r#"{ sigmaProp(proveDlog(groupGenerator).isProven) }"#);
}
