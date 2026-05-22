---
name: S38 atom-hoist candidate table — Probe 1 falsification
description: Per-atom byte math empirically falsifies CLOSE-GLUON-S38-PREDICATE-A.md's premise that atom-level hoist closes the +60B gluon residual
type: research
---

# S38 Atom-Hoist Candidate Table — Probe 1 Output

**HEAD:** `d2895e8a` (S37 stage 13 swap-symmetric pair merger LANDED; gluon 2343B both HC=0 and v3).
**Probe 1 source:** `/tmp/g_node_s38.txt` (combined NODE + LOCAL+v3 dump with verbose outer-shape diff via `SIG15_DUMP_OUTER_SHAPES_VERBOSE=1`).
**Canonical baseline:** `probe_sig15_local_hex` HC=0 and v3 both report **gluon 2343B** (identical hex).

---

## 1. Byte encoding formula

In ergotree serialization the marginal cost of hoisting an inline atom of size `X` across `N` use-sites is:

```
Δbytes = − N·X                       (inline atoms removed)
       + 2 + X                       (one new outer ValDef: 2B header + X RHS)
       + 2·N                         (N ValUse references at sites)

       = X·(N−1) − 2·(N+1)
```

Hoist saves bytes iff `X·(N−1) > 2·(N+1)`, i.e.:

| N (use sites) | Min atom size X to save bytes |
|---|---|
| 2 | X > 6  |
| 3 | X > 4  |
| 4 | X > 3.33 |
| 5 | X > 3  |

Atom-size lookup (gluon multiset members):
- `ValUse(id)` = **2B** (`72 NN`)
- `ExtractAmount(VU)` = **3B** (`c1 72 NN`)
- `BinOp(VU, VU)` = **5B** (`91 72 NN 72 NN`, e.g. `<>`)
- `BinOp(VU, ExtractAmount(VU))` = **6B**
- `BinOp(VU, OptionGet[ExtractRegister<RX>[VU]])` ≈ **7–8B**
- `ProjectColl<tokens>[VU]` ≈ **3–4B**
- `OptionGet[ExtractRegister[VU]]` ≈ **4–5B**

---

## 2. Per-atom candidate table

Sourced from `/tmp/g_node_s38.txt:336–355` SHAPE DIFF multiset.

| Atom shape (NODE outer ValDef) | Δ L→N | Use-sites (N) | X (inline B) | Δbytes per hoist | Hoist verdict |
|---|---|---|---|---|---|
| `BO<<>[VU,VU]` (Neutrons.<) | L=0 N=1 of 2 | 2 (Fusion d19, BetaDecayMinus d21) | 5 | **−1B** | **NO (regresses)** |
| `BO<<>[VU,VU]` (Protons.<)  | L=0 N=1 of 2 | 2 (Fusion d19, BetaDecayPlus d20)  | 5 | **−1B** | **NO** |
| `BO<>>[VU,VU]` (Neutrons.>) | L=0 N=1 of 2 | 2 (Fission d18, BetaDecayPlus d20) | 5 | **−1B** | **NO** |
| `BO<>>[VU,VU]` (Protons.>)  | L=0 N=1 of 2 | 2 (Fission d18, BetaDecayMinus d21)| 5 | **−1B** | **NO** |
| `BO<==>[VU,VU]` (Value.==)  | L=0 N=1     | 2 (d20, d21; with `ExAmt[VU]` operand also hoisted) | 5 | **−1B** | **NO** |
| `ExAmt[VU]` (OUT box.value) | L=0 N=1     | 4 (d18 pos3, d19 pos3, d20 pos3, d21 pos3) | 3 | **−1B** | **NO** (X·3 − 10 = −1) |
| `PC<tokens>[VU]` (OUT box.tokens) | L=0 N=1 | 2 (d4, d6 ByIdx inner) | ~4 | **0B** | break-even |
| `OGet[ExReg<R5>[VU]]` (R5 on OUT) | L=0 N=1 | 1 (only inside d12 pos 6) | ~4 | n/a (single use) | NO |
| `BO<==>[VU,OGet[ExReg<R4>[VU]]]` | L=0 N=1 | 1 (only inside d12 pos 5) | ~7 | n/a (single use) | NO |

**Sum of per-atom Δbytes if ALL 6 multi-site hoists land verbatim:** roughly **−6 to −8 bytes (REGRESSION)**.

---

## 3. Cross-check against §2a §2c framing

`GLUON-EMPIRICAL-MAP.md` §2c #1 claims:

> "Each redundant BO inline costs ~3-5B over the ValUse it replaces. With ~9 hoist-misses × ~3-4 sites each, this alone accounts for ~+70B raw, partially offset by LOCAL skipping a few outer ValDef headers."

The §2c framing is **arithmetically inconsistent** with the multiset count and the ergotree encoding:

1. The multiset shows ~6 distinct multi-site atom shapes, not 9.
2. Site counts cap at N=2 for the BO atoms; only ExAmt[VU] reaches N=4.
3. The ValUse-vs-inline byte saving per site is 3B (5B atom → 2B VU), NOT 3–5B, AND must net out the ValDef header (2B) + outer RHS (5B) once.
4. Per-atom marginal Δbytes for these N=2 atoms is **−1B** (regression), not "+3–5B per site savings".

The §2c "+70B raw" estimate appears to have **double-counted** the inline-atom savings without subtracting the outer-ValDef-and-VU-ref overheads. This is a clean Probe 1 falsification of the §2c bytes-headline that drove CLOSE-GLUON-S38-PREDICATE-A.md's premise.

---

## 4. Where does the +60B residual actually live?

Per §2a/§2c, LOCAL has +60B over NODE. The atom-level hoist layer (above) is at best ~−2B savings, more likely ~+6–8B regression. So the +60B residual lives elsewhere:

| Residual class | Estimated B | Where to look |
|---|---|---|
| `K(Int)`/`K(Long)` stale outer ValDef noise (L d 22, 27, 31, 32, 33) | ~+10–15B | over-promotion of single-use constants to outer ValDefs |
| d12 internal atom replacements (atom positions 5/6/7 inside `__gluonWBoxPersistedValueCheck`) — NODE replaces inline atoms with VU refs to N d 9 (`BO<==>[VU,OGet[ExReg<R4>[VU]]]`), N d 11 (`OGet[ExReg<R5>[VU]]`), N d 14 (`Sel<#2>[VU]`) | ~+10–14B | inside-d12 atom positions (atom 5 saves ~+5B if hoist BO atom; atom 6 saves ~+2B; atom 7 saves ~+2B) |
| Constants pool layout difference (LOCAL 108 entries vs NODE 114) | ~+5–10B | pool packing of consants vs outer-ValDef-as-constant emission |
| `PC<tokens>[VU]` inline-vs-hoist (L d 4, d 6 ByIdx have `PC<tokens>[VU]` inside; NODE hoists once) | ~+2–4B | break-even per Section 2 but enables d4/d6 byte-shrink |
| Structural atom hoists in d18–d21 | ~+6–9B regression IF replicated | (do NOT replicate; falsified above) |

Total accounted: ~+27–43B. Residual unaccounted ~+17–33B is **structural-divergence** at deeper nesting that requires Probe 1 zoom into specific positions inside d18–d21 and the chained-If body.

---

## 5. Cross-fixture insulation re-check (Probe 2 skipped)

Per `CLOSE-GLUON-S38-PREDICATE-A.md` Step 4 and `GLUON-EMPIRICAL-MAP.md` §9d:

- Predicate-A's `arity_1: chained-If depth ≥ 3` gate insulates all 11 v3 MATCH fixtures + paideia + sigmao (none have chained-If depth ≥ 3 except gluon; chaincash has 2 < 3).
- So cross-fixture regression risk from Predicate-A is empirically ≈ 0.

The gating discipline is sound. The byte math is what fails.

---

## 6. Stop-table verdict

Per CLOSE-GLUON-S38-PREDICATE-A.md Stop conditions:

> "| All atom candidates falsify (cross-fixture or byte-negative) | diag — atom-level layer also exhausted for gluon | accept gluon +60 residual; document |"

**This row matches.** Per the byte-encoding formula, 6 of 6 multi-site atom hoists from the §2a multiset are byte-negative (−1B each) or break-even. The atom-level hoist layer cannot close the +60B residual.

---

## 7. Recommendation — pivot S39+

Rather than implement Predicate-A as a byte-negative pass, the next byte-mover for gluon should target one of these residual classes (in descending expected yield):

1. **K-noise removal (~+10–15B):** identify the 5 stale K(Int)/K(Long) outer ValDefs (L d 22/27/31/32/33) and either:
   - inline-them at use sites (if single-use), OR
   - convert them to PC pool references (LOCAL's existing dedup_inner_consts gate already does this for inner-scope; extend to outer ValDefs).
2. **d12-internal atom hoist (~+10–14B):** hoist the 3 atoms at d12 positions 5/6/7 (`BO<==>[VU,OGet[ExReg<R4>[VU]]]` at pos 5; `OGet[ExReg<R5>[VU]]` at pos 6; `Sel<#2>[VU]` at pos 7). These ARE byte-positive when hoisted (X=7,4,4 with N=2+ via shared atoms across d12 and chained-If branches).
3. **PC<tokens>[VU] hoist (~+2–4B):** the `tokens` PropertyCall on VU(d2) appears in d4 and d6's ByIdx; hoist once.

These three layers cumulatively could close ~+22–33B, vs Predicate-A's −7B regression.

---

## 8. Falsification fingerprint

This is the **78th cumulative falsification fingerprint** in the WS-G sig-15 arc, of class:

> "Probe 1 byte-encoding-math falsifies handoff bytes-headline before any code change."

Closest archive precedents:
- S77 (sigmao): Probe 1 IR-dump preempted speculation; handoff direction was inverted from empirics.
- S74b (sigmao): Probe 1 IR-dump preempted 76th fp on K>=1→K==1 narrow.
- S60 (paideia): bytes-first probe found root cause skip sweep in one session vs 6 theory sessions.

Per `feedback_falsification_fingerprint` and `feedback_bytes_first_before_theory`, the empirical math counts BEFORE the theory.

---

## 9. Artifacts

- `/tmp/g_node_s38.txt` — Probe 1 combined NODE + LOCAL+v3 dump (22114 lines)
- `/tmp/g_local_s38.txt` — Probe 1 LOCAL+v3 dump under `CSE_HC_V3=1 SIG15_DUMP_OUTER_SHAPES_VERBOSE=1`
- (no code changes)
