# S39 K-noise candidates — Probe 1 byte-encoding + pool-multiset FALSIFICATION

**HEAD:** `58da9b9a` (S38 diag; sister falsification arc continues).

**Session position:** S39 picks pivot #1 from S38 §7 (K-noise removal, expected +10–15B yield per handoff). Probe 1 falsifies the yield premise via two empirical anchors:

1. The 5 candidate K outer ValDefs are NOT single-use; they are multi-use (5–7 uses each).
2. Per-ValDef inlining is byte-NEGATIVE in segregated mode because LOCAL and NODE constant pools do **not** deduplicate — NODE inlines K and pays N pool entries per K value, while LOCAL extracts K and pays 1 pool entry + 1 ValDef wrapper.

---

## §1 — Probe 0: Scala authority

`processAstGraph` in `sigma.compiler.ir.TreeBuilding` (scJVM) gates ValDef
creation with:

```scala
if (mainG.hasManyUsagesGlobal(s)
  && IsContextProperty.unapply(d).isEmpty
  && IsInternalDef.unapply(d).isEmpty
  && IsConstantDef.unapply(d).isEmpty)  // _: Const[_] -> Some
{ build ValDef }
```

Inline comment: *"to increase effect of constant segregation we need to treat
the constants specially and don't create ValDef even if the constant is used
more than one time, because two equal constants don't always have the same
meaning."*

So Scala unconditionally rejects bare `Const(_)` from ValDef creation
regardless of use count. The handoff cited this as the basis for the "+10–15B
K-noise yield." **The rule is correct but the yield direction is not what the
handoff assumed.** See §3.

---

## §2 — Probe 1: empirical 5-candidate audit (cached `g_local_s38.txt`)

LOCAL outer ValDefs for `gluon_box_guard` at HEAD (5 bare K's):

| ValDef | RHS shape | value | use count |
|---|---|---|---|
| d22 | `K(I,200)` | 200 (SInt) | 5 |
| d27 | `K(I,720)` | 720 (SInt) | 5 |
| d31 | `K(L,1000000)` | 1000000 (SLong) | 7 |
| d32 | `K(I,1000)` | 1000 (SInt) | 5 |
| d33 | `K(I,0)` | 0 (SInt) | 5 |

Use counts derived from the AST dump (`grep -A2 "ValUse {" /tmp/g_local_s38.txt
| awk '...val_id...' | uniq -c`). Handoff §S39 step 1 description ("single-use")
is FALSIFIED — all 5 are multi-use (≥5).

NODE outer ValDefs: ZERO with shape `K(_)`. Multiset diff confirms
`Δ L=4 N=0 K(Int)` and `Δ L=1 N=0 K(Long)`.

---

## §3 — Probe 1 byte-encoding math (segregated, no pool dedup)

Per-ValDef byte cost in segregated mode (header byte 0x10):

```
LOCAL extract (RHS=Const(SInt|SLong) of varint size V, N uses):
  pool entry      : 1B (type tag) + V (varint value)
  ValDef wrapper  : 1B (tag) + 1B (id varint) + 2B (Placeholder ref)  = 4B
  N ValUse refs   : N × 2B
  TOTAL           : (1 + V) + 4 + 2N  =  5 + V + 2N

NODE inline (no ValDef, N independent Const visits to ConstantStore):
  N pool entries  : N × (1 + V)
  N PH refs       : N × 2B
  TOTAL           : N(1 + V) + 2N  =  3N + NV

Δ(inline - extract) = (3N + NV) - (5 + V + 2N) = (N-1)V + (N - 5)
```

For each candidate:

| d# | V | N | LOCAL | NODE-inline | Δ (regression if inlined) |
|---|---|---|---|---|---|
| d22 | 2 | 5 | 17 | 25 | **+8** |
| d27 | 2 | 5 | 17 | 25 | **+8** |
| d31 | 3 | 7 | 23 | 42 | **+19** |
| d32 | 2 | 5 | 17 | 25 | **+8** |
| d33 | 1 | 5 | 17 | 20 | **+3** |
| Σ |  |  | **91** | **137** | **+46B regression** |

Inlining all 5 K ValDefs would push gluon `2343 → 2389B`, diverging FURTHER from
NODE 2283 (Δ +60 → +106). The handoff's "+10–15B yield" was derived from a
single-use formula but the candidates are multi-use, and segregated-pool
arithmetic without dedup INVERTS the savings sign.

---

## §4 — Probe 1 pool-multiset empirical anchor

NODE `gluon` constant pool entries (sample top values):

| value | NODE count | LOCAL count |
|---|---|---|
| 720: SInt | **6** (no dedup; one per inline use) | 1 (single ValDef body) |
| 1000000: SLong | **4** | 2 |
| 0: SInt | 24 | 25 |
| 14: SInt | 20 | 22 |

The 6× `720:SInt` in NODE is decisive: NODE's emission visits each shared
Const sym at every use site and `ConstantStore.put` appends a fresh pool entry
each time (per Scala `s.put(constant)` semantics in `buildValue`). No
compile-time dedup. LOCAL gets 6× compressed to 1 ValDef + 5 ValUse refs.

---

## §5 — Cross-fixture audit

`probe_sig15_local_hex` + `SIG15_DUMP_OUTER_SHAPES_VERBOSE=1` shows bare K
outer ValDefs in 5 of 15 fixtures (HC=0):

| Fixture | bare K outer ValDefs | Current status |
|---|---|---|
| gluon_box_guard | 5 (target) | HC=0=v3=2343, Δ +60 |
| duckpools_child_interest | 1 (K(L,100M)) | v3 598 byte-MATCH ✓ |
| ergomixer_fullmix | 1 (K(Coll[Byte])) | v3-MATCH ✓ |
| ergoraffle_active | 1 (K(Coll[Byte])) | v3-MATCH ✓ |
| paideia_stake_state | 2 (K(Coll[Byte]) ×2) | 1468 sacred |
| 10 others | 0 | clean |

The 4 sacred fixtures with bare K outer ValDefs would also regress by
analogous per-ValDef arithmetic. A blanket rejection of bare-Const extraction
is byte-NEGATIVE in 5 of 5 sites — no narrow per-D variant rescues yield.

---

## §6 — Pipeline-layer probe (empirical null)

Probe 2 site #1 — narrowing `pure_const_root_ok` in
`process_ast_graph_hash_cons` to exclude bare `Const(_)` / `ConstPlaceholder(_)`
admission (mirror Scala's `IsConstantDef` rule). Env-gated
`CSE_PROBE_S39_REJECT_BARE_CONST=1` toggle, ran preflight: ZERO byte movement
across all 15 fixtures. The K outer ValDefs do NOT originate from this
admission path. They survive from upstream MIR lowering (source `val
BLOCKS_PER_VOLUME_BUCKET: Int = 720` at `gluon_box_guard.es:104` lowers to a
top-level ValDef) and pass through `inline_single_use_vals` because their use
counts exceed 1.

Independently, `is_extractable` at `mir/cse.rs:7283` already rejects
`Const(SInt|SLong|...)` (admits only `Const(SBigInt)` per S54). So the
extraction is NOT happening in HC=0's intern-walk-driven candidate iteration
either. Confirmation: the K ValDefs are USER-DECLARED in source, surviving
HIR→MIR lowering, and CSE has no incentive to inline them per §3 byte
arithmetic.

Probe site #1 (`pure_const_root_ok` narrow) reverted — gate would be wrong
layer AND yield-NEGATIVE even at the right layer.

---

## §7 — Stop table verdict

Per `CLOSE-GLUON-S39-K-NOISE.md` stop conditions:

> | All 5 candidates falsify (cross-fixture or byte-negative) | diag — K-noise also exhausted | pivot to d12 hoists OR pool layout (#2/#3 in S38 §7) |

S39 lands in this row. Falsification class: **byte-negative-multi-use-pool-arithmetic**
(extension of S38 §1 single-use formula to multi-use + segregation/no-dedup
pool semantics).

---

## §8 — 79th cumulative falsification fingerprint AVOIDED

S39 is the 79th opportunity for a multi-session speculative arc. Avoided
via Probe 1 byte-encoding math + pool-multiset empirical evidence BEFORE
implementation cycle. Sister falsifications on the same anti-pattern axis:
- S38 (78th) — atom-level hoist byte-math falsification before code change.
- S78 sigmao (79th candidate counterpart) — multiset-undercount falsification at structurally analogous layer.
- S77 sigmao (77th) — multiset-direction inversion via Probe 1.
- S60 paideia (closure precedent) — bytes-first probe over theory-stalled multi-session arc.

---

## §9 — S40+ pivot recommendation

Per S38 §7 ranking AND the empirical falsification of K-noise:

1. **d12-internal atom hoist at positions 5/6/7** — S38 §7 #2.
   - X=7/4/4 inline cost with N=2+ via shared atoms cross-scope.
   - Byte-POSITIVE per Δ=X·(N-1)−2·(N+1) formula: e.g. `BO<==>[VU,OGet[ExReg<R4>[VU]]]` (X≈7) N=2 saves +2B.
   - Verify N≥2 use count empirically per atom before implementation.
   - Implementation surface: medium (in-place rewrite of d12 RHS during
     post-CSE pass, scope-limited by `is_d12_internal` check).
   - Expected close: ~+10–14B.

2. **PC<tokens>[VU] hoist** — S38 §7 #3.
   - At d4/d6 ByIdx args. Expected close: ~+2–4B. Small surface, small yield.

3. **Pool-layout reorder** (deep) — sigma_byte_writer LOCAL pool ordering vs
   NODE. The 6-entry pool-size delta (LOCAL 108 vs NODE 114) is mostly
   "NODE has more entries because NODE inlines K everywhere"; LOCAL's pool is
   structurally smaller. No yield from pool reorder per se.

4. **Accept +60B residual** as architectural plateau for gluon at v3=HC=0=2343B.
   Per `feedback_no_ship_off_ramp.md` — only after empirical exhaustion of (1).

---

## §10 — Artifacts

- Cached: `/tmp/g_local_s38.txt` (S38 LOCAL+NODE pool + outer-VD verbose dump).
- New: `/tmp/s39_hc0_outers.txt` (HC=0 preflight verbose dump, this session).
- New: `/tmp/s39_v3_outers.txt` (v3 preflight verbose dump, this session).
- No code change (probe site #1 reverted as empirically null + theoretically wrong-direction).
