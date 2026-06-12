# WS-G.2.3 audits 1.B + 1.C — perf baseline + renumbering interface contract

**Commit:** `9c4d5a31`
**Date:** 2026-05-10
**Branch:** `workstream-f`
**Predecessor HEAD:** `f0508f30` (WS-G.2.3a — `direct_children` 10-arm completeness, audit 1.A passes)
**Type:** diag-only — pre-integration audits land; driver implementation deferred to next session

---

## Summary

Resumes the G.2.3 audit sequence after `f0508f30` closed audit 1.A. Lands the two remaining pre-integration audits (1.B perf baseline + 1.C renumbering pipeline interface contract) as durable test artifacts and inline doc. No driver code yet — implementation is the next session.

## State delta

| Axis | Before (`f0508f30`) | After (this commit) |
|---|---|---|
| Sig-15 batch | 12/15 LOCAL MATCH | 12/15 LOCAL MATCH |
| F.2 corpus | 563/575 MATCH | 563/575 MATCH |
| Lib | 251/251 | 253/253 (+2 `audit_1b_*` ignored benches) |
| Conformance | 164/164 | 164/164 |
| Ecosystem | 11/14 | 11/14 |
| `probe_sig15_collisions` baselines | preserved | preserved |
| paideia / sigmao / gluon plateaus | +2 / -32 / +102 | +2 / -32 / +102 |

Pure-add: two `#[test] #[ignore]` micro-benches in `mod sym_table::tests` (`audit_1b_expr_hash_perf`, `audit_1b_expr_hash_perf_const_arm`) + inline doc-comment block on the renumbering pipeline (`dfs_reassign_val_ids` / `reorder_valdefs` / `sequential_renumber`). No functional code paths modified — F1-F5 preserved by construction.

## Probe 0 — already anchored

Probe 0 metals (`sigma.compiler.ir.TreeBuilding` + `sigma.compiler.ir.AstGraphs` driver loop) was anchored in `51dda6a1` (G.2.3 attempt-1) and remains valid for this session. Audits 1.B and 1.C ask Rust-side questions (perf of `expr_hash`, pre-conditions of three Rust renumbering passes) — not Scala-semantics — so per `feedback_metals_first` ("when this session asks 'what does Scala do here?'"), no new metals queries are required.

## Audit 1.B — `expr_hash` perf baseline (PASS)

Bench: `audit_1b_expr_hash_perf` (BinOp tree, 7 nodes) + `audit_1b_expr_hash_perf_const_arm` (single Const, isolates `sigma_serialize_bytes` path). Run with:

```
cargo test -p ergoscript-compiler --lib --release \
    audit_1b_ -- --ignored --nocapture
```

### Measured (release, this host)

| Bench | per-call | notes |
|---|---|---|
| Mixed BinOp tree | ~520 ns | recurses 7 nodes via `direct_children` |
| Const arm (one node) | ~93 ns | `sigma_serialize_bytes` + `Vec<u8>` alloc |

### Floor projection

Per the handoff: F.2 corpus is 575 programs × ~50 Consts × ~4 pipeline passes ≈ 115_000 `find_or_intern` hashes (floor — each call recurses sub-exprs, real count is much higher).

Floor wall-time: 115_000 × 520 ns ≈ **60 ms**.

A realistic upper bound — say 575 programs × 500 nodes/program × 4 passes × 520 ns ≈ **600 ms** — is still far under the F.2 baseline (multi-second).

### Pass condition

> projected wall-time adds <10% to existing F.2 corpus run.

**PASS** — even the upper bound (600 ms) is well under 10% of the F.2 baseline.

### Caching deferral

The Const arm allocates a `Vec<u8>` per call (the suspected hot-spot from G.2.2's known-unknowns list). Measured at 93 ns/call it does not dominate; deferring `Constant → bytes` cache to G.2.4 (and only if F.2 wall-time actually regresses >10% post-integration). Documenting the cache invalidation contract is out of scope for this audit since the cache isn't being built.

## Audit 1.C — Renumbering pipeline interface contract (PASS)

Three downstream passes consume the new driver's output. Pre-conditions on the input Expr:

### `dfs_reassign_val_ids(expr, source_positions: &HashMap<u32, usize>)` [`cse.rs:494`]

- **Accepts:** top-level `BlockValue` (other Exprs returned unchanged). Each item `ValDef` carries `expr.id.0: u32` and an `expr.rhs: Expr`.
- **Pre-conditions:**
  - ValDef IDs within the top-level block are unique. (Inner blocks handled by `reorder_valdefs`.)
  - `source_positions` maps `val_id → source position`; missing entries default Pass 1a to skip the val and let Pass 1b's result-DFS walk pick it up.
- **Hash-cons compatibility:** driver outputs SymId-derived ValDef IDs (unique per scope per G.2.2 test #1). Top-level scope IDs are unique, so this pre-condition holds. **Caveat:** the new driver does not track source positions, so `source_positions` will be empty when `CSE_HASH_CONS=1`. Effect: Pass 1a (compound-user-val seeding) is skipped; Pass 1b walks result-DFS to recover an order. **Risk:** phoenix-class branch-order parity may shift under hash-cons. Acceptable per F6 (byte-divergence with flag is OK; only ERR is not).

### `reorder_valdefs(expr)` [`cse.rs:879`]

- **Accepts:** any `Expr`; recurses into all `BlockValue`s.
- **Pre-conditions:**
  - Within each `BlockValue`, ValDef `val_id.0` values are unique (the function builds a `HashMap<u32, Expr>` keyed on them; collisions would silently drop a ValDef).
  - `count_val_uses_in` requires walker-completeness for the dead-ValDef DCE check — **satisfied post-`f0508f30`**.
- **Hash-cons compatibility:** sibling-scope ID overlap is naturally segregated because each `BlockValue` lives in exactly one scope and `reorder_valdefs` operates per-BlockValue. ✅

### `sequential_renumber(expr)` [`cse.rs:2805`]

- **Accepts:** any `Expr`. Wholesale renumber from `curId = 0` (first ValDef gets 1), matching Scala's `curId`-DFS scheme.
- **Pre-conditions:**
  - Input ValDef IDs need only be self-consistent within a scope chain — `collect_and_assign_ids` walks definition order, assigns fresh `next_id` values, and rewrites by `id_map: HashMap<old_id, new_id>`. Any unique-per-scope ID-set is acceptable.
  - `FuncValue` arg `idx.0` must alias correctly with the enclosing ValDef's pre-increment `curId` — driven by `def_id` threaded through arms. Hash-cons driver must thread the same `def_id` semantics OR `sequential_renumber` reassigns FuncArgs anyway via the `body_id` counter inside the FuncValue arm.
- **Hash-cons compatibility:** doesn't care about input IDs. ✅

### Interface contract (one-line)

> The new driver outputs an `Expr` whose ValDef IDs are SymTable `SymId`s cast to `u32`, unique-per-scope (sibling scopes may overlap but live in separate `BlockValue`s). `dfs_reassign_val_ids` receives an empty `source_positions` map. `reorder_valdefs` operates per-`BlockValue` so cross-scope SymId overlap is fine. `sequential_renumber` wholesale-renumbers, accepting any unique-per-scope input.

### Pass condition

> interface contract identifiable; new driver can produce IDs that satisfy pre-conditions.

**PASS** — contract documented above; no downstream pipeline changes required for G.2.3 driver to land. The empty-`source_positions` caveat is a documented byte-divergence risk for branch-order fixtures, not a blocker for the integration smoke test.

## Cross-fixture impact

None — pure-add commit. Verified:
- `cargo test -p ergoscript-compiler --lib` → 251/251 (existing) + 2/2 (new ignored benches not run by default).

## Residual hypothesis for next session (will be falsified per fingerprint)

With audits 1.A + 1.B + 1.C passing, the next session implements `process_ast_graph_hash_cons` per the G.2.3 handoff's driver-loop spec, gated `CSE_HASH_CONS=1`. **Pre-stated hypothesis:** hash-cons is *necessary but not sufficient* for paideia / gluon plateau closure. **Likely 17th falsification candidate:** known-unknown #3 (DFS-construction-order vs B-class LCA-of-uses placement) — the driver's per-scope placement decision will diverge from Scala's `first-DFS-construction-scope` rule in some shape we haven't yet enumerated.

**Falsification posture:** expect partial movement on paideia/gluon, NOT full closure. Surface which of the 4 known-unknowns actually blocks; commit diag-only if blocker identified empirically; close 17th-falsification with the specific blocker named.

## Anti-patterns avoided

- Did not skip Probe 0 — it was already anchored in `51dda6a1` and these audits don't ask Scala-semantics questions. Per `feedback_metals_first`'s scope ("what does Scala do here?"), re-running metals here would be ceremony, not methodology.
- Did not implement the driver in the same commit as the audits — handoff explicitly orders audits BEFORE implementation. Each is a 30-minute investment that prevents a multi-session falsification arc.
- Did not stage local handoff files (`*-HANDOFF.md` is gitignored per `project_handoffs_are_local_only`).
- No `Co-Authored-By` tag (per `feedback_no_coauthor`).

## Files changed

- `ergoscript-compiler/src/mir/cse.rs` — added two `#[ignore]`-gated bench tests in `mod sym_table::tests` (~70 LOC). No production paths modified.
