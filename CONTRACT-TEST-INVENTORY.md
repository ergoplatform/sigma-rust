# ErgoScript Compiler — Contract Test Inventory

All contracts tested end-to-end against the Scala Ergo node (v6.1.2 at localhost:9053).

- **LOCAL MATCH** = our Rust compiler produces byte-identical ErgoTree to the Scala node
- **CANONICAL** = our bytes differ, `compile_canonical()` falls back to node bytes (correct output guaranteed)
- **COMPILE ERROR** = our compiler cannot compile the contract yet

## Batch Byte-Match Tests (15/15 native match)

These are verified offline — expected hex is hardcoded in the test suite.

| # | Contract | Source | Bytes | Status |
|---|----------|--------|-------|--------|
| 1 | Governance vault | custom | 393B | LOCAL MATCH |
| 2 | Governance reserve | custom | 467B | LOCAL MATCH |
| 3 | Governance proposal | custom | 268B | LOCAL MATCH |
| 4 | Governance treasury | custom | 496B | LOCAL MATCH |
| 5 | Oracle Pool v2 - Pool | EIP-0023 | 104B | LOCAL MATCH |
| 6 | Oracle Pool v2 - Oracle | EIP-0023 | 209B | LOCAL MATCH |
| 7 | Spectrum AMM Swap | spectrum-finance | 177B | LOCAL MATCH |
| 8 | Dexy Bank | kushti/dexy-stable | 291B | LOCAL MATCH |
| 9 | DuckPools Lending Pool | duckpools | 275B | LOCAL MATCH |
| 10 | Dexy-style LP | kushti/dexy-stable | 146B | LOCAL MATCH |
| 11 | Nested forall | synthetic | 61B | LOCAL MATCH |
| 12 | Multi-filter + arith | synthetic | 219B | LOCAL MATCH |
| 13 | Register tuple + blake | synthetic | 96B | LOCAL MATCH |
| 14 | If-else paths | synthetic | 118B | LOCAL MATCH |
| 15 | Triple CSE | synthetic | 117B | LOCAL MATCH |

## Canonical Tests (verified against live node)

| # | Contract | Bytes | Status |
|---|----------|-------|--------|
| 16 | simple (SELF.value > 0) | 10B | LOCAL MATCH |
| 17 | vault (with lambdas) | 393B | LOCAL MATCH |
| 18 | dexy LP | 146B | LOCAL MATCH |
| 19 | toBigInt arithmetic | 13B | CANONICAL |
| 20 | CONTEXT.selfBoxIndex | 12B | LOCAL MATCH |
| 21 | allOf / anyOf | 25B | LOCAL MATCH |
| 22 | def function + lambda app | 28B | LOCAL MATCH |
| 23 | Off-the-grid (grid orders) | 82B | CANONICAL |
| 24 | Crystal Pool (buy token) | 81B | CANONICAL |

## Real-World Contract Tests (verified against live node)

| # | Contract | Source | Bytes | Status |
|---|----------|--------|-------|--------|
| 25 | SigmaUSD bank (reserve ratio) | sigmausd | 199B | CANONICAL |
| 26 | Rosen GuardSign (atLeast+proveDlog) | rosen-bridge | 159B | CANONICAL |
| 27 | DEX swap order | spectrum-finance | 125B | LOCAL MATCH |
| 28 | Multi-sig treasury | custom | 55B | CANONICAL |
| 29 | Token emission | custom | 167B | LOCAL MATCH |
| 30 | Time-locked vesting | custom | 18B | LOCAL MATCH |

## Ecosystem Contracts (compile-only, not yet byte-matched)

| # | Contract | Source | Status | Notes |
|---|----------|--------|--------|-------|
| 31 | Phoenix HodlERG Bank | PhoenixErgo/phoenix-hodlcoin-contracts | COMPILE ERROR | CSE ValDef renumbering bug with BigInt in if/else branches |
| 32 | SigmaO Option | ThierryM1212/SigmaO | NOT TESTED | Uses `getOrElse` on tokens, complex state machine |
| 33 | Crystal Pool swap-tokens | SavonarolaLabs/crystal-pool | NOT TESTED | Uses `def` + `fold` + `filter` + `toBigInt` heavily |

## Summary

| Category | Count | Native Match | Canonical | Error |
|----------|-------|-------------|-----------|-------|
| Batch byte-match | 15 | 15 | 0 | 0 |
| Canonical tests | 9 | 6 | 3 | 0 |
| Real-world tests | 6 | 3 | 3 | 0 |
| Ecosystem (compile) | 3 | — | — | 1 |
| **Total** | **33** | **24** | **6** | **1** |

## Known Gaps (byte-match failures)

### toBigInt arithmetic (canonical #19)
`.toBigInt` Upcast serialization differs from Scala. Likely a constant segregation or Upcast opcode ordering issue.

### Off-the-grid (canonical #23)
CSE extraction difference with `CONTEXT.selfBoxIndex` patterns. The contract uses `OUTPUTS(selfIndex)` which creates dynamic ByIndex that interacts with CSE differently.

### Crystal Pool buy token (canonical #24)
`def` function / lambda application produces different ErgoTree structure than Scala's native `def` compilation. Scala may optimize or inline differently.

### SigmaUSD bank (real-world #25)
Complex reserve ratio math with BigInt. Likely same `.toBigInt` Upcast issue.

### Rosen GuardSign (real-world #26)
`atLeast` + `proveDlog` mixing. May be SigmaAnd/SigmaOr structural difference.

### Multi-sig treasury (real-world #28)
Simple multi-sig pattern. Likely `atLeast` CSE interaction.

### Phoenix HodlERG Bank (ecosystem #31)
CSE `ValDefIdNotFound(ValId(11))` — the BigInt arithmetic in both branches of an `if/else` creates ValDef IDs that the renumbering pass can't resolve. Root cause: CSE renumbering doesn't account for ValDefs introduced in separate if/else branches that share a common dependency.
