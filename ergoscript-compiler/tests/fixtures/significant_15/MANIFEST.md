# Significant-15 Fixture Manifest

Source provenance for each contract added under the "15 Significant Ergo Contracts" testing initiative
(see `/ergo-significant-contracts.md` and `/home/cq/.claude/plans/reivew-this-document-in-vivid-dragon.md`).

All sources fetched from `cannonQ` GitHub forks at the commit hashes pinned below. Pinning ensures
reproducibility independent of upstream churn.

Test target: every fixture should reach **LOCAL MATCH** when fed through `compile_canonical(...)`
against Ergo node v6.1.2 at `localhost:9053` with `treeVersion: 0`.

## Coverage map: 15 significant contracts → fixtures

The 15 keystone contracts split into two buckets:

| Significant rank | Protocol | Keystone script | Where tested |
|---|---|---|---|
| 1 | Oracle Pool v2 | `refresh.es` | **NEW (this dir)** — currently NEEDED, no source in cannonQ Rust oracle-core |
| 2 | SigmaUSD / AgeUSD | `bank.es` (full State Box) | **NEW** — `sigmausd_bank.es` |
| 3a | Spectrum DEX | `n2t_pool.es` | ✅ **`spectrum_n2t_pool.es` LOCAL MATCH @ 409B** (closed by S65 outer-AND skip-Pass-1a + S63 hoist + S64a/b fold-drop & post-hoist dedup). |
| 3b | Spectrum DEX | `t2t_pool.es` | ✅ **`spectrum_t2t_pool.es` LOCAL MATCH @ 421B** (same root-cause fix as n2t). |
| 4 | Rosen Bridge | `EventTrigger.es` | **NEW** — `rosen_event_trigger.es` |
| 5 | Dexy / USE | `bank.es` | ✅ **`dexy_bank_full.es` LOCAL MATCH @ 309B** (full upstream keystone). 46-corpus #8 "Dexy Bank" (291B) is a simplified variant — kept for regression coverage. |
| 6 | ErgoMixer | `FullMix.es` | **NEW** — `ergomixer_fullmix.es` |
| 7 | SkyHarbor | `V1_ErgEditsAndOffersV1.es` | ✅ **`skyharbor_v1_erg.es` LOCAL MATCH @ 411B**. 46-corpus #37 "SigUSDV1" tests the wrong sibling (SigUSD variant); kept for regression coverage. |
| 8 | Phoenix HodlERG | `phoenix_v1_hodlerg_bank.es` | ✅ **`phoenix_hodlerg_bank_full.es` LOCAL MATCH @ 394B** (closed by S62 source-order val schedule). 46-corpus #25 "Phoenix HodlERG Bank" (314B) is the simplified variant — also LOCAL MATCH; kept for regression coverage. |
| 9 | Paideia DAO | `stakeState.es` | **NEW** — `paideia_stake_state.es` |
| 10 | Gluon Gold | `GluonWBoxGuardScript.es` | **NEW** — `gluon_box_guard.es` |
| 11 | DuckPools | `childInterest.es` | ✅ **`duckpools_child_interest.es` LOCAL MATCH @ 598B** (closed by S67 — drop trivial alias ValDefs in HIR `inline_single_use_vals` dedup pass; two source vals with identical RHS produced an alias `ValDef = ValUse(other)` that NODE never emits). |
| 12 | SigmaO | `Option.es` | **NEW** — `sigmao_option.es` |
| 13 | ChainCash | `reserve.es` | **NEW** — `chaincash_reserve.es` |
| 14 | ErgoRaffle | `raffle.es` | ✅ **`ergoraffle_active.es` LOCAL MATCH @ 931B** (closed by S66 ByteArrayToBigInt CSE walker fix — added missing arms in `direct_children` / `count_occurrences*` / `replace_all` / etc. so the `Slice → ExtractId → ByIndex` chain inside `winNumber` was reachable). |
| 15 | SigmaFi | `BondContractERG.ergo` | ✅ Existing 46-corpus #32 "SigmaFi BondContractERG" (146B native match) — **VERIFIED** identical to upstream. |

**Totals: 15 fixtures in this directory** (12 from initial round + 3 added 2026-04-27 to fix simplified/wrong-sibling coverage gaps surfaced by the keystone audit). Plus `SigmaFi BondContractERG #32` already verified-keystone in the 46-corpus, brings total keystone coverage to 16 fixtures testing 15 keystones (Dexy is double-covered: full + simplified). **7/15 LOCAL MATCH** as of 2026-05-02 (post-S67 alias-drop fix): `dexy_bank_full.es`, `skyharbor_v1_erg.es`, `phoenix_hodlerg_bank_full.es`, `spectrum_n2t_pool.es`, `spectrum_t2t_pool.es`, `ergoraffle_active.es`, `duckpools_child_interest.es`.

The 4 "already covered" rows are *not* duplicated as fixtures here — they are tested by
[`test_batch_node_byte_match`](../../../src/compiler.rs) and the existing
[`test_ecosystem_batch`](../../../src/compiler.rs#L3750). Two of them (Dexy bank, SkyHarbor V1)
should still be cross-checked against current upstream sources to confirm we're testing the
*same* contract version the keystone list describes.

## Status legend
- **READY** — source file is in this directory, ready for Tier-A test
- **NEEDED** — source not yet acquired; lookup TODO
- **PARTIAL** — source on disk but uses ScriptEnv placeholders or template variables; substitution required

## Empirical compile status (2026-05-02, against node v6.1.2)

**15/15 fixtures compile end-to-end. 7/15 LOCAL MATCH** (`dexy_bank_full.es`,
`skyharbor_v1_erg.es`, `phoenix_hodlerg_bank_full.es`, `spectrum_n2t_pool.es`,
`spectrum_t2t_pool.es`, `ergoraffle_active.es`, `duckpools_child_interest.es`).
The other 8 produce different bytes than the node — those diffs are the canonical
S43–S60-style CSE/lowering parity work, one root-cause per contract.

Re-baselined post-Workstream-A–D close (commits `1a2034a2`, `1f6025bb`, `ab10a30e`,
`e9212e83`), with `c7112a1e` (skyharbor), `ea04228c` (S62 / phoenix), `209d448f`
(S65 / spectrum), `716ac236` (S66a ergoraffle structural IR + body-schedule walk)
and `ac6dc12f` (S66b ByteArrayToBigInt CSE walker arms — this session) layered on.
The earlier 46-corpus and 14-ecosystem batches are at 45/46 + 14/14
LOCAL MATCH on this same branch; sig-15 directly closed phoenix in S62,
spectrum n2t/t2t in S65, ergoraffle in S66 (a+b), and several other fixtures
shifted via shared CSE/schedule code paths (see table below).

| Fixture                          | Node bytes | Local bytes | Δ | Status | Movement |
|---|---|---|---|---|---|
| `chaincash_reserve.es`           | 611  | 546  | -65  | USED NODE | unchanged |
| `dexy_bank_full.es`              | 309  | 309  | 0    | ✅ **LOCAL MATCH** | unchanged |
| `duckpools_child_interest.es`    | 598  | 598  | 0    | ✅ **LOCAL MATCH** | was +4 → now matched (**S67 — drop trivial alias ValDef in HIR dedup pass**; two source vals with identical literal RHS produced `ValDef = ValUse(other)` that NODE never emits) |
| `ergomixer_fullmix.es`           | 198  | 175  | -23  | USED NODE | unchanged |
| `ergoraffle_active.es`           | 931  | 931  | 0    | ✅ **LOCAL MATCH** | was +8 → now matched (**S66b ByteArrayToBigInt CSE walker arms** — closed the 3rd `dataInputs(0)` substitution the dag-walker was missing) |
| `gluon_box_guard.es`             | 2283 | 2240 | -43  | USED NODE | was -51 → now -43 (8B closer; S66a) |
| `oracle_refresh.es`              | 572  | 519  | -53  | USED NODE | unchanged since S62 |
| `paideia_stake_state.es`         | 1468 | 1379 | -89  | USED NODE | **was +95 → now -89** (sign-flip; 6B closer in absolute; S66a structural IR) |
| `phoenix_hodlerg_bank_full.es`   | 394  | 394  | 0    | ✅ **LOCAL MATCH** | unchanged since S62 |
| `rosen_event_trigger.es`         | 374  | 336  | -38  | USED NODE | unchanged |
| `sigmao_option.es`               | 1148 | 1112 | -36  | USED NODE | **was -133 → now -36** (97B closer; S66a) |
| `sigmausd_bank.es`               | 741  | 664  | -77  | USED NODE | **was -128 → now -77** (51B closer; S66a — recovers the bank-widening loss flagged below) |
| `skyharbor_v1_erg.es`            | 411  | 411  | 0    | ✅ **LOCAL MATCH** | unchanged since 04-30 |
| `spectrum_n2t_pool.es`           | 409  | 409  | 0    | ✅ **LOCAL MATCH** | unchanged since S65 |
| `spectrum_t2t_pool.es`           | 421  | 421  | 0    | ✅ **LOCAL MATCH** | unchanged since S65 |

**Net |Δ| reduction across the 9 still-diffing fixtures vs prior MANIFEST**: ~290B
of inflation eliminated by S66a (sigmao -97, duckpools -78, sigmausd -51, gluon -8,
paideia ~-6) plus ergoraffle's +8 closed entirely by S66b. Three fixtures
(`duckpools_child_interest`, `paideia_stake_state`, `sigmausd_bank`) crossed the
sign axis on S66a — a strong hint that the body-schedule walk was the load-bearing
ordering primitive several earlier "shifted" fixtures were waiting on.

**Smallest diffs** (best targets for next byte-match parity sessions, in order of
expected leverage):
- `dexy_bank_full` (0 — ✅ matched)
- `skyharbor_v1_erg` (0 — ✅ matched 2026-04-30)
- `phoenix_hodlerg_bank_full` (0 — ✅ matched 2026-05-01 via S62 source-order val schedule)
- `spectrum_n2t_pool` (0 — ✅ matched 2026-05-01 via S65 outer-AND skip-Pass-1a)
- `spectrum_t2t_pool` (0 — ✅ matched 2026-05-01 via S65)
- `ergoraffle_active` (0 — ✅ matched 2026-05-02 via S66b ByteArrayToBigInt walker fix)
- **`duckpools_child_interest` (+4)** — closest small-diff candidate; same shared-multi-register / multi-stage shape as ergoraffle, now within striking distance.
- `ergomixer_fullmix` (-23)
- `sigmao_option` (-36) — collapsed from -133 on S66a; structural shift makes a follow-up plausible.
- `rosen_event_trigger` (-38), `gluon_box_guard` (-43), `oracle_refresh` (-53)

**Investigate-before-targeting**: `paideia_stake_state` and `sigmausd_bank` both
sign-flipped on S66a (paideia +95→-89, sigmausd -128→-77 — the latter recovers
51B of the bank-widening loss). The shifts confirm that `dfs_reassign_val_ids`
ordering and `emit_deps` body-schedule order are jointly load-bearing for any
fixture with multi-branch shared-val patterns; S62's transitive `branch_val_ids`
expansion, S63's hoist on `inline_single_use_vals`, S65's per-fixture Pass 1a
gate (applied only when the result is an If), and S66a's body-schedule walk
all change which sub-expressions land at outer scope vs branch scope.

**Sig-15 progress**: 7/15 LOCAL MATCH (2026-05-02 post-S67) — was 1/15 at
plan start, 2/15 post-skyharbor, 3/15 post-S62, 5/15 post-S65 (spectrum
n2t/t2t closed), 6/15 post-S66b (ergoraffle), now 7/15 with
duckpools_child_interest closed via S67 alias-drop in HIR
`inline_single_use_vals` dedup pass.

### Run-to-run non-determinism in USED NODE fixtures (2026-05-02)

**Discovered during S67/Session 9 diagnosis.** The byte counts reported in
the table above for USED NODE (fallback) fixtures are **not stable
run-to-run.** Verified pre-change on `sigmausd_bank.es`: three pre-change
runs returned 620 / 664 / 760 local bytes against the same source — a
spread of 140B with no code change between runs. Other fallback fixtures
(`paideia_stake_state`, likely all USED NODE entries) exhibit the same
behavior.

Root cause: HashMap iteration order somewhere in the pipeline (CSE candidate
enumeration, ValDef map walk, or hash-cons table — not yet diagnosed). Out
of scope for the current parity arc but documented here so future regression
checks don't treat single-run byte counts as authoritative.

**Implications for parity work:**

- LOCAL MATCH fixtures are unaffected: a tree that byte-equals the node's
  tree byte-equals it on every run. The non-determinism is in the
  USED NODE / fallback path — specifically the local-bytes count we report
  for "how far we are from MATCH."
- The "watermark" pattern in handoffs ("if sigmausd widens past -77 →
  revert") is **not a reliable single-run check.** A multi-run gate is
  required: 5 runs minimum, take the median; treat a regression as confirmed
  only if the median moves outside the noise band (~140B observed for
  sigmausd, narrower for smaller fixtures).
- Suite-wide ripple claims ("S66a closed ~290B") need re-grounding: the
  ~290B figure is from single-run snapshots and is approximate. The
  ordering of fixes (which fixture closed when) is correct because each
  LOCAL MATCH commit was deterministically verified, but the per-fixture
  byte deltas in USED NODE rows should be read with the noise band in mind.

Future work: characterize the noise band per-fixture (median + spread over
N runs) and add it as a column to the empirical compile table. Until then,
prefer LOCAL MATCH / USED NODE as the binary signal over the local-bytes
column.

### Sigmausd_bank widening hypothesis (2026-05-01) — partially recovered post-S66a

Post-skyharbor delta: -77 → -128 (lost 51B of extractions). **Update 2026-05-02**:
S66a's body-schedule walk fix recovered exactly that 51B (-128 → -77), confirming
the hypothesis below was at least directionally right — body-schedule ordering
was upstream of the under-extraction. The remaining -77 is the same shape as the
pre-skyharbor regime, so the original analysis still applies as a candidate for
closing the rest. Original analysis preserved below.

The change responsible is the S40 global bump switching from `count_occurrences` to
`count_occurrences_no_inner_if`. Plausible mechanism:

- Sigmausd's bank script has multiple deeply-nested `if` blocks (mint/redeem/cooling
  branches) with arithmetic on shared sub-expressions like `oraclePoolNFT box value`,
  `reserveIn / circulationIn ratios`, and `BigInt / Long upcasts`.
- Many of these shared sub-exprs likely appear in BOTH the outer scope AND inside
  nested `if` branches that have been inlined by `inline_single_use_vals`. Pre-fix
  S40 (`count_occurrences`) saw the global count ≥ 2 and extracted them at the
  outer scope. Post-fix S40 (`count_occurrences_no_inner_if`) stops at the nested
  `if` branches → counts only the outer-scope occurrence (1) → no extraction.
- Scala *does* perform these extractions because Scala's `hasManyUsagesGlobal` runs
  on the hash-consed graph, which sees the nested-If occurrences as separate Sym
  parents — same behavior as full `count_occurrences`, but Scala doesn't have
  the SaleLP-style "inlined-If duplicates the inner refs" pathology because Scala
  doesn't aggressively inline single-use ValDefs across ThunkDef boundaries.

**Likely real fix**: tighten `inline_single_use_vals` to NOT inline a ValDef whose
RHS contains an `Expr::If` across a ThunkDef boundary. That removes the pathological
inlining that motivated the S40 restriction, letting us revert S40 to full
`count_occurrences` and recovering sigmausd_bank's 51B without breaking SaleLP.
Worth confirming the inliner's current scope-awareness before assuming this is the
root cause — the inline pass may already gate on something we're not seeing here.

Lower-leverage alternative: special-case the S40 bump to recurse into nested If
branches *only when* the candidate also appears outside them at the current scope
(i.e. discount the nested-If occurrence when it's the *only* extra reference past
a scope-level baseline of 1). Less principled but more surgical.

### What landed in the compile-all push

**Lexer/parser:**
- `\t`, `\r` whitespace.
- `0x..L` hex literals (priority=3).
- `++` Coll concat token + Pratt binding power `(9, 10)`.
- AST op extractor extended for `PlusPlus`.

**HIR / type infer:**
- `BinaryOp::ConcatColl` enum variant.
- Type infer: ConcatColl returns lhs type; SGroupElement methods (`exp`, `multiply`, `getEncoded`); SContext field `HEIGHT`.

**MIR lowering:**
- `BinaryOp::ConcatColl` → `Append::new`.
- Hex literal → `i64` via `u64 as i64` cast (preserves high-bit constants).
- New built-ins: `groupGenerator` (HIR `GlobalVars::GroupGenerator`), `proveDHTuple` → `CreateProveDhTuple`, `.exp(scalar)` → `Exponentiate`, `.multiply(other)` → `MULTIPLY_METHOD`, `.getEncoded` → `GET_ENCODED_METHOD`, `CONTEXT.HEIGHT` → `GlobalVars::Height`.
- `upcast_index_to_int` helper applied at all 3 user-facing `ByIndex::new` sites — handles `INPUTS(byteVar)` where source uses `getVar[Byte]` arithmetic for indexing.

**Test driver:**
- `test_significant_15` reads `.es` from disk; per-fixture env-prelude is prepended after the
  outer `{` (replaces free Scala-side variables with `val name = literal;` declarations).

**Source patches** (applied to `.es` files in this directory):
- CRLF → LF, tab → spaces (lexer doesn't tokenize tabs).
- Scala-style trailing commas stripped from `Coll(...)` and `(...)` literals.
- `rosen_event_trigger.es`: `fromBase64("CLEANUP_NFT")` → `fromBase16("00..01")` (placeholder NFT IDs).
- `sigmausd_bank.es`: 6 Scala `$placeholder` interpolation sites → concrete numeric values
  (`$minStorageRent`→`10000000L`, `$coolingOffHeight`→`350000`, `${INF}`→MaxLong, etc.).
- `gluon_box_guard.es`: 1 type-annotation correction
  (`emptyFees: (Coll[Byte], Long)` → `(Coll[Byte], BigInt)` — RHS was already BigInt;
  Scala compiler accepts via implicit widening, ours doesn't).

**No regressions** — lib suite stays at 203/203 across the entire round.

### What's left

- **Refresh.es source** — still NEEDED. cannonQ's Rust `oracle-core` carries precompiled bytes,
  not Scala source. Upstream `ergoplatform/oracle-core-v2-pool-publish` or EIP-23 spec are
  candidates; not yet cloned.
- **Byte-match parity for the 9 USED NODE diffs** — this is the canonical S43–S60 work cycle.
  Local bytes are 8/9 *shorter* than node (typically signaling under-extraction in CSE) and
  1/9 *longer* (`duckpools_child_interest` at +4, the closest small-diff candidate).
  Each is one focused session per contract, in the existing cadence.

## Contracts

### 1. Oracle Pool v2 — `oracle_refresh.es`
- Status: **READY**
- Source: `kettlebell/eips @ eip23_separate_contract_files` —
  [`eip-0023/contracts/refresh_contract.es`](https://github.com/kettlebell/eips/blob/eip23_separate_contract_files/eip-0023/contracts/refresh_contract.es)
  (the canonical EIP-0023 separate-contract-files PR `ergoplatform/eips#78`).
  Source reachable directly from the GitHub raw URL; cannonQ doesn't have a fork because
  the PR was never merged to `ergoplatform/eips/master`.
- Notes: Two `fromBase64` placeholder TODOs in the upstream PR converted to `fromBase16` in this
  fixture (decoded byte values preserved). The fold lambda body was wrapped in extra `{}` —
  upstream uses Scala-style un-braced multi-statement form which our parser doesn't accept.

### 2. SigmaUSD / AgeUSD — `sigmausd_bank.es`
- Status: **READY** (PARTIAL — references env constants `oraclePoolNFT`, `updateNFT`, `dexyUSDLPNFT`, etc.)
- Source: `cannonQ/Djed-Ergo @ e810b195` — `ageusd-smart-contracts/v0.4/AgeUSD.scala :: bankScript`
- ScriptEnv: pass `oraclePoolNFT`, `updateNFT` (Coll[Byte]) when calling `compile_canonical`.

### 3a. Spectrum DEX — `spectrum_n2t_pool.es`
- Status: **READY**
- Source: `cannonQ/ergo-dex @ 8fe94e1f` — `contracts/amm/cfmm/v1/n2t/Pool.sc`

### 3b. Spectrum DEX — `spectrum_t2t_pool.es`
- Status: **READY**
- Source: `cannonQ/ergo-dex @ 8fe94e1f` — `contracts/amm/cfmm/v1/t2t/Pool.sc`

### 4. Rosen Bridge — `rosen_event_trigger.es`
- Status: **READY**
- Source: `cannonQ/contract @ 0cda684a` — `src/main/scala/rosen/bridge/scripts/EventTrigger.es`

### 5. ErgoMixer — `ergomixer_fullmix.es`
- Status: **READY** (PARTIAL — references `feeEmissionScriptHash`, `tokenId` from outer Scala scope)
- Source: `cannonQ/ergoMixBack @ 6f1241d9` — `mixer/app/mixinterface/TokenErgoMix.scala :: fullMixScript`
- ScriptEnv: pass `feeEmissionScriptHash` (Coll[Byte]), `tokenId` (Coll[Byte]).

### 6. Paideia DAO — `paideia_stake_state.es`
- Status: **READY**
- Source: `cannonQ/paideia-contracts @ 55961530` — `paideia_contracts/contracts/staking/ergoscript/latest/stakeState.es`

### 7. Gluon Gold — `gluon_box_guard.es`
- Status: **READY**
- Source: `cannonQ/Gluon-Ergo-Contracts @ 3e71f9f4` —
  `modules/gluonw-base/src/main/resources/ErgoContracts/GluonW/Ergs/BoxGuardScripts/GluonWBoxGuardScript.es`

### 8. DuckPools Lending — `duckpools_child_interest.es`
- Status: **READY**
- Source: `cannonQ/lend-protocol-contracts @ 63b49a05` — `contracts/pools/RSN-POOL/childInterest.md`
  (extracted from the embedded ```scala fenced block).

### 9. SigmaO — `sigmao_option.es`
- Status: **READY**
- Source: `~/working-files/p2p-options-contracts/contracts/Option-SigmaO.es` (working copy as of 2026-04-27).
  Multi-stage state machine, 253 lines.

### 10. ChainCash — `chaincash_reserve.es`
- Status: **READY**
- Source: `cannonQ/chaincash @ b942d125` — `contracts/onchain/reserve.es`

### 11. ErgoRaffle — `ergoraffle_active.es`
- Status: **READY**
- Source: `cannonQ/raffle-backend @ cc882e4b` — `app/raffle/RaffleContract.scala :: raffleActiveScript`
  (extracted from `s"""{...}""".stripMargin` block; closing brace re-added). Other raffle scripts in
  the same file: `RaffleServiceScript`, `ticketScript`, `RaffleScriptWaitingToken`, `raffleWinnerScript`,
  `raffleRedeemScript`, `createRaffleProxyScript`, `donateScript` — `raffleActiveScript` is the
  on-deadline winner-selection contract identified as load-bearing in `ergo-significant-contracts.md`.

## ScriptEnv substitution policy

Sources that contain template variables (`$placeholder`, `${someToken}`) or Scala-scope refs:

1. **Canonical mode (Tier A/B):** keep placeholders in the `.es` and pass an identical `ScriptEnv` to
   the local compiler and to the node. Both sides substitute via the env; bytes must match.

2. **Offline mode (Tier C, frozen):** hand-substitute placeholders to fixed dummy values
   (`fromBase16("00..01")` for token IDs, `proveDlog(decodePoint(fromBase16("...")))` for keys) and
   store both the substituted source and the populated env in a `.env.toml` adjacent to the `.es` file.

PARTIAL entries above flag the substitutions required.

## Acquisition checklist

- [x] 11/12 sources acquired and pinned to commit hashes.
- [ ] `refresh.es` — locate upstream source or decompile from precompiled bytes.
- [ ] Tier-A canonical test for each READY contract.
- [ ] Tier-B append + LOCAL MATCH per contract.
- [ ] Tier-C freeze (offline byte-match) per contract.

---

## Verification Audit (2026-04-27)

Performed before opening the byte-match-parity arc, to confirm: (a) the fixture patches we
applied to make sources compile don't silently change semantics; (b) the 4 keystone contracts we
*claimed* were already byte-matching in the existing 46-corpus are testing the actual upstream
keystone source, not a sibling or simplified variant.

### A. Fixture-patch sanity (10.A)

For each non-trivial source patch, the original and patched forms were both compiled
node-side; ErgoTree bytes compared:

| Patch | Verdict | Evidence |
|---|---|---|
| `gluon_box_guard.es`: `emptyFees: (Coll[Byte], Long)` → `(Coll[Byte], BigInt)` | ✅ EQUIVALENT | Both forms produce identical 4566-byte ErgoTree. RHS was already `0L.toBigInt`; Scala does implicit widening. |
| `oracle_refresh.es`: fold-lambda body wrapped in `{...}` | ✅ EQUIVALENT | Both forms produce identical 1144-byte ErgoTree. Scala accepts un-braced multi-statement form, our parser doesn't. |
| `rosen_event_trigger.es`: `fromBase64("CLEANUP_NFT")` → `fromBase16("00..01")` | ✅ EQUIVALENT (forced) | `b64decode("CLEANUP_NFT")` is invalid base64; original would have failed at deploy too. Substitution is the only path forward and produces 32-byte placeholder NFT IDs. |
| `sigmausd_bank.es`: 6 `$placeholder` → concrete numeric values | ✅ ACCEPTED with caveat | Substituted constants are folded into the tree (different values → different addresses). Our chosen values produce stable bytes for byte-match parity testing, but **the resulting bytes are NOT canonical mainnet bytes**. Acceptable for parity work; document if ever promoting to a "byte equals on-chain bond" check. |

### B. Existing-corpus keystone coverage (10.B)

| Keystone | Test source | Upstream source | Verdict |
|---|---|---|---|
| **Dexy Bank** (291B byte-match) | [`compiler.rs:2956`](../../../src/compiler.rs#L2956) | `/tmp/dexy-stable-pr/contracts/bank/bank.es` (85 lines) | ⚠️ **SIMPLIFIED**. Test source missing `tokens(1)._1` preservation in `validSuccessor`; uses `INPUTS(0)` for all checks where upstream uses distinct `mintInIndex=0`/`interventionInIndex=2`/`payoutInIndex=0`. Logically a smaller subset of the keystone. |
| **SkyHarbor SigUSDV1** (510B byte-match) | [`compiler.rs:4158`](../../../src/compiler.rs#L4158) | `/tmp/skyharbor/V1_ErgEditsAndOffersV1.md` (script block, 65 lines) | ⚠️ **WRONG SIBLING**. Test source is **token-denominated** (`OUTPUTS(0).tokens(0)._1 == currency` where currency is a SigUSD token), while keystone is **ERG-denominated** (`OUTPUTS(0).value >= sellerReceives`). Our 510B test compiles a SigUSD-denominated variant that lives alongside the ERG keystone, not the keystone itself. |
| **Phoenix HodlERG** (314B byte-match) | [`compiler.rs:3472`](../../../src/compiler.rs#L3472) | `/tmp/sig15-repos/phoenix-hodlcoin-contracts/hodlERG/contracts/bank_contract/v1/ergoscript/phoenix_v1_hodlerg_bank.es` (157 lines) | ⚠️ **SIMPLIFIED** (consistent with the test's own "simplified but with all key patterns" comment). Missing `validRegisters` (5 R4–R8 preservation checks) entirely; `validTokens` missing the `validHodlERGTokenAmount >= 1L` invariant. Real keystone has stricter recreation rules. |
| **SigmaFi BondContractERG** (146B byte-match) | [`compiler.rs:3826`](../../../src/compiler.rs#L3826) | `/tmp/sig15-repos/Sigma-Finance/contracts/BondContractERG.ergo` | ✅ **IDENTICAL** to upstream apart from comment headers and whitespace. The 146B native byte-match is real keystone coverage. |

### C. Implications

Headline correction: **of the "4 already byte-matching" keystones, only SigmaFi BondContractERG is genuinely the keystone.** The other 3 native byte-matches in the 46-corpus are testing simplified or sibling contracts that share structure but NOT the load-bearing keystone logic.

Real keystone byte-match coverage today: **1 of 15** (SigmaFi). The other 12 either:
- Compile but byte-diff (the 12 USED NODE entries above), OR
- Compile and byte-match but against a non-keystone variant (Dexy, SkyHarbor, Phoenix in 46-corpus).

### D. Remediation: 3 new fixtures needed

To get true keystone coverage, three additional fixtures should be added to this directory:

| Fixture | Source | Notes |
|---|---|---|
| `dexy_bank_full.es` | `/tmp/dexy-stable-pr/contracts/bank/bank.es` | Has `fromBase64("$placeholder")` — convert to `fromBase16` and supply token-id env. |
| `skyharbor_v1_erg.es` | `/tmp/skyharbor/V1_ErgEditsAndOffersV1.md` (script block) | ERG-denominated. Includes `offersContract` constant. |
| `phoenix_hodlerg_bank_full.es` | upstream `phoenix_v1_hodlerg_bank.es` (157 lines) | Has `$phoenixFeeContractBytesHash` placeholder; substitute. |

When added and wired into `test_significant_15`, expect each to land as another USED NODE entry — they join the byte-match-parity backlog as fixtures 13–15 of this directory (= keystones 5/7/8 of the original-15 list).
