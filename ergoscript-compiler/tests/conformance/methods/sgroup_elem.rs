//! `SGroupElementMethods` — method-call lowering tests for `GroupElement`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (case object `SGroupElementMethods`).
//!
//! S67: wired `negate` (V0 property). Combined with `getEncoded`/`exp`/`multiply`
//! already wired, 4/5 SGroupElement methods are covered.
//!
//! S69: wired `expUnsigned` (V3+). Now 5/5 SGroupElement methods are covered.
//! The Rust IR struct in ergotree-ir was already in place
//! (`EXPONENTIATE_UNSIGNED_METHOD` at method_id=6); only the compiler dispatch
//! arm and type_infer entry were missing.
//!
//! Note: Scala has no `isIdentity` method on GroupElement. Earlier coverage
//! matrix was wrong; corrected S67.

use ergotree_ir::types::stype::SType;

use crate::compile_ok;
use crate::compile_tree;

#[test]
fn property_negate() {
    // GroupElement.negate: GroupElement
    let expr = compile_ok(r#"{ groupGenerator.negate }"#);
    assert_eq!(expr.tpe(), SType::SGroupElement);
}

#[test]
fn property_negate_compiles_v0() {
    // V0-tier method — full pipeline through ErgoTree should also succeed.
    compile_tree(r#"{ sigmaProp(groupGenerator.negate.getEncoded.size > 0) }"#);
}

#[test]
fn method_exp_unsigned_v6() {
    // GroupElement.expUnsigned(UnsignedBigInt) → GroupElement (V3+)
    let expr = compile_ok(
        r#"{
            val k: UnsignedBigInt = unsignedBigInt("7")
            groupGenerator.expUnsigned(k)
        }"#,
    );
    assert_eq!(expr.tpe(), SType::SGroupElement);
}
