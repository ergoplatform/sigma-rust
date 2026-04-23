# ErgoScript Compiler — Session Handoff (2026-04-22, Session 17)

## Current State: 31/31 native byte-match, 196 tests, 0 errors, 0 canonical fallbacks

## What was done in Session 17: Phoenix HodlERG → LOCAL MATCH

### Five changes to close the 23-byte gap (309B → 314B exact match):

1. **Type propagation pass** (`lower.rs`): `propagate_val_types()` walks MIR after lowering, collects actual ValDef RHS types, updates ValUse types (fixing `val x: Long = BigInt_expr`), re-applies `numeric_upcast_pair` on BinOps. Wired into pipeline between `lower()` and `apply_cse()`.

2. **CSE If-branch ThunkDef scoping** (`cse.rs`): Extended `appears_in_main_scope_inner` to treat If branches as ThunkDef scopes. Added `Upcast(ValUse(_) | Const(_))` to `needs_scope_check`.

3. **Post-CSE single-use val inlining** (`cse.rs`): `inline_single_use_vals()` inlines vals that become single-use after CSE extraction — combines e.g. `ExtractAmount(Self)` + `Upcast(ValUse, BigInt)` into `Upcast(ExtractAmount(Self), BigInt)`.

4. **Inner-block constant dedup** (`cse.rs`): `deduplicate_inner_consts()` extracts duplicate constants as vals within If-branch BlockValues (prevents duplicate ConstantStore entries).

5. **If-branch val ordering** (`cse.rs`): Changed `emit_deps` for If to collect all branch val refs, sort by val ID (matching Scala's symbol-ID-ordered ThunkDef freeVars). Made `reorder_valdefs` recursive via `map_children`.

## What was done in Session 16

### 1. Crystal Pool (#24) → LOCAL MATCH
**Root cause:** `ByIndex(Outputs, 0)` appeared only inside separate `&&` right arms. In Scala's graph IR, each `&&` right arm is a ThunkDef with independent symbol scope. Two `ByIndex(Outputs, 0)` in separate ThunkDefs are separate graph symbols (each usage=1), so Scala doesn't extract them. Our CSE was extracting them because both had the same structural identity → dag_count=2.

**Fix:** Extended the `needs_scope_check` pattern in `process_ast_graph` (cse.rs) to include `ByIndex` with `GlobalVars` input (not just `ValUse`/`PropertyCall`). Now `ByIndex(Outputs, 0)` used only in ThunkDef scopes is skipped, matching Scala's behavior.

**Key insight:** Crystal Pool does NOT take the `has_lambdas` path (despite having FuncValue inside Apply). The `contains_func_value` function has `_ => false` for Apply, so FuncValues inside Apply nodes are invisible. Crystal Pool takes the `process_ast_graph` (no-lambdas) path.

### 2. BigInt auto-upcast in MIR lowering
**What:** Added `numeric_upcast_pair()` in `lower.rs`. When a `BinOp` mixes BigInt with a smaller numeric type (Byte/Short/Int/Long), the narrower operand is auto-upcast to BigInt via `Upcast`. This matches the Scala ErgoScript compiler's implicit numeric promotion.

**Also:** Relaxed the MIR type check to accept BigInt results assigned to Long-annotated vals (the Scala REST API accepts this too, even though the strict Scala type checker rejects it).

**Impact:** Phoenix HodlERG output changed from 291B to 309B (closer to target 314B).

### Phoenix HodlERG (#25) — not fixed (309B vs 314B)

**Remaining issues (5 bytes):**

1. **Duplicate `feeDenom` constant (10 vs 9 constants).** `feeDenom = 1000L` is inlined by the HIR optimizer, creating two separate `Const(1000L)` nodes in the tree. The serializer's `ConstantStore::put` creates separate ConstPlaceholders for each. In Scala's graph IR, identical constants are hash-consed (one Sym), so `put` is called once. We can't dedup in ConstantStore because other contracts (vault, etc.) intentionally have duplicate constants in the Scala output.

2. **Type propagation through val chains.** The Scala compiler computes expression types from the graph IR, so `val expectedAmountBeforeFees: Long = BigInt_expr` is typed as BigInt in the graph. Our compiler uses the user's type annotation (Long), so downstream operations like `expectedAmountBeforeFees * bankFeeNum` are `Long * Long` (no auto-upcast) instead of `BigInt * Long` (with auto-upcast). This causes different Upcast insertion patterns.

3. **ValDef ordering.** `validBankRecreation` appears at val 10 in our output but val 13 in the node, because the Upcast vals (10, 11 in the node) come first.

**Fix direction for Phoenix:**
- **Short-term:** After BigInt auto-upcast, propagate the actual computed type back to the val's type. When `val x: Long = BigInt_expr`, update x's type to BigInt so downstream uses get correct auto-upcast. This requires modifying `compile_from_hir` to do a type-fixup pass.
- **Long-term:** Constant dedup needs graph-level sharing, not ConstantStore-level dedup. Consider a post-CSE pass that assigns a canonical index to each unique constant value and replaces `Const` nodes with `ConstPlaceholder` references before serialization.

## Files modified this session

- `ergoscript-compiler/src/mir/cse.rs` — ThunkDef scope check extended for ByIndex(GlobalVars); cleaned up duplicate comments
- `ergoscript-compiler/src/mir/lower.rs` — `numeric_upcast_pair()` for BigInt auto-upcast; `numeric_rank()` helper; relaxed type check for BigInt→Long assignment
- `ergoscript-compiler/src/compiler.rs` — Cleaned up debug code
- `ERGOSCRIPT-COMPILER-STATUS.md` — Updated to 31/31
- `SKILL-how-to.md` — Added Session 16 notes

## Remaining contracts: NONE

All 31 contracts produce bytecode identical to the Scala reference node.

## Files modified in Session 17

- `ergoscript-compiler/src/mir/lower.rs` — `propagate_val_types()`, `propagate_inner()` (type propagation pass); added `HashMap` and `Spanned` imports
- `ergoscript-compiler/src/mir/cse.rs` — `inline_single_use_vals()`, `count_val_uses_in()`, `deduplicate_inner_consts()`, `dedup_consts_in_block()`, `collect_consts()`, `collect_all_val_uses()` (new functions); extended `needs_scope_check` for `Upcast(Const(_))`; If-branch ThunkDef scoping in `appears_in_main_scope_inner`; If-branch freeVars sorting in `emit_deps`; recursive `reorder_valdefs`
- `ergoscript-compiler/src/compiler.rs` — Added `propagate_val_types` call in pipeline; cleaned up debug code
- `ERGOSCRIPT-COMPILER-STATUS.md` — Updated to 31/31 with 0 canonical fallbacks
- `SKILL-how-to.md` — Added Session 17 notes
- `HANDOFF-CSE-PARITY.md` — Updated to Session 17

## Critical rule: test after EVERY change

```bash
cargo test -p ergoscript-compiler
# Must show 196+ passed, 0 failed

source ~/.secrets
cargo test -p ergoscript-compiler test_real_world -- --ignored --nocapture
cargo test -p ergoscript-compiler test_canonical -- --ignored --nocapture
```
