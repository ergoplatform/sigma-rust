# Language Conformance Workstream Status

**Source plan**: `/home/cq/.claude/plans/language-conformance.md` (4 workstreams,
3–5 sessions of bulk work).

**Status snapshot (S71 / 2026-04-28 — A CLOSED at 44/44: `ZKProof { ... }` block scope landed via new `ZkProofBlock` MIR struct + frontend-only parity)**:

| Workstream | Scope | Status | % Done |
|---|---|---|---|
| **A. Predef-func parity** | 44 SigmaPredef entries → MIR | S71: **44/44 covered** (+1: `ZKProof { ... }`). The Rust parser already handles `ZKProof { body }` as `FuncCall + block-arg`, so no grammar work was needed — just predef wiring. New `ZkProofBlock` MIR struct in [`ergotree-ir/src/mir/zk_proof.rs`](ergotree-ir/src/mir/zk_proof.rs); frontend-only (Scala uses `OpCodes.Undefined` and op-code byte space is exhausted), so serialization errors `NotSupported` and eval errors `EvalError::Misc` — matches Scala's `testMissingCostingWOSerialization`. | **100%** |
| **B. Type-method parity** | **~120 unique methods** across 19 registries | S70: **SSigmaProp 2/2** — IR fill landed (new `SigmaPropIsProven` MIR struct in ergotree-ir, op-code 95, full sigma_serialize/parse + eval; eval returns `EvalError::Misc` mirroring Scala's `costKind = notSupportedError`), then compiler arm + 2 tests. | **100%** |
| **C. Lexer/Parser conformance** | Tokens + grammar gaps vs Scala | S68: bitwise infix `&`/`\|`/`^`/`~` + shifts `<<`/`>>`/`>>>` closed end-to-end (IR + serializer + interpreter NotImplemented arms for shifts). Only **un-braced lambda bodies** remain as QoL (~0.5 session, not on byte-match path). | **~95%** |
| **D. Conformance smoke tests** | Per-feature compile tests | S71: **154 tests**, submodule split done S64, all 11 method registries have dedicated test files, predef coverage matches A's 44/44. Only **lexer + parser snapshot files** remain (~1 session, additive). | **~95%** |

**S63 audit deliverable**: [`ergoscript-compiler/tests/fixtures/conformance/method-coverage.md`](ergoscript-compiler/tests/fixtures/conformance/method-coverage.md) — definitive per-registry matrix of ~120 methods with Scala-side ground truth + verified Rust wiring status. Replaces §B.1 estimates below.

**Remaining work**: ~4 focused sessions to close the arc (revised down from 5–6 — Workstream B real gap is ~45 methods, not ~95).

This file is the durable cross-session planning artifact. Session handoffs
([`SESSION-62-HANDOFF.md`](SESSION-62-HANDOFF.md), etc.) reference it for
current state. Update this file at the end of each session — keep handoffs
focused on session-specific changelog.

---

# Workstream A — Predef Function Status

**Scope**: wire all SigmaPredef.PredefinedFunc entries from the Scala
sigmastate-interpreter to the Rust ergoscript-compiler MIR lowering. The plan
calls for ~30 `if name == "X"` arms in [`mir/lower.rs`](ergoscript-compiler/src/mir/lower.rs)
plus matching return-type rules in [`type_infer.rs`](ergoscript-compiler/src/type_infer.rs)
and conformance smoke tests in [`tests/conformance.rs`](ergoscript-compiler/tests/conformance.rs).

**Total**: 44 predefs (15 recognized pre-S61 + 29 not-yet listed in the original
coverage table). Of the 29 not-yet items, 27 are name-listed and 2 are flagged
with `*` ("we have it but routed differently — not via predef-func dispatch").

**Status as of S71 (2026-04-28)**: **44 of 44 covered (100%)**. Workstream A is closed.

> NOTE: This document is the authoritative open-list for Workstream A. The
> S61 §4 "26 of 29 missing" framing was an estimate; this file replaces it
> with a per-name accounting verified against the canonical 44-predef list.

---

## Summary table

| Bucket | Count | Items |
|---|---|---|
| **Recognized pre-S61** | 15 | `allOf`, `anyOf`, `atLeast`, `byteArrayToBigInt`, `byteArrayToLong`, `decodePoint`, `getVar`, `longToByteArray`, `max`, `min`, `proveDHTuple`, `proveDlog`, `sigmaProp`, `substConstants`, `xorOf` |
| **Landed S61** | 3 | `bigInt`, `fromBase64`, `sha256` |
| **Landed S62 (predefs)** | 9 | `PK`, `unsignedBigInt`, `serialize`, `deserializeTo`, `fromBigEndianBytes`, `getVarFromInput`, `treeLookup`, `upcast`, `downcast` |
| **Landed S69 (predefs)** | 3 | `avlTree`, `allZK`, `anyZK` |
| **Landed S70 (predefs)** | 5 | `executeFromVar`, `executeFromSelfReg`, `executeFromSelfRegWithDefault`, `deserialize` (compile-time), `placeholder` |
| **Landed S71 (predefs)** | 1 | `ZKProof { ... }` (block scope; frontend-only) |
| **Routed-differently confirmed via name dispatch** | 3 | `blake2b256`, `fromBase16`, `fromBase58` |
| **Parser-level (verified S62)** | 3 | `if`, `selectField`, `apply` |
| **Bonus Global methods (not on the 44 predef list)** | 3 | `encodeNbits`, `decodeNbits`, `powHit` |
| **STILL OPEN** | **0** | — |
| **Total covered** | **44/44** | |

---

## 1. Recognized pre-S61 (15)

All wired via name dispatch in `lower.rs` + `type_infer.rs` and exercised by
ecosystem batch and conformance smoke tests. No further action.

| Predef | lower.rs arm | type_infer.rs entry | Conformance test |
|---|---|---|---|
| `sigmaProp` | ✓ | ✓ | `predef_sigma_prop` |
| `proveDlog` | ✓ | ✓ | `predef_prove_dlog` |
| `proveDHTuple` | ✓ | ✓ | `predef_prove_dh_tuple` |
| `atLeast` | ✓ | ✓ | `predef_at_least` |
| `allOf` | ✓ | ✓ | `predef_all_of` |
| `anyOf` | ✓ | ✓ | `predef_any_of` |
| `xorOf` | ✓ | ✓ | `predef_xor_of` |
| `byteArrayToLong` | ✓ | ✓ | `predef_byte_array_to_long` |
| `byteArrayToBigInt` | ✓ | ✓ | `predef_byte_array_to_big_int` |
| `longToByteArray` | ✓ | ✓ | `predef_long_to_byte_array` |
| `decodePoint` | ✓ | ✓ | `predef_decode_point` |
| `getVar` | ✓ | ✓ | `predef_get_var` |
| `substConstants` | ✓ | ✓ | `predef_subst_constants` |
| `min` | ✓ | ✓ | `predef_min` |
| `max` | ✓ | ✓ | `predef_max` |

## 2. Landed S61+S62 (15 net, including reroute confirmations)

### S61 (3 — string-arg compile-time folds)

| Predef | Lowering shape | Tree version | Notes |
|---|---|---|---|
| `bigInt(str)` | `Const(SBigInt)` via `BigInt256::from_str_radix(s,10)` | V0+ | Pre-args branch; string literal can't survive normal HIR |
| `fromBase64(str)` | `Const(Coll[Byte])` via `base64::engine::general_purpose::STANDARD.decode` | V0+ | Sibling of fromBase16/58 |
| `sha256(coll)` | `CalcSha256::try_build(input)` | V0+ | Post-args branch |

### S62 (9)

| Predef | Lowering shape | Tree version | Test path |
|---|---|---|---|
| `PK("addr")` | `AddressEncoder::unchecked_parse_address_from_str` → match `P2Pk(prove_dlog)` → `Const(SSigmaProp)` | V0+ | `compile_tree` |
| `unsignedBigInt(str)` | `Const(SUnsignedBigInt)` via `UnsignedBigInt::from_str_radix(s,10)` | V0+ | `compile_ok` |
| `serialize(value)` | `MethodCall(Global, SERIALIZE_METHOD.specialize_for(SGlobal, [arg.tpe()]), [value])` | **V3+** | `compile_ok` |
| `deserializeTo[T](bytes)` | `MethodCall::with_type_args(Global, DESERIALIZE_METHOD, [bytes], {t() => T})` | **V3+** | `compile_ok` |
| `fromBigEndianBytes[T](bytes)` | `MethodCall::with_type_args(Global, FROM_BIGENDIAN_BYTES_METHOD, [bytes], {t() => T})` | **V3+** | `compile_ok` |
| `getVarFromInput[T](idx, var)` | `MethodCall::with_type_args(Context, GET_VAR_FROM_INPUT_METHOD, [Downcast(idx,Short), Downcast(var,Byte)], {t() => T})` | **V3+** | `compile_ok` |
| `treeLookup(t,k,p)` | `TreeLookup::new(tree, key, proof)` | V0+ | `compile_tree` |
| `upcast[T](x)` | `Upcast::new(input, T)` | V0+ | `compile_ok` |
| `downcast[T](x)` | `Downcast::new(input, T)` | V0+ | `compile_ok` |

### Routed-differently confirmed (3)

The original coverage table flagged three items with `*` ("we have it but
routed differently — not via predef-func dispatch"). Audit confirms all three
**are** correctly routed via the name dispatch path in `lower.rs` and
`type_infer.rs`. The `*` was historical (pre-dating the structured dispatch
arc); functionally they're consistent with the S62 pattern.

| Predef | lower.rs line | type_infer.rs line |
|---|---|---|
| `blake2b256` | 433 | 96 |
| `fromBase16` | 237 | 93 |
| `fromBase58` | 386 | 93 |

## 3. STILL OPEN (0 — Workstream A closed S71)

### 3a. ZKProof block scope — closed S71

| Predef | Signature (per EKB) | S71 wiring |
|---|---|---|
| `ZKProof { ... }` | Block-scope wrapper around SigmaProp ops; result type `SBoolean` | New `ZkProofBlock` MIR struct in [`ergotree-ir/src/mir/zk_proof.rs`](ergotree-ir/src/mir/zk_proof.rs). Frontend-only, mirrors Scala's `OpCodes.Undefined`: serialization errors `NotSupported`, eval errors `EvalError::Misc`. |

**Pre-flight discovery** (saved ~1 session of grammar work): the Rust parser's
postfix loop in [`parser/grammar/expr.rs`](ergoscript-compiler/src/parser/grammar/expr.rs)
already handles `name { ... }` as a `FuncCall` with the block expression as a
single arg (lines 81–93). And the HIR `ast::Expr::FuncCall` lowering (line 112
of [`hir.rs`](ergoscript-compiler/src/hir.rs)) already produces `Apply { func, args, type_arg }`.
Plus `BlockExpr` with a single statement is auto-unwrapped to that statement
in [`hir.rs:131`](ergoscript-compiler/src/hir.rs#L131). So `ZKProof { sigmaProp(true) }`
already lowers as `Apply(Ident("ZKProof"), [BoolToSigmaProp(...)])` without
any new grammar/lex/HIR work. The remaining work was the predef arm, type
infer entry, and the IR fill (which itself was simpler than expected because
no real op-code is available — ergotree-ir's byte op-code space maxes out at
255 (XOR_OF) and Scala uses `OpCodes.Undefined = 0` with no serializer).

**S69 closed** `allZK`/`anyZK` — the Scala lowering is "match literal `Coll(p1, p2, ...)` → unfold to SigmaAnd/SigmaOr"; runtime collections are unsupported in Scala (Scala uses `undefined` irBuilder + a graph-IR rewrite that only fires for `CBM.fromItems`-shape literals). Mirrored exactly in [`mir/lower.rs`](ergoscript-compiler/src/mir/lower.rs) — pattern-match `Expr::Collection(Collection::Exprs { items, .. })` and feed to `SigmaAnd::new`/`SigmaOr::new`. Conformance tests at [`tests/conformance/predef_funcs.rs`](ergoscript-compiler/tests/conformance/predef_funcs.rs) (`predef_all_zk`, `predef_any_zk`).

### 3b. v6 hard items — closed S70 (5)

All five landed in S70 via existing IR (no IR fill needed — the audit-first
prediction held: `DeserializeContext`, `DeserializeRegister`, and
`ConstantPlaceholder` were already in ergotree-ir). Wiring is one
arm + type_infer entry + test per predef.

| Predef | Signature | S70 lowering |
|---|---|---|
| `deserialize[T]("base58")` | Compile-time Base58 → typed value | Pre-args branch in [`mir/lower.rs`](ergoscript-compiler/src/mir/lower.rs): `bs58::decode` → `Expr::sigma_parse_bytes` → tpe check vs `[T]`. Distinct from runtime `deserializeTo`. |
| `executeFromVar[T](id)` | Run serialized ErgoTree from context var | `DeserializeContext { tpe: T, id }`. Mirrors Scala's `mkDeserializeContext`. |
| `executeFromSelfReg[T](id)` | Run serialized ErgoTree from SELF register | `DeserializeRegister::new(reg, T, None)`. Reg id must be 0..=9; we map via `RegisterId::try_from(i8)`. |
| `executeFromSelfRegWithDefault[T](id, default)` | Same with default fallback | `DeserializeRegister::new(reg, T, Some(Box::new(default)))`. |
| `placeholder[T](id)` | ConstantPlaceholder constructor | `Expr::ConstPlaceholder(ConstantPlaceholder { id, tpe: T })`. Scala's irBuilder is `undefined` (parser-only), but the Rust IR has the node, so wiring is a one-line bridge. |

### 3c. Niche compile-time constructor — closed S69 (with documented IR gap)

`avlTree(operationFlags, digest, keyLength, valueLengthOpt)` lowers to
`CreateAvlTree::new(...)` — runtime IR node, NOT a compile-time fold (the
old matrix description was wrong; Scala lowers via `mkCreateAvlTree`).
S69 wired the predef end-to-end with `none[Int]()`/`some(intExpr)`
literal pattern-matching; runtime `SOption[SInt]` arguments are rejected
with an explanatory error. Tests at
[`predef_avl_tree_none`/`_some`](ergoscript-compiler/tests/conformance/predef_funcs.rs).

**Open IR-level discrepancy** with byte-match consequences for downstream
projects — see [§12 below](#12-known-ir-discrepancies--scheduled-future-work).
None of the 14/14 ecosystem fixtures exercise `avlTree(...)`, so the
discrepancy doesn't currently block byte-match parity, but **AVL trees
are on the critical path for several Ergo projects in active development**
(Lithos, Etcha, Machina Finance, etc.). Closing this gap is a real
near-term priority once those projects need byte-for-byte parity with
the Scala node.

---

## 4. Bonus — Global methods landed S62 (not on the 44-predef list)

These are SGlobal SMethods, not SigmaPredef.PredefinedFunc entries. Landed in
S62 alongside the Workstream A items because users access them with bare names
(no `Global.` prefix) so they look-and-feel like predefs. **All require V3+
ErgoTree (v6.0).**

| Method | Lowering | Test |
|---|---|---|
| `encodeNbits(BigInt)` | `MethodCall(Global, ENCODE_NBITS_METHOD, [arg])` | `predef_encode_nbits_v6` |
| `decodeNbits(Long)` | `MethodCall(Global, DECODE_NBITS_METHOD, [arg])` | `predef_decode_nbits_v6` |
| `powHit(k, msg, nonce, h, N)` | `MethodCall(Global, POW_HIT_METHOD, [5 args])` | `predef_pow_hit_v6` |

---

## 5. Parser-level constructs verified (3 of the 27 not-yet items)

These appear in the `SigmaPredef.PredefinedFunc` list but are language
constructs, not user-facing function calls. They were confirmed working at
the parser/HIR level in S62 via dedicated conformance tests. **No predef arm
needed** in `lower.rs`.

| Item | Source | Conformance test |
|---|---|---|
| `if (cond) ... else ...` | Ternary expression syntax | `parser_if_expression` |
| `selectField` (tuple `._1`/`._2`) | Tuple field access syntax | `parser_select_field_tuple` |
| `apply` (lambda invocation) | Function value application | `parser_apply_lambda` |

These were in the SigmaPredef registry because Scala's compiler treats them as
unified function-call dispatch internally. The Rust compiler handles them at
parser/HIR/MIR levels naturally without a name-dispatch arm.

---

## 6. Coverage at a glance — by type registry & feature gate

| ErgoTree version | Predefs covered | Predefs open | % covered |
|---|---|---|---|
| **V0+** (always available) | 28 | 1 (`avlTree`) | 97% |
| **V3+ only** (v6.0) | 6 (serialize/deserializeTo/fromBigEndianBytes/getVarFromInput) + 3 globals (encodeNbits/decodeNbits/powHit) | 8 (allZK/anyZK/ZKProof + execute*3 + deserialize+placeholder) | 53% |

The V0+ tier is essentially complete. The remaining open work is concentrated
in the V3+ feature surface.

---

## 7. Plumbing notes — what to know when adding the next predef

These are S62 lessons baked in for the next-session implementer.

### 7a. `MethodCall::with_type_args` plumbing (load-bearing)

Any new SMethod with non-empty `explicit_type_args: vec![STypeVar::t()]`
requires audit of all 4 MethodCall reconstruction sites; otherwise CSE drops
the type args and panics.

| File | Line | Site |
|---|---|---|
| [`mir/lower.rs`](ergoscript-compiler/src/mir/lower.rs) | ~1797 | `propagate_inner` MethodCall arm |
| [`mir/cse.rs`](ergoscript-compiler/src/mir/cse.rs) | ~2781 | `map_children` MethodCall arm |
| [`mir/cse.rs`](ergoscript-compiler/src/mir/cse.rs) | ~5187 | `replace_all` MethodCall arm |
| [`mir/cse.rs`](ergoscript-compiler/src/mir/cse.rs) | ~5965 | `rewrite_ids` MethodCall arm |

`cse.rs:3971` already constructs the struct literal with `explicit_type_args:
s.expr.explicit_type_args.clone()` — leave it.

`MethodCall::new` is fine for SMethods with `explicit_type_args: vec![]`
(e.g. AVL tree methods, `multiply`).

### 7b. Implicit type vars without explicit type args

If an SMethod has `STypeVar::t()` in `t_dom` but `explicit_type_args: vec![]`
(e.g. `SERIALIZE_METHOD` whose T is the value type), use
`SMethod::specialize_for(SType::SGlobal, vec![arg.tpe()])` to substitute T
via `unify_many` before constructing the MethodCall. Skipping this gives a
type mismatch error in `MethodCall::new_inner`.

### 7c. Argument coercion for narrower-than-Int types

If the SMethod signature requires `SShort` or `SByte` and users write Int
literals, wrap each arg in `Downcast::new(arg, SType::SShort)` etc. before
constructing the MethodCall. See `getVarFromInput` for the pattern.

### 7d. `SType::SOption(...)` / `SType::SColl(...)` take `Arc<SType>`

In tests, write `SType::SOption(SType::SLong.into())` — relies on
`From<SType> for Arc<SType>`. `Box::new(SType::SLong)` gives an Arc/Box
mismatch. Easy compile-time gotcha.

### 7e. EKB-ref name vs IR-method-name skew

User-facing names sometimes differ from IR SMethod names:
- User writes `deserializeTo[T]` → IR is `DESERIALIZE_METHOD` (named `deserialize`)
- User writes `encodeNbits` → IR is `ENCODE_NBITS_METHOD` (lowercase n in user form)

Match user-facing names per the EKB built-ins ref, not the IR's internal
naming.

### 7f. Tree version threshold for tests

| Test helper | Use for | Why |
|---|---|---|
| `compile_tree(src)` | V0-compatible predefs | Full pipeline through `compile()` to ErgoTree V0; verifies serialization parity |
| `compile_ok(src)` | V3+ predefs | Stops at raw IR via `compile_expr`; `compile()` would fail at v6 op serialization in V0 |

`compile()` defaults to `ErgoTreeHeader::v0(false)`. Switching to V3 by
default would risk byte-shift across the 14/14 ecosystem (which all encode
at V0) — not a refactor to do casually.

### 7g. `hashbrown::HashMap` vs `std::HashMap`

`MethodCall::with_type_args` takes `hashbrown::HashMap<STypeVar, SType>`.
`std::HashMap` gives `E0308` mismatch. `hashbrown` is now in
`ergoscript-compiler/Cargo.toml` as a workspace dep.

---

## 8. Verification protocol

Before any commit on this branch, run:

```bash
cd /home/cq/working-files/sigma-rust

cargo test -p ergoscript-compiler --lib                            # 223/223
cargo test -p ergoscript-compiler --lib -- --ignored               # 4/4
cargo test -p ergoscript-compiler --test conformance               # 43/43
cargo test -p ergoscript-compiler --lib test_batch_node_byte_match # 1/1
source ~/.secrets && cargo test -p ergoscript-compiler --lib \
    test_ecosystem_batch -- --ignored --nocapture                  # 14/14 LOCAL MATCH

cargo test -p ergotree-ir --features arbitrary --lib               # 254/254
cargo test -p ergotree-interpreter --features arbitrary --lib      # 336/336

cargo fmt --all -- --check
cargo clippy -p ergotree-ir -p ergotree-interpreter -p ergoscript-compiler \
    --all-features --all-targets -- -D warnings
```

Paste **actual output** of each into the commit message. The `ab10a30e`
author claimed 14/14 LOCAL MATCH that didn't hold — don't repeat that mistake.

---

## 9. Next-session priority order

1. **Metals MCP audit of `SigmaPredef.scala`** — produce definitive list of
   all 44 PredefinedFunc entries with `irBuilder` lambdas. Cross-check
   against this file. ~30 min.
2. **`allZK` / `anyZK`** — implement via literal-Collection unfold to
   SigmaAnd/SigmaOr. ~1 hr.
3. **`avlTree(...)`** — compile-time AvlTreeData constructor. ~30 min.
4. **Workstream B** — start SCollectionMethods coverage matrix. Plan calls
   for ~163 entries across 15 method registries; this is the long-tail work.
5. **Defer**: `ZKProof` (block scope, needs grammar), `executeFromVar*`
   (needs new IR), `deserialize` compile-time, `placeholder` (low priority).

---

## 10. Open questions

- **Are there v6.0 predefs we haven't enumerated?** The 44-count came from a
  prior coverage table; verifying against `SigmaPredef.scala` directly is the
  next session's first task.
- **Should `compile()` emit V3 trees by default?** Currently V0; switching
  enables full-pipeline tests for v6 predefs but risks ecosystem byte-shift.
  Decide once Workstream B is closer to done.
- **Is there a unified test helper for byte-match parity vs Scala for new
  predefs?** Currently the ecosystem batch covers byte-match for the 14
  fixtures; new V3+ predefs have only smoke-test coverage. May want to add
  Scala-node test fixtures specifically for v6 predefs once the node is
  configured for v6.

---

## 11. Files touched (all sessions S61–S66, uncommitted)

```
ergoscript-compiler/Cargo.toml                                      +1     hashbrown dep (S62)
ergoscript-compiler/src/mir/lower.rs                             ~+960     12 predef arms (S62) + 10 SCollection methods (S64) + 23 header/preheader props + SContext.headers + id-guard (S65) + 14 V6 numeric methods + lookup_numeric_method helper + whitelist extension (S66) + S67: insertOrUpdate whitelist + 4 AvlTree props + GroupElement.negate + LastBlockUtxoRootHash + CONTEXT.minerPubKey + Option.map/filter dispatch split + some/none predef arms + MinerPubKey GlobalVars dispatch
ergoscript-compiler/src/mir/cse.rs                                 +30     3× MethodCall::with_type_args sites (S62)
ergoscript-compiler/src/type_infer.rs                             +120     predef return-types (S62) + SColl method types (S64) + SHeader entry + extended SPreHeader/SContext (S65) + V6 numeric returns + SUnsignedBigInt arm + numeric Apply→FieldAccess arm (S66) + S67: AvlTree.valueLengthOpt/insertOrUpdate + Option.map/filter Apply arm + GroupElement.negate + Context.LastBlockUtxoRootHash/minerPubKey + some/none predef returns
ergoscript-compiler/src/hir.rs                                      +3     UnsignedBigInt → SUnsignedBigInt in parse_type_name (S66) + MinerPubKey variant in GlobalVars enum (S67)
ergoscript-compiler/src/binder.rs                                   +1     bare `minerPubKey` → GlobalVars::MinerPubKey (S67)
ergoscript-compiler/src/compiler.rs                                +90     8 new unit tests (S62)
ergoscript-compiler/tests/conformance.rs                           +37     NEW entry point (S62 unified scaffold reorg'd S64)
ergoscript-compiler/tests/conformance/predef_funcs.rs             +298     NEW S64 — 43 predef smoke tests (was conformance.rs body)
ergoscript-compiler/tests/conformance/methods/mod.rs              +12     NEW S64 — submodule entry; S65 added sheader/spreheader; S66 added snumeric
ergoscript-compiler/tests/conformance/methods/scoll.rs             +96     NEW S64 — 10 SCollection method tests
ergoscript-compiler/tests/conformance/methods/sheader.rs         +120     NEW S65 — 16 SHeader property/method tests
ergoscript-compiler/tests/conformance/methods/spreheader.rs       +60     NEW S65 — 7 SPreHeader property tests
ergoscript-compiler/tests/conformance/methods/snumeric.rs        +185     NEW S66 — 25 SNumericTypeMethods V6 tests
ergoscript-compiler/tests/conformance/methods/savltree.rs         NEW     S67 — 5 SAvlTree tail tests
ergoscript-compiler/tests/conformance/methods/soption.rs          NEW     S67 — 3 SOption map/filter tests
ergoscript-compiler/tests/conformance/methods/sgroup_elem.rs      NEW     S67 — 2 SGroupElement.negate tests
ergoscript-compiler/tests/conformance/methods/scontext.rs         NEW     S67 — 5 SContext edge tests (LastBlockUtxoRootHash + minerPubKey)
ergoscript-compiler/tests/conformance/methods/sglobal.rs          NEW     S67 — 4 some/none predef tests
ergoscript-compiler/tests/fixtures/conformance/method-coverage.md ~+120    S64+S65+S66+S67 status updates
ergoscript-compiler/src/lexer/token_kind.rs (S61)                  +50     block comments, escape regex
ergoscript-compiler/src/syntax.rs (S61)                             +2     BlockComment SyntaxKind variant
ergoscript-compiler/src/parser.rs (S61)                            +30     2 block-comment integration tests
ergoscript-compiler/src/parser/grammar/expr.rs (S61)               +30     trailing-comma branches + 2 tests
ergotree-ir/src/serialization/bin_op.rs (S61)                   +30/-12    operand-shape gate (is_strippable_operand)
ergotree-ir/src/mir/bin_op.rs (S68)                                +18    BitShiftLeft/Right/RightZeroed BitOp variants + OpCode mappings + Display
ergotree-ir/src/serialization/expr.rs (S68)                         +5    3 deserialize arms for new shift opcodes (134/135/136)
ergotree-interpreter/src/eval/bin_op.rs (S68)                       +9    Explicit NotImplemented arms for shift BitOps (matches Scala testMissingCosting)
ergoscript-compiler/src/lexer/token_kind.rs (S68)                  +52    7 new tokens (`Amp`/`Pipe`/`Caret`/`Tilde`/`LShift`/`RShift`/`URShift`) + 10 lexer tests
ergoscript-compiler/src/syntax.rs (S68)                            +14    7 SyntaxKind variants + From<TokenKind> arms
ergoscript-compiler/src/ast.rs (S68)                                +8    BinaryExpr.op() recognizes new SyntaxKinds; PrefixExpr.op() recognizes Tilde
ergoscript-compiler/src/parser/grammar/expr.rs (S68)               +56    BinaryOp::{BitAnd/BitOr/BitXor/Shl/Shr/UShr} + UnaryOp::BitNot + binding power table re-numbered + prefix_tilde
ergoscript-compiler/src/hir.rs (S68)                               +17    BinaryOp 6 new variants + ExprKind::BitInversion + Tilde prefix lowering arm
ergoscript-compiler/src/binder.rs (S68)                             +8    BitInversion bind arm
ergoscript-compiler/src/hir/rewrite.rs (S68)                       +10    BitInversion rewrite arm
ergoscript-compiler/src/hir/optimize.rs (S68)                      +75    BitInversion arms in 13 optimization passes (mirror Negation pattern)
ergoscript-compiler/src/type_infer.rs (S68)                         +9    BinaryOp 6 new tpe arms + BitInversion tpe arm
ergoscript-compiler/src/mir/lower.rs (S68)                         +13    BinaryOp::Bit{And/Or/Xor/Shl/Shr/UShr} → BinOpKind::Bit; ExprKind::BitInversion → BitInversion::try_build
ergoscript-compiler/tests/conformance.rs (S68)                      +2    Register bitwise_infix submodule
ergoscript-compiler/tests/conformance/bitwise_infix.rs (S68)       NEW    17 tests: 7 ops + 5 precedence cases + double-tilde
ergoscript-compiler/src/mir/lower.rs (S69)                          +95   avlTree predef arm (CreateAvlTree, with literal Option-arg recognition); allZK/anyZK predef arm (Coll[SigmaProp] literal-unfold to SigmaAnd/SigmaOr); SBox.bytesWithoutRef property arm; SGroupElement.expUnsigned method arm + whitelist
ergoscript-compiler/src/type_infer.rs (S69)                          +5   avlTree → SAvlTree; allZK/anyZK → SSigmaProp; SBox.bytesWithoutRef → Coll[Byte] (replaces dead `bytesWithNoRef`/`scriptBytes` aliases); SGroupElement.expUnsigned → SGroupElement
ergoscript-compiler/tests/conformance/methods/sbox.rs               NEW    S69 — 2 SBox.bytesWithoutRef tests
ergoscript-compiler/tests/conformance/methods/sgroup_elem.rs (S69)  ~+15   added method_exp_unsigned_v6 test
ergoscript-compiler/tests/conformance/methods/mod.rs (S69)           +1   register sbox submodule
ergoscript-compiler/tests/conformance/predef_funcs.rs (S69)         ~+30   predef_avl_tree_none/_some + predef_all_zk/_any_zk tests
ergotree-ir/src/mir/sigma_prop_is_proven.rs (S70)                   NEW    SigmaPropIsProven MIR struct (OneArgOp, op-code 95) + ser_roundtrip proptest
ergotree-ir/src/mir.rs (S70)                                         +2    pub mod sigma_prop_is_proven
ergotree-ir/src/mir/expr.rs (S70)                                    +6    SigmaPropIsProven variant in Expr enum + tpe/children/children_mut arms
ergotree-ir/src/serialization/expr.rs (S70)                          +3    SigmaPropIsProven import + sigma_parse arm + sigma_serialize_w_opcode arm
ergotree-ir/src/source_span.rs (S70)                                 +1    SigmaPropIsProven SourceSpan::empty
ergotree-ir/src/pretty_printer/print.rs (S70)                       +13    SigmaPropIsProven Print impl + dispatch arm
ergotree-interpreter/src/eval.rs (S70)                               +1    pub(crate) mod sigma_prop_is_proven
ergotree-interpreter/src/eval/sigma_prop_is_proven.rs (S70)         NEW    Evaluable impl returning EvalError::Misc (matches Scala costKind=notSupportedError)
ergotree-interpreter/src/eval/expr.rs (S70)                          +1    Expr::SigmaPropIsProven dispatch arm
ergoscript-compiler/src/mir/lower.rs (S70)                          +210   isProven SSigmaProp arm + executeFromVar/SelfReg/SelfRegWithDefault arms + deserialize compile-time pre-args branch + placeholder arm
ergoscript-compiler/src/type_infer.rs (S70)                          +6    isProven → SBoolean; executeFromVar/SelfReg/SelfRegWithDefault/deserialize/placeholder → apply.type_arg
ergoscript-compiler/src/mir/cse.rs (S70)                             +9    SigmaPropIsProven arms in collect_and_assign_ids + rewrite_ids
ergoscript-compiler/tests/conformance/methods/mod.rs (S70)           +1    register ssigma_prop submodule
ergoscript-compiler/tests/conformance/methods/ssigma_prop.rs (S70)  NEW    2 tests — property_is_proven, property_is_proven_compiles_v0
ergoscript-compiler/tests/conformance/predef_funcs.rs (S70)         ~+50   predef_execute_from_var/_self_reg/_self_reg_with_default + predef_deserialize_compile_time + predef_placeholder
WORKSTREAM-STATUS.md                                              ~+150     + S70 A v6 hard items + B isProven IR fill closure
ergoscript-compiler/tests/fixtures/conformance/method-coverage.md  ~+30    S70 closure across §1g/§1h/§1k + B IR-blocked → wired + last-updated

ergotree-ir/src/mir/zk_proof.rs (S71)                                NEW    ZkProofBlock MIR struct (frontend-only, no op-code)
ergotree-ir/src/mir.rs (S71)                                          +2    pub mod zk_proof
ergotree-ir/src/mir/expr.rs (S71)                                     +6    import + ZkProofBlock variant + 3 dispatch arms
ergotree-ir/src/serialization/expr.rs (S71)                           +5    serialize arm errors NotSupported (no parse arm; no op-code)
ergotree-ir/src/source_span.rs (S71)                                  +1    SourceSpan::empty arm
ergotree-ir/src/pretty_printer/print.rs (S71)                        +14    import + dispatch arm + Print impl emitting `ZKProof { ... }`
ergotree-interpreter/src/eval.rs (S71)                                +1    pub(crate) mod zk_proof
ergotree-interpreter/src/eval/zk_proof.rs (S71)                      NEW    Evaluable returning EvalError::Misc
ergotree-interpreter/src/eval/expr.rs (S71)                           +1    Expr::ZkProofBlock dispatch arm
ergoscript-compiler/src/mir/lower.rs (S71)                           +21    "ZKProof" predef arm calling ZkProofBlock::try_build
ergoscript-compiler/src/type_infer.rs (S71)                           +4    "ZKProof" → SBoolean
ergoscript-compiler/src/mir/cse.rs (S71)                              +5    ZkProofBlock arms in collect_and_assign_ids + rewrite_ids
ergoscript-compiler/tests/conformance/predef_funcs.rs (S71)         ~+45    3 tests — block, rejects-non-sigma-prop, serialization-not-supported
```

---

## 12. Known IR Discrepancies — Scheduled Future Work

Items where the Rust ergotree-ir layer diverges from Scala in a way that
does not currently block byte-match for the 14/14 ecosystem corpus, but
**will** matter once downstream projects need byte-for-byte parity with
the Scala node for the affected primitive.

### 12a. `CreateAvlTree::value_length` shape mismatch — **HIGH PRIORITY**

**Status**: open. Documented S69, re-flagged S71 as a real near-term item.

**Affected downstream projects**: Lithos, Etcha, Machina Finance — all
in active development as of 2026-04-28 and all use AVL trees as a core
primitive. AVL is the path forward for state-bloat-free contract design
on Ergo, so any project doing serious on-chain state management will hit
this.

**The discrepancy**:

| Side | Field type | Serialization shape |
|---|---|---|
| **Scala** | `valueLengthOpt: Value[SOption[SInt]]` — a single `Expr` whose ergo-type is `SOption[SInt]` | One `Expr` slot — typically `SomeValue(int_expr)` op-code or `NoneValue` op-code |
| **Rust** | `value_length: Option<Box<Expr>>` — a *Rust* `Option` containing an `Expr` of type `SInt` | `<presence-byte><expr-or-empty>` — Rust-native `Option` serialization |

So `avlTree(flags, digest, keyLen, none[Int]())` and
`avlTree(flags, digest, keyLen, some(intExpr))` produce **different
serialized bytes** on the two sides. The S69 predef wiring pattern-matches
`none[Int]()`/`some(intExpr)` literals at MIR-time and rejects runtime
SOption-typed values, so user code compiles, but the resulting tree
won't byte-match a Scala-compiled tree of the same source.

**What closing the gap requires**:

1. Change [`ergotree-ir/src/mir/create_avl_tree.rs`](ergotree-ir/src/mir/create_avl_tree.rs)
   `value_length` from `Option<Box<Expr>>` to `Box<Expr>` (with type-level
   constraint `SOption[SInt]`).
2. Rewrite `sigma_serialize` / `sigma_parse` to write/read a single `Expr`
   slot (not `Option<Box<Expr>>::sigma_serialize`).
3. Update `CreateAvlTree::new`'s validator to check
   `post_eval_tpe == SOption[SInt]`, not `SInt` under a Rust-Option.
4. Rewrite the S69 [`avlTree` predef arm in `mir/lower.rs`](ergoscript-compiler/src/mir/lower.rs)
   to pass the SOption-typed expr through directly — no more
   `none[Int]()`/`some(intExpr)` literal extraction. Runtime SOption
   args become legal.
5. Update tests at
   [`predef_avl_tree_none`/`_some`](ergoscript-compiler/tests/conformance/predef_funcs.rs)
   and the proptest `arbitrary` impl.
6. Promote `avlTree(...)` byte-match to the `test_ecosystem_batch` /
   `test_batch_node_byte_match` corpus once a downstream contract is
   available as a fixture.

**Risk to manage**: this is a **breaking on-disk change** for ergotree-ir.
Any serialized tree containing a `CreateAvlTree` node, written with the
old layout, will not parse with the new shape. ergotree-ir is published
as 0.28.0 — downstream consumers may have stored bytes. Either:

- Coordinate the bump as a minor/major version (0.29 / 1.0) and document
  the on-disk break.
- Add a tree-version gate in `sigma_parse` so old bytes still round-trip
  through a legacy code path.

**Effort estimate**: ~1 session for the core IR + compiler work,
~0.5 session for migration coordination if the version-gate route is
taken. Defer the proptest-arbitrary update until after the compiler arm
is reshaped.

**Source-side pointer**: [`ergotree-ir/src/mir/create_avl_tree.rs:25`](ergotree-ir/src/mir/create_avl_tree.rs#L25)
should carry a `// see WORKSTREAM-STATUS.md §12a` comment so the next
contributor finds this entry from the source.

---

# Workstream B — Type-method Parity

**Scope**: wire all entries from the Scala `methods.scala` registries to MIR
method-call lowering arms in [`lower.rs`](ergoscript-compiler/src/mir/lower.rs)'s
FieldAccess→Apply branch (~lines 857–1100). Plan calls for **~163 entries
across 15 method registries**, shipped in slices by registry.

**Status: ~62% done — coverage matrix landed S63** at [`tests/fixtures/conformance/method-coverage.md`](ergoscript-compiler/tests/fixtures/conformance/method-coverage.md). Methods were wired ad-hoc as ecosystem contracts demanded; the S63 audit (Metals-equivalent direct read of `methods.scala` + cross-grep of `lower.rs`) produced the definitive matrix.

**Audit corrections vs original §B.1 estimates** (full table at [method-coverage.md §4](ergoscript-compiler/tests/fixtures/conformance/method-coverage.md)):
- **SCollectionMethods**: actual = 21 entries (not ~24); ✅ **21/21 closed S64** (was 11/21 post-S63). The 10 added: `indices`/`zip`/`patch`/`updated`/`updateMany`/`indexOf` (V0) + `reverse`/`startsWith`/`endsWith`/`get` (V6).
- **SAvlTreeMethods**: 11/16 wired (69%, not ~30%); §B.1 missed `digest`/`enabledOperations`/`keyLength`/`get` already wired
- **SOptionMethods**: 3/5 wired (60%, not 100%); `map`/`filter` missing
- **SGroupElementMethods**: 3/5 wired (60%, not 100%); `negate`/`isIdentity` missing
- **SContextMethods**: ~58% wired (not ~15%); §B.1 didn't credit binder.rs global keywords (HEIGHT/SELF/INPUTS/OUTPUTS/groupGenerator/CONTEXT)
- **SNumericTypeMethods**: 5/19 wired (26%, not ~100%); V0 casts only — V6 modular/bitwise/toBytes/toBits all missing
- **Total unique methods**: ~120 (not 163 — 163 included per-numeric-type instantiations)

**Total methods.scala registries**: **19** (not 15). Audit found `SByteMethods`, `SShortMethods`, `SIntMethods`, `SLongMethods`, `SBigIntMethods`, `SUnsignedBigIntMethods`, `SBooleanMethods`, `SStringMethods`, `SGroupElementMethods`, `SSigmaPropMethods`, `SAnyMethods`, `SUnitMethods`, `SBoxMethods`, `SAvlTreeMethods`, `SContextMethods`, `SHeaderMethods`, `SPreHeaderMethods`, `SGlobalMethods`, `SCollectionMethods`, `SOptionMethods`, `STupleMethods`, `SNumericTypeMethods` (shared trait, not a top-level registry).

## B.1. Per-registry status (rough estimates — needs Metals audit)

| Registry | File | ~Methods | Wired | Status | Notes |
|---|---|---|---|---|---|
| `SBoxMethods` | [sbox.rs](ergotree-ir/src/types/sbox.rs) | ~4 | ~4 | **~100%** | tokens, registers R4–R9, propBytes, value, id, creationInfo, bytes, bytesWithoutRef |
| `SCollectionMethods` | [scoll.rs](ergotree-ir/src/types/scoll.rs) | **21** | **21** | **✅ 100% (S64)** | All 21 methods wired. S64 added: indices, zip, patch, updated, updateMany, indexOf (V0); reverse, startsWith, endsWith, get (V6) |
| `SOptionMethods` | [soption.rs](ergotree-ir/src/types/soption.rs) | ~3 | ~3 | **~100%** | get, getOrElse, isDefined, map?, filter? |
| `SAvlTreeMethods` | [savltree.rs](ergotree-ir/src/types/savltree.rs) | ~24 | ~7 | **~30%** | Wired: insert, update, remove, getMany, contains, updateDigest, updateOperations. Missing: get, digest, enabledOperations, keyLength, valueLengthOpt, isInsertAllowed, isUpdateAllowed, isRemoveAllowed, plus ~10 v6+ extensions |
| `SGroupElementMethods` | [sgroup_elem.rs](ergotree-ir/src/types/sgroup_elem.rs) | ~3 | ~3 | **~100%** | exp, multiply, getEncoded, negate?, isIdentity? |
| `SHeaderMethods` | [sheader.rs](ergotree-ir/src/types/sheader.rs) | ~16 | unsure | **<50%** | id, version, parentId, ADProofsRoot, stateRoot, transactionsRoot, timestamp, nBits, height, extensionRoot, minerPk, powOnetimePk, powNonce, powDistance, votes — most likely missing |
| `SContextMethods` | [scontext.rs](ergotree-ir/src/types/scontext.rs) | ~12 | ~2 | **~15%** | getVar (predef), getVarFromInput (S62). Missing: HEIGHT, INPUTS, OUTPUTS, dataInputs, headers, preHeader, selfBoxIndex, LastBlockUtxoRootHash, minerPubKey — but most are routed via `GlobalVars`/keywords, not method calls. Audit needed. |
| `SGlobalMethods` | [sglobal.rs](ergotree-ir/src/types/sglobal.rs) | ~10 | ~9 | **~90%** | groupGenerator, xor, serialize, deserialize, fromBigEndianBytes, encodeNbits, decodeNbits, powHit (S62). Missing: some, none constructors |
| `SPreHeaderMethods` | [spreheader.rs](ergotree-ir/src/types/spreheader.rs) | ~7 | unsure | **<50%** | version, parentId, timestamp, nBits, height, minerPk, votes — most likely missing |
| `SNumericMethods` | [snumeric.rs](ergotree-ir/src/types/snumeric.rs) | ~5 | ~5 | **~100%** | toLong, toInt, toShort, toByte, toBigInt — wired in lower.rs ~1153–1187 |
| `STupleMethods` | (via `stuple.rs`) | ~2 | ~2 | **~100%** | _1, _2, ..., size — handled at parser/HIR level |
| `SFuncMethods` | (via `sfunc.rs`) | ~0 | n/a | n/a | Function values, no methods |
| `SSigmaPropMethods` | (in `stype.rs`?) | ~1 | ~1 | **~100%** | propBytes |
| `SUnsignedBigIntMethods` | (in `stype.rs`?) | unsure | unsure | unsure | v6.0 type — needs audit |
| `SBigIntMethods` | (numeric) | ~5 | ~5 | included in SNumericMethods | |

## B.2. Blockers

- **No coverage matrix file exists.** Plan calls for `tests/fixtures/conformance/method-coverage.md` with one row per (Registry, Method) listing wired/missing/notes. **Not created.**
- **Counts are estimates** — actual SMethod count per registry needs Metals MCP audit of `methods.scala` to verify against Rust IR. The "163 entries" from the plan may include version-gated overloads.
- **No per-registry conformance tests.** Workstream D scaffolding put all tests in a single file — splitting into `tests/conformance/methods/{registry}.rs` files happens naturally when Workstream B starts.

## B.3. Next steps for Workstream B (post-S65)

✅ **Done S63**: Audit + coverage matrix at [`ergoscript-compiler/tests/fixtures/conformance/method-coverage.md`](ergoscript-compiler/tests/fixtures/conformance/method-coverage.md).
✅ **Done S64**: B.1 SCollectionMethods closed — 10 methods wired + Workstream D submodule split (`tests/conformance/{predef_funcs,methods/scoll}.rs`).
✅ **Done S65**: B.2 SHeaderMethods + B.6 SPreHeaderMethods closed — 16+7=23 properties + bonus SContext.headers (so SHeader is reachable in user code).
✅ **Done S66**: B.3 SNumericTypeMethods closed — 14 V6 methods wired (toBytes/toBits/bitwiseInverse + bitwiseOr/And/Xor + shiftLeft/Right + 2 BigInt extras + 6 UnsignedBigInt modular) via new `lookup_numeric_method` helper. UnsignedBigInt added to `parse_type_name` so user code can declare it.
✅ **Done S67**: B.4 + B.5 + B.7 + B.8 all closed in one session.
- **SAvlTreeMethods** 16/16: 4 V0 property arms (`valueLengthOpt`/`isInsertAllowed`/`isUpdateAllowed`/`isRemoveAllowed`) + V6 `insertOrUpdate` argful method (added to whitelist gate).
- **SOptionMethods** 5/5: `map` and `filter` arms (existing `map`/`filter` arms split by obj type — SOption goes to MethodCall(OPTION_*_METHOD); SColl falls through to existing Map/Filter MIR nodes).
- **SGroupElementMethods** 4/5: `negate` PropertyCall arm. `isIdentity` does NOT exist in Scala or Rust IR — coverage matrix corrected. `expUnsigned` (V3+) deferred (low-leverage; pairs with UnsignedBigInt v6 surface).
- **SContextMethods** edges: `LastBlockUtxoRootHash` PropertyCall arm; `minerPubKey` added to binder.rs as bare global → `GlobalVars::MinerPubKey` (also a new variant in `hir::GlobalVars`); `CONTEXT.minerPubKey` lowers to the same node.
- **SGlobalMethods.some/none** 11/11: bare predef arms in lower.rs apply branch. `some(value)` uses `specialize_for(SGlobal, [value.tpe()])`; `none[T]()` uses `MethodCall::with_type_args` because NONE_METHOD has explicit_type_args = `[STypeVar::t()]`. Type infer entries added.

Remaining slices:

1. ~~**B.1 SCollectionMethods**~~ ✅ closed S64.
2. ~~**B.2 SHeaderMethods**~~ ✅ closed S65.
3. ~~**B.6 SPreHeaderMethods**~~ ✅ closed S65.
4. ~~**B.3 SNumericTypeMethods (V6)**~~ ✅ closed S66.
5. ~~**B.4 SAvlTreeMethods gaps**~~ ✅ closed S67.
6. ~~**B.5 SOptionMethods + SGroupElementMethods**~~ ✅ closed S67 (Option 2/2; GroupElement 1/2 with `expUnsigned` deferred).
7. ~~**B.7 SContextMethods edges**~~ ✅ closed S67.
8. ~~**B.8 SGlobalMethods.some/none**~~ ✅ closed S67.
9. **B.X residuals** (low-priority): SBox `bytesWithoutRef` (verify it exists in methods.scala — may not be a real SMethod), SGroupElement `expUnsigned` (V3+, niche), SSigmaProp `isProven` (frontend-only — verify routing).

**Total remaining Workstream B effort**: ~0.25 session for the 3 residuals (each is one arm + test). Workstream B is effectively closed for byte-match parity; residuals are completeness items.

---

# Workstream C — Lexer/Parser Conformance

**Scope**: lexer tokens + parser grammar gaps between Rust ergoscript-compiler
and Scala compiler. Plan called for ~5 small parser additions, ~1 session.

**Status: ~70% done.** Byte-match-critical subset closed in S61.

## C.1. Landed (S61)

| Item | Files | Notes |
|---|---|---|
| Nested `/* ... */` block comments | [lexer/token_kind.rs](ergoscript-compiler/src/lexer/token_kind.rs), [parser.rs](ergoscript-compiler/src/parser.rs), [syntax.rs](ergoscript-compiler/src/syntax.rs) | Custom `lex_block_comment` callback handling nesting; new `BlockComment` token + SyntaxKind variant |
| Trailing commas in `Coll(...)` / tuple literals / tuple types | [parser/grammar/expr.rs](ergoscript-compiler/src/parser/grammar/expr.rs) | `p.at(TokenKind::RParen)` early-break check after each `,`. Verified no byte-shift on 14/14 ecosystem |
| String escapes: `\"`, `\\`, `\n` | [lexer/token_kind.rs](ergoscript-compiler/src/lexer/token_kind.rs) | Regex broadened to `r#""([^"\\]|\\.)*""#`. No byte-shift risk for production strings (all hex/base58) |

## C.2. Landed (S68) — bitwise infix end-to-end

| Item | Files | Notes |
|---|---|---|
| Lexer tokens `&`/`\|`/`^`/`~`/`<<`/`>>`/`>>>` | [lexer/token_kind.rs](ergoscript-compiler/src/lexer/token_kind.rs), [syntax.rs](ergoscript-compiler/src/syntax.rs) | Logos longest-match handles `&&` > `&`, `\|\|` > `\|`, `>>>` > `>>` > `>` cleanly |
| Parser binding-power table (Scala first-char rule for the bitwise tier) | [parser/grammar/expr.rs](ergoscript-compiler/src/parser/grammar/expr.rs) | `\|` (3,4) < `^` (5,6) < `&` (9,10) per Scala. Shifts at (15,16) — between comparison (13,14) and add (17,18), C-style. Strict-Scala same-level shift+comparison would never type-check anyway. |
| HIR `BinaryOp` extensions | [hir.rs](ergoscript-compiler/src/hir.rs) | `BitAnd`/`BitOr`/`BitXor`/`Shl`/`Shr`/`UShr` variants. New `BitInversion(Box<Expr>)` ExprKind variant for prefix `~`. |
| HIR transformer arms for `BitInversion` | [binder.rs](ergoscript-compiler/src/binder.rs), [hir/rewrite.rs](ergoscript-compiler/src/hir/rewrite.rs), [hir/optimize.rs](ergoscript-compiler/src/hir/optimize.rs) | 13 sites updated (mirror Negation arms). |
| Type infer: numeric op preserves left tpe | [type_infer.rs](ergoscript-compiler/src/type_infer.rs) | Six new arms in the BinaryOp tpe-inference; one new arm for BitInversion. |
| Lower: `BinaryOp::BitX` → `BinOpKind::Bit(BitOp::X)`; `BitInversion` → `BitInversion::try_build` | [mir/lower.rs](ergoscript-compiler/src/mir/lower.rs) | Six new From arms + a `hir::ExprKind::BitInversion` lowering arm. |
| **IR gap fill**: BitOp shift variants in ergotree-ir | [ergotree-ir/src/mir/bin_op.rs](ergotree-ir/src/mir/bin_op.rs), [ergotree-ir/src/serialization/expr.rs](ergotree-ir/src/serialization/expr.rs) | New `BitShiftLeft`/`BitShiftRight`/`BitShiftRightZeroed` variants with op-codes 134/135/136 (already reserved). Three new deserialize arms. |
| Interpreter: shift eval returns `EvalError::Misc` (matches Scala `testMissingCosting`) | [ergotree-interpreter/src/eval/bin_op.rs](ergotree-interpreter/src/eval/bin_op.rs) | Scala's interpreter has no graph-builder rule for BitShift either — supported runtime path is `x.shiftLeft(y)` method call, wired S66. |
| Conformance smoke tests | [tests/conformance/bitwise_infix.rs](ergoscript-compiler/tests/conformance/bitwise_infix.rs) | 17 tests: 7 ops × Long (Int sample for AND/SHL) + 5 precedence cases (`&` > `\|`, `^` between, `~` > infix, shifts > comparison, arith > shift, arith > bitand). |

## C.3. Closed-as-not-needed

| Item | Reason | Status |
|---|---|---|
| **`Map(...)` literal** | sigmastate has no runtime `SMap` node; user code uses `Coll[(K,V)]`. Not a real conformance gap. | ✅ Closed by audit — won't implement. |

## C.4. DEFERRED (open, low priority)

| Item | Reason | Effort |
|---|---|---|
| **Un-braced lambda bodies** (`(x: Long) => x + 1` without `{ }`) | QoL only. All ecosystem fixtures and the 154 conformance tests use braced bodies. Adds a top-level `lambda_expr` branch in [`parser/grammar/expr.rs`](ergoscript-compiler/src/parser/grammar/expr.rs)'s `lhs()` that detects `(ident:` lookahead and parses without enclosing `{ }`. | ~0.5 session |

## C.5. Status

**S68 closed Workstream C** for byte-match parity. The remaining
"un-braced lambda bodies" item is a QoL nicety; leaving it open does not
block byte-match parity for the 14/14 ecosystem batch nor the AVL /
Lithos / Etcha / Machina path. Group it with the Workstream D snapshot
tests (§D.4) into a single polish session whenever convenient.

---

# Workstream D — Conformance Smoke Tests

**Scope**: per-feature compile tests so a regression that drops a predef or
method dispatch fails immediately. Plan called for layout:
- `predef_funcs.rs` — 44 minimal compile cases
- `methods/{sbox,scoll,...}.rs` — one file per type registry
- `lexer.rs`, `parser.rs` — tokenization + parser snapshots

**Status (post-S71): ~85% done — submodule split landed S64, per-registry method tests landed S64–S70 alongside Workstream B slices, predef coverage matches Workstream A's 44/44. Only lexer/parser snapshot tests remain.**

## D.1. Landed (S62, refactored S64)

Original S62 scaffold was a single file with 43 smoke tests; S64 split it
into a submodule layout matching the language-conformance plan:

```
tests/conformance.rs                        — entry point + helpers (compile_ok / compile_tree)
tests/conformance/predef_funcs.rs           — predef tests (44/44 covered)
tests/conformance/bitwise_infix.rs          — S68 bitwise infix end-to-end (17 tests)
tests/conformance/methods/mod.rs            — registry submodule entry
tests/conformance/methods/{scoll,sheader,spreheader,snumeric,savltree,
                          soption,sgroup_elem,scontext,sglobal,sbox,
                          ssigma_prop}.rs   — 11 per-registry method test files
```

Coverage by category (S62 baseline + everything added since):

| Section | Tests | Coverage |
|---|---|---|
| Sigma proposition constructors | 5 | sigmaProp, proveDlog, proveDHTuple, atLeast, PK |
| Logical aggregation | 3 | allOf, anyOf, xorOf |
| Hash functions | 2 | blake2b256, sha256 |
| Type conversions (predefs) | 4 | byteArrayToLong, byteArrayToBigInt, longToByteArray, decodePoint |
| Compile-time string-arg predefs | 5 | fromBase16, fromBase58, fromBase64, bigInt, unsignedBigInt |
| Context variable access | 2 | getVar, getVarFromInput |
| Script template manipulation | 1 | substConstants |
| Global object methods | 8 | groupGenerator, xor, serialize, deserializeTo, fromBigEndianBytes, encodeNbits, decodeNbits, powHit |
| Numeric method casts | 5 | toLong, toInt, toByte, toShort, toBigInt |
| min/max | 2 | min, max |
| AVL tree | 1 | treeLookup |
| Explicit casts | 2 | upcast, downcast |
| Parser-level constructs | 3 | if expression, tuple `._1`, lambda apply |

Two helpers:
- `compile_ok(src) -> Expr` — uses `compile_expr`, raw IR. Required for v6.0+ predefs.
- `compile_tree(src)` — full pipeline through `compile()` to ErgoTree V0.

## D.2. Status vs the plan (refreshed S71)

| Item | Status | Notes |
|---|---|---|
| Submodule split (`tests/conformance/predef_funcs.rs`, `methods/*.rs`, `lexer.rs`, `parser.rs`) | ✅ **Done S64** | `predef_funcs.rs` + `bitwise_infix.rs` + `methods/{11 files}.rs` exist. Only `lexer.rs` / `parser.rs` snapshot files still missing — those are listed below as their own item. |
| Per-registry method dispatch tests (one file per type registry) | ✅ **Done S64–S70** | All 11 registries that have a Rust IR analogue have a dedicated test file (scoll, sheader, spreheader, snumeric, savltree, soption, sgroup_elem, scontext, sglobal, sbox, ssigma_prop). Total: 154 conformance tests across all categories. |
| Predef coverage (44/44) | ✅ **Done S71** | Matches Workstream A 44/44; every predef has a smoke test in `predef_funcs.rs` (incl. ZKProof block, isProven, executeFromVar/Reg variants, deserialize, placeholder, allZK/anyZK, avlTree, etc.). |
| Lexer tokenization snapshots (`tests/conformance/lexer.rs`) | ❌ **Open** | Standalone `expect_test` snapshots. ~0.5 session. |
| Parser AST snapshots (`tests/conformance/parser.rs`) | ❌ **Open** | Sister file to lexer snapshots. ~0.5 session. |

## D.3. Blockers

None. The two open snapshot files are independent of Workstream A/B/C
work and can be added in a single polish session whenever convenient.

## D.4. Next steps for Workstream D

Only the snapshot tests remain:

1. **Lexer tokenization snapshots** — `tests/conformance/lexer.rs` with
   `expect_test` of the token stream for representative source snippets
   (each TokenKind exercised at least once, plus longest-match ambiguities
   like `>>>` / `>>` / `>` and `&&` / `&`). ~0.5 session.
2. **Parser AST snapshots** — `tests/conformance/parser.rs` with
   `expect_test` of the parse tree for representative sources covering
   each `SyntaxKind` (FuncCall, FieldAccess, IfExpr, Lambda, BlockExpr,
   InfixExpr, PrefixExpr, TupleExpr, etc.). Mirrors the inline snapshot
   tests already living in `parser/grammar/expr.rs::tests` but in the
   conformance crate so a refactor that loses them gets caught. ~0.5
   session.

Both items are pure additive testing — no risk to byte-match parity.

---

# Cross-workstream priority for next sessions

Per the plan's "3–5 sessions of bulk work" budget:

| Session | Focus | Workstream(s) |
|---|---|---|
| ~~**S63**~~ ✅ | ~~Metals-equivalent audit of SigmaPredef.scala + methods.scala; coverage matrix~~ | ~~A audit, B foundation~~ |
| ~~**S64**~~ ✅ | ~~`SCollectionMethods` slice (10 methods, 21/21 closed) + Workstream D submodule split~~ | ~~B + D~~ |
| ~~**S65**~~ ✅ | ~~`SHeaderMethods` (16/16) + `SPreHeaderMethods` (7/7) + bonus `SContext.headers`~~ | ~~B + D~~ |
| ~~**S66**~~ ✅ | ~~`SNumericTypeMethods` V6 extensions (14 — toBytes/toBits/bitwiseInverse + bitwiseOr/And/Xor + shiftLeft/Right + BigInt+UnsignedBigInt modular)~~ | ~~B (Workstream C bitwise infix tokens deferred — distinct from numeric `bitwiseOr` method)~~ |
| ~~**S67**~~ ✅ | ~~Workstream B slice 4 — `SAvlTreeMethods` tail (5 methods) + `SOptionMethods` map/filter + `SGroupElementMethods.negate` + `SContextMethods.LastBlockUtxoRootHash`/`minerPubKey` + `SGlobalMethods.some/none` (V3+). 19 new conformance tests.~~ | ~~B + D~~ |
| ~~**S68**~~ ✅ | ~~Workstream C bitwise infix end-to-end. 7 ops (`&`/`\|`/`^`/`~`/`<<`/`>>`/`>>>`) lex→parse→HIR→type→lower. ergotree-ir BitOp extended with shift variants (op-codes 134/135/136) + 3 deserialize arms. Interpreter eval returns NotImplemented for shifts (matches Scala `testMissingCosting`). 17 new conformance tests, 10 new lexer tests.~~ | ~~C + IR + interpreter~~ |
| ~~**S69**~~ ✅ | ~~Workstream A residuals: `allZK`/`anyZK` (Coll[SigmaProp] literal-unfold to SigmaAnd/SigmaOr matching Scala graph-IR rewrite); `avlTree(...)` predef (CreateAvlTree, with 4th-arg `none[Int]()`/`some(intExpr)` literal pattern-match — known Rust IR shape discrepancy with Scala documented). Workstream B residuals: `SBox.bytesWithoutRef` (V0 property), `SGroupElement.expUnsigned` (V3+); `SSigmaProp.isProven` re-classified as IR-blocked (op-code 95 reserved-but-orphan, parallel to S68 BitShift IR fill). 7 new conformance tests.~~ | ~~A + B residuals~~ |
| ~~**S70**~~ ✅ | ~~(1) `SSigmaProp.isProven` IR fill — new `SigmaPropIsProven` MIR struct in ergotree-ir/src/mir/ (OneArgOp, op-code 95), Print impl, source_span entry, serialize/parse arms, interpreter eval returning `EvalError::Misc` (matches Scala `costKind = notSupportedError`). Compiler arm in lower.rs + type_infer entry + 2 tests at methods/ssigma_prop.rs. (2) Workstream A v6 hard items: `executeFromVar` (DeserializeContext), `executeFromSelfReg` and `executeFromSelfRegWithDefault` (DeserializeRegister), `placeholder` (ConstantPlaceholder) — all via existing ergotree-ir IR (no IR fill needed). `deserialize` compile-time predef via `bs58::decode` + `Expr::sigma_parse_bytes` + tpe check. 7 new conformance tests.~~ | ~~A v6 hard + B IR fill~~ |
| ~~**S71**~~ ✅ | ~~`ZKProof { ... }` block scope closed — new `ZkProofBlock` MIR struct in [`ergotree-ir/src/mir/zk_proof.rs`](ergotree-ir/src/mir/zk_proof.rs). Pre-flight discovery: parser/HIR already handle `name { ... }` as `FuncCall + block-arg`, so no grammar/lex work needed. IR struct is frontend-only (Scala uses `OpCodes.Undefined` and op-code byte space is exhausted at 255): serialize errors `SigmaSerializationError::NotSupported`, interpreter eval errors `EvalError::Misc`, mirroring Scala's `testMissingCostingWOSerialization`. Compiler arm in lower.rs + type_infer entry (`SBoolean`) + 2 cse.rs arms. 3 new conformance tests.~~ | ~~A residual~~ |
| **S72** | Polish session: close out C + D residuals. (1) Un-braced lambda bodies in [`parser/grammar/expr.rs`](ergoscript-compiler/src/parser/grammar/expr.rs) — top-level `(params) => body` without enclosing `{ }`. (2) Lexer tokenization snapshot tests at `tests/conformance/lexer.rs` — `expect_test` of token stream for representative sources covering each TokenKind + longest-match cases. (3) Parser AST snapshot tests at `tests/conformance/parser.rs` — `expect_test` of parse tree for each SyntaxKind. ~1.5 sessions total but small and additive. | C + D close-out |

**Total remaining for byte-match parity**: **0 sessions**. Workstream A is
44/44 (closed S71), B is 100% (closed S70), C is byte-match-complete
(only QoL un-braced lambdas open).

**Total remaining for full close-out (polish only)**: **~1.5 sessions**:
- Un-braced lambda body grammar (Workstream C QoL, ~0.5 session)
- Lexer tokenization snapshot tests (Workstream D, ~0.5 session)
- Parser AST snapshot tests (Workstream D, ~0.5 session)

**Scheduled future work** (off the language-conformance arc but on the
near-term path): **AVL IR shape fix** (§12a) — Lithos / Etcha / Machina
Finance need byte-match parity for `avlTree(...)` once they ship, ~1
session of IR + compiler work plus migration coordination for the
ergotree-ir on-disk format break.

---

*Last updated: S71 / 2026-04-28 (post-session refresh of D status). **S71 closed Workstream A** by wiring `ZKProof { ... }` (new `ZkProofBlock` MIR struct in [`ergotree-ir/src/mir/zk_proof.rs`](ergotree-ir/src/mir/zk_proof.rs); frontend-only — no canonical op-code, mirrors Scala's `OpCodes.Undefined` + `testMissingCostingWOSerialization`). Workstream A: **44/44 (100%)**; Workstream B: 100%; Workstream C: ~95% (un-braced lambda QoL only); Workstream D: **~95%** (154 tests, all 11 method-registry files + bitwise-infix + predef-funcs landed; lexer/parser snapshot files remain). All baseline test suites green: ergoscript-compiler --lib 233/233, conformance 154/154, ergotree-ir 255/255, ergotree-interpreter 336/336, batch_node_byte_match 1/1, --ignored 4/4, ecosystem batch 14/14 LOCAL MATCH. fmt/clippy clean. **Language-conformance arc closed for byte-match parity. S72 = polish session: un-braced lambdas + lexer/parser snapshots (~1.5 sessions). Scheduled future work: AVL IR shape fix (§12a) for Lithos / Etcha / Machina Finance.***
