# ErgoScript Rust Compiler — Status

## What is it?

The `ergoscript-compiler` crate compiles ErgoScript source code to ErgoTree bytecode in pure Rust. It lives in the sigma-rust monorepo alongside `ergotree-ir` and `ergotree-interpreter`.

## Current state: production-ready

**203 tests passing. 45/46 contracts byte-match the Scala node natively (1 skipped — CSE stack overflow on deeply nested BigInt polynomial).**

### Two compilation modes

```rust
use ergoscript_compiler::compiler::{compile, compile_canonical};
use ergoscript_compiler::script_env::ScriptEnv;

// Pure Rust — no network, no dependencies. 45/46 contracts byte-match Scala exactly.
let tree = compile(source, ScriptEnv::new())?;

// Canonical — verifies against Ergo node, uses node bytes if local differs.
// Guarantees identical P2S addresses to the Scala toolchain.
let result = compile_canonical(source, ScriptEnv::new(), "http://localhost:9053", &api_key)?;
// result.matched: Some(true) = local match, Some(false) = used node, None = node unavailable
```

### Language features supported

| Feature | Status |
|---------|--------|
| Literals (Int, Long, Bool, String, Hex) | Complete |
| Arithmetic (`+`, `-`, `*`, `/`, `%`, `min`, `max`) | Complete |
| Comparisons (`==`, `!=`, `>`, `<`, `>=`, `<=`) | Complete |
| Logical (`&&`, `\|\|`, `!`) | Complete |
| Val bindings with type annotations | Complete |
| If/else expressions | Complete |
| Block expressions | Complete |
| Field access (`box.value`, `tuple._1`) | Complete |
| Method calls (`tokens(0)`, `filter{...}`) | Complete |
| Lambda expressions | Complete |
| Collection ops (`filter`, `map`, `flatMap`, `fold`, `exists`, `forall`, `size`, `append`, `slice`) | Complete |
| Tuple construction and access | Complete |
| Global variables (`SELF`, `INPUTS`, `OUTPUTS`, `HEIGHT`, `CONTEXT`) | Complete |
| Register access (`R4[Long].get`, `R5[Any].isDefined`) | Complete |
| Built-in functions (`sigmaProp`, `proveDlog`, `atLeast`, `blake2b256`, `fromBase16`, `fromBase58`, `getVar`, `decodePoint`, `longToByteArray`, `byteArrayToLong`, `byteArrayToBigInt`, `substConstants`, `xor`, `xorOf`) | Complete |
| Sigma protocols (`proveDlog`, `atLeast`, `&&`/`\|\|` on SigmaProp) | Complete |
| Bool-to-SigmaProp auto-promotion in `&&`/`\|\|` | Complete |
| Context extensions (`getVar[T](id)`) | Complete |
| Data inputs (`CONTEXT.dataInputs`) | Complete |
| Constant segregation | Complete |

### Optimization passes

1. **Constant folding** — evaluates compile-time constant expressions
2. **SizeOf(Map) rewrite** — `mapVal.size` becomes `collection.size` (map preserves length)
3. **Single-use val inlining** — inlines vals used once, dead-code eliminates unused vals
4. **Negation elimination** — `!(a > b)` becomes `a <= b`
5. **BigInt type propagation** — after MIR lowering, propagates actual types from ValDef RHS to ValUse references (fixes `val x: Long = BigInt_expr` annotation mismatches), re-applies numeric upcasts
6. **Common Subexpression Elimination (CSE)** — graph IR approach porting the Scala compiler's `processAstGraph`:
   - DAG hash-consing with selective sharing (matches Scala IR behavior)
   - DFS schedule generation
   - Iterative extraction with candidate updating
   - ThunkDef scope modeling for `&&`/`||` right-arm scoping and If-branch scoping
   - Lambda-scope fallback for contracts with filter/fold/exists/forall
   - Post-CSE single-use val inlining (folds e.g. `ExtractAmount(Self)` into `Upcast(ExtractAmount(Self), BigInt)`)
   - Inner-block constant deduplication (extracts duplicate constants as vals within If-branch blocks)
   - If-branch val ordering via symbol-ID-sorted freeVars (matches Scala's ThunkDef scheduling)

### Contract test inventory (46 contracts)

All contracts produce bytecode identical to the Scala reference node, with one exception (#39, see Notes).

| # | Contract | Source | Bytes | Match | Notes |
|---|----------|--------|-------|-------|-------|
| 1 | Governance vault | custom | 393B | Y | |
| 2 | Governance reserve | custom | 467B | Y | |
| 3 | Governance proposal | custom | 268B | Y | |
| 4 | Governance treasury | custom | 496B | Y | |
| 5 | Oracle Pool v2 - Pool | EIP-0023 | 104B | Y | |
| 6 | Oracle Pool v2 - Oracle | EIP-0023 | 209B | Y | |
| 7 | Spectrum AMM Swap | spectrum-finance | 177B | Y | |
| 8 | Dexy Bank | kushti/dexy-stable | 291B | Y | |
| 9 | DuckPools Lending Pool | duckpools | 275B | Y | |
| 10 | Dexy-style LP | kushti/dexy-stable | 146B | Y | |
| 11 | Nested forall | synthetic | 61B | Y | |
| 12 | Multi-filter + arith | synthetic | 219B | Y | |
| 13 | Register tuple + blake | synthetic | 96B | Y | |
| 14 | If-else paths | synthetic | 118B | Y | |
| 15 | Triple CSE | synthetic | 117B | Y | |
| 16 | simple (SELF.value > 0) | synthetic | 10B | Y | |
| 17 | vault (with lambdas) | custom | 393B | Y | |
| 18 | dexy LP (canonical test) | kushti/dexy-stable | 146B | Y | |
| 19 | toBigInt arithmetic | synthetic | 13B | Y | |
| 20 | CONTEXT.selfBoxIndex | synthetic | 12B | Y | |
| 21 | allOf / anyOf | synthetic | 25B | Y | |
| 22 | def function + lambda app | synthetic | 28B | Y | |
| 23 | Off-the-grid (grid orders) | Telefragged/off-the-grid | 82B | Y | |
| 24 | Crystal Pool (buy token) | SavonarolaLabs/crystal-pool | 81B | Y | |
| 25 | Phoenix HodlERG Bank | PhoenixErgo/phoenix-hodlcoin | 314B | Y | |
| 26 | SigmaUSD bank | sigmausd | 199B | Y | |
| 27 | Rosen GuardSign | rosen-bridge | 159B | Y | |
| 28 | DEX swap order | spectrum-finance | 125B | Y | |
| 29 | Multi-sig treasury | custom | 55B | Y | |
| 30 | Token emission | custom | 167B | Y | |
| 31 | Time-locked vesting | custom | 18B | Y | |
| 32 | SigmaFi BondContractERG | K-Singh/Sigma-Finance | 146B | Y | |
| 33 | SigmaFi BondContractToken | K-Singh/Sigma-Finance | 223B | Y | |
| 34 | SigmaFi EXP_BondContractERG | K-Singh/Sigma-Finance | 182B | Y | |
| 35 | SigmaFi OpenOrderERG | K-Singh/Sigma-Finance | 471B | Y | BigInt fees |
| 36 | SigmaFi OpenOrderToken | K-Singh/Sigma-Finance | 638B | Y | no-segregation fallback |
| 37 | SkyHarbor SigUSDV1 | skyharbor-market | 510B | Y | fromBase58, R6[Box], no-segregation |
| 38 | DuckPools ERG Repayment | duckpools | 189B | Y | fromBase58 |
| 39 | DuckPools ERG InterestRate | duckpools | — | SKIP | CSE stack overflow (deep BigInt polynomial) |
| 40 | DuckPools ERG ParentInterest | duckpools | 412B | Y | append, fold |
| 41 | DuckPools ERG ProxyBorrow | duckpools | 440B | Y | fromBase58, complex if/else |
| 42 | Lilium CollectionIssuer | LiliumErgo/scala-api | 85B | Y | getVar[Coll[Byte]] |
| 43 | Lilium CollectionIssuance | LiliumErgo/scala-api | 113B | Y | getVar[Box] |
| 44 | Lilium PreMintIssuer | LiliumErgo/scala-api | 90B | Y | |
| 45 | Lilium WhitelistIssuer | LiliumErgo/scala-api | 90B | Y | |
| 46 | Lilium SaleLP | LiliumErgo/scala-api | 317B | Y | flatMap, no-segregation fallback |

## How to run tests

```bash
# All unit tests (pure Rust, no node needed)
cargo test -p ergoscript-compiler
# 203 passed, 0 failed, 3 ignored

# Canonical compilation tests (requires running Ergo node at localhost:9053)
source ~/.secrets  # sets API_KEY
cargo test -p ergoscript-compiler test_canonical -- --ignored --nocapture
cargo test -p ergoscript-compiler test_real_world -- --ignored --nocapture
cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture
```

## Open items for future work

### Language features not yet implemented

- `serialize` / `deserialize` — on-chain serialization
- `indexOf` on collections
- `fromBase64` — compile-time Base64 decode
- Multi-line string literals
- Pattern matching (not commonly used in contracts)

### Known issues

- **CSE stack overflow on deeply nested BigInt polynomials** — DuckPools InterestRate contract has `(f * x) / D * x / M * x / M * x / M * x / M` which causes recursive CSE to overflow. Needs iterative CSE or depth limit.
- **Constant segregation roundtrip failure** — Some contracts with complex CSE-extracted vals fail the `ErgoTree::new` serialize→deserialize roundtrip (ValDefIdNotFound). Root cause: `ErgoTree::new` with constant segregation does serialize→re-parse internally; CSE-extracted vals in ThunkDef scopes (If branches, &&/|| right arms) produce ValUse references before their ValDef in the linear serialization order. Workaround: fall back to non-segregated ErgoTree. Affects 3 ecosystem contracts.

### Architecture improvements

- Remove `curl` dependency in `compile_canonical` — use a Rust HTTP client
- Add `compile_canonical` to the public crate API with proper error types
- WASM target support for browser-based compilation
- LSP server for IDE integration
