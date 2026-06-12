//! `SOptionMethods` — method-call lowering tests for `Option[T]`.
//!
//! Source of truth: `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala`
//! (object `SOptionMethods`).
//!
//! S67: wired `map` and `filter`. Combined with `get`/`isDefined`/`getOrElse`
//! already wired, all 5 unique SOption methods are covered.

use ergotree_ir::types::stype::SType;

use crate::compile_ok;

#[test]
fn method_map_int_to_long() {
    // Option[Int].map(x => x.toLong): Option[Long]
    let src = r#"{
        val o = SELF.R4[Int].get
        val opt: Option[Int] = SELF.R5[Int]
        opt.map({ (x: Int) => x.toLong + o.toLong - o.toLong })
    }"#;
    let expr = compile_ok(src);
    assert_eq!(expr.tpe(), SType::SOption(SType::SLong.into()));
}

#[test]
fn method_map_preserves_when_lambda_returns_same_type() {
    // Option[Long].map(x => x + 1L): Option[Long]
    let src = r#"{
        val opt: Option[Long] = SELF.R4[Long]
        opt.map({ (x: Long) => x + 1L })
    }"#;
    let expr = compile_ok(src);
    assert_eq!(expr.tpe(), SType::SOption(SType::SLong.into()));
}

#[test]
fn method_get_or_else_inlined() {
    // Option[T].getOrElse(default: T): T — inlined receiver `SELF.R4[Long].getOrElse(0L)`.
    let src = r#"{ sigmaProp((SELF.R4[Long].getOrElse(0L) >= 0L) && (SELF.bytes.size > 0)) }"#;
    let expr = compile_ok(src);
    assert_eq!(expr.tpe(), SType::SSigmaProp);
}

#[test]
fn method_get_or_else_val_bound() {
    // Option[T].getOrElse(default: T): T — val-bound Option receiver.
    let src = r#"{
        val o_s0 = SELF.R4[Long]
        val v_s0 = o_s0.getOrElse(0L)
        sigmaProp((v_s0 >= 0L) && (SELF.bytes.size > 0))
    }"#;
    let expr = compile_ok(src);
    assert_eq!(expr.tpe(), SType::SSigmaProp);
}

#[test]
fn method_filter() {
    // Option[Long].filter(p): Option[Long] (preserves T)
    let src = r#"{
        val opt: Option[Long] = SELF.R4[Long]
        opt.filter({ (x: Long) => x > 0L })
    }"#;
    let expr = compile_ok(src);
    assert_eq!(expr.tpe(), SType::SOption(SType::SLong.into()));
}
