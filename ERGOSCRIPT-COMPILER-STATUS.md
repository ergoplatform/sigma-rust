# ErgoScript Rust Compiler — Status

## What is it?

The `ergoscript-compiler` crate compiles ErgoScript source code to ErgoTree bytecode in pure Rust. It lives in the sigma-rust monorepo alongside `ergotree-ir` and `ergotree-interpreter`.

## Current state: production-usable

**180 tests passing. 12/15 production contracts byte-match the Scala node natively. 15/15 match via canonical compilation (node fallback).**

### Two compilation modes

```rust
use ergoscript_compiler::compiler::{compile, compile_canonical};
use ergoscript_compiler::script_env::ScriptEnv;

// Pure Rust — no network, no dependencies. 12/15 contracts match Scala exactly.
let tree = compile(source, ScriptEnv::new())?;

// Canonical — verifies against Ergo node, uses node bytes if local differs.
// Guarantees identical P2S addresses to the Scala toolchain. 15/15 match.
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
| Collection ops (`filter`, `map`, `fold`, `exists`, `forall`, `size`, `append`, `slice`) | Complete |
| Tuple construction and access | Complete |
| Global variables (`SELF`, `INPUTS`, `OUTPUTS`, `HEIGHT`, `CONTEXT`) | Complete |
| Register access (`R4[Long].get`, `R5[Any].isDefined`) | Complete |
| Built-in functions (`sigmaProp`, `proveDlog`, `atLeast`, `blake2b256`, `fromBase16`, `getVar`, `decodePoint`, `longToByteArray`) | Complete |
| Sigma protocols (`proveDlog`, `atLeast`, `&&`/`\|\|` on SigmaProp) | Complete |
| Context extensions (`getVar[T](id)`) | Complete |
| Data inputs (`CONTEXT.dataInputs`) | Complete |
| Constant segregation | Complete |

### Optimization passes

1. **Constant folding** — evaluates compile-time constant expressions
2. **SizeOf(Map) rewrite** — `mapVal.size` becomes `collection.size` (map preserves length)
3. **Single-use val inlining** — inlines vals used once, dead-code eliminates unused vals
4. **Negation elimination** — `!(a > b)` becomes `a <= b`
5. **Common Subexpression Elimination (CSE)** — graph IR approach porting the Scala compiler's `processAstGraph`:
   - DAG hash-consing with selective sharing (matches Scala IR behavior)
   - DFS schedule generation
   - Iterative extraction with candidate updating
   - Lambda-scope fallback for contracts with filter/fold/exists/forall

### Byte-match scorecard

| Contract | Source | Bytes | Status |
|----------|--------|-------|--------|
| Governance vault | custom | 393B | Native match |
| Governance reserve | custom | 467B | Native match |
| Governance proposal | custom | 268B | Native match |
| Governance treasury | custom | 496B | Native match |
| Oracle Pool v2 - Pool | EIP-0023 | 104B | Native match |
| Spectrum AMM Swap | spectrum-finance | 177B | Native match |
| Dexy Bank | kushti/dexy-stable | 291B | Native match |
| Nested forall | synthetic | 61B | Native match |
| Multi-filter + arith | synthetic | 219B | Native match |
| If-else paths | synthetic | 118B | Native match |
| Map + size | synthetic | 31B | Native match |
| Triple CSE | synthetic | 117B | Native match |
| Dexy-style LP | kushti/dexy-stable | 146B | Canonical (local +1B) |
| DuckPools Lending | duckpools | 275B | Canonical (local +2B) |
| Oracle Pool v2 - Oracle | EIP-0023 | 209B | Canonical (local -4B) |

Additional contracts tested via `compile_canonical`:
- SigmaUSD bank (reserve ratio) — 199B, canonical
- Rosen Bridge GuardSign (atLeast + proveDlog) — 159B, canonical
- DEX swap order — 125B, canonical
- Multi-sig treasury — 55B, canonical
- Token emission — 167B, canonical
- Time-locked vesting — 18B, native match

### Why 3 contracts need canonical fallback

The Scala compiler builds a DAG (directed acyclic graph) with hash-consing during compilation. Some node types are NOT deduplicated in the Scala IR — specifically `MethodCall` nodes and `Equals` comparisons create separate graph symbols per source occurrence. Our tree-based CSE merges structurally identical expressions, which causes slight over-extraction (1-4 extra bytes).

This was confirmed by building the Scala `sigmastate-interpreter` from source with debug logging in `processAstGraph`. The exact behavior depends on Scala IR internals (`rewriteDef`, implicit `Elem` singletons) that can't be replicated without porting the full Scala graph builder.

The `compile_canonical` function solves this completely by verifying against the Ergo node and using canonical bytes when they differ.

## How to run tests

```bash
# All unit tests (pure Rust, no node needed)
cargo test -p ergoscript-compiler
# 180 passed, 0 failed, 2 ignored

# Canonical compilation tests (requires running Ergo node at localhost:9053)
source ~/.secrets  # sets API_KEY
cargo test -p ergoscript-compiler test_canonical -- --ignored --nocapture
cargo test -p ergoscript-compiler test_real_world -- --ignored --nocapture
```

## Open items for future work

### Closing the native parity gap (3 contracts)

The remaining 3 contracts differ because the Scala IR doesn't hash-cons `MethodCall` and `Equals[A: Elem]` nodes. To fix natively:

1. **In `is_graph_shared`**: enumerate which MIR expression types correspond to non-shared Scala IR nodes
2. **In `count_dag_usages`**: don't deduplicate non-shared types
3. **Validate**: use the instrumented sigmastate-interpreter at `~/working-files/sigmastate-interpreter/` to check each contract

Debug infrastructure is in place:
- `sigmastate-interpreter/sc/jvm/src/test/scala/sigmastate/CseDebugTest.scala`
- `TreeBuilding.scala` instrumented with schedule/extraction logging
- Run: `cd sigmastate-interpreter && sbt "scJVM/testOnly sigmastate.CseDebugTest"`

### Language features not yet implemented

- `substConstants` — template-based contract instantiation
- `serialize` / `deserialize` — on-chain serialization
- Multi-line string literals
- Pattern matching (not commonly used in contracts)
- `xor` on byte collections
- `byteArrayToLong`, `byteArrayToBigInt`

### Architecture improvements

- Remove `curl` dependency in `compile_canonical` — use a Rust HTTP client
- Add `compile_canonical` to the public crate API with proper error types
- WASM target support for browser-based compilation
- LSP server for IDE integration
