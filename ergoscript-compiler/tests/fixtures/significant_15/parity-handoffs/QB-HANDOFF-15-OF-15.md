# Quarterback Handoff — Ship sig-15 at 15/15

**Target:** sig-15 → **15/15** LOCAL byte-MATCH against Scala reference (Ergo node v6.1.2).
**Branch:** `workstream-f`
**HEAD:** `46451e9d`
**Inherited state:** 12/15 already MATCH. 3 plateaus remaining: `paideia_stake_state` +2, `sigmao_option` -26, `gluon_box_guard` +102.
**No off-ramp.** Plan continues until 15/15 or empirical evidence reframes the next probe target.

---

## 0. Your role — READ THIS FIRST

**You are the coordinator, NOT the implementer.** You do not write Rust code, run `cargo test`, modify `mir/cse.rs`, or execute probes yourself. Single sessions do that — invoked via `/audit-fixture-session <fixture> <N>` against fresh agent contexts.

### What the user does

1. Reads your next-session handoff doc.
2. Invokes a fresh session: `/audit-fixture-session <fixture> <N>` with `SESSION-SPECIFIC CONTEXT — see <handoff>.md`.
3. That session runs in its own context, does the actual work (Probe 0 metals, Probe 1 instrumentation, fix implementation, validation, commit), and reports the commit hash back to you.

### What you do (per session-cycle, your only job)

1. **Receive** a commit hash from the user (e.g., "Commit: abc12345").
2. **Inspect** the commit: `git log -1 --format="%H%n%s%n%n%b" <hash>` to read what was done.
3. **Verify** housekeeping artifacts the session should have left:
   - `target/diff_fuzz/clusters/closed/<hash>_*.md` archive (if not present, write it from the commit body).
   - `~/.claude/projects/-home-cq-working-files-sigma-rust/memory/project_*.md` entry.
   - `MEMORY.md` index updated.
   - `README-HANDOFF.md` §"Bug-surface state" updated.
4. **Synthesize** what the session found vs the prior plan. Update the state model.
5. **Write the next-session handoff** (a new `.md` file in `parity-handoffs/`) with the empirical findings from this session baked in. Each handoff is a self-contained kickoff for a fresh agent.
6. **Tell the user** the concrete invocation: the `/audit-fixture-session` line + which handoff to reference.

You write docs and coordinate state. Sessions move bytes.

### Why this matters

A session has limited context budget. The handoff doc is its full operating manual — written by you (the coordinator) from accumulated empirical evidence across all prior sessions. The session reads the handoff, executes the probe/fix per its instructions, commits, and reports back. Then the next session is invoked with a fresh context against the next handoff you write.

**If you attempt to do the implementation work yourself in your own context, you'll burn through context window inspecting code, running tests, and debugging — none of which is what the coordinator role requires.** The session's job is implementation; yours is direction.

### Coordinator anti-patterns (from this workstream's history)

- **Don't over-narrate.** State variables and byte deltas after each session are what matter. 100-line prose summaries to the user padded sessions without moving bytes.
- **Don't speculate beyond empirical evidence.** Each next-session handoff is grounded in the prior session's commit data, not in extrapolated theory.
- **Don't propose multi-session plans without per-session stop conditions.** Each handoff = one session = one stop condition.
- **Don't write code yourself even for "small" things.** You modify only doc files (handoffs, README, memory, archives). Implementation = sessions. Hard line.
- **Don't push to remote.** Per `feedback_no_push_before_test`. The user pushes when ready.

### Coordinator cadence per session-cycle (the loop you run)

```
USER  → "Commit: <hash>"
QB    → inspect commit body
QB    → write/update target/diff_fuzz/clusters/closed/<hash>_*.md  (cluster archive)
QB    → update memory/project_<topic>.md  (per-session findings)
QB    → update MEMORY.md  (index)
QB    → update README-HANDOFF.md §"Bug-surface state"  (live state)
QB    → write parity-handoffs/<NEXT-HANDOFF>.md  (next session kickoff)
QB    → reply to user with: state variables (Section 6) + invocation for next session
USER  → invokes next session via /audit-fixture-session
        (loop repeats)
```

Each cycle takes ~5-15 minutes of your context. Sessions take 30-90 minutes of fresh context each.

---

## 1. The codebase you're inheriting

### Production parity (committed; ship-ready for wave-2 PR after 15/15 lands)

| Test suite | Result |
|---|---|
| `test_significant_15` | **12/15 LOCAL byte-MATCH** |
| `test_diff_fuzz` (F.2 corpus, 575 generated programs) | **563 MATCH / 10 DIFF / 2 SCALA_FAIL = 97.9%** |
| `test_ecosystem_batch` (14 real contracts) | **11 LOCAL MATCH + 3 USED NODE** |
| `cargo test -p ergoscript-compiler --lib` | **251/251** |
| `cargo test -p ergoscript-compiler --test conformance` | **164/164** |
| `ergotree-ir --features arbitrary --lib` | **255/255** |
| `ergotree-interpreter --features arbitrary --lib` | **336/336** |

### Sig-15 fixtures byte-MATCH (12 of 15)

`chaincash_reserve`, `dexy_bank_full`, `duckpools_child_interest`, `ergomixer_fullmix`, `ergoraffle_active`, `oracle_refresh`, `phoenix_hodlerg_bank_full`, `rosen_event_trigger`, `sigmausd_bank`, `skyharbor_v1_erg`, `spectrum_n2t_pool`, `spectrum_t2t_pool`.

### The 3 plateaus remaining

| Fixture | Δ | Empirical class (S13 convergence at `46451e9d`) |
|---|---|---|
| `paideia_stake_state` | +2 | DAG-identity hash-cons / scope-chain semantics |
| `sigmao_option` | -26 | Same class (pool-order = ValDef-shape divergence, per S13 multiset proof) |
| `gluon_box_guard` | +102 | Same class (extraction-decision-time divergence) |

**All three converge on one code surface:** `process_ast_graph_hash_cons`'s scope-chain semantics in `mir/cse.rs`. The convergence was empirically established by 24 falsifications + S13's slot-by-slot pool-order measurement showing sigmao's residual is same-multiset-permutation (proves LOCAL extracts a different SET of subexpressions than NODE; same root cause as paideia's over-hoist and gluon's per-branch redundancy).

---

## 2. Infrastructure already built (use it; don't rebuild it)

### Hash-cons primitive + driver (BEHIND FEATURE FLAG)

| Asset | Where | Status |
|---|---|---|
| `mir/cse.rs::sym_table::ExprKey` | `mir/cse.rs` | Newtype on `Expr` with structural `Hash`; SourceSpan-stripped. |
| `mir/cse.rs::sym_table::SymTable` | `mir/cse.rs` | `scope_parents` + `scope_defs` + 5 methods (`find`, `intern`, `find_or_intern`, `new_scope`, `is_ancestor`). 8 unit tests pass. |
| `mir/cse.rs::process_ast_graph_hash_cons` | `mir/cse.rs` | Parallel-path driver gated by env var `CSE_HASH_CONS=1`. No panics on any sig-15 fixture. Default path (`process_ast_graph_impl`) untouched when flag unset. |
| Dispatch | `mir/cse.rs::process_ast_graph` + `process_ast_graph_branch` | One-line env check. |

**The hash-cons machinery is built and tested. What's missing is the scope-chain logic correction inside `process_ast_graph_hash_cons` — Session 1 measures, Session 2 fixes.**

Current behavior with `CSE_HASH_CONS=1`:
- Sig-15 batch: 10/15 (regressions: `dexy_bank_full` MATCH→fallback, `duckpools_child_interest` MATCH→fallback, `paideia` +2→+27 over-extract).
- The gate is bidirectionally divergent vs default-path's accreted scope-gate hierarchy — admits some candidates the default rejects AND rejects some candidates the default accepts.

### Walker-completeness (3 walkers complete at HEAD)

| Walker | Status |
|---|---|
| `replace_all` | 7 arms added (`83a962f0`); single-input wrappers + Xor + SubstConstants |
| `contains_val_use` | 1-line `direct_children` recursion fallthrough (`64dc94ce`) |
| `direct_children` | 10 arms added (`f0508f30`); permanent gate via `walker_completeness_probe` test |

**DO NOT** reintroduce explicit-arm walker patterns. Walker-completeness is a load-bearing methodology rule documented in `project_walker_completeness_dominant_cse_bug_class.md`.

### Diagnostic probes (committed in `mir/cse.rs` + `compiler.rs`)

| Probe | Purpose |
|---|---|
| `walker_completeness_probe` (test, runs by default) | Permanent gate; FAILS if any new `direct_children` arm-gap appears |
| `probe_sig15_collisions` (`#[ignore]`) | Per-fixture ValDef-ID collisions, type-conflicts, segregation OK/FAIL |
| `probe_gluon_scopes` (`#[ignore]`) | AST-consistency: unresolvable `OptionGet(ValUse)` sites |
| `probe_gluon_sibling_redundancy` (`#[ignore]`) | Single-fixture sibling-equal ValDef inventory (B1/B2/B3 classification) |
| `probe_sig15_sibling_redundancy` (`#[ignore]`) | Cross-fixture sibling-redundancy inventory (mandatory pre-flight per 21st falsification rule) |
| `audit_1b_expr_hash_perf` (`#[ignore]`) | Perf bench for `expr_hash`; ~520 ns/call BinOp, ~93 ns/call Const |
| `debug_paideia` / `debug_gluon` / `debug_sigmao` / `debug_oracle` / `debug_duckpools` / `debug_sigmausd` / `debug_rosen` / `debug_dexy` etc. | LOCAL vs NODE IR + bytes dump per fixture |

Env-gated trace flags:
- `CSE_TRACE_EXTRACT=1` — per-extraction shape logger
- `CSE_TRACE_SLOT_SHIFT=1` — orphan-VU detector across all 12 pipeline stages
- `CSE_TRACE_VU_PATH=1` + `CSE_TRACE_VD_FOR=<id>` — type-aware orphan-VU walker + ValDef-placement walker
- `CSE_TRACE_CONST_COUNTS=1` — per-stage Const-Sym count walker
- `CSE_TRACE_PRE_EXTRACT=1` — per-extraction shape logger inside `extract_if_cond_shared`
- `CSE_TRACE_SCOPE_CHAIN=1` — *to be added in Session 1*

### Audit framework

| Doc | Location | Purpose |
|---|---|---|
| `AUDIT-FRAMEWORK-GUIDE.md` | repo root | Collaborator-facing methodology guide. Read first for context. |
| `docs/sigma-audit-mcp-design.md` | repo root | MCP server design (not yet implemented; ~½ day to build). |
| `ERGOSCRIPT-COMPILER-STATUS.md` | repo root | Current production status of the compiler crate. |

### Skills (user-global, `~/.claude/skills/`)

| Skill | Invocation |
|---|---|
| `/audit-fixture-session <fixture> [session-N]` | Expands per-fixture arc cadence + methodology rules automatically |
| `/audit-residual-classify <fixture> [Δ]` | Classifies residual into 8 known methodology classes |

### Memory rules (user-global, `~/.claude/projects/-home-cq-working-files-sigma-rust/memory/`)

Mandatory reading before Session 1:

| Rule file | Summary |
|---|---|
| `feedback_metals_first.md` | Probe 0 metals queries on Scala source BEFORE Rust-side instrumentation when asking Scala-semantics questions |
| `feedback_probe_before_third_speculation.md` | Install instrumentation that directly observes the claimed mechanism BEFORE proposing a fix |
| `feedback_falsification_fingerprint.md` | Probe each candidate fix in isolation BEFORE declaring coupling (24 instances) |
| `feedback_no_push_before_test.md` | Local commits only; never push to remote without explicit user approval |
| `feedback_no_coauthor.md` | No `Co-Authored-By` lines in commits |
| `feedback_no_fake_session_split.md` | Don't rewrite history; honest single commits |
| `feedback_handoff_premise_reanchor.md` | When a handoff cites a specific sym/number from a prior archive, re-anchor at HEAD via cheapest empirical probe BEFORE building bucket frameworks |
| `feedback_run_all_tests.md` | CSE tweaks silently break other contracts; run full battery after every edit |
| `project_walker_completeness_dominant_cse_bug_class.md` | Walker-completeness is the dominant CSE bug class; preserve `direct_children` discipline |

**Cross-fixture pre-flight rule (codified in 21st falsification archive `51f4effc_*`):** every fix candidate must validate against 12 MATCH fixtures BEFORE commit. MATCH fixtures contain patterns that LOOK like the target bug; naive fixes regress them silently.

---

## 3. Empirical context (compressed)

**24 falsification instances** logged across:
- WS-G architectural migration (11 sessions, 0 byte movement on plateaus)
- Inversions A/B/C (3 sessions; surgical fix-spaces empirically empty at extraction-decision-time)
- Bisections A1-A5 (5 sessions; 6B moved on sigmao count layer; remaining sigmao residual converges with WS-G architectural class)

**First plateau byte movement:** sigmao -32 → -26 at `eb3ec4cd` (S12). Surgical narrowing in `extract_if_cond_shared` for `BinOp(Relation(Eq|NEq))` candidates without ValUse children. Closed the constant-pool count layer.

**Convergence finding (S13 `46451e9d`):** sigmao's remaining -26 is same-multiset slot-permutation in the constants pool — direct evidence that LOCAL and NODE construct the same 61 Constants in different recursive traversal orders. Combined with paideia +2 over-hoist and gluon +102 per-branch redundancy findings, all three plateaus converge on `process_ast_graph_hash_cons` scope-chain semantics.

**The unrun probe:** G.2.4c scope-chain trace. Originally queued as the final probe in the architectural migration arc; never executed. With the empirical convergence locked in, this is the next session's target.

---

## 4. The plan — sessions to 15/15

Each session ends in a single commit (or diag-only commit). Each session has one stop condition. Sessions chain via sub-handoffs.

### Session 1 — Scope-chain probe (MEASUREMENT)

**Handoff:** [`G2.4c-RESUME-SCOPE-CHAIN-PROBE-HANDOFF.md`](G2.4c-RESUME-SCOPE-CHAIN-PROBE-HANDOFF.md)

**What:** Add `CSE_TRACE_SCOPE_CHAIN=1` env-gated probe to `SymTable::find_or_intern` and `process_ast_graph_hash_cons`. For each `find_or_intern` call on paideia / sigmao / gluon under `CSE_HASH_CONS=1`: log node, current scope, full chain walk, hit/miss per scope, final result, gate decision.

Compare against `process_ast_graph_impl` (default path) extraction decisions on the same source positions in each fixture.

Cross-fixture pre-flight: sigmausd / rosen / oracle (MATCH fixtures) must NOT trigger the bug pattern.

**Stop condition:** classify the divergence into one of:
- **D1**: same scope-chain bug across all 3 plateaus → Session 2 fixes all 3 in one change.
- **D2**: 2 fixtures share the bug, 1 differs → Session 2 fixes 2, Session 3 handles third.
- **D3**: each fixture has distinct scope-chain bug → 3 separate fixes across Sessions 2-4.
- **D4**: no clear scope-chain divergence → Session 1b runs a different probe (scope-tree construction, iteration order, or Const-Sym interning timing).

Diag-only commit. The empirical trace IS the deliverable.

### Session 2 — Surgical fix per Session 1's D-result

**What:** Implement the scope-chain correction per Session 1's empirical data.

**Cross-fixture validation (BLOCKING):**
- All 12 currently-MATCH fixtures hold under `CSE_HASH_CONS=1`.
- F.2 corpus ≥ 563/575 under flag.
- Ecosystem ≥ 11/14 under flag.

**Target outcomes:**
- D1 fix: sig-15 → 15/15 under flag.
- D2 fix: sig-15 → 14/15 under flag.
- D3 fix: sig-15 → 13/15 under flag (Sessions 3-4 close remaining).

**Stop condition:** cross-fixture clean + plateaus close as expected → commit. If cross-fixture regresses → narrow with strict-subset gating (precedent: sigmausd S4/S5 `d763d262` / `23fbe431`); don't commit until clean.

### Session 3 — Iteration / remaining plateaus

**Triggers:**
- Session 2 had cross-fixture regression → tighten the fix with strict-subset gating.
- Session 2 partial close (D2/D3) → apply same probe-localize-fix pattern to next remaining plateau.

**Stop condition:** sig-15 batch under flag ≥ 14/15 with zero MATCH regressions.

### Session 1b / 1c / ... — Alternative probes if Session 1 D4

If Session 1's scope-chain probe shows no clear divergence:
- **Session 1b**: `CSE_TRACE_SCOPE_CONSTRUCTION=1` — measure scope-tree shape divergence (scope push/pop ordering inside `process_ast_graph_hash_cons`).
- **Session 1c**: Per-pass Const-Sym interning timing across HIR → MIR → CSE → emission for the specific 27 divergent sigmao slots from S13.
- **Session 1d+**: continue narrowing until the bug surface is localized.

**The methodology guarantees empirical narrowing every session.** Each falsification eliminates a layer; the search space is bounded.

### Final Session — Flip default + remove old path

**Trigger:** sig-15 under flag at 15/15 + cross-fixture stable.

1. Make `CSE_HASH_CONS=1` the default in dispatch (`process_ast_graph` + `process_ast_graph_branch`).
2. Run full battery. Bytes match prior flag=1 results.
3. Remove `process_ast_graph_impl` (old path) as dead code.
4. Commit. **15/15 shipped.**

---

## 5. Falsification axes (every commit)

- **F1**: sig-15 ≥ 12/15 with `CSE_HASH_CONS` UNSET. Default path is sacred until the final session flips it.
- **F2**: F.2 corpus ≥ 563/575 with default flag.
- **F3**: lib 251/251 + conformance 164/164 with default flag.
- **F4**: `probe_sig15_collisions` baselines preserved with default flag.
- **F5**: with `CSE_HASH_CONS=1`, no panics / no ERRs.
- **F6**: every session must EITHER move bytes OR sharpen the next probe with empirical evidence. Hand-waving = revert.
- **F7**: every fix candidate cross-fixture-validates against 12 MATCH fixtures BEFORE commit (21st falsification rule).

If F1 ever fails — the integration leaked into the default path. **REVERT immediately.**

---

## 6. State variables to report after every session

```
sig-15 (default):  X/15      (must be ≥ 12)
sig-15 (HC=1):     X/15      (must be ≥ 10; target 13→14→15)
F.2 (default):     XXX/575   (must be ≥ 563)
F.2 (HC=1):        XXX/575   (must be ≥ 563)
Lib + conformance: 251 + 164 (must hold)
Ecosystem:         X/14      (must be ≥ 11)
Bytes moved this session: X
Cross-fixture regressions on commit: 0 (REQUIRED)
```

---

## 7. Pre-flight reads (~45 minutes)

Read in this order:

1. **`README-HANDOFF.md`** §"Bug-surface state post-`46451e9d`" — top of file. Current state in one paragraph.
2. **`AUDIT-FRAMEWORK-GUIDE.md`** — methodology in §2; 8 residual classes in §4; 3 gating patterns in §6; 9 anti-patterns in §7.
3. **`G2.4c-RESUME-SCOPE-CHAIN-PROBE-HANDOFF.md`** — Session 1's full procedure.
4. **`target/diff_fuzz/clusters/closed/46451e9d_*`** — S13 convergence archive (justifies the plan).
5. **`target/diff_fuzz/clusters/closed/eb3ec4cd_*`** — S12 (first plateau byte movement; surgical-narrowing precedent).
6. **`target/diff_fuzz/clusters/closed/3b56db63_*`** — G.2.3 driver landing (bidirectional divergence finding).
7. **`mir/cse.rs::process_ast_graph_hash_cons`** — READ (function under modification).
8. **`mir/cse.rs::sym_table::SymTable::find_or_intern`** — READ (the scope-chain walker).
9. **`mir/cse.rs::process_ast_graph_impl`** — READ-ONLY reference. **DO NOT modify.**

Memory files (mandatory):
- `feedback_metals_first.md`
- `feedback_probe_before_third_speculation.md`
- `feedback_falsification_fingerprint.md` (skim; 24 instances)
- `project_walker_completeness_dominant_cse_bug_class.md`
- `feedback_handoff_premise_reanchor.md`
- `feedback_no_push_before_test.md`
- `feedback_no_coauthor.md`

---

## 8. Commit cadence (every session)

Each session ends in one commit. Commit body MUST include:
- Residual class diagnosed (per S13 convergence: scope-chain or alternative-probe layer).
- Fix shape (or "none — diag only" for measurement sessions).
- Falsification probes run.
- Exact byte deltas per fixture.
- Cross-fixture impact.
- Residual hypothesis for next session.

Then write:
- Archive at `target/diff_fuzz/clusters/closed/<commit-hash>_*.md`.
- Memory entry at `~/.claude/projects/-home-cq-working-files-sigma-rust/memory/project_<topic>.md`.
- Update `MEMORY.md` index.
- Update `README-HANDOFF.md` §"Bug-surface state" with new state.

Rules:
- **Local commits only.** Never push to remote (per `feedback_no_push_before_test`).
- **No `Co-Authored-By` lines** (per `feedback_no_coauthor`).
- **No prose padding in commit messages.** Numbers > narrative. Empirical evidence > hand-waving.

---

## 9. Anti-patterns

- **Don't relaunch the inversion arc.** Inversions A/B/C closed at falsification 20/21/22. The surgical fix-space at extraction-decision-time is empirically empty for paideia and gluon.
- **Don't restart wholesale architectural framing.** WS-G's "build the full migration" framing produced 11 sessions, 0 byte movement on plateaus.
- **Don't expand scope mid-session.** Each session has one stop condition; either hit it or commit diag-only.
- **Don't propose a fix without empirical Probe 1 data.** Per probe-before-third-speculation (24-instance-validated).
- **Don't skip cross-fixture pre-flight.** 21st falsification rule.
- **Don't write long prose in commits or session updates.** State variables and byte deltas are the deliverable.
- **Don't push to remote without explicit user confirmation.**
- **Don't touch `process_ast_graph_impl`.** Default path is sacred until the final session flips it.

---

## 10. Concrete invocation for Session 1 — give this to the user

Hand the user this invocation. **You do not run it.** A fresh agent will pick it up in its own context.

```
/audit-fixture-session paideia_stake_state 22

SESSION-SPECIFIC CONTEXT — see parity-handoffs/G2.4c-RESUME-SCOPE-CHAIN-PROBE-HANDOFF.md

PRIORITY: Session 1 of QB-HANDOFF-15-OF-15. Read QB-HANDOFF-15-OF-15.md
first for the multi-session plan. Goal is sig-15 → 15/15.
```

The probe runs against all 3 plateau fixtures (sigmao + paideia + gluon) under `CSE_HASH_CONS=1`. The `paideia_stake_state` fixture name in the invocation is just the session identifier; the trace covers all three.

When the session completes, the user will reply to you with something like `"Commit: <hash>"`. That's your trigger to execute the coordinator cadence in §0: inspect commit → archive → memory → README → write next handoff → reply with invocation.

---

## 11. Working tree state at handoff

```
HEAD: 46451e9d
Branch: workstream-f
Uncommitted (pre-existing parked work; not related to plateau closure):
  M ergoscript-compiler/tests/diff_fuzz.rs
  M ergoscript-compiler/tests/diff_fuzz_gen.rs
  ?? ergoscript-compiler/tests/fixtures/significant_15/SIGNIFICANT-15-PLAN.md
```

The parked lint edits have been in the working tree for multiple sessions. They are NOT load-bearing for the plateau work and can be left as-is. Resolve them in a separate cleanup session before any wave-2 PR push, not before Session 1.

---

## 12. Quick orientation for the new agent

This is the sigma-rust ergoscript-compiler parity workstream. The compiler produces ErgoTree bytecode from ErgoScript source code; the goal is byte-identical output to the reference Scala compiler.

**What's been built:** a production-parity compiler that byte-matches Scala for 12 of 15 keystone contracts + 97.9% of a generated fuzz corpus + 11 of 14 real-world contracts. Audit framework v1 with reusable methodology IP. Diagnostic infrastructure for any future Scala→Rust parity work.

**What remains:** 3 contract fixtures whose Δ-bytes vs Scala converge on one specific Rust code surface (`mir/cse.rs::process_ast_graph_hash_cons` scope-chain semantics). Hash-cons primitive is built and tested behind a feature flag; the scope-chain logic inside the driver needs correction.

**The plan:** measure the scope-chain divergence empirically (Session 1), fix per the empirical data (Sessions 2-3), flip the default flag and remove the old path (final session). Sessions chain via sub-handoffs written as predecessors land.

**The methodology:** probe-before-third-speculation, metals-first for Scala questions, falsification fingerprint discipline (24 instances logged), cross-fixture pre-flight (mandatory per 21st falsification rule), walker-completeness preserved across `direct_children`/`replace_all`/`contains_val_use`.

**The target:** 15/15. The plan continues until it lands.

**Your role one more time:** coordinator. You write handoffs and update state docs. Sessions write code and run tests. See §0.
