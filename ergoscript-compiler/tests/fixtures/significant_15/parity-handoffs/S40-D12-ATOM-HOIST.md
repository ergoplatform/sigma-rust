# S40 — d12-internal atom hoist + cross-scope dedupe FALSIFIED via Probe 1+2

**HEAD:** `67928016` (S39 diag — K-noise byte-NEGATIVE falsified).

**Session position:** S40 pursued pivot #2 from S38 §7 / S39 §9 — d12-internal atom hoist at positions 5/6/7 with predicted yield ~+10–14B. Probe 1 byte-arithmetic falsified the yield premise for 6 of 7 atom-class candidates; Probe 2 (cross-scope dedupe of inner ValDef RHS matching outer ValDef RHS) found ZERO empirical matches across 4 pipeline-stage placements.

---

## §1 — Probe 1 byte-arithmetic per atom (extended S38 §1 formula)

Per-atom marginal Δbytes from hoisting an N-site atom of inline size X:

```
Δ = X·(N−1) − 2·(N+1)
Saves iff X·(N-1) > 2·(N+1)
```

For the 7 N-only NODE shapes (per S38 multiset diff), assuming N=2 cross-fixture conservative minimum:

| N-only shape | X | Δ at N=2 | Δ at N=3 | Δ at N=4 |
|---|---|---|---|---|
| BO<==>[VU,OGet[ExReg<R4>[VU]]] | 7 | **+1** | +8 | +15 |
| OGet[ExReg<R5>[VU]] | 4 | -2 | 0 | +2 |
| PC<tokens>[VU] | 4 | -2 | 0 | +2 |
| BO<>>[VU,VU] (×2) | 5 | -1 | +2 | +5 |
| BO<<>[VU,VU] (×2) | 5 | -1 | +2 | +5 |
| BO<==>[VU,VU] | 5 | -1 | +2 | +5 |
| ExAmt[VU] | 3 | -3 | -2 | -1 |

Empirical N counts from NODE bytecode (`grep -oE "7209" /tmp/node_hex.txt | wc -l` etc):
- `BO<==>[VU,OGet[ExReg<R4>[VU]]]` (N d9): N=2
- `OGet[ExReg<R5>[VU]]` (N d11): N=2
- `OGet[ExReg<R6>[VU]]` (N d14): N=3 (already extracted in LOCAL as d10)

**Result:** at N=2, only `BO<==>[VU,OGet[ExReg<R4>[VU]]]` is byte-positive (+1B). All other N-only atoms regress 1–3B at the N=2 cross-fixture conservative minimum. Summing all 7 atoms = **net -11B regression** for full atom-hoist class.

NODE's net byte advantage of -60B does NOT come from atom hoist arithmetic. It comes from the structural pattern at NODE d25/d28/d30/d31 = `And[Coll[VU,VU,VU,VU]]` where 3 inline atoms are replaced by 3 ValUse refs (savings 3×(5-2) = 9B per And shape, ×4 = +36B). This requires BOTH atom hoist (extraction) AND inline atom→ValUse substitution (restructuring). Without coordinated restructuring, atom hoist alone is byte-negative as shown above.

---

## §2 — Probe 2 (empirical) — cross-scope inner ValDef RHS dedupe

### Hypothesis

`apply_cse_within_branches` extracts CSE candidates per-branch without consulting the OUTER scope; if shape S is already extracted at outer ValDef d_o, and the same shape S appears inline inside an inner branch and is shared within that branch, the branch pass extracts a SECOND inner ValDef with shape S. A dedupe pass rewriting inner ValDef RHS to `ValUse(d_o)` would eliminate the redundant inner extraction.

Empirical driver in cached `/tmp/g_local_ir.txt`: gluon LOCAL final IR shows inner `ValDef(8) = OGet[ExReg<R4>[Self]]` (lines 11514-11551) where outer `d7 = OGet[ExReg<R4>[Self]]` per the verbose outer-VD dump.

### Implementation

Env-gated probe `CSE_PROBE_S40_INNER_OUTER_DEDUPE=1` (~150 LOC, threading outer-RHS slice via thread-local for closure-less recursion through `map_children`). Tested at 4 pipeline-stage placements:

| Pipeline stage | Outer ValDef count | Inner ValDef matches |
|---|---|---|
| 05c (post-`inline_single_use_vals` 2nd pass) | 36 | 0 |
| 08b (post-`rewrite_byindex_globalvars_chain`) | 36 | 0 |
| 12b (post-`sequential_renumber`) | 36 | 0 |
| 14 (post-S37 `merge_stage12_swap_symmetric_pairs`) | 36 | 0 |

`CSE_TRACE_S40_INNER=1` dumped ALL 48 inner ValDefs encountered at stage 12b — none structurally matched any of the 36 outer ValDef RHSs.

### Why the match doesn't fire

The "duplicates" visible in the FINAL post-everything IR dump are **id-LOCAL to their inner scopes** (per-scope sequential renumbering). They look like outer ValDef RHSs at byte-encoding time but their structural form at intermediate pipeline stages contains different ValUse IDs (pre-renumber inner-scope ids vs outer-scope ids). Structural equality `==` on Expr trees compares ValUse val_ids exactly; identical-shape-different-id ValUses are NOT equal.

### Result

Gluon byte count: 2343B unchanged across all 4 placement variants. Zero byte movement. Hypothesis empirically FALSIFIED.

Probe 2 code reverted (would have been a ~150-LOC durable no-op artifact).

---

## §3 — Stop verdict

Per S38 §7 + S39 §9 pivot ranking and S40 falsification:

| Pivot | Status |
|---|---|
| #1 K-noise removal | S39 byte-NEGATIVE FALSIFIED (+46B regression if applied) |
| #2 d12-internal atom hoist 5/6/7 | S40 byte-arithmetic FALSIFIED (6 of 7 atoms regress at N=2; only one +1B candidate, gated by walker-completeness scope issues that 4 placements failed to address) |
| #3 PC<tokens>[VU] hoist | byte-arithmetic FALSIFIED (X=4 N=2 → -2B regression) |
| #4 pool-layout reorder | S38 commit "no yield from pool reorder per se" |
| #5 ~+17–33B structural divergence at deeper nesting (S38 §4) | requires WS-G architectural rewrite per S29/S78 precedent |

**S40 outcome: third consecutive falsification (S38 → S39 → S40) in the gluon byte-residual reduction class.**

Per `feedback_close_the_gap_not_phase_plumbing`: two byte-neutral diag commits in a row = escalate single-gap focus. S40 = third. Per `feedback_no_ship_off_ramp`: don't pre-decide the off-ramp — commit to closure, document falsification cadence.

The empirical evidence (S38 atom-byte-math + S39 K-noise pool-arithmetic + S40 cross-scope dedupe empirical null) **establishes that the gluon +60B residual is not closable via any surgical CSE-pipeline modification at the current architecture**. Closure requires the **WS-G DAG-identity hash-cons migration** (per `QB-HANDOFF-15-OF-15.md` §0 architectural rewrite class, sister to sigmao S29 segregation-pipeline-rewrite blocker, S78 ThunkScope.findDef cross-thunk-distinct sym semantic).

---

## §4 — 80th cumulative falsification fingerprint AVOIDED

S40 falsified BEFORE landing any persistent code change. Sister falsifications in the same arc:
- S38 (78th) — atom-level hoist byte-math falsification.
- S77 sigmao (77th) — multiset direction inversion via Probe 1.
- S78 sigmao (79th) — multiset undercount + PASS-3 probe REVERTED.
- S39 (79th + counter) — K-noise multi-use pool arithmetic.
- S40 (80th) — d12 atom hoist + cross-scope dedupe empirical null.

Six consecutive Probe-1-first sessions (S38-S40 + S77-S78) preserving the byte/segregation baselines while exhausting the speculative-implementation surface.

---

## §5 — S41+ recommendation

**Honest closure: accept gluon +60B = HC=0 = v3 = 2343B as architectural plateau.**

Per `feedback_close_the_gap_not_phase_plumbing.md`, two byte-neutral diags in a row signals "escalate single-gap focus" — done at S38/S39/S40. The single-gap focus has empirically exhausted the surgical-fix surface. Per `feedback_no_ship_off_ramp.md`, the off-ramp wasn't pre-decided; it is post-decided by the falsification cadence and architectural-ceiling diagnosis.

Closure paths beyond surgical:

1. **WS-G architectural rewrite — DAG-identity hash-cons migration.** Per `QB-HANDOFF-15-OF-15.md` §0. Estimated months of work. Would close gluon + sigmao + potentially paideia simultaneously.

2. **Pivot to non-gluon byte-yielding work.** Other fixtures' residuals (paideia Δ -3, sigmao ABANDONED) are smaller; F.2 corpus has 12 remaining DIFF programs per memory; ecosystem fixtures (Lilium, AVL-IR per `project_avl_priority.md`) have known fix candidates.

3. **Decline further sigma-rust gluon work.** State of the union: 15+ gluon sessions (S1-S40), all exhausting at architectural plateau.

User direction needed for selection between (1)/(2)/(3).

---

## §6 — Preflight (post-revert HEAD)

| Axis | Result |
|---|---|
| HC=0 sig-15 12/15 | sigmao 1124, paideia 1471, gluon 2343 baseline ✓ |
| v3 sig-15 13/15 byte-MATCH | paideia 1468, sigmao 1142, gluon 2343 Δ +60 ✓ |
| 4 sacred K-host fixtures | byte-match preserved ✓ |
| `lib` 257/257 | passed ✓ |
| `conformance` 164/164 | passed ✓ |
| `probe_sig15_collisions` | OK ✓ |

---

## §7 — Artifacts

- `/tmp/s39_hc0_outers.txt`, `/tmp/s39_v3_outers.txt` — reused from S39
- `/tmp/g_local_s38.txt`, `/tmp/node_hex.txt` — cached IR + NODE hex
- Empirical NODE use counts: `grep -oE "7209|720b|720e" /tmp/node_hex.txt | wc -l` per ValUse(N) byte sequence

No code change (Probe 2 dedupe code reverted as empirically null).
