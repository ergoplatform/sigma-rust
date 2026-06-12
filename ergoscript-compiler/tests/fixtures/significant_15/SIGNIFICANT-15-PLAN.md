# Significant-15 Byte-Match Parity Plan (post-WS-A–D)

**Branch:** `ergoscript-compiler-v2`
**Re-baselined:** 2026-04-30 against node v6.1.2 @ `localhost:9053`
**Starting state:** 1/15 LOCAL MATCH (`dexy_bank_full.es`), 14 USED NODE

## Approach (one root-cause per session)

Each fixture is one focused session in the established S43–S60 cadence:
1. Diff local vs node ErgoTree at the IR level (decode both, compare structure).
2. Identify the single root-cause (most common: CSE extraction order, inner-block
   const dedup, BigInt type propagation, ValDef ordering in ThunkDef scopes,
   Upcast placement).
3. Fix in `compiler.rs` / `cse.rs` / lowering passes.
4. Re-run **full test suite** after every edit (CSE tweaks silently break other
   contracts — this is in MEMORY for a reason).
5. Commit with `feat(ergoscript-compiler): <fixture> byte-match — <root cause>`.

## Session order (smallest diff → largest leverage first)

The smallest-diff fixtures are usually one-line CSE ordering or Upcast-placement
fixes; landing them first keeps momentum and validates the diagnosis flow.

| Order | Fixture | Δ | Hypothesis |
|---|---|---|---|
| 1 | `skyharbor_v1_erg.es` | -1 | One missed inner-block constant — likely a single byte literal not deduped. |
| 2 | `phoenix_hodlerg_bank_full.es` | +2 | Already had a parity fix (commit `1a2034a2`) for the simplified variant; full variant likely needs the same BigInt propagation in one more spot. |
| 3 | `spectrum_n2t_pool.es` | +2 | DEX pools historically share a CSE pattern — likely identical fix to t2t. |
| 4 | `spectrum_t2t_pool.es` | +2 | See above; pair with n2t. |
| 5 | `ergoraffle_active.es` | +7 | Slightly larger; multiple register reads, candidate for ValDef ordering tweak. |
| 6 | `ergomixer_fullmix.es` | -23 | Under-extraction (local *shorter*). Mixer uses `getEncoded` heavily — CSE may be inlining a sigma-expression that node extracts. |

## Pause-and-investigate before targeting (3 fixtures)

These shifted post-WS-A–D. Don't treat as fresh root-causes until we know
*what changed*:

- `oracle_refresh.es` — was +2, now -53.
- `paideia_stake_state.es` — was -67, now +97 (sign flip).
- `sigmausd_bank.es` — was +17, now -77 (sign flip).

**Investigation step:** for each, regenerate the Apr-27 local bytes (git checkout
the pre-WS-D commit, run with same prelude, capture local tree), diff the IR
against current local tree. The delta tells us which WS-A–D pass changed
extraction. Often this reveals a bug *introduced* by the conformance work that
the 46-corpus/14-ecosystem suites didn't catch because no fixture there
exercises the same shape.

## Larger-diff backlog (defer until smaller fixes land)

| Fixture | Δ | Notes |
|---|---|---|
| `rosen_event_trigger.es` | -38 | Multi-stage signature-validation contract; under-extraction. |
| `chaincash_reserve.es` | -65 | Reserve/payout state machine; under-extraction. |
| `duckpools_child_interest.es` | -82 | Lending-pool interest; same family as DuckPools #39 (CSE stack overflow) — watch for depth issues. |
| `gluon_box_guard.es` | -90 | 2283-byte contract, 20+ env vars; biggest single fixture, save for last. |
| `sigmao_option.es` | -133 | Largest under-extraction; complex multi-stage option contract. |

## Out-of-scope for this arc

- **AVL IR fix (§12a)** — flagged in MEMORY as near-term but doesn't block any
  sig-15 fixture (none use runtime-`SOption` `avlTree`). Track separately.
- **Un-braced lambda body grammar** — Workstream C residual, QoL only.
- **Lexer/parser snapshot tests** — Workstream D residual.

## Test commands

```bash
# Full sig-15 batch (node required)
source ~/.secrets && cargo test -p ergoscript-compiler test_significant_15 -- --ignored --nocapture

# Single fixture
SIG15_FILTER=skyharbor cargo test -p ergoscript-compiler test_significant_15 -- --ignored --nocapture

# Regression guard (run after EVERY edit per MEMORY)
cargo test -p ergoscript-compiler --lib                                   # 233/233
cargo test -p ergoscript-compiler --test conformance                      # 154/154
cargo test -p ergoscript-compiler --lib test_batch_node_byte_match        # 1/1
source ~/.secrets && cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture  # 14/14
```

## Definition of done

- 15/15 LOCAL MATCH on `test_significant_15`.
- All baseline suites still green (no regressions in 46-corpus or 14-ecosystem).
- MANIFEST.md updated; ERGOSCRIPT-COMPILER-STATUS.md updated to add sig-15 row
  to the test coverage table.
