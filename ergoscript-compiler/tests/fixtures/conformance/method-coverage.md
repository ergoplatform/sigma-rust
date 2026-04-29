# Language Conformance Coverage Matrix

**Sources of truth (Scala canonical)**:
- `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/SigmaPredef.scala` (754 lines, ~44 PredefinedFunc entries)
- `~/working-files/sigmastate-interpreter/data/shared/src/main/scala/sigma/ast/methods.scala` (2023 lines, 19 MethodsContainer registries)

**Rust dispatch sites (verified by direct grep, S63)**:
- Predef name dispatch: [`mir/lower.rs`](../../../src/mir/lower.rs) lines 415–1000 (apply-of-name branch)
- Method-call dispatch: [`mir/lower.rs`](../../../src/mir/lower.rs) lines 957–1280 (FieldAccess→Apply branch)
- Property accessors: [`mir/lower.rs`](../../../src/mir/lower.rs) lines 1359–1500 (FieldAccess branch)
- Global keywords: [`binder.rs`](../../../src/binder.rs) lines 90–104
- Type inference returns: [`type_infer.rs`](../../../src/type_infer.rs) lines 92–135
- Conformance smoke tests: [`tests/conformance.rs`](../../conformance.rs)

**Wiring legend**:
- ✅ **Wired** — has lower.rs arm + type_infer.rs entry + conformance test
- 🟡 **Partial** — IR exists in ergotree-ir, but no lower.rs dispatch arm (still callable via raw IR but compiler rejects)
- ❌ **Missing** — no Rust dispatch (would need new lowering arm and possibly new IR)
- ◽ **Auto** — handled by parser/HIR/binder (no name-dispatch arm needed)

**Audit history**: This file generated S63 / 2026-04-28. Cross-checks two Explore-agent reports (SigmaPredef + methods.scala) against the Rust source. **Some agent claims were re-verified manually** — see §4 for corrections.

---

## §1. SigmaPredef.PredefinedFunc — 44 entries (35 wired)

### §1a. Sigma propositions & ZK conjuncts (7 — 6 wired, 1 missing)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `sigmaProp` | V0 | `(Boolean) → SigmaProp` | `BoolToSigmaProp` | ✅ | `predef_sigma_prop` |
| `proveDlog` | V0 | `(GroupElement) → SigmaProp` | `CreateProveDlog` | ✅ | `predef_prove_dlog` |
| `proveDHTuple` | V0 | `(GroupElement)×4 → SigmaProp` | `CreateProveDHTuple` | ✅ | `predef_prove_dh_tuple` |
| `atLeast` | V0 | `(Int, Coll[SigmaProp]) → SigmaProp` | `AtLeast` | ✅ | `predef_at_least` |
| `allZK` | V0 | `(Coll[SigmaProp]) → SigmaProp` | `SigmaAnd(coll)` | ✅ (S69) | `predef_all_zk` |
| `anyZK` | V0 | `(Coll[SigmaProp]) → SigmaProp` | `SigmaOr(coll)` | ✅ (S69) | `predef_any_zk` |
| `ZKProof` | V0 | block-scope | `ZKProofBlock` | ❌ | — |

**S69 closure note**: `allZK`/`anyZK` use Scala's `undefined` irBuilder + a graph-IR rewrite that only matches literal `Coll(p1, p2, ...)` shapes (`CBM.fromItems.unapply`). Mirrored exactly: pattern-match `Expr::Collection(Collection::Exprs { items, .. })` at MIR-time and unfold to `SigmaAnd::new(items)`/`SigmaOr::new(items)` (≥2 items required). Runtime `Coll[SigmaProp]` rejected with explanatory error.

**Open work**: `ZKProof` needs grammar work (block keyword) + new IR.

### §1b. Logical aggregation (3 — 3 wired)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `allOf` | V0 | `(Coll[Boolean]) → Boolean` | `SigmaAnd` | ✅ | `predef_all_of` |
| `anyOf` | V0 | `(Coll[Boolean]) → Boolean` | `SigmaOr` | ✅ | `predef_any_of` |
| `xorOf` | V0 | `(Coll[Boolean]) → Boolean` | `XorOf` | ✅ | `predef_xor_of` |

### §1c. Hash functions (2 — 2 wired)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `blake2b256` | V0 | `(Coll[Byte]) → Coll[Byte]` | `CalcBlake2b256` | ✅ | `predef_blake2b256` |
| `sha256` | V0 | `(Coll[Byte]) → Coll[Byte]` | `CalcSha256` | ✅ | `predef_sha256` |

### §1d. Type conversions (4 — 4 wired)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `byteArrayToBigInt` | V0 | `(Coll[Byte]) → BigInt` | `ByteArrayToBigInt` | ✅ | `predef_byte_array_to_big_int` |
| `byteArrayToLong` | V0 | `(Coll[Byte]) → Long` | `ByteArrayToLong` | ✅ | `predef_byte_array_to_long` |
| `longToByteArray` | V0 | `(Long) → Coll[Byte]` | `LongToByteArray` | ✅ | `predef_long_to_byte_array` |
| `decodePoint` | V0 | `(Coll[Byte]) → GroupElement` | `DecodePoint` | ✅ | `predef_decode_point` |

### §1e. Compile-time string-arg constructors (5 — 5 wired)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `fromBase16` | V0 | `(String) → Coll[Byte]` | `Const(SColl(SByte))` | ✅ | `predef_from_base16` |
| `fromBase58` | V0 | `(String) → Coll[Byte]` | `Const(SColl(SByte))` | ✅ | `predef_from_base58` |
| `fromBase64` | V0 | `(String) → Coll[Byte]` | `Const(SColl(SByte))` | ✅ | `predef_from_base64` |
| `bigInt` | V0 | `(String) → BigInt` | `Const(SBigInt)` | ✅ | `predef_big_int_decimal` |
| `unsignedBigInt` | V0 | `(String) → UnsignedBigInt` | `Const(SUnsignedBigInt)` | ✅ | `predef_unsigned_big_int_decimal` |

### §1f. Address/key constructors (1 — 1 wired)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `PK` | V0 | `(String) → SigmaProp` | base58 decode → `Const(SSigmaProp(ProveDlog))` | ✅ | `predef_pk` |

### §1g. Global / serialization (V3+ tier — 4 wired, 1 missing)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `serialize` | **V3+** | `[T](T) → Coll[Byte]` | `MethodCall(Global, SERIALIZE_METHOD, [v])` (specialize_for) | ✅ | `predef_serialize_v6` |
| `deserializeTo` | **V3+** | `[T](Coll[Byte]) → T` | `MethodCall::with_type_args(Global, DESERIALIZE_METHOD)` | ✅ | `predef_deserialize_to_v6` |
| `fromBigEndianBytes` | **V3+** | `[T](Coll[Byte]) → T` | `MethodCall::with_type_args(Global, FROM_BIGENDIAN_BYTES_METHOD)` | ✅ | `predef_from_big_endian_bytes_v6` |
| `substConstants` | V0 | `[T](Coll[Byte], Coll[Int], Coll[T]) → Coll[Byte]` | `SubstConstants` | ✅ | `predef_subst_constants` |
| `deserialize` | V0 | `[T](String) → T` | base58 decode + sigma_parse → inlined Expr | ✅ (S70) | `predef_deserialize_compile_time` |

**S70 closure note**: `deserialize[T]("base58")` mirrors `fromBase58` plumbing — pre-args branch in [`mir/lower.rs`](../../../src/mir/lower.rs) decodes the string literal at compile time, calls `Expr::sigma_parse_bytes`, and verifies the resulting expr's tpe matches the requested `[T]`. Distinct from `deserializeTo[T](bytes)` (runtime, V3+).

### §1h. Context variables (5 — 2 wired, 3 missing)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `getVar` | V0 | `[T](Byte) → Option[T]` | `GetVar(id, T)` | ✅ | `predef_get_var` |
| `getVarFromInput` | **V3+** | `[T](Short, Byte) → Option[T]` | `MethodCall::with_type_args(Context, GET_VAR_FROM_INPUT)` + Downcast args | ✅ | `predef_get_var_from_input_v6` |
| `executeFromVar` | V0 | `[T](Byte) → T` | `DeserializeContext(id, T)` | ✅ (S70) | `predef_execute_from_var` |
| `executeFromSelfReg` | V0 | `[T](Int) → T` | `DeserializeRegister(regId, T, None)` | ✅ (S70) | `predef_execute_from_self_reg` |
| `executeFromSelfRegWithDefault` | V0 | `[T](Int, T) → T` | `DeserializeRegister(regId, T, Some(default))` | ✅ (S70) | `predef_execute_from_self_reg_with_default` |

**S70 closure note**: All three landed via existing IR — `DeserializeContext` and `DeserializeRegister` were already in [`ergotree-ir/src/mir/`](../../../../../ergotree-ir/src/mir/) with full sigma_parse/serialize/eval. Only the compiler dispatch arms + type_infer entries were missing. The audit-first prediction in S69 §4b held — these were one-arm-each wires.

### §1i. AVL tree (2 — 2 wired)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `treeLookup` | V0 | `(AvlTree, Coll[Byte], Coll[Byte]) → Option[Coll[Byte]]` | `TreeLookup` | ✅ | `predef_tree_lookup` |
| `avlTree` | V0 | `(Byte, Coll[Byte], Int, Option[Int]) → AvlTree` | `CreateAvlTree(...)` (NOT compile-time fold; old matrix entry was wrong) | ✅ (S69) | `predef_avl_tree_none`/`_some` |

**S69 closure note**: `avlTree(...)` lowers to `CreateAvlTree::new(flags, digest, key_length, value_length)`. **Known IR-shape discrepancy**: Rust's `CreateAvlTree::value_length` is `Option<Box<Expr>>` (compile-time Option of an Int Expr), while Scala carries a runtime `SOption[SInt]` expression. The S69 wiring pattern-matches `none[Int]()`/`some(intExpr)` literals at MIR-time and rejects runtime Option-typed values with an explanatory error. Byte-match against Scala for this predef is a known IR-level discrepancy; none of the 14/14 ecosystem fixtures exercise it.

### §1j. Numeric (4 — 4 wired)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `min` | V0 | `[T](T, T) → T` | `ArithOp(Min)` | ✅ | `predef_min` |
| `max` | V0 | `[T](T, T) → T` | `ArithOp(Max)` | ✅ | `predef_max` |
| `upcast` | V0 | `[T,R](T) → R` | `Upcast(input, R)` | ✅ | `predef_upcast` |
| `downcast` | V0 | `[T,R](T) → R` | `Downcast(input, R)` | ✅ | `predef_downcast` |

### §1k. Special (2 — 1 wired, 1 missing)

| Predef | MinVersion | Signature | irBuilder | Status | Test |
|---|---|---|---|---|---|
| `xor` | V0 | `(Coll[Byte], Coll[Byte]) → Coll[Byte]` | `Xor` | ✅ | `predef_xor_byte_array` |
| `placeholder` | V0 | `[T](Int) → T` | `ConstantPlaceholder(id, T)` | ✅ (S70) | `predef_placeholder` |

**S70 closure note**: Wired even though Scala's irBuilder is `undefined` (not user-callable in Scala) — the Rust IR has the node, so the compiler dispatch is one arm + type_infer entry. Internal const-segregation primitive; not exercised by user contracts.

### §1l. Parser-level constructs (3 — 3 wired ◽)

| Item | Source-form | How handled in Rust | Test |
|---|---|---|---|
| `if` | `if (cond) ... else ...` | Parser/HIR → MIR `If` | `parser_if_expression` ◽ |
| `selectField` | `tuple._1` / `._2` | Parser/HIR → MIR `SelectField` | `parser_select_field_tuple` ◽ |
| `apply` | `f(x)` lambda invocation | Parser/HIR → MIR `Apply`/`FuncValue` | `parser_apply_lambda` ◽ |

### §1m. Operators (NOT in 44-count, generated via helpers)

Infix arithmetic (`+`/`-`/`*`/`/`/`%`), comparison (`<`/`<=`/`>`/`>=`/`==`/`!=`), logical (`&&`/`||`/`^`), and unary (`!`/`-`) all wired via parser→HIR→MIR `BinOp`/`UnaryOp` lowering. Not name-dispatch entries.

**Bitwise** (`&`/`|`/`^`/`<<`/`>>`/`>>>`/`~`): parser tokens NOT yet implemented in Rust (Workstream C deferred).

### §1n. Bonus — Global SMethods user-facing as predefs (3 — 3 wired)

These live in `SGlobalMethods` (not `SigmaPredef`) but users access with bare names so feel like predefs. **All V3+.**

| Method | Signature | irBuilder | Status | Test |
|---|---|---|---|---|
| `encodeNbits` | `(BigInt) → Long` | `MethodCall(Global, ENCODE_NBITS_METHOD)` | ✅ | `predef_encode_nbits_v6` |
| `decodeNbits` | `(Long) → BigInt` | `MethodCall(Global, DECODE_NBITS_METHOD)` | ✅ | `predef_decode_nbits_v6` |
| `powHit` | `(Int, Coll[Byte], Coll[Byte], Long, BigInt) → BigInt` | `MethodCall(Global, POW_HIT_METHOD)` (5 args) | ✅ | `predef_pow_hit_v6` |

---

## §2. SMethod registries — 19 containers in methods.scala

Counts are unique method signatures (not per-numeric-type instantiations). Status verified by direct grep of [`mir/lower.rs`](../../../src/mir/lower.rs) S63.

### §2a. SCollectionMethods (21 unique — 21 wired, 0 missing) ✅

| Method | MinVersion | Signature (Coll[T] receiver) | IR symbol | Status | lower.rs |
|---|---|---|---|---|---|
| `size` | V0 | `→ Int` | `SizeOf` (special) | ✅ | 1364 |
| `getOrElse` | V0 | `(Int, T) → T` | `ByIndex` (3rd arg) | ✅ | 1186 |
| `apply` | V0 | `(Int) → T` | `ByIndex` | ✅ | 1001 (auto via index) |
| `map` | V0 | `(T → U) → Coll[U]` | `Map` | ✅ | 1076 |
| `filter` | V0 | `(T → Bool) → Coll[T]` | `Filter` | ✅ | 1016 |
| `exists` | V0 | `(T → Bool) → Bool` | `Exists` | ✅ | 1029 |
| `forall` | V0 | `(T → Bool) → Bool` | `ForAll` | ✅ | 1042 |
| `fold` | V0 | `(U, (U,T) → U) → U` | `Fold` | ✅ | 1055 |
| `flatMap` | V0 | `(T → Coll[U]) → Coll[U]` | `FlatMap` | ✅ | 1102 |
| `slice` | V0 | `(Int, Int) → Coll[T]` | `Slice` | ✅ | 1206 |
| `append` | V0 | `(Coll[T]) → Coll[T]` | `Append` | ✅ | 1089 |
| `indices` | V0 | `→ Coll[Int]` | `INDICES_METHOD` (PropertyCall) | ✅ | 1366 (S64) |
| `zip` | V0 | `(Coll[U]) → Coll[(T,U)]` | `ZIP_METHOD` (specialize_for) | ✅ | 1228 (S64) |
| `patch` | V0 | `(Int, Coll[T], Int) → Coll[T]` | `PATCH_METHOD` (specialize_for) | ✅ | 1248 (S64) |
| `updated` | V0 | `(Int, T) → Coll[T]` | `UPDATED_METHOD` (specialize_for) | ✅ | 1271 (S64) |
| `updateMany` | V0 | `(Coll[Int], Coll[T]) → Coll[T]` | `UPDATE_MANY_METHOD` (specialize_for) | ✅ | 1293 (S64) |
| `indexOf` | V0 | `(T, Int) → Int` | `INDEX_OF_METHOD` (specialize_for) | ✅ | 1316 (S64) |
| `reverse` | **V6** | `→ Coll[T]` | `REVERSE_METHOD` (PropertyCall) | ✅ | 1376 (S64) |
| `startsWith` | **V6** | `(Coll[T]) → Bool` | `STARTS_WITH_METHOD` (specialize_for) | ✅ | 1338 (S64) |
| `endsWith` | **V6** | `(Coll[T]) → Bool` | `ENDS_WITH_METHOD` (specialize_for) | ✅ | 1361 (S64) |
| `get` | **V6** | `(Int) → Option[T]` | `GET_METHOD` (V6, specialize_for, gated `if SType::SColl(_)`) | ✅ | 1387 (S64) |

**S64 implementation notes** (Workstream B slice 1):
- All 10 newly-wired methods use `SMethod::specialize_for(obj.tpe(), arg_tpes)` to substitute the receiver's `STypeVar::t()` (and `STypeVar::iv()` for `zip`) before constructing `MethodCall`/`PropertyCall`. None require `MethodCall::with_type_args` since `explicit_type_args` is empty for every Coll method.
- `indices` and `reverse` are no-arg → `PropertyCall` in the FieldAccess branch (lower.rs:1366/1376), gated on `Some(SType::SColl(_))` to avoid name shadowing on other types.
- `Coll.get` is V6 and clashes with both `AvlTree.get` (handled at lower.rs:1124 with type guard) and `Option.get` (FieldAccess property at lower.rs:1405). The guard at the matches!() filter (lower.rs ~966) was widened from `Some(SType::SAvlTree)` to `Some(SType::SAvlTree | SType::SColl(_))` so dispatch reaches the right arm.
- Conformance tests live in [`tests/conformance/methods/scoll.rs`](../../conformance/methods/scoll.rs) — 10 tests (6 V0 via `compile_tree`, 4 V6 via `compile_ok`).
- Workstream D submodule split landed alongside: [`tests/conformance.rs`](../../conformance.rs) is now an entry-point declaring `mod predef_funcs; mod methods;` via `#[path = "..."]` attributes (required because integration-test crate roots resolve siblings, not subdirectory descendants).

**§B.1 correction (S63 finding, retained)**: WORKSTREAM-STATUS.md §B.1 listed "indices/zip/patch/updated/distinct/reverse/sortBy/take/drop/head/tail/last/init/min/max/sum/product/find/count/groupBy/mkString/unzip/flatten" as missing. **The actual scoll.rs has only 21 methods.** `distinct`/`sortBy`/`take`/`drop`/`head`/`tail`/`last`/`init`/`sum`/`product`/`find`/`count`/`groupBy`/`mkString`/`unzip`/`flatten` are NOT in methods.scala — they don't exist in Scala sigmastate.

**Wired count: 21/21 (100%) — closed S64.**

### §2b. SAvlTreeMethods (16 unique — 16 wired, 0 missing) ✅

| Method | MinVersion | Signature | IR symbol | Status | lower.rs |
|---|---|---|---|---|---|
| `digest` | V0 | `→ Coll[Byte]` (property) | `DIGEST_METHOD` | ✅ | 1483 |
| `enabledOperations` | V0 | `→ Byte` (property) | `ENABLED_OPERATIONS_METHOD` | ✅ | 1489 |
| `keyLength` | V0 | `→ Int` (property) | `KEY_LENGTH_METHOD` | ✅ | 1495 |
| `contains` | V0 | `(Coll[Byte], Coll[Byte]) → Bool` | `CONTAINS_METHOD` | ✅ | 1168 |
| `get` | V0 | `(Coll[Byte], Coll[Byte]) → Option[Coll[Byte]]` | `GET_METHOD` | ✅ | 1124 |
| `getMany` | V0 | `(Coll[Coll[Byte]], Coll[Byte]) → Coll[Option[Coll[Byte]]]` | `GET_MANY_METHOD` | ✅ | 1168 |
| `insert` | V0 | `(Coll[(Coll[Byte],Coll[Byte])], Coll[Byte]) → Option[AvlTree]` | `INSERT_METHOD` | ✅ | 1168 |
| `update` | V0 | `(Coll[(Coll[Byte],Coll[Byte])], Coll[Byte]) → Option[AvlTree]` | `UPDATE_METHOD` | ✅ | 1168 |
| `remove` | V0 | `(Coll[Coll[Byte]], Coll[Byte]) → Option[AvlTree]` | `REMOVE_METHOD` | ✅ | 1168 |
| `updateDigest` | V0 | `(Coll[Byte]) → AvlTree` | `UPDATE_DIGEST_METHOD` | ✅ | 1168 |
| `updateOperations` | V0 | `(Byte) → AvlTree` | `UPDATE_OPERATIONS_METHOD` | ✅ | 1168 |
| `valueLengthOpt` | V0 | `→ Option[Int]` (property) | `VALUE_LENGTH_OPT_METHOD` | ✅ | S67 |
| `isInsertAllowed` | V0 | `→ Bool` (property) | `IS_INSERT_ALLOWED_METHOD` | ✅ | S67 |
| `isUpdateAllowed` | V0 | `→ Bool` (property) | `IS_UPDATE_ALLOWED_METHOD` | ✅ | S67 |
| `isRemoveAllowed` | V0 | `→ Bool` (property) | `IS_REMOVE_ALLOWED_METHOD` | ✅ | S67 |
| `insertOrUpdate` | **V6** | `(Coll[(Coll[Byte],Coll[Byte])], Coll[Byte]) → Option[AvlTree]` | `INSERT_OR_UPDATE_METHOD` | ✅ | S67 (whitelist + MethodCall) |

**S67 implementation notes**:
- 4 V0 property arms in the FieldAccess match (gated on `Some(SType::SAvlTree)`); existing `enabledOperations`/`keyLength` arms also gated S67 to disambiguate from same-named arms on other types if added later.
- `insertOrUpdate` (V6) is argful → added to whitelist gate at lower.rs:1002; arm in Apply→FieldAccess. INSERT_OR_UPDATE_METHOD has empty explicit_type_args and no STypeVar in t_dom — direct `MethodCall::new` works without `specialize_for`.
- type_infer.rs: `valueLengthOpt → Some(SOption(SInt))` in FieldAccess returns; `insertOrUpdate → Some(SOption(SAvlTree))` in Apply→FieldAccess returns.
- Conformance tests at [`tests/conformance/methods/savltree.rs`](../../conformance/methods/savltree.rs) — 5 tests via `compile_ok` (insertOrUpdate is V6; properties are V0 but use V6 `R4[AvlTree].get` in tests for ergonomics).

**Wired count: 16/16 (100%) — closed S67.**

### §2c. SBoxMethods (8 named methods + R0–R9 register accessors — 8+10 wired, 0 missing) ✅

| Method | MinVersion | Signature | IR symbol | Status | lower.rs |
|---|---|---|---|---|---|
| `value` | V0 | `→ Long` | `ExtractAmount` | ✅ | 1359 |
| `propositionBytes` | V0 | `→ Coll[Byte]` | `ExtractScriptBytes` | ✅ | 1360 |
| `id` | V0 | `→ Coll[Byte]` | `ExtractId` | ✅ | 1361 |
| `creationInfo` | V0 | `→ (Int, Coll[Byte])` | `ExtractCreationInfo` | ✅ | 1362 |
| `bytes` | V0 | `→ Coll[Byte]` | `ExtractBytes` | ✅ | 1363 |
| `tokens` | V0 | `→ Coll[(Coll[Byte], Long)]` | (custom lowering) | ✅ | 1365 |
| `R0`..`R9` | V0 | `→ Option[T]` (parameterized by reg id + type arg) | `ExtractRegisterAs` | ✅ | 1501 (regex match) |
| `bytesWithoutRef` | V0 | `→ Coll[Byte]` | `ExtractBytesWithNoRef` (op-code 84) | ✅ (S69) | post-`bytes` arm |

**S69 closure note**: `bytesWithoutRef` SMethod confirmed at methods.scala:1312 (`BytesWithoutRefMethod`, method_id=4). The Rust IR struct `ExtractBytesWithNoRef` already existed in [`ergotree-ir/src/mir/extract_bytes_with_no_ref.rs`](../../../../ergotree-ir/src/mir/extract_bytes_with_no_ref.rs); only the compiler dispatch arm and type_infer entry were missing. Pre-S69 type_infer.rs had dead `bytesWithNoRef`/`scriptBytes` aliases (never reachable from any user input) — replaced with the canonical `bytesWithoutRef`. Tests at [`tests/conformance/methods/sbox.rs`](../../conformance/methods/sbox.rs).

**Wired count: 8/8 named + 10/10 registers (100%) — closed S69.**

### §2d. SOptionMethods (5 unique — 5 wired, 0 missing) ✅

| Method | MinVersion | Signature (Option[T] receiver) | IR symbol | Status | lower.rs |
|---|---|---|---|---|---|
| `get` | V0 | `→ T` | (special — `OptionGet`) | ✅ | 1405 |
| `isDefined` | V0 | `→ Bool` | `OptionIsDefined` | ✅ | 1408 |
| `getOrElse` | V0 | `(T) → T` | `OptionGetOrElse` (or shared with Coll) | ✅ | 1186 (shared arm) |
| `map` | V0 | `(T → U) → Option[U]` | `MAP_METHOD` (specialize_for) | ✅ | S67 (split from Coll.map by obj.tpe()) |
| `filter` | V0 | `(T → Bool) → Option[T]` | `FILTER_METHOD` (specialize_for) | ✅ | S67 (split from Coll.filter by obj.tpe()) |

**S67 implementation notes**:
- The whitelist already had `map`/`filter` for Coll. S67 split each existing arm: a new arm gated on `matches!(obj.tpe(), SType::SOption(_))` runs first, dispatches to `MethodCall(obj, OPTION_*_METHOD.specialize_for(obj.tpe(), [arg.tpe()]), [arg])`; falls through to the existing `Filter::new`/`Map::new` collection path otherwise.
- Both MAP_METHOD/FILTER_METHOD have `explicit_type_args: vec![]` and use STypeVar::iv()/STypeVar::ov() in t_dom/t_range — `specialize_for` handles all substitutions, no need for `with_type_args`.
- type_infer.rs: new `Some(SType::SOption(inner))` arm in Apply→FieldAccess returns: `filter → SOption(inner)` (preserves T); `map → SOption(lambda.t_range)`.
- Conformance tests at [`tests/conformance/methods/soption.rs`](../../conformance/methods/soption.rs) — 3 tests via `compile_ok`.

**Wired count: 5/5 (100%) — closed S67.**

### §2e. SGroupElementMethods (5 unique in Scala — 5 wired, 0 missing) ✅

| Method | MinVersion | Signature | IR symbol | Status | lower.rs |
|---|---|---|---|---|---|
| `getEncoded` | V0 | `→ Coll[Byte]` | `GetEncoded` (PropertyCall) | ✅ | 1459 |
| `exp` | V0 | `(BigInt) → GroupElement` | `Exponentiate` | ✅ | 1144 |
| `multiply` | V0 | `(GroupElement) → GroupElement` | `MultiplyGroup` | ✅ | 1158 |
| `negate` | V0 | `→ GroupElement` (property) | `NEGATE_METHOD` (PropertyCall) | ✅ | S67 |
| `expUnsigned` | **V3+** | `(UnsignedBigInt) → GroupElement` | `EXPONENTIATE_UNSIGNED_METHOD` (method_id=6, MethodCall) | ✅ (S69) | post-`multiply` arm |

**S67 corrections vs prior matrix**: the matrix claimed `isIdentity` was a missing 5th method. **`isIdentity` does NOT exist in Scala methods.scala or in Rust ergotree-ir.** The actual 5th method is `expUnsigned` (V3+, niche). Only 4 base methods exist in V0; `expUnsigned` is added in V3+.

**S67 implementation notes**:
- `negate` is a PropertyCall (no args) gated on `Some(SType::SGroupElement)` in the FieldAccess match.
- type_infer.rs SGroupElement arm extended with `negate → Some(SGroupElement)`.
- Conformance tests at [`tests/conformance/methods/sgroup_elem.rs`](../../conformance/methods/sgroup_elem.rs) — 2 tests (raw IR + V0 pipeline).

**S69 implementation notes**:
- Method-name dispatch arm in lower.rs (gated on `Some(SType::SGroupElement)`) constructs `MethodCall::new(obj, EXPONENTIATE_UNSIGNED_METHOD.clone(), args)`. The SMethod has `explicit_type_args: vec![]` and no STypeVar in `t_dom` → `MethodCall::new` is sufficient (no `specialize_for`/`with_type_args` needed).
- Whitelist gate (lower.rs:~1071) extended with `"expUnsigned"`.
- type_infer.rs SGroupElement arm: `"exp" | "expUnsigned" | "multiply" → SGroupElement`.
- Note: Rust IR's `EXPONENTIATE_UNSIGNED_METHOD_DESC.name = "exponentiate"` (not `"expUnsigned"`); name only matters for SMethod-by-name lookup paths the compiler doesn't take. Method dispatch by user name → SMethod selection happens in the lower.rs arm directly. Pre-existing IR-side naming inconsistency, low impact.
- Test: `method_exp_unsigned_v6` at [`tests/conformance/methods/sgroup_elem.rs`](../../conformance/methods/sgroup_elem.rs).

**Wired count: 5/5 (100%) — closed S69.**

### §2f. SHeaderMethods (16 unique — 16 wired, 0 missing) ✅

| Method | MinVersion | Signature | IR symbol | Status | lower.rs |
|---|---|---|---|---|---|
| `id` | V0 | `→ Coll[Byte]` | `ID_PROPERTY` | ✅ | 1700 (S65) |
| `version` | V0 | `→ Byte` | `VERSION_PROPERTY` | ✅ | 1706 (S65) |
| `parentId` | V0 | `→ Coll[Byte]` | `PARENT_ID_PROPERTY` | ✅ | 1712 (S65) |
| `ADProofsRoot` | V0 | `→ Coll[Byte]` | `AD_PROOFS_ROOT_PROPERTY` | ✅ | 1718 (S65) |
| `stateRoot` | V0 | `→ AvlTree` | `STATE_ROOT_PROPERTY` | ✅ | 1724 (S65) |
| `transactionsRoot` | V0 | `→ Coll[Byte]` | `TRANSACTIONS_ROOT_PROPERTY` | ✅ | 1730 (S65) |
| `timestamp` | V0 | `→ Long` | `TIMESTAMP_PROPERTY` | ✅ | 1736 (S65) |
| `nBits` | V0 | `→ Long` | `N_BITS_PROPERTY` | ✅ | 1742 (S65) |
| `height` | V0 | `→ Int` | `HEIGHT_PROPERTY` | ✅ | 1748 (S65) |
| `extensionRoot` | V0 | `→ Coll[Byte]` | `EXTENSION_ROOT_PROPERTY` | ✅ | 1754 (S65) |
| `minerPk` | V0 | `→ GroupElement` | `MINER_PK_PROPERTY` | ✅ | 1760 (S65) |
| `powOnetimePk` | V0 | `→ GroupElement` | `POW_ONETIME_PK_PROPERTY` | ✅ | 1766 (S65) |
| `powNonce` | V0 | `→ Coll[Byte]` | `POW_NONCE_PROPERTY` | ✅ | 1772 (S65) |
| `powDistance` | V0 | `→ BigInt` | `POW_DISTANCE_PROPERTY` | ✅ | 1778 (S65) |
| `votes` | V0 | `→ Coll[Byte]` | `VOTES_PROPERTY` | ✅ | 1784 (S65) |
| `checkPow` | **V3** | `→ Boolean` | `CHECK_POW_METHOD` | ✅ | 1790 (S65) |

**S65 implementation notes**:
- All 16 are `PropertyCall::new(obj, METHOD.clone())` — no `STypeVar`, no `specialize_for` needed.
- Every arm is gated on `matches!(fa.object.tpe, Some(SType::SHeader))` to disambiguate from same-named SBox arms (`id`) and SPreHeader arms (`version`/`parentId`/`timestamp`/`nBits`/`height`/`minerPk`/`votes`).
- `id` previously dispatched unconditionally to `ExtractId` (SBox extractor); arm at lower.rs:1542 is now gated on `Some(SType::SBox)` so the SHeader.id arm can fire.
- `checkPow` is V3+ (test must use `compile_ok`); structurally a property since `t_dom == [SHeader]`.
- `Header` values reachable in user code via `CONTEXT.headers(idx)`. `HEADERS_PROPERTY` arm wired in lower.rs (lines 1660–1665, S65) + type_infer.rs SContext arm — natural extension since SHeader tests needed it.
- Conformance tests at [`tests/conformance/methods/sheader.rs`](../../conformance/methods/sheader.rs) — 16 tests (15 V0 via both `compile_ok` and `compile_tree`, 1 V6 via `compile_ok`).

**Wired count: 16/16 (100%) — closed S65.**

### §2g. SPreHeaderMethods (7 unique — 7 wired, 0 missing) ✅

| Method | MinVersion | Signature | IR symbol | Status | lower.rs |
|---|---|---|---|---|---|
| `version` | V0 | `→ Byte` | `VERSION_PROPERTY` | ✅ | 1797 (S65) |
| `parentId` | V0 | `→ Coll[Byte]` | `PARENT_ID_PROPERTY` | ✅ | 1803 (S65) |
| `timestamp` | V0 | `→ Long` | `TIMESTAMP_PROPERTY` | ✅ | 1809 (S65, formerly unguarded) |
| `nBits` | V0 | `→ Long` | `N_BITS_PROPERTY` | ✅ | 1815 (S65) |
| `height` | V0 | `→ Int` | `HEIGHT_PROPERTY` | ✅ | 1821 (existing, gated) |
| `minerPk` | V0 | `→ GroupElement` | `MINER_PK_PROPERTY` | ✅ | 1827 (S65, formerly unguarded) |
| `votes` | V0 | `→ Coll[Byte]` | `VOTES_PROPERTY` | ✅ | 1833 (S65) |

**S65 implementation notes**:
- Pre-S65 the `timestamp`/`minerPk` arms in lower.rs were unguarded — would mis-dispatch SHeader receivers to SPreHeader IR. S65 added `if matches!(fa.object.tpe, Some(SType::SPreHeader))` guards and added 4 new arms (`version`/`parentId`/`nBits`/`votes`).
- type_infer.rs SPreHeader entry extended: was `version`/`timestamp`/`height`/`minerPk`; now also `parentId`/`nBits`/`votes`.
- `PreHeader` values reachable via `CONTEXT.preHeader` (PRE_HEADER_PROPERTY arm at lower.rs:1654, pre-existing).
- Conformance tests at [`tests/conformance/methods/spreheader.rs`](../../conformance/methods/spreheader.rs) — 7 tests, all V0 via both `compile_ok` and `compile_tree`.

**Wired count: 7/7 (100%) — closed S65.**

### §2h. SContextMethods (12 — fully wired)

Wired in lower.rs (FieldAccess on `CONTEXT` or via globals):
| Method | MinVersion | IR symbol | Status | Source |
|---|---|---|---|---|
| `dataInputs` | V0 | `DATA_INPUTS_PROPERTY` | ✅ | lower.rs:1389 |
| `selfBoxIndex` | V0 | `SELF_BOX_INDEX_PROPERTY` | ✅ | lower.rs:1395 |
| `HEIGHT` (via CONTEXT) | V0 | `Height` GlobalVars | ✅ | lower.rs:1401 |
| `preHeader` | V0 | `PRE_HEADER_PROPERTY` | ✅ | lower.rs:1453 |
| `headers` | V0 | `HEADERS_PROPERTY` | ✅ | lower.rs (S65) |
| `LastBlockUtxoRootHash` | V0 | `LAST_BLOCK_UTXO_ROOT_HASH_PROPERTY` (PropertyCall) | ✅ | S67 |
| `minerPubKey` (CONTEXT.) | V0 | `GlobalVars::MinerPubKey` (same node as bare) | ✅ | S67 |
| `getVar` | V0 | predef `getVar` | ✅ | predef arm |
| `getVarFromInput` | V3+ | predef `getVarFromInput` | ✅ | predef arm |

Routed through bare global names in [`binder.rs`](../../../src/binder.rs):
| Name | GlobalVars variant | Status |
|---|---|---|
| `HEIGHT` | `Height` | ✅ binder.rs:91 |
| `SELF` | `SelfBox` | ✅ binder.rs:92 |
| `INPUTS` | `Inputs` | ✅ binder.rs:93 |
| `OUTPUTS` | `Outputs` | ✅ binder.rs:94 |
| `groupGenerator` | `GroupGenerator` | ✅ binder.rs:95 |
| `CONTEXT` | (Context expression) | ✅ binder.rs:96 |
| `minerPubKey` | `MinerPubKey` | ✅ binder.rs (S67 — new variant in `hir::GlobalVars`) |

**S67 implementation notes**:
- `LastBlockUtxoRootHash` is a regular PropertyCall (no GlobalVars variant exists in Rust IR for it — only its OpCode `0x36` and the SMethod constant).
- `minerPubKey` has both forms: bare `minerPubKey` → binder routes to `GlobalVars::MinerPubKey`; `CONTEXT.minerPubKey` → same `GlobalVars::MinerPubKey` IR node (matching the existing `CONTEXT.HEIGHT` pattern).
- New `MinerPubKey` variant added to `hir::GlobalVars` enum (was missing), forwarded to `ergotree_ir::mir::global_vars::GlobalVars::MinerPubKey` in `lower.rs`.
- Conformance tests at [`tests/conformance/methods/scontext.rs`](../../conformance/methods/scontext.rs) — 5 tests including V0 pipeline assertions.

**Wired count: 12/12 (100%) — closed S67.**

### §2i. SGlobalMethods (10 — fully wired) ✅

| Method | MinVersion | Signature | Status | Source |
|---|---|---|---|---|
| `groupGenerator` | V0 | `→ GroupElement` | ✅ | binder.rs (bare) |
| `xor` | V0 | `(Coll[Byte], Coll[Byte]) → Coll[Byte]` | ✅ | lower.rs (bare predef) |
| `serialize` | V3+ | `[T](T) → Coll[Byte]` | ✅ | lower.rs (S62, specialize_for) |
| `deserializeTo` | V3+ | `[T](Coll[Byte]) → T` | ✅ | lower.rs (S62, with_type_args) |
| `fromBigEndianBytes` | V3+ | `[T](Coll[Byte]) → T` | ✅ | lower.rs (S62, with_type_args) |
| `encodeNbits` | V3+ | `(BigInt) → Long` | ✅ | lower.rs (S62) |
| `decodeNbits` | V3+ | `(Long) → BigInt` | ✅ | lower.rs (S62) |
| `powHit` | V3+ | `(Int, Coll[Byte], Coll[Byte], Coll[Byte], Int) → UnsignedBigInt` | ✅ | lower.rs (S62) |
| `some` | V3+ | `[T](T) → Option[T]` | ✅ | S67 (specialize_for) |
| `none` | V3+ | `[T]() → Option[T]` | ✅ | S67 (with_type_args — explicit_type_args = `[STypeVar::t()]`) |

**S67 implementation notes**:
- `some` and `none` exposed as bare-name predefs in the Apply branch of `lower.rs` (not via `Global.some(...)` since there's no `Global` binder symbol — same pattern as `serialize`/`deserializeTo`).
- `some(value)`: SOME_METHOD has STypeVar::t() in t_dom but `explicit_type_args: vec![]`. `specialize_for(SGlobal, [arg.tpe()])` substitutes T from the value; then `MethodCall::new(Expr::Global, specialized, [value])`.
- `none[T]()`: NONE_METHOD has `explicit_type_args: vec![STypeVar::t()]`. Use `MethodCall::with_type_args(Expr::Global, NONE_METHOD, [], {t() => T})`. The 4 CSE plumbing sites (per S62 carry-forward) preserve `explicit_type_args` correctly.
- type_infer.rs: `some` reads first arg's type and wraps in SOption; `none` reads `apply.type_arg` and wraps in SOption.
- Conformance tests at [`tests/conformance/methods/sglobal.rs`](../../conformance/methods/sglobal.rs) — 4 tests via `compile_ok` (all V3+).

**Wired count: 10/10 (100%) — closed S67.**

### §2j. SNumericTypeMethods (shared across SByte/SShort/SInt/SLong/SBigInt/SUnsignedBigInt — 13 base + 8 modular for V6)

| Method | MinVersion | Signature | Status | lower.rs |
|---|---|---|---|---|
| `toByte` | V0 | `T → Byte` | ✅ | 1439 |
| `toShort` | V0 | `T → Short` | ✅ | 1442 |
| `toInt` | V0 | `T → Int` | ✅ | 1427 |
| `toLong` | V0 | `T → Long` | ✅ | 1414 |
| `toBigInt` | V0 | `T → BigInt` | ✅ | 1417 |
| `toBytes` | V6 | `T → Coll[Byte]` | ✅ | ~1700 (S66) |
| `toBits` | V6 | `T → Coll[Bool]` | ✅ | ~1700 (S66) |
| `bitwiseInverse` | V6 | `T → T` | ✅ | ~1700 (S66) |
| `bitwiseOr/And/Xor` | V6 | `(T, T) → T` | ✅ | ~1465 (S66, 3 methods) |
| `shiftLeft/Right` | V6 | `(T, Int) → T` | ✅ | ~1465 (S66, 2 methods) |
| `toUnsigned` (BigInt) | V6 | `BigInt → UnsignedBigInt` | ✅ | ~1716 (S66) |
| `toUnsignedMod` (BigInt) | V6 | `(BigInt, UnsignedBigInt) → UnsignedBigInt` | ✅ | ~1490 (S66) |
| `modInverse` (UnsignedBigInt) | V6 | `(U, U) → U` | ✅ | ~1505 (S66) |
| `plusMod` (UnsignedBigInt) | V6 | `(U, U, U) → U` | ✅ | ~1505 (S66) |
| `subtractMod` (UnsignedBigInt) | V6 | `(U, U, U) → U` | ✅ | ~1505 (S66) |
| `multiplyMod` (UnsignedBigInt) | V6 | `(U, U, U) → U` | ✅ | ~1505 (S66) |
| `mod` (UnsignedBigInt) | V6 | `(U, U) → U` | ✅ | ~1505 (S66) |
| `toSigned` (UnsignedBigInt) | V6 | `UnsignedBigInt → BigInt` | ✅ | ~1728 (S66) |

**S66 implementation notes** (Workstream B slice 3):
- All V6 methods routed through new `lookup_numeric_method(receiver, name)` helper at lower.rs:150-181, which indexes the per-type METHODS Vec from `ergotree_ir::types::snumeric::{sbyte,sshort,sint,slong,sbigint,sunsignedbigint}::METHODS`. The Vecs are pre-specialized at lazy_static time — no `specialize_for` needed at lowering.
- Property-style (no args) arms wrap `PropertyCall::new`; argful arms wrap `MethodCall::new`. Both work directly because the per-type METHODS already have the type substituted (no STypeVar leftover).
- Whitelist gate at lower.rs:1027–1041 (the Apply→FieldAccess method-vs-indexing dispatch) extended with all 11 argful method names: `bitwiseOr/And/Xor/shiftLeft/Right`, `toUnsignedMod`, `modInverse/plusMod/subtractMod/multiplyMod/mod`. Without this, the apply branch falls through to "indexing" and rejects the call as "Unknown field on numeric type".
- type_infer.rs: each numeric SType arm (SByte/SShort/SInt/SLong/SBigInt/SUnsignedBigInt) extended with no-args V6 returns; new SUnsignedBigInt arm added (was none prior to S66); new Apply→FieldAccess match arm for numeric receivers covers argful return types.
- hir.rs `parse_type_name` extended with `"UnsignedBigInt" => Some(SType::SUnsignedBigInt)` so user code like `val u: UnsignedBigInt = ...` parses.
- `as_method` on SMethodDesc is `pub(crate)`, so direct construction from `*_METHOD_DESC` is not callable from ergoscript-compiler. The `lookup_numeric_method` indirection sidesteps the visibility gap.
- Conformance tests at [`tests/conformance/methods/snumeric.rs`](../../conformance/methods/snumeric.rs) — 25 representative tests across 6 numeric types and all 13 V6 method shapes. All use `compile_ok` (V3+ ops would fail V0 ErgoTree serialization).

**Wired count: 19/19 (100%) — closed S66.** (V0 casts 5 + V6 base 8 + BigInt extras 2 + UnsignedBigInt extras 6 — but `bitwiseOr/And/Xor` count as 3, `shiftLeft/Right` as 2 in the row tally above for completeness.)

### §2k. STupleMethods, SSigmaPropMethods, SBooleanMethods, SStringMethods, SAnyMethods, SUnitMethods

| Registry | Status |
|---|---|
| `STupleMethods` | `_1`/`_2`/etc. handled at parser level (◽ Auto). `size` → `SizeOf`. ✅ |
| `SSigmaPropMethods` | `propBytes` ✅ (lower.rs:1411). `isProven` ✅ (S70) — IR fill landed: new `SigmaPropIsProven` MIR struct in [`ergotree-ir/src/mir/sigma_prop_is_proven.rs`](../../../../../ergotree-ir/src/mir/sigma_prop_is_proven.rs) using `OneArgOp` (op-code 95), `Print` impl, source_span entry, serialize/parse arms in [`serialization/expr.rs`](../../../../../ergotree-ir/src/serialization/expr.rs), interpreter eval returns `EvalError::Misc` (mirrors Scala's `costKind = notSupportedError` — graph-IR rewrite removes the node before evaluation). Compiler arm in [`mir/lower.rs`](../../../src/mir/lower.rs) at the SSigmaProp method branch; tests `methods::ssigma_prop::property_is_proven`/`_compiles_v0`. |
| `SBooleanMethods` | Inherits from MonoTypeMethods; no instance methods. ✅ |
| `SStringMethods` | Inherits; strings are compile-time literals only in user surface. ✅ |
| `SAnyMethods`, `SUnitMethods` | Inherit; nothing to wire. ✅ |

---

## §3. Verified totals (S63 audit)

| Layer | Total | Wired | % |
|---|---|---|---|
| **Predefs** (SigmaPredef.PredefinedFunc) | 44 | 38 | 86% (S69: +allZK/+anyZK/+avlTree) |
| **Predef bonuses** (Global SMethods, user-facing) | 3 | 3 | 100% |
| **SCollectionMethods** | 21 | 21 | 100% (S64) |
| **SAvlTreeMethods** | 16 | 16 | 100% (S67) |
| **SBoxMethods** (named + registers) | 8+10 reg | 8+10 | 100% (S69) |
| **SOptionMethods** | 5 | 5 | 100% (S67) |
| **SGroupElementMethods** | 5 | 5 | 100% (S69) |
| **SHeaderMethods** | 16 | 16 | 100% (S65) |
| **SPreHeaderMethods** | 7 | 7 | 100% (S65) |
| **SContextMethods** | 12 | 12 | 100% (S67) |
| **SGlobalMethods** | 10 | 10 | 100% (S67) |
| **SNumericTypeMethods** | 19 (V0+V6) | 19 | 100% (S66) |
| **STupleMethods/SSigmaPropMethods/etc.** | 6 | 6 | ~100% |

**Estimated unique-method total**: ~120 (not 163 — the 163 figure includes per-numeric-type instantiations and overloads).

---

## §4. Audit corrections vs WORKSTREAM-STATUS.md §B.1

| Cell | §B.1 estimate | S63 finding | Correction |
|---|---|---|---|
| SCollectionMethods missing | 14 (incl. distinct/sortBy/take/drop/head/tail/last/init/min/max/sum/product/find/count/groupBy/mkString/unzip/flatten) | **10** (real Scala SMethods: indices/zip/patch/updated/updateMany/indexOf + V6: reverse/startsWith/endsWith/get) | 14 of those names DON'T exist as Scala SMethods; remove from gap list |
| SCollectionMethods wired | 10 (~40%) | **11/21 (52%)** | Includes `flatMap` which §B.1 missed |
| SAvlTreeMethods wired | ~7 (~30%) | **11/16 (69%)** | §B.1 underestimated; `digest`/`enabledOperations`/`keyLength`/`get` are wired |
| SBoxMethods wired | ~100% | **94%** | `bytesWithoutRef` missing (if it's a real SMethod) |
| SOptionMethods wired | ~100% | **60%** | `map`/`filter` not wired |
| SGroupElementMethods wired | ~100% | **60%** | `negate`/`isIdentity` not wired |
| SContextMethods wired | ~15% | **~58%** | §B.1 didn't credit binder global keywords (HEIGHT/SELF/INPUTS/OUTPUTS/groupGenerator/CONTEXT) |
| SNumericTypeMethods | ~100% | **26%** | §B.1 only counted V0 casts; V6 modular + bitwise + toBytes/toBits all missing |
| Total unique methods | 163 | **~120** | 163 includes per-type overloads |

---

## §5. Implementation slice priority for Workstream B

Rank by leverage (size of gap × user-facing frequency):

| Slice | Methods | Effort |
|---|---|---|
| ~~B.1 SCollectionMethods~~ ✅ closed S64 | ~~10 missing — `indices`/`zip`/`patch`/`updated`/`updateMany`/`indexOf` (V0); `reverse`/`startsWith`/`endsWith`/`get` (V6)~~ | ~~1 session~~ |
| ~~B.2 SHeaderMethods~~ ✅ closed S65 | ~~16 wired (V0 ×15 + V6 `checkPow`); SContext.headers wired as bonus~~ | ~~0.5 session~~ |
| ~~B.6 SPreHeaderMethods~~ ✅ closed S65 | ~~7 wired (4 new + guards on existing 3 unguarded `timestamp`/`minerPk` arms)~~ | ~~0.25 session — coupled with B.2~~ |
| ~~B.3 SNumericTypeMethods (V6)~~ ✅ closed S66 | ~~14 wired — `toBytes`/`toBits`/`bitwiseInverse`/`bitwiseOr`/`bitwiseAnd`/`bitwiseXor`/`shiftLeft`/`shiftRight` + 8 modular (`toUnsigned`/`toUnsignedMod` on BigInt; `modInverse`/`plusMod`/`subtractMod`/`multiplyMod`/`mod`/`toSigned` on UnsignedBigInt)~~ | ~~1 session~~ |
| ~~B.4 SAvlTreeMethods gaps~~ ✅ closed S67 | ~~4 V0 properties (`valueLengthOpt`/`isInsertAllowed`/`isUpdateAllowed`/`isRemoveAllowed`) + V6 `insertOrUpdate`~~ | ~~0.25 session~~ |
| ~~B.5 SOptionMethods + SGroupElementMethods~~ ✅ closed S67 | ~~SOption: `map`/`filter` (split from collection arms by obj.tpe()); SGroupElement: `negate`. `isIdentity` does not exist; `expUnsigned` V3+ deferred.~~ | ~~0.25 session~~ |
| ~~B.7 SContextMethods edges~~ ✅ closed S67 | ~~`LastBlockUtxoRootHash` (PropertyCall) + `minerPubKey` (new GlobalVars hir variant; bare + CONTEXT.)~~ | ~~0.25 session~~ |
| ~~B.8 SGlobalMethods.some/none~~ ✅ closed S67 | ~~`some(value)` (specialize_for) + `none[T]()` (with_type_args)~~ | ~~0.25 session~~ |
| ~~B.X residuals (S69)~~ ✅ | ~~`SBox.bytesWithoutRef` (V0 property — wired; existing IR struct), `SGroupElement.expUnsigned` (V3+ method — wired). `SSigmaProp.isProven` re-classified as **IR-blocked** (op-code 95 reserved-but-orphan; needs `SigmaPropIsProven` MIR struct).~~ | ~~each <0.25 session except isProven~~ |
| ~~**B.IR-blocked** (S70)~~ ✅ | ~~`SSigmaProp.isProven` — IR fill landed: `SigmaPropIsProven` MIR struct + serialize/parse + eval. Compiler dispatch arm + 2 conformance tests.~~ | ~~~0.5 session~~ |

**Total effort**: Workstream B is feature-complete. All 19 type-method registries fully wired (where the underlying ergotree-ir IR exists).

---

## §6. Carry-forward caveats from S62 (still load-bearing for Workstream B)

- **`MethodCall::with_type_args` plumbing** (4 sites: `lower.rs:propagate_inner`, `cse.rs:{map_children, replace_all, rewrite_ids}`) required for any new SMethod with non-empty `explicit_type_args: vec![STypeVar::t()]`. `MethodCall::new` silently drops them; CSE later panics.
- **`SMethod::specialize_for(obj_tpe, args)`** for SMethods with `STypeVar` in `t_dom` but `explicit_type_args: vec![]` (e.g. `SERIALIZE_METHOD`).
- **`hashbrown::HashMap` vs `std::HashMap`** — `MethodCall::with_type_args` takes `hashbrown::HashMap<STypeVar, SType>`. Already in `Cargo.toml`.
- **Argument coercion**: SMethods with narrower types than user input (e.g. `getVarFromInput` takes `SShort, SByte` but users write Int literals) need `Downcast::new(arg, target)` wrappers.
- **EKB-name vs IR-name skew**: user-facing `deserializeTo` → IR `DESERIALIZE_METHOD`; user-facing `encodeNbits` → IR `ENCODE_NBITS_METHOD`. Match user-facing names per EKB built-ins ref.
- **`compile_expr` vs `compile()` for V3+ methods in tests**: V3+ method calls need `compile_expr` (raw IR); `compile()` emits V0 ErgoTree which rejects V3+ ops at serialization.

---

*Last updated: S70 / 2026-04-28. S64 closed SCollectionMethods 21/21; S65 closed SHeaderMethods 16/16 + SPreHeaderMethods 7/7 + bonus SContext.headers; S66 closed SNumericTypeMethods 19/19; S67 closed SAvlTreeMethods 16/16 + SOptionMethods 5/5 + SGroupElementMethods 4/5 (negate) + SContextMethods 12/12 + SGlobalMethods 10/10 (some/none V3+). S69 closed predef tail (allZK/anyZK + avlTree) and B residuals (SBox.bytesWithoutRef + SGroupElement.expUnsigned). **S70 closed `SSigmaProp.isProven` (new `SigmaPropIsProven` MIR struct in ergotree-ir + compiler arm) and the Workstream A v6 hard items (`executeFromVar`/`executeFromSelfReg`/`executeFromSelfRegWithDefault` via existing `DeserializeContext`/`Register` IR; `deserialize` compile-time predef; `placeholder` via existing `ConstantPlaceholder` IR).** Workstream A: 43/44 (98%); only `ZKProof { ... }` block-scope remains (own session).*
