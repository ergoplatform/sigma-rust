//! `SBoxMethods` — method-call lowering tests for `Box`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (case object `SBoxMethods`).
//!
//! Most SBox properties (`value`/`propositionBytes`/`id`/`bytes`/`creationInfo`/
//! `tokens`/registers R0–R9) were wired ad-hoc pre-conformance arc. S69 closes
//! the residual `bytesWithoutRef` (V0 property) — the SMethod exists in Scala
//! at method_id=4 and the `ExtractBytesWithNoRef` IR node existed in
//! ergotree-ir; only the compiler dispatch arm and type_infer entry were
//! missing.

use ergotree_ir::types::stype::SType;

use crate::compile_ok;
use crate::compile_tree;

#[test]
fn property_bytes_without_ref() {
    // Box.bytesWithoutRef: Coll[Byte] — serialized box bytes excluding
    // transactionId and box index.
    let expr = compile_ok(r#"{ SELF.bytesWithoutRef }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn property_bytes_without_ref_compiles_v0() {
    // V0 method — full pipeline through ErgoTree should also succeed.
    compile_tree(r#"{ sigmaProp(SELF.bytesWithoutRef.size > 0) }"#);
}
