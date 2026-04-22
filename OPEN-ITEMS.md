# ErgoScript Compiler — Open Items

## Full Contract Status (34 contracts)

### BATCH BYTE-MATCH: 15/15 native match (hardcoded hex, offline)

| # | Contract | Bytes | Status |
|---|----------|-------|--------|
| 1 | Governance vault | 393B | MATCH |
| 2 | Governance reserve | 467B | MATCH |
| 3 | Governance proposal | 268B | MATCH |
| 4 | Governance treasury | 496B | MATCH |
| 5 | Oracle Pool v2 - Pool | 104B | MATCH |
| 6 | Oracle Pool v2 - Oracle | 209B | MATCH |
| 7 | Spectrum AMM Swap | 177B | MATCH |
| 8 | Dexy Bank | 291B | MATCH |
| 9 | DuckPools Lending Pool | 275B | MATCH |
| 10 | Dexy-style LP | 146B | MATCH |
| 11 | Nested forall | 61B | MATCH |
| 12 | Multi-filter + arith | 219B | MATCH |
| 13 | Register tuple + blake | 96B | MATCH |
| 14 | If-else paths | 118B | MATCH |
| 15 | Triple CSE | 117B | MATCH |

### CANONICAL E2E: 7 match, 3 canonical (verified against live node)

| # | Contract | Bytes | Status |
|---|----------|-------|--------|
| 16 | simple | 10B | MATCH |
| 17 | vault (with lambdas) | 393B | MATCH |
| 18 | dexy LP | 146B | MATCH |
| 19 | toBigInt arithmetic | 13B | MATCH |
| 20 | CONTEXT.selfBoxIndex | 12B | MATCH |
| 21 | allOf / anyOf | 25B | MATCH |
| 22 | def + lambda application | 28B | MATCH |
| 23 | Off-the-grid (grid orders) | 82B | CANONICAL |
| 24 | Crystal Pool (buy token) | 81B | CANONICAL |
| 25 | Phoenix HodlERG Bank | 314B | CANONICAL |

### REAL-WORLD E2E: 3 match, 3 canonical (verified against live node)

| # | Contract | Bytes | Status |
|---|----------|-------|--------|
| 26 | DEX swap order | 125B | MATCH |
| 27 | Token emission | 167B | MATCH |
| 28 | Time-locked vesting | 18B | MATCH |
| 29 | SigmaUSD bank | 199B | CANONICAL |
| 30 | Rosen GuardSign | 159B | CANONICAL |
| 31 | Multi-sig treasury | 55B | CANONICAL |

### ECOSYSTEM COMPILE: 3/3 compile (canonical verified)

| # | Contract | Bytes | Status |
|---|----------|-------|--------|
| 32 | Phoenix HodlERG Bank (full) | 314B | Compiles, CANONICAL |
| 33 | Off-the-grid multi-grid | 82B | Compiles, CANONICAL |
| 34 | Crystal Pool buy-token | 81B | Compiles, CANONICAL |

### Totals: 196 tests, 0 failures, 0 compile errors

| | Match | Canonical | Error | Total |
|---|---|---|---|---|
| Batch | 15 | 0 | 0 | 15 |
| Canonical e2e | 7 | 3 | 0 | 10 |
| Real-world e2e | 3 | 3 | 0 | 6 |
| Ecosystem | 0 | 3 | 0 | 3 |
| **Total** | **25** | **9** | **0** | **34** |

---

## 6 Byte-Match Gaps (all produce correct output via canonical fallback)

### Gap 1: Off-the-grid (82B) �� Spanned PartialEq prevents CSE dedup

**CSE path:** `process_ast_graph`

**Problem:** `ExtractRegisterAs(SELF, R4)` appears twice in the tree (once in `proveDlog(ownerPK)`, once in `recreatedBox.R4 == SELF.R4`). Scala extracts it as a CSE ValDef. We don't because `Spanned<ExtractRegisterAs>` includes source position in `PartialEq`, making two identical register accesses from different source locations appear as different nodes. Our span-stripping pass (`strip_source_spans`) normalizes spans before CSE, but `count_dag_usages` still reports dag_count=1 — investigation needed into whether the stripping is incomplete or a nested comparison issue.

**Fix direction:** Implement span-ignoring equality for CSE, or verify strip_source_spans covers all nested Spanned types.

### Gap 2: Crystal Pool buy-token (81B) — def/lambda Apply structure

**CSE path:** `has_lambdas` (due to no FuncValue... actually Crystal Pool uses `def` which desugars to lambda + Apply, so `contains_func_value` returns true → `has_lambdas` path)

**Problem:** `def f(x: Box) = expr` desugars to `val f = { (x: Box) => expr }`, then `f(SELF)` becomes `Apply(FuncValue, [SELF])`. Scala compiles `def` natively — the function body is inlined at each call site, no Apply node. Our desugaring introduces FuncValue+Apply overhead that changes the tree structure and constant segregation.

**Fix direction:** Inline `def`-created lambdas at call sites before MIR lowering (beta reduction: replace `Apply(FuncValue(params, body), args)` with `body[params := args]`).

### Gap 3: Phoenix HodlERG Bank (314B) — allOf + BigInt + if/else CSE

**CSE path:** `process_ast_graph`

**Problem:** Complex interaction of `allOf(Coll[Boolean](...))` containing val references inside if/else branches, combined with BigInt arithmetic. The `And`/`Or`/`Collection` CSE traversal is working (compile error fixed), but the extraction decisions and ValDef ordering differ from Scala. Likely the same Spanned PartialEq issue affecting multiple nodes.

**Fix direction:** Same as Gap 1 — once span-ignoring equality works, this may resolve automatically.

### Gap 4: SigmaUSD bank (199B) — CSE extraction on process_ast_graph path

**CSE path:** `process_ast_graph` (no lambdas)

**Problem:** Not fully root-caused. Contract has: `tokens(0)._1` comparisons (2x with different NFT IDs), `tokens(1)._2` extractions (2x), `CONTEXT.dataInputs`, `R4[Long].get`, arithmetic. Likely the Spanned PartialEq issue preventing proper dedup of repeated token access patterns, or a CSE ordering difference for the `&&` chain.

**Fix direction:** Root-cause by hex comparison once node API permissions are resolved. Likely same underlying issue as Gap 1.

### Gap 5: Rosen GuardSign (159B) — has_lambdas CSE path

**CSE path:** `has_lambdas` (has `filter` + `map` lambdas)

**Problem:** The `has_lambdas` path uses a savings-based extraction approach instead of the graph IR. It blocks `contains_val_use` expressions from extraction. The `pks.map { proveDlog }` + `atLeast(threshold, sigmas)` pattern creates FuncValue nodes that trigger this path. The savings-based heuristic produces different extraction decisions than Scala's graph approach.

**Fix direction:** Improve the `has_lambdas` CSE path to better match Scala's behavior, OR find a way to use the graph IR path for these contracts (separate lambda body CSE from top-level CSE).

### Gap 6: Multi-sig treasury (55B) — has_lambdas CSE path

**CSE path:** `has_lambdas` (has `map` lambda)

**Problem:** Same root cause as Gap 5 — the `pks.map { proveDlog }` pattern triggers the `has_lambdas` path. Simpler contract but same CSE path difference.

**Fix direction:** Same as Gap 5.

---

## Root Cause Summary

| Root Cause | Gaps Affected | Effort |
|------------|---------------|--------|
| Spanned PartialEq in CSE dedup | #1, #3, #4 | Medium — need span-ignoring Expr comparison |
| def/lambda not inlined (beta reduction) | #2 | Medium — inline Apply(FuncValue, args) |
| has_lambdas CSE path heuristics | #5, #6 | Hard — improve savings-based CSE or refactor paths |

---

## Language Features Not Yet Implemented

| Feature | Needed By | Effort |
|---------|-----------|--------|
| `serialize` / `deserialize` | Rare contracts | Medium |
| `getOrElse` on token collections | SigmaO Option (full) | Small |
| Multi-line string literals | None known | Small |
| Pattern matching | None known | Hard |

---

## Architecture / Tooling

| Item | Notes |
|------|-------|
| Replace `curl` in `compile_canonical` | Use Rust HTTP client |
| Public API for `compile_canonical` | Proper error types, not String |
| WASM target | Browser-based compilation |
| LSP server | IDE integration |
