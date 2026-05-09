# ErgoScript Parity-Audit Framework — User Guide

**Status:** in use; framework v1 landed 2026-05-08 (concurrent with sig-15 10/15 milestone).
**Audience:** anyone diagnosing or closing `ergoscript-compiler` byte-match parity gaps against the Scala reference compiler.

This guide describes the workflow + artifacts for the methodology that produced 10/15 byte-MATCH across the "15 Significant Ergo Contracts" initiative, ~98% on the WS-F differential fuzz corpus, and the diagnostic infrastructure (`probe_sig15_collisions`, `probe_gluon_scopes`) for the remaining gaps.

---

## 1. Sig-15 status (post-`52ca2c14`, 2026-05-08)

**10/15 LOCAL byte-MATCH** with the Scala node:
- `chaincash_reserve`, `dexy_bank_full`, `ergomixer_fullmix`, `ergoraffle_active`, `phoenix_hodlerg_bank_full`, `rosen_event_trigger`, `sigmausd_bank`, `skyharbor_v1_erg`, `spectrum_n2t_pool`, `spectrum_t2t_pool`

**5 still open** (USED NODE — local compiles, but bytes diverge from Scala reference; canonical mode falls back to node bytes):

| Fixture | Δ | Class | Path forward |
|---|---|---|---|
| `paideia_stake_state` | +2 | Cross-branch coordination plateau (5 sessions, 5 falsifications) | CSE-pass invariant work — shared sym across sibling thunks |
| `gluon_box_guard` | -154 | Post-CSE AST-consistency emit-bug plateau | Add invariant: every `ValUse(id, tpe)` has matching `ValDef(id, _: tpe)` in scope chain |
| `sigmao_option` | -32 | Renumbering pipeline rewrite plateau | `mir/cse.rs` module-level refactor of `dfs_reassign_val_ids → reorder_valdefs → sequential_renumber` |
| `oracle_refresh` | -2 | Suspected silent cascade regression since CLOSE at `3038845e` | Verify at HEAD via `probe_sig15_collisions`; restore if regressed |
| `duckpools_child_interest` | -4 | Suspected silent cascade regression since CLOSE at S67 | Same as oracle |

The three diagnosed plateaus (paideia, gluon, sigmao) all share a structural pattern: post-CSE structural integrity. Each requires module-level CSE-pass invariant work, not per-fixture predicate tweaks. Recommend pairing them as a unified CSE-pass workstream rather than chasing them one by one.

The two suspected regressions (oracle, duckpools) are likely the same class of silent-cascade regression that `52ca2c14` (sigmao S3) caught and restored — bisection from each fixture's CLOSE commit to HEAD localizes the cascade that drifted them.

---

## 2. The methodology in one paragraph

The audit cycle is: pick a fixture with a Δ vs the Scala node; classify the residual into one of 8 known methodology classes via IR-dump comparison; pattern-match against prior closed-cluster archives for similar shapes; **front-load metals MCP queries** against `sigmastate-interpreter` to anchor *what Scala actually does* (Probe 0); propose a structural fix using one of the established gating patterns (strict-subset, two-axis evidence, LCA-aware permissive scope check); empirically probe regression-then-restore (Probe 2) to confirm the fix doesn't mask a compensating bug; validate full sig-15 + F.2 + lib + conformance + ecosystem to protect baselines; commit with a structured commit body that doubles as the cluster-archive seed; update memory + status. **When a single fix regresses, expect a compensating-bugs structure (sign-flip fingerprint).** Some fixtures close in 3 sessions (rosen, oracle); some take 5+ (sigmausd, paideia → plateau). Equal cardinality ≠ multiset parity. **Verify Scala-side ground truth via metals BEFORE structural fixes** — 5 of the 9 known falsification-fingerprint instances skipped this step.

---

## 3. Artifacts inventory

The framework is split across user-global (per-engineer) and project-local (in-repo) artifacts.

### 3a. User-global — `~/.claude/skills/`

Two slash-command skills installed at user scope. To use these on another machine, copy the directories.

| Skill | Path | Purpose |
|---|---|---|
| `/audit-fixture-session` | `~/.claude/skills/audit-fixture-session/SKILL.md` | Run a per-fixture audit session end-to-end. Encodes pre-flight reads, Probe 0 metals (BLOCKING), Probe 1 empirical IR, Probe 2 falsification, commit/archive cadence, anti-patterns. |
| `/audit-residual-classify` | `~/.claude/skills/audit-residual-classify/SKILL.md` | Given fixture + Δ, classify residual into 8 known methodology classes; suggest highest-yield Probe 0 FQCNs and relevant archive references. |

Invoke as `/audit-fixture-session <fixture> [session-N]` or `/audit-residual-classify <fixture> [Δ]`.

### 3b. User-global — `~/.claude/projects/-home-cq-working-files-sigma-rust/memory/`

Persistent methodology rules that survive across conversations. Notable entries:

| Entry | Rule |
|---|---|
| `feedback_metals_first.md` | Per-fixture sessions MUST front-load metals MCP queries when the question is Scala-side semantics. 5/9 falsifications skipped this step. |
| `feedback_falsification_fingerprint.md` | Single-fix regression doesn't prove partner-fix is correct. Probe each candidate in isolation BEFORE declaring coupling. 9 instances logged. |
| `feedback_run_all_tests.md` | CSE tweaks silently break other contracts. Run full suite after every edit. |
| `feedback_no_push_before_test.md` | Never push to remote without explicit user confirmation that changes work locally. |
| `project_collisions_are_legitimate_parser_bug_class.md` | 4 currently-MATCH fixtures have ValId collisions baseline. Don't make CSE emit globally-unique IDs. AST-consistency check is the layer-classifier. |
| `project_sig15_<fixture>_*.md` (per fixture) | Per-session findings, falsifications, and residual-class diagnoses. |

These files document the *why* behind decisions. They are not skills (no slash-command surface).

### 3c. Project-local — `ergoscript-compiler/`

Diagnostic infrastructure committed to the repo:

| Artifact | Location | Purpose |
|---|---|---|
| `debug_<fixture>` helpers | `ergoscript-compiler/src/compiler.rs` (per-fixture) | Dump LOCAL vs Scala-node IR + bytes for one fixture. Mirror the shape when adding new fixtures. |
| `probe_sig15_collisions` | `ergoscript-compiler/src/compiler.rs` | Walks post-CSE Expr; counts ValDef-ID collisions, type-conflicts, segregation OK/FAIL per fixture. ~1s deterministic. Forward-pace metric for parser-layer changes. |
| `probe_gluon_scopes` | `ergoscript-compiler/src/compiler.rs` | AST-consistency check — reports `OptionGet(ValUse(id))` sites with no resolvable `ValDef(id, tpe)` in any ancestor scope. Forward-pace metric for emitter-layer changes. |
| F.1 ecosystem harness | `ergoscript-compiler/tests/diff_fuzz.rs` | 14-contract real-world parity batch. |
| F.2 typed-AST generator | `ergoscript-compiler/tests/diff_fuzz_gen.rs` | Synthetic ES program corpus generator (575 programs). |
| `CSE_TRACE` instrumentation | `ergoscript-compiler/src/mir/cse.rs` | Env-flag-controlled tracing of CSE candidate decisions. |
| Cluster archives | `ergoscript-compiler/target/diff_fuzz/clusters/closed/*.md` | Per-commit historical record of every closed-or-diagnosed cluster. **Gitignored under `target/`** but kept locally — this is the audit trail. |

### 3d. Project-local design specs

| Artifact | Location |
|---|---|
| MCP server design spec | [`docs/sigma-audit-mcp-design.md`](docs/sigma-audit-mcp-design.md) — `sigma-audit-mcp` MVP (3 Rust-side tools); not yet implemented. |

The MCP server, when built, will wrap the existing `debug_<fixture>` + cargo invocations into structured JSON tools (`dump_ir`, `compare_constant_pools`, `run_corpus`). Estimated half-day implementation. Skill invocations work today without it; the MCP is a force-multiplier on top.

### 3e. External dependency — `metals` MCP

The framework depends on the `metals` MCP being wired up against a local checkout of the Scala reference compiler:

```
target repo: github.com/scorexfoundation/sigmastate-interpreter
local path:  /home/cq/sigmastate-interpreter (CannonQ's setup; configurable)
build state: imported (.bloop, .metals, .bsp, .mcp.json present)
```

`mcp__metals__list-modules` should return 27 build targets including `scJVM`. The workhorse query is `mcp__metals__get-source(fqcn=..., module="scJVM", detailed=true)`, which returns full Scala bodies. See the FQCN cheat-sheet (sec 5 below) for the canonical entry points.

If you don't have `sigmastate-interpreter` cloned + metals-imported, the framework falls back to manual Scala-source reading + `p2sAddress` probes — slower, error-prone (5/9 falsifications resulted from skipping metals).

---

## 4. The 8 methodology classes

| # | Class | Detection signal | Highest-yield FQCN |
|---|---|---|---|
| 1 | Constant-fold per-arm/per-shape | LOCAL doesn't fold a Const+Const op (or folds where Scala doesn't) | `sigma.compiler.ir.DefRewriting` (`propagateBinOp`, `propagateUnOp`, `op.shouldPropagate`) |
| 2 | Walker-arm completeness | Specific AST node (Append/Slice/Fold/CreateProveDlog) doesn't recurse where Scala does | `sigma.compiler.ir.AstGraphs` (`sym.node.syms` walking) |
| 3 | Extraction-count under | LOCAL inlines what NODE extracts; outer ValDefs LOCAL < NODE | `sigma.compiler.ir.AstGraphs.buildUsageMap(usingDeps=false)` + `hasManyUsagesGlobal` |
| 4 | Extraction-count over | LOCAL extracts what NODE inlines; outer ValDefs LOCAL > NODE | Same + check `IsContextProperty`/`IsInternalDef`/`IsConstantDef` filters |
| 5 | Body-walker-order / scope-placement | Same multiset of ValDefs but different IDs / different scope (root vs inner BlockValue) | `sigma.compiler.ir.AstGraphs` (schedule, flatSchedule) + `sigma.compiler.ir.TreeBuilding` |
| 6 | Alias-ValDef HIR/MIR seam | Source-level `val x = OUTPUTS(N)` survives HIR; pins branch-local ValId; downstream guards reject | (Rust-side bug); cross-check `AstGraphs` |
| 7 | AST-consistency emitter-layer | `ValUse(N, T)` has no matching `ValDef(N, _: T)` in any ancestor scope; segregation roundtrip rejects | (Rust-side CSE bug); confirm via `sigma.compiler.ir.TreeBuilding` |
| 8 | Cross-branch coordination (PLATEAU class) | Sibling If-branch ThunkDefs each independently extract the same expression; NODE's graph IR shares one sym across thunks | `sigma.compiler.ir.AstGraphs` (mainG sym scope) + `sigma.compiler.ir.TreeBuilding` (Thunk handling) |

Run `/audit-residual-classify <fixture>` to apply detection heuristics in order; outputs the suspected class + Probe 0 plan.

---

## 5. Scala-side FQCN cheat-sheet (seed)

All FQCNs verified to resolve via metals on 2026-05-08 unless marked `[unverified]`.

| Topic | FQCN |
|---|---|
| Top-level rewrite dispatch | `sigma.compiler.ir.DefRewriting` |
| Const-fold gate | `sigma.compiler.ir.DefRewriting.propagateBinOp` (specifically `op.shouldPropagate(xVal, yVal)`) |
| MethodCall + `tryInvoke` + `mkMethodCall` | `sigma.compiler.ir.MethodCalls` |
| Extraction / schedule | `sigma.compiler.ir.AstGraphs` (members: `Schedule`, `flatSchedule`, `buildFlatSchedule`, `buildUsageMap`) |
| Const node basics | `sigma.compiler.ir.Base` (`extractConst`, `Const`/`Def`/`Sym`/`Ref`) |
| Tree builder (graph → ErgoTree) | `sigma.compiler.ir.TreeBuilding` |
| Pass config (constant-prop flag) | `currentPass.config.constantPropagation` (search `Transforming` / `IRContext`) |

**Symbols that LOOK right but don't exist** (avoid wasting `glob-search` calls): `EliminateCommonSubexpressions`, `numericUpcast`, `cseStrategy`, `commonSub*`. CSE is implicit in `AstGraphs`/`Schedule`, not a named pass.

When a session surfaces a useful new FQCN, append it to this table.

---

## 6. Three gating patterns (for fix proposals)

The methodology toolkit has three validated gate-design patterns. Use these instead of inventing one-off predicates.

| Pattern | Origin | When to apply |
|---|---|---|
| **Strict-subset gating** | sigmausd S4 (`d763d262`) `parent_is_or`; sigmausd S5 (`23fbe431`) `has_direct_interleave && !dense_outer` | Existing flag fires too broadly. Gate via additional state thread so flag fires in strict subset of prior cases. Regression-safe by construction. |
| **Two-axis evidence gating** | paideia S2 (`c9ea218a`) `inline_alias_vals` eager-vs-in-branch | Inline/extract decision needs evidence the post-fix state matches NODE; single-axis insufficient. |
| **LCA-aware permissive scope check** | paideia S3 (`d6ba8d82`) `count_distinct_top_level_containers` | Strict `appears_in_main_scope` rejects valid root-LCA candidates whose uses are all inside If branches but at sibling LCA scope. |

---

## 7. Common anti-patterns

Documented across 9 falsification-fingerprint instances:

- **Don't skip Probe 0 metals.** 5/9 falsifications resulted from this. The audit guide's most-violated rule.
- **Don't trust prior cluster-archive predictions of "S(N+1) path" without re-validating at HEAD.** Cascades shift the landscape under stale plans (sigmao S3 retrospective).
- **Don't declare extraction-count parity from cardinality alone.** Verify multiset (paideia S4 retrospective).
- **Don't commit on hope after 7+ falsifications looks "almost there."** Diag-only is a valid honest outcome.
- **Don't treat segregation-roundtrip-failure as a parser bug without running the AST-consistency probe** (`probe_gluon_scopes`). Wrong layer guess wastes a session (gluon S3 retrospective).
- **Don't make CSE emit globally-unique IDs.** 4 currently-MATCH fixtures have ValId collisions baseline because Scala emits them. Globally-unique counters regress those fixtures.
- **Don't add Co-Authored-By lines to commits** (per `feedback_no_coauthor.md`).
- **Don't push to remote without user testing locally first** (per `feedback_no_push_before_test.md`).

---

## 8. How to run an audit session

For a fresh contributor with this repo cloned + metals + the skills installed:

```bash
# 1. Pre-flight: confirm framework health
mcp__metals__list-modules                              # should return 27 targets
cargo test -p ergoscript-compiler --lib --release      # 233/233 baseline

# 2. Pick a target — let's say sigmao_option
/audit-residual-classify sigmao_option -32             # classify residual

# 3. Run the session (skill expands the full workflow)
/audit-fixture-session sigmao_option 5

# 4. The skill drives:
#    - reading prior archives + memory
#    - Probe 0 metals queries
#    - Probe 1 empirical IR via debug_<fixture>
#    - Probe 2 falsification regression-then-restore
#    - commit + archive + memory updates if MATCH or partial
#    - diag-only commit if probes don't converge
```

---

## 9. Wave-2 ship status (this commit)

This commit captures the framework v1 + sig-15 10/15 milestone state. It does not push to remote — per `feedback_no_push_before_test.md`, push is a separate user-approved step.

What's frozen at this commit:
- Sig-15: 10/15 LOCAL byte-MATCH; 3 diagnosed plateaus + 2 suspected silent regressions.
- F.2 corpus: 563/575 = 97.9% byte-MATCH.
- Ecosystem: 10/14 LOCAL MATCH + 4 USED NODE.
- Lib + conformance: 233/233 + 164/164.
- Diagnostic infrastructure: `debug_<fixture>` per fixture, `probe_sig15_collisions`, `probe_gluon_scopes`.
- 9 falsification-fingerprint instances logged as methodology rules.

What's deferred for the next CSE-pass workstream:
- The 3 diagnosed plateaus (paideia, gluon, sigmao) → unified CSE-pass invariant work.
- Oracle / duckpools regression verification + restoration (likely 1 session each).
- AVL IR fix (CreateAvlTree::value_length shape mismatch — separate from sig-15 work; blocks Lithos / Etcha / Machina Finance byte-match).

---

## 10. Maintaining this guide

When a new methodology class surfaces, append to §4. When a new useful FQCN is verified, append to §5. When a falsification instance demonstrates a new anti-pattern, append to §7. The guide is a living artifact tracked under git so the methodology stays discoverable as the codebase evolves.
