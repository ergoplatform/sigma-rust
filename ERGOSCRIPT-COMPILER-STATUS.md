# ErgoScript Rust Compiler — Status

## What is it?

The `ergoscript-compiler` crate compiles ErgoScript source code to ErgoTree bytecode in pure Rust. It lives in the sigma-rust monorepo alongside `ergotree-ir` and `ergotree-interpreter`.

## Current state: production-ready

**Test coverage**

| Suite | Result |
|---|---|
| `ergoscript-compiler --lib` | 233/233 |
| `ergoscript-compiler --lib -- --ignored` | 4/4 |
| `ergoscript-compiler --test conformance` | 154/154 |
| `test_batch_node_byte_match` | 1/1 |
| `test_ecosystem_batch` (auth-gated, vs `localhost:9053`) | 14/14 LOCAL MATCH |
| `test_significant_15` (auth-gated, vs `localhost:9053`) | 9/15 LOCAL MATCH |
| `ergotree-ir --features arbitrary --lib` | 255/255 |
| `ergotree-interpreter --features arbitrary --lib` | 336/336 |

**Byte-match parity with the Scala node**: 45/46 legacy contract fixtures (1 skipped — CSE stack overflow on a deeply nested BigInt polynomial; see Known issues), the 14 ecosystem contracts in the auth-gated batch (SigmaFi, SkyHarbor, DuckPools, Lilium), **and 9/15 keystone contracts from the "15 Significant Ergo Contracts" initiative** (skyharbor V1, phoenix HodlERG bank, spectrum n2t/t2t pools, dexy bank, ergoraffle, duckpools child interest, ergomixer fullmix, chaincash reserve). Per-fixture provenance and remaining-backlog status in [`ergoscript-compiler/tests/fixtures/significant_15/MANIFEST.md`](ergoscript-compiler/tests/fixtures/significant_15/MANIFEST.md).

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
| Bitwise infix (`&`, `\|`, `^`, `~`, `<<`, `>>`, `>>>`) | Complete (lex→parse→HIR→type→lower; shift eval is `NotImplemented`, matching Scala `testMissingCosting`) |
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
| Predef / built-in functions (44/44 — see "Predef function coverage") | Complete |
| Sigma protocols (`proveDlog`, `proveDHTuple`, `atLeast`, `&&`/`\|\|` on SigmaProp) | Complete |
| Bool-to-SigmaProp auto-promotion in `&&`/`\|\|` | Complete |
| Context extensions (`getVar[T](id)`, `getVarFromInput`) | Complete |
| Data inputs (`CONTEXT.dataInputs`) | Complete |
| Constant segregation | Complete |
| `ZKProof { ... }` block scope | Complete (frontend-only IR — no canonical op-code; serializing errors with `NotSupported`, mirroring Scala's `OpCodes.Undefined` + `testMissingCostingWOSerialization`) |
| Method registries (SColl, SOption, SAvlTree, SBox, SContext, SHeader, SPreHeader, SGroupElement, SGlobal, SNumeric, SBigInt/SUnsignedBigInt) | Complete (per-method ground truth in [`tests/fixtures/conformance/method-coverage.md`](ergoscript-compiler/tests/fixtures/conformance/method-coverage.md)) |

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

### Language conformance

The compiler tracks parity with the Scala reference (`sigmastate-interpreter`) along four workstreams. Per-method and per-predef ground truth lives in [`ergoscript-compiler/tests/fixtures/conformance/method-coverage.md`](ergoscript-compiler/tests/fixtures/conformance/method-coverage.md); durable status with file pointers lives in [`WORKSTREAM-STATUS.md`](WORKSTREAM-STATUS.md).

| Workstream | Scope | Status |
|---|---|---|
| A — Predef functions | `SigmaPredef` parity (the 44 globally-named built-ins: `sigmaProp`, `bigInt`, `unsignedBigInt`, `proveDlog`, `proveDHTuple`, `PK`, `atLeast`, `min`/`max`, `longToByteArray`, `byteArrayToLong`, `byteArrayToBigInt`, `fromBigEndianBytes`, `decodePoint`, `xor`, `xorOf`, `allOf`, `anyOf`, `allZK`, `anyZK`, `getVar`, `getVarFromInput`, `serialize`, `deserializeTo`, `some`, `none`, `encodeNbits`, `decodeNbits`, `powHit`, `avlTree`, `treeLookup`, `substConstants`, `upcast`, `downcast`, `placeholder`, `fromBase16`/`fromBase58`/`fromBase64`, `blake2b256`, `sha256`, `executeFromVar`, `ZKProof { ... }`, etc.) | **44/44** (100%) |
| B — Method registries | Method-call parity across the 11 type registries (SColl, SOption, SAvlTree, SBox, SContext, SHeader, SPreHeader, SGroupElement, SGlobal, SNumeric, SBigInt/SUnsignedBigInt) including V6 numeric extensions (`toBytes`/`toBits`/`bitwiseInverse`, `bitwiseOr`/`And`/`Xor`, `shiftLeft`/`Right`, BigInt+UnsignedBigInt modular arithmetic) | **100%** |
| C — Lexer / parser conformance | Surface-syntax parity: bitwise infix tokens (`&`/`\|`/`^`/`~`/`<<`/`>>`/`>>>`), `expr { block }` application form, post-fix method dispatch, all literal forms required by ecosystem contracts | **byte-match-complete** (~95%; un-braced lambda body grammar remains as QoL — see Open items) |
| D — Conformance smoke tests | Per-registry submodule layout under `ergoscript-compiler/tests/conformance/` mirroring the Scala test surface | **154 tests**; lexer-tokenization and parser-AST snapshot suites still to land (~95%) |

The byte-op-code space (`OpCodes` 0..=255) is **exhausted** — `XOR_OF = 255` is the last entry. Newer frontend constructs that have no Scala op-code (e.g. `ZkProofBlock`) are wired as frontend-only IR nodes whose serialize arm returns `SigmaSerializationError::NotSupported`, matching how Scala marks them with `OpCodes.Undefined`.

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
cargo test -p ergoscript-compiler --lib                      # 233 / 0 / 4 ignored
cargo test -p ergoscript-compiler --test conformance         # 154 / 0
cargo test -p ergoscript-compiler --lib -- --ignored         # 4 / 0
cargo test -p ergoscript-compiler --lib test_batch_node_byte_match  # 1 / 0

# Canonical compilation tests (requires running Ergo node at localhost:9053)
source ~/.secrets  # sets API_KEY
cargo test -p ergoscript-compiler test_canonical -- --ignored --nocapture
cargo test -p ergoscript-compiler test_real_world -- --ignored --nocapture
cargo test -p ergoscript-compiler test_ecosystem_batch -- --ignored --nocapture
```

## Open items for future work

### Residuals from the language-conformance arc

These are off the byte-match critical path — none block any current ecosystem fixture — but they are the natural next batch of polish.

- **`avlTree` IR shape mismatch** (flagged near-term priority). Rust's [`CreateAvlTree::value_length: Option<Box<Expr>>`](ergotree-ir/src/mir/create_avl_tree.rs) vs Scala's `valueLengthOpt: Value[SOption[SInt]]` (a runtime `SOption`-typed expression). The current `avlTree(...)` predef pattern-matches `none[Int]()` / `some(intExpr)` literals at compile time; runtime `SOption` arguments are rejected with a clear error. None of the 14/14 ecosystem fixtures call `avlTree` with a runtime SOption, so this is a documented IR-level discrepancy rather than a parity blocker — but fixing it unblocks Lithos / Etcha / Machina Finance byte-match. See `WORKSTREAM-STATUS.md §12a`.
- **Un-braced lambda body grammar** — accept `(x: Long) => x + 1` without the surrounding `{ }`. Surface-syntax QoL only; not used by any current ecosystem fixture. Workstream C residual.
- **Lexer / parser snapshot tests** — Workstream D §D.4 has stub plans for token-stream and parse-tree snapshot suites under `tests/conformance/lexer/` and `tests/conformance/parser/`. Independent of the byte-match path, good warm-up work.

### Known issues

- **CSE stack overflow on deeply nested BigInt polynomials** — DuckPools InterestRate contract has `(f * x) / D * x / M * x / M * x / M * x / M` which causes recursive CSE to overflow. Needs iterative CSE or depth limit.
- **Constant segregation roundtrip failure** — Some contracts with complex CSE-extracted vals fail the `ErgoTree::new` serialize→deserialize roundtrip (ValDefIdNotFound). Root cause: `ErgoTree::new` with constant segregation does serialize→re-parse internally; CSE-extracted vals in ThunkDef scopes (If branches, &&/|| right arms) produce ValUse references before their ValDef in the linear serialization order. Workaround: fall back to non-segregated ErgoTree. Affects 3 ecosystem contracts.

### Language features not yet implemented

- `indexOf` on collections (top-level — the method form on `Coll[T]` is supported)
- Multi-line string literals
- Pattern matching (not commonly used in contracts)

### Architecture improvements

- Remove `curl` dependency in `compile_canonical` — use a Rust HTTP client
- Add `compile_canonical` to the public crate API with proper error types
- WASM target support for browser-based compilation
- LSP server for IDE integration
