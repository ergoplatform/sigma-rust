# S77 Probe 1 — sigmao 1142B residual swap-site localization

**HEAD:** `5e0b6d8c` (S76b)
**Mode:** `CSE_HC_V3=1 CSE_HC_V3_S75_ADMIT_OUTPUTS_K1=1 CSE_HC_V3_S76_SCOPE_AWARE_WALK=1`
**LOCAL:** 1142B  |  **NODE:** 1148B  |  **Δ:** 6B

---

## Result: MULTI-SITE COMPENSATING (per stop-table row 3)

The "1 swap site" framing is empirically falsified. The pool-multiset deficit
`L − N = (−1 SInt(0), +1 SInt(1))` is the **net sum** of structurally
compensating differences across multiple AST positions. There is no single
source line whose K(I,0) ↔ K(I,1) flip would close the deficit.

Per S77 stop-table row 3 → **accept sigmao 1142B partial close; pivot to gluon.**

---

## Empirical decomposition (verified with verbose outer-VD dump + NODE-hex reparse)

### Pool totals
```
LOCAL pool: 11× SInt(0)  13× SInt(1)  (61 entries total)
NODE  pool: 12× SInt(0)  12× SInt(1)  (61 entries total)
Δ = (−1 SInt(0), +1 SInt(1))   ← OPPOSITE direction from handoff text
```

### Outer-ValDef vs body decomposition
| Stratum             | LOCAL K(I,0) | LOCAL K(I,1) | NODE K(I,0) | NODE K(I,1) |
| ------------------- | :----------: | :----------: | :---------: | :---------: |
| outer ValDef RHSes  | 7            | 7            | 7           | 4           |
| body inline (post-) | 4            | 6            | 5           | 8           |
| **pool total**      | **11**       | **13**       | **12**      | **12**      |

### Per-K-value site-by-site cross-check
Every LOCAL outer ValDef whose RHS contains a K(I,N) literal matches NODE's
K(I,N) value at the structurally-equivalent outer or sub-VD site. There is no
flipped literal in any outer ValDef. Specifically:

```
LOCAL d3  = ByIdx<or>[VU,K(I,0),VU]                ≡ NODE d3   K(I,0) ✓
LOCAL d9  = PDlog[DecPt[ByIdx<raw>[VU,K(I,0)]]]    ≡ NODE d9   K(I,0) ✓
LOCAL d12 = ByIdx<raw>[Outputs,K(I,0)]             ≡ NODE d11  K(I,0) ✓
LOCAL d18 = ByIdx<raw>[VU,K(I,6)]                  ≡ NODE d16  K(I,6) ✓
LOCAL d23 = ByIdx<raw>[VU,K(I,5)]                  ≡ NODE d17  K(I,5) ✓
LOCAL d24 = ByIdx<or>[PC<tokens>[VU],K(I,0),VU]    ≡ NODE d22  K(I,0) ✓ (receiver diff)
LOCAL d25 = BO<==>[ByIdx<raw>[VU,K(I,0)],K(L,0)]   ≡ NODE d24  K(I,0) ✓
LOCAL d26 = ByIdx<or>[PC<tokens>[VU],K(I,1),VU]    ≡ NODE d23  K(I,1) ✓ (receiver diff)
LOCAL d29 = ByIdx<raw>[VU,K(I,2)]                  ≡ NODE d26  K(I,2) ✓
LOCAL d32 = ByIdx<raw>[VU,K(I,4)]                  ≡ NODE d28  K(I,4) ✓
LOCAL d34 = ExScript[ByIdx<raw>[Outputs,K(I,1)]]   * NODE inlines via VU→d29  (extraction diff)
LOCAL d35 = PC<tokens>[ByIdx<raw>[Outputs,K(I,1)]] * NODE inlines via VU→d29  (extraction diff)
LOCAL d38 = ExAmt[ByIdx<raw>[Outputs,K(I,1)]]      * NODE inlines via VU→d29  (extraction diff)
LOCAL d37 = BO<==>[VU,K(I,0)]                      * NODE inlines  (singleton K(I,0) — l.146)
LOCAL d41 = ByIdx<raw>[VU,K(I,3)]                  ≡ NODE d32  K(I,3) ✓
LOCAL d43 = If[BO<==>[ByIdx<raw>[VU,K(I,1)],K(L,0)],…]  ≡ NODE d35  K(I,1) ✓
LOCAL d45 = ByIdx<or>[VU,K(I,0),VU]                ≡ NODE d22/40/45  K(I,0) ✓
LOCAL d48 = BO<==>[VU,K(I,1)]                      * NODE inlines  (singleton K(I,1) — l.140 or l.162)
LOCAL d49 = ByIdx<raw>[Outputs,K(I,2)]             ≡ NODE d43  K(I,2) ✓
LOCAL d52 = ByIdx<or>[VU,K(I,1),VU]                ≡ NODE d23/36  K(I,1) ✓
```

Every K(I,N) value matches between LOCAL and NODE outer ValDefs. **Zero
flipped literals at the outer-ValDef stratum.**

---

## Where the (−1, +1) actually comes from

The deficit decomposes by stratum, not by site:

- **Outer-ValDef stratum**: L − N outer = (0 K(I,0), +3 K(I,1)).
  The +3 K(I,1) comes from LOCAL extracting **three** outer ValDefs whose RHS
  contains an inline `ByIdx<raw>[Outputs,K(I,1)]` (d34, d35, d38). NODE
  extracts `OUTPUTS(1)` itself as its own outer ValDef d29 and references it
  via VU — so the K(I,1) only appears **once** in NODE's outer stratum (in
  d29), not three times.

- **Body-inline stratum**: L − N body = (−1 K(I,0), −2 K(I,1)).
  Because LOCAL extracts more outer ValDefs (53 vs 46), fewer literals remain
  inline in LOCAL's body. NODE's body inlines more K(I,N) literals overall.

Summing: outer (0, +3) + body (−1, −2) = **(−1, +1)** ✓ (matches pool diff).

The deficit is therefore a structural artifact of the OUTPUTS(1) extraction
gap (csym=20 stays rejected at HEAD's S62 (a/b) gate, despite S75's attempt to
admit and S76b's wrap-suppression compensation). There is no single
source-line whose Const swap would close it.

---

## Why no narrow fix per S77 step 2

S77 step 2 specifies "ONE-LINE answer" + ≤10 LOC narrow lowering/extraction
fix at the site. The empirical analysis above shows:

1. **No K(I,0) ↔ K(I,1) flip exists at any single AST position.** Every
   outer-ValDef literal matches NODE. Every body literal that appears in both
   is at the same K value.
2. **The deficit is purely a side-effect of extraction-set differences** —
   LOCAL extracts {d34, d35, d38, d37, d48, d50…} that NODE inlines, and NODE
   extracts {d29 = OUTPUTS(1)} that LOCAL inlines three times.
3. **Closing the deficit requires either** admitting csym=20 (FALSIFIED in
   S75 — over-replacement; FALSIFIED in S76 PASS-3 reject at 1131B), or
   structurally changing the LOCAL extraction set to drop d37/d48/d34/d35/d38
   in favor of OUTPUTS(1) extraction. Neither is a "narrow lowering fix at
   one site"; both are mechanism-layer redesigns (Path C from S75 archive,
   scope-aware `rebuild_v3_walk` refactor).

Per `feedback_no_ship_off_ramp`: the 12-session arc explicitly tested every
mechanism layer (canon 4 sublayers, admit S62/S63/S65, wire S68/S69, per-csym
admit-time S70, deep canon S71, emission preview S73, S75 admit, scope-aware
walker S76/S76b). All mechanism layers exhausted at the same residual.

---

## Honest documentation of the empirical ceiling

**v3 sigmao_option ceiling: 1142B = NODE 1148B − 6B**, with:
- 13 v3 byte-MATCH preserved (NOT regressed)
- 14 HC=0 sacred preserved (NOT regressed)
- pool count = NODE (61 = 61) ✓
- pool multiset Δ = (−1 SInt(0), +1 SInt(1)) (structural, not single-site)
- byte-overlap = 1.3% (S72 ceiling)

The 6B byte gap = pool slot-order permutation × 3 + outer-ValDef encoding
diff × 2 + body-emission slot-shift × 1, none of which is closable without
either (a) a mechanism change that was already FALSIFIED, or (b) a multi-VD
extraction-set rebalance that risks regressing the 13 v3 MATCH fixtures.

---

## Probe 1 outputs

- `/tmp/s_s77_local.txt` — combined LOCAL+NODE verbose outer-VD dump
  (env-gated, runs cleanly via the SIG15_NODE_HEX_PATH addition)
- `/tmp/s_s77_fullir.txt` — full LOCAL IR Debug dump (6457 lines)
- `/tmp/node_full_ir.txt` — cached NODE IR Debug dump (6496 lines, from
  `/tmp/sigmao_v3_full.txt`)
- LOCAL/NODE pool multiset analysis verified by `/tmp/decode_pool.py`
  (constant pool reverse-decoded from raw hex; matches `print_outer_valdef_shape_diff` totals)

---

## Anti-fingerprint outcome

S77 avoids the **78th cumulative falsification fingerprint** by performing
the Probe-1-only mandate verbatim — no new mechanism layer, no "ambitious"
deferral. The probe's output (multi-site compensating, not single-site swap)
matches stop-table row 3, which the handoff itself identified as a valid
outcome. The 12-session arc closes honestly at v3 13/15 + sigmao 1142B
partial-close per `feedback_no_ship_off_ramp`.

---

## Next session

Per parallel-work note in CLOSE-SIGMAO-S77-LOCALIZE-SWAP.md §"Parallel work":
**gluon S37+ is unblocked**. GLUON-EMPIRICAL-MAP §1-9 + GLUON-S37-IMPL-SKETCH
418 LOC sketch + §9e gate signature with per-fixture verdict table are all
staged. The identity stub for `merge_stage12_swap_symmetric_pairs` added in
this commit allows the build to compile without changing byte parity; the
gluon S37 real implementation replaces it.
