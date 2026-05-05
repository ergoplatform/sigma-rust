//! One smoke test per SigmaPredef.PredefinedFunc entry — covers all predefs
//! wired in `lower.rs`'s name-dispatch branch, plus parser-level constructs
//! (if/selectField/apply) that the SigmaPredef registry treats as predefs but
//! are handled at the HIR level in this compiler.
//!
//! See `tests/fixtures/conformance/method-coverage.md` for the canonical
//! Scala-side ground truth and per-name wiring status.

use ergotree_ir::types::stype::SType;

use super::{compile_ok, compile_tree};

// ---------------------------------------------------------------------------
// Sigma proposition constructors
// ---------------------------------------------------------------------------

#[test]
fn predef_sigma_prop() {
    compile_tree(r#"{ sigmaProp(HEIGHT > 0) }"#);
}

#[test]
fn predef_prove_dlog() {
    compile_tree(r#"{ proveDlog(groupGenerator) }"#);
}

#[test]
fn predef_prove_dh_tuple() {
    compile_tree(
        r#"{ proveDHTuple(groupGenerator, groupGenerator, groupGenerator, groupGenerator) }"#,
    );
}

#[test]
fn predef_at_least() {
    compile_tree(r#"{ atLeast(1, Coll(proveDlog(groupGenerator))) }"#);
}

#[test]
fn predef_at_least_dedup_provedlog_siblings() {
    // WS-F cluster 001 (DIFF agreement-prefix 100204020400):
    // `Coll(proveDlog(g), proveDlog(g))` inside `atLeast` has two
    // structurally-identical siblings. Scala's graph IR hash-conses them
    // into one ValDef + two ValUses; Rust must mirror that by walking
    // Atleast's children in CSE and counting edge multiplicity inside a
    // single Coll/SigmaAnd/etc. parent.
    use ergoscript_compiler::compiler::compile;
    use ergoscript_compiler::script_env::ScriptEnv;
    use ergotree_ir::serialization::SigmaSerializable;
    let src = r#"{
  val sp = atLeast(1, Coll(proveDlog(groupGenerator), proveDlog(groupGenerator)))
  sigmaProp(sp.propBytes.size > 0)
}"#;
    let tree = compile(src, ScriptEnv::new()).expect("compile");
    let bytes = tree.sigma_serialize_bytes().expect("serialize");
    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(
        hex, "100204020400d801d601cddb6a01ddd191b1d0987300830208720172017301",
        "WS-F cluster 001: must emit one ValDef for proveDlog(g) and ValUse it twice"
    );
}

#[test]
fn predef_pk() {
    // Mainnet P2PK address (real, from ergotree-ir test fixtures).
    compile_tree(r#"{ PK("9hzP24a2q8KLPVCUk7gdMDXYc7vinmGuxmLp5KU7k9UwptgYBYV") }"#);
}

// ---------------------------------------------------------------------------
// Logical aggregation
// ---------------------------------------------------------------------------

#[test]
fn predef_all_of() {
    compile_tree(r#"{ sigmaProp(allOf(Coll(true, false))) }"#);
}

#[test]
fn predef_any_of() {
    compile_tree(r#"{ sigmaProp(anyOf(Coll(true, false))) }"#);
}

#[test]
fn predef_xor_of() {
    compile_tree(r#"{ sigmaProp(xorOf(Coll(true, false))) }"#);
}

// ---------------------------------------------------------------------------
// ZK-conjuncts (Coll[SigmaProp] versions of allOf/anyOf)
// ---------------------------------------------------------------------------

#[test]
fn predef_all_zk() {
    // allZK(Coll(p1, p2)): SigmaProp. Mirrors Scala graph-IR rewrite which
    // unfolds a literal Coll[SigmaProp] argument into SigmaAnd. Runtime Colls
    // are rejected (matches Scala's `undefined` irBuilder).
    compile_tree(r#"{ allZK(Coll(proveDlog(groupGenerator), proveDlog(groupGenerator))) }"#);
}

#[test]
fn predef_any_zk() {
    // anyZK(Coll(p1, p2)): SigmaProp → SigmaOr.
    compile_tree(r#"{ anyZK(Coll(proveDlog(groupGenerator), proveDlog(groupGenerator))) }"#);
}

// ---------------------------------------------------------------------------
// Hash functions
// ---------------------------------------------------------------------------

#[test]
fn predef_blake2b256() {
    compile_tree(r#"{ sigmaProp(blake2b256(SELF.propositionBytes).size > 0) }"#);
}

#[test]
fn predef_sha256() {
    compile_tree(r#"{ sigmaProp(sha256(SELF.propositionBytes).size > 0) }"#);
}

// ---------------------------------------------------------------------------
// Type conversions (predefs)
// ---------------------------------------------------------------------------

#[test]
fn predef_byte_array_to_long() {
    compile_tree(r#"{ sigmaProp(byteArrayToLong(SELF.propositionBytes) > 0L) }"#);
}

#[test]
fn predef_byte_array_to_big_int() {
    compile_tree(r#"{ sigmaProp(byteArrayToBigInt(SELF.propositionBytes) > 0.toBigInt) }"#);
}

#[test]
fn predef_long_to_byte_array() {
    compile_tree(r#"{ sigmaProp(longToByteArray(0L).size > 0) }"#);
}

#[test]
fn predef_decode_point() {
    compile_tree(r#"{ proveDlog(decodePoint(SELF.propositionBytes)) }"#);
}

// ---------------------------------------------------------------------------
// Compile-time string-arg predefs
// ---------------------------------------------------------------------------

#[test]
fn predef_from_base16() {
    compile_tree(r#"{ val x = fromBase16("deadbeef"); sigmaProp(x.size > 0) }"#);
}

#[test]
fn predef_from_base58() {
    compile_tree(r#"{ val x = fromBase58("1"); sigmaProp(x.size > 0) }"#);
}

#[test]
fn predef_from_base64() {
    compile_tree(r#"{ val x = fromBase64("AA=="); sigmaProp(x.size > 0) }"#);
}

#[test]
fn predef_big_int_decimal() {
    let expr = compile_ok(r#"{ bigInt("12345") }"#);
    assert_eq!(expr.tpe(), SType::SBigInt);
}

#[test]
fn predef_unsigned_big_int_decimal() {
    let expr = compile_ok(r#"{ unsignedBigInt("12345") }"#);
    assert_eq!(expr.tpe(), SType::SUnsignedBigInt);
}

// ---------------------------------------------------------------------------
// Context variable access
// ---------------------------------------------------------------------------

#[test]
fn predef_get_var() {
    compile_tree(r#"{ sigmaProp(getVar[Long](0).isDefined) }"#);
}

#[test]
fn predef_get_var_from_input_v6() {
    // v6.0+ Context method. Test raw expr build; full ErgoTree serialization
    // requires V3 tree version which `compile()` doesn't currently emit.
    let expr = compile_ok(r#"{ getVarFromInput[Long](0, 1) }"#);
    assert_eq!(expr.tpe(), SType::SOption(SType::SLong.into()));
}

// ---------------------------------------------------------------------------
// Script template manipulation
// ---------------------------------------------------------------------------

#[test]
fn predef_subst_constants() {
    compile_tree(
        r#"{ sigmaProp(substConstants(SELF.propositionBytes, Coll(0), Coll(1L)).size > 0) }"#,
    );
}

// ---------------------------------------------------------------------------
// Global object — methods accessible without `Global.` prefix
// ---------------------------------------------------------------------------

#[test]
fn predef_group_generator() {
    // Property accessed via the binder's GlobalVars::GroupGenerator.
    compile_tree(r#"{ proveDlog(groupGenerator) }"#);
}

#[test]
fn predef_xor_byte_array() {
    compile_tree(r#"{ sigmaProp(xor(SELF.propositionBytes, SELF.propositionBytes).size > 0) }"#);
}

#[test]
fn predef_serialize_v6() {
    let expr = compile_ok(r#"{ serialize(SELF.value) }"#);
    assert_eq!(expr.tpe(), SType::SColl(SType::SByte.into()));
}

#[test]
fn predef_deserialize_to_v6() {
    let expr = compile_ok(r#"{ deserializeTo[Long](SELF.propositionBytes) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_from_big_endian_bytes_v6() {
    let expr = compile_ok(r#"{ fromBigEndianBytes[Long](SELF.propositionBytes) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_encode_nbits_v6() {
    let expr = compile_ok(r#"{ encodeNbits(bigInt("123456")) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_decode_nbits_v6() {
    let expr = compile_ok(r#"{ decodeNbits(0L) }"#);
    assert_eq!(expr.tpe(), SType::SBigInt);
}

#[test]
fn predef_pow_hit_v6() {
    let expr = compile_ok(
        r#"{ powHit(0, SELF.propositionBytes, SELF.propositionBytes, SELF.propositionBytes, 0) }"#,
    );
    assert_eq!(expr.tpe(), SType::SBoolean);
}

// ---------------------------------------------------------------------------
// Numeric method casts (user-facing forms of upcast/downcast)
// ---------------------------------------------------------------------------

#[test]
fn methods_to_long() {
    compile_tree(r#"{ sigmaProp(0.toLong + 1L > 0L) }"#);
}

#[test]
fn methods_to_int() {
    compile_tree(r#"{ sigmaProp(0L.toInt + 1 > 0) }"#);
}

#[test]
fn methods_to_byte() {
    // toByte is a Downcast. Used inside an Int comparison so the result type fits.
    compile_tree(r#"{ sigmaProp(0.toByte.toInt + 1 > 0) }"#);
}

#[test]
fn methods_to_short() {
    compile_tree(r#"{ sigmaProp(0.toShort.toInt + 1 > 0) }"#);
}

#[test]
fn methods_to_big_int() {
    compile_tree(r#"{ sigmaProp(0.toBigInt > 0.toBigInt) }"#);
}

// ---------------------------------------------------------------------------
// Min/max
// ---------------------------------------------------------------------------

#[test]
fn predef_min() {
    compile_tree(r#"{ sigmaProp(min(0L, 1L) >= 0L) }"#);
}

#[test]
fn predef_max() {
    compile_tree(r#"{ sigmaProp(max(0L, 1L) >= 0L) }"#);
}

// ---------------------------------------------------------------------------
// AVL tree lookup
// ---------------------------------------------------------------------------

#[test]
fn predef_avl_tree_none() {
    // avlTree(operationFlags, digest, keyLength, valueLengthOpt) → AvlTree.
    // valueLengthOpt = none[Int]() → Rust IR's compile-time None branch.
    // (Byte-match against Scala for this predef has a known IR-shape
    //  discrepancy — Rust's CreateAvlTree carries Option<Box<Expr>> while
    //  Scala carries a runtime SOption[SInt] expression. Smoke test only.)
    let expr = compile_ok(r#"{ avlTree(0.toByte, fromBase16("0102030405"), 32, none[Int]()) }"#);
    assert_eq!(expr.tpe(), SType::SAvlTree);
}

#[test]
fn predef_avl_tree_some() {
    // valueLengthOpt = some(8) → Rust IR's Some(Box<Expr>) branch.
    let expr = compile_ok(r#"{ avlTree(0.toByte, fromBase16("0102030405"), 32, some(8)) }"#);
    assert_eq!(expr.tpe(), SType::SAvlTree);
}

#[test]
fn predef_tree_lookup() {
    // treeLookup(tree, key, proof) → Option[Coll[Byte]].
    // Returns SOption(SColl(SByte)). Use SELF.R4 as a placeholder AvlTree.
    compile_tree(
        r#"{ sigmaProp(treeLookup(SELF.R4[AvlTree].get, SELF.propositionBytes, SELF.propositionBytes).isDefined) }"#,
    );
}

// ---------------------------------------------------------------------------
// Explicit numeric casts (SigmaPredef list — distinct from .toLong/.toBigInt methods)
// ---------------------------------------------------------------------------

#[test]
fn predef_upcast() {
    // upcast[Long](Int) — explicit widening. Produces SLong.
    let expr = compile_ok(r#"{ upcast[Long](0) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_downcast() {
    // downcast[Byte](Int) — explicit narrowing. Produces SByte.
    let expr = compile_ok(r#"{ downcast[Byte](0) }"#);
    assert_eq!(expr.tpe(), SType::SByte);
}

// ---------------------------------------------------------------------------
// MAST / segregation primitives (S70)
// ---------------------------------------------------------------------------

#[test]
fn predef_execute_from_var() {
    // executeFromVar[T](id) → T. Lowers to DeserializeContext { tpe: T, id }.
    // Mirrors Scala's mkDeserializeContext (SigmaPredef.scala:402).
    let expr = compile_ok(r#"{ executeFromVar[Long](0) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_execute_from_self_reg() {
    // executeFromSelfReg[T](id) → T. Lowers to DeserializeRegister { reg, tpe: T, default: None }.
    // Mirrors Scala's mkDeserializeRegister(r, rtpe, None) (SigmaPredef.scala:505).
    let expr = compile_ok(r#"{ executeFromSelfReg[Long](4) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_execute_from_self_reg_with_default() {
    // executeFromSelfRegWithDefault[T](id, default) → T.
    // Lowers to DeserializeRegister { reg, tpe: T, default: Some(default) }.
    // Mirrors Scala's mkDeserializeRegister(r, rtpe, Some(default)) (SigmaPredef.scala:419).
    let expr = compile_ok(r#"{ executeFromSelfRegWithDefault[Long](4, 0L) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_placeholder() {
    // placeholder[T](id) → T. Lowers to ConstantPlaceholder { id, tpe: T }.
    // Internal segregation primitive; user code may not normally call it,
    // but the IR node is wired for completeness (Scala has it as parser-only
    // with `undefined` irBuilder, SigmaPredef.scala:730).
    let expr = compile_ok(r#"{ placeholder[Long](0) }"#);
    assert_eq!(expr.tpe(), SType::SLong);
}

#[test]
fn predef_deserialize_compile_time() {
    // deserialize[T]("base58") → T. Compile-time base58 decode + sigma_parse,
    // inlining the resulting expression. Mirrors Scala's DeserializeFunc
    // (SigmaPredef.scala:169).
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::serialization::SigmaSerializable;

    // Build a known expr, serialize, base58-encode, and run it back through
    // the predef. Must round-trip to the original.
    let original: Expr = Constant::from(42i64).into();
    let bytes = original.sigma_serialize_bytes().unwrap();
    let b58 = bs58::encode(&bytes).into_string();
    let src = format!(r#"{{ deserialize[Long]("{}") }}"#, b58);
    let result = compile_ok(&src);
    assert_eq!(result.tpe(), SType::SLong);
    assert_eq!(result, original);
}

#[test]
fn predef_zk_proof_block() {
    // ZKProof { sigmaPropExpr } → SBoolean. Mirrors Scala's ZKProofFunc
    // (SigmaPredef.scala:125; mkZKProofBlock irBuilder). Frontend-only: typer
    // accepts it, but serialization fails with NotSupported and eval errors —
    // matching Scala's `OpCodes.Undefined` + `testMissingCostingWOSerialization`.
    use ergotree_ir::mir::expr::Expr;
    let expr = compile_ok(r#"{ ZKProof { sigmaProp(HEIGHT > 1000) } }"#);
    assert_eq!(expr.tpe(), SType::SBoolean);
    assert!(matches!(expr, Expr::ZkProofBlock(_)));
}

#[test]
fn predef_zk_proof_block_rejects_non_sigma_prop() {
    // The block body must be a SigmaProp value. A bare Boolean body is
    // rejected by `try_build` (mirrors Scala's typer that requires
    // SigmaPropValue). The error surfaces as a MIR lowering failure.
    use ergoscript_compiler::compiler::compile_expr;
    use ergoscript_compiler::script_env::ScriptEnv;
    let res = compile_expr(r#"{ ZKProof { HEIGHT > 1000 } }"#, ScriptEnv::new());
    assert!(
        res.is_err(),
        "expected ZKProof with non-SigmaProp body to fail lowering, got: {:?}",
        res
    );
}

#[test]
fn predef_zk_proof_block_serialization_not_supported() {
    // Once an Expr::ZkProofBlock IR node is built, attempting to serialize it
    // must fail (no canonical op-code; mirrors Scala's `OpCodes.Undefined`).
    use ergotree_ir::serialization::SigmaSerializable;
    let expr = compile_ok(r#"{ ZKProof { sigmaProp(HEIGHT > 1000) } }"#);
    let res = expr.sigma_serialize_bytes();
    assert!(
        res.is_err(),
        "expected ZKProof serialization to fail, got Ok({:?} bytes)",
        res.as_ref().map(|b| b.len())
    );
}

// ---------------------------------------------------------------------------
// Parser-level constructs (NOT predef funcs; verify language syntax works)
// ---------------------------------------------------------------------------

#[test]
fn parser_if_expression() {
    // `if (cond) ... else ...` — ternary expression syntax.
    compile_tree(r#"{ sigmaProp(if (HEIGHT > 0) true else false) }"#);
}

#[test]
fn parser_select_field_tuple() {
    // tuple `._1` / `._2` field selection (selectField in SigmaPredef).
    compile_tree(r#"{ sigmaProp((1L, 2L)._1 > 0L) }"#);
}

#[test]
fn parser_apply_lambda() {
    // Function value application (apply in SigmaPredef).
    compile_tree(r#"{ sigmaProp({ (x: Long) => x > 0L }(1L)) }"#);
}
