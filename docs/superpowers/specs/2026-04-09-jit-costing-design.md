# JIT Costing for ergotree-interpreter

## Problem

`VerificationResult.cost` and `ReductionResult.cost` are hardcoded to `0`. The `CostAccumulator` and `Costs` types exist but are dead code. ErgoScript evaluation has no cost tracking, which means the node can't enforce `MaxBlockCost` or rate-limit expensive transactions.

## Goal

Port the per-opcode JIT cost table from the JVM sigmastate-interpreter so that after `reduce_to_crypto` returns, `ReductionResult.cost` contains the actual accumulated evaluation cost, and after verification, the caller can read the total cost.

## Architecture Decision: Context-Embedded Accumulator

Rather than adding `cost_accum: &mut CostAccumulator` to the `Evaluable` trait (which would change ~70 file signatures), the accumulator is embedded in `Context` using `Cell<u64>`. Each eval impl calls `ctx.add_jit_cost(N)?` internally. This matches the existing `Cell<ErgoTreeVersion>` pattern in Context and avoids any trait signature changes.

## Cost Model

JitCost values are in a 10x scale relative to block costs. `JitCost.to_block_cost()` divides by 10.

### Cost Kinds

| Kind | Formula |
|------|---------|
| `Fixed(N)` | cost = N |
| `PerItem(base, perChunk, chunkSize)` | cost = base + ceil(nItems / chunkSize) * perChunk |
| `TypeBased(bigint, other)` | cost = bigint if BigInt arg, else other |
| `Dynamic` | cost = sum of sub-operation costs (no additional charge) |

### Complete Cost Table

**Tree operations:**

| Operation | CostKind |
|-----------|----------|
| Constant | Fixed(5) |
| ConstantPlaceholder | Fixed(1) |
| TaggedVariable (ValUse) | Fixed(5) |
| GroupGenerator | Fixed(10) |
| Tuple | Fixed(15) |
| ConcreteCollection | Fixed(20) |
| BlockValue | PerItem(1, 1, 10) |
| FuncValue | Fixed(5) |
| Apply | Fixed(30) |
| MethodCall | Fixed(4) |
| PropertyCall | Fixed(4) |
| If | Fixed(10) |
| LogicalNot | Fixed(15) |
| BoolToSigmaProp | Fixed(15) |
| CreateProveDlog | Fixed(10) |
| CreateProveDHTuple | Fixed(20) |
| SigmaAnd | PerItem(10, 2, 1) |
| SigmaOr | PerItem(10, 2, 1) |
| OR | PerItem(5, 5, 64) |
| XorOf | PerItem(20, 5, 32) |
| AND | PerItem(10, 5, 32) |
| AtLeast | PerItem(20, 3, 5) |
| Upcast | TypeBased(bigint=30, other=10) |
| Downcast | TypeBased(bigint=30, other=10) |
| LongToByteArray | Fixed(17) |
| ByteArrayToLong | Fixed(16) |
| ByteArrayToBigInt | Fixed(30) |
| DecodePoint | Fixed(300) |
| CalcBlake2b256 | PerItem(20, 7, 128) |
| CalcSha256 | PerItem(80, 8, 64) |
| SubstConstants | PerItem(100, 100, 1) |
| ArithOp Plus/Minus | TypeBased(bigint=20, other=15) |
| ArithOp Multiply/Division/Modulo | TypeBased(bigint=25, other=15) |
| ArithOp Min/Max | TypeBased(bigint=10, other=5) |
| Negation | Fixed(30) |
| BitOp (all 6) | Fixed(1) |
| ModQ, ModQArithOp | Fixed(1) |
| Xor | PerItem(10, 2, 128) |
| Exponentiate | Fixed(900) |
| MultiplyGroup | Fixed(40) |
| LT, LE, GT, GE | TypeBased(bigint=20, other=20) |
| EQ, NEQ | Dynamic |
| BinOr, BinAnd, BinXor | Fixed(20) |

**Transformer operations:**

| Operation | CostKind |
|-----------|----------|
| MapCollection | PerItem(20, 1, 10) |
| Append | PerItem(20, 2, 100) |
| Slice | PerItem(10, 2, 100) |
| Filter | PerItem(20, 1, 10) |
| Exists | PerItem(3, 1, 10) |
| ForAll | PerItem(3, 1, 10) |
| Fold | PerItem(3, 1, 10) |
| ByIndex | Fixed(30) |
| SelectField | Fixed(10) |
| SigmaPropBytes | PerItem(35, 6, 1) |
| SizeOf | Fixed(14) |
| ExtractAmount | Fixed(8) |
| ExtractScriptBytes | Fixed(10) |
| ExtractBytes | Fixed(12) |
| ExtractBytesWithNoRef | Fixed(12) |
| ExtractId | Fixed(12) |
| ExtractRegisterAs | Fixed(50) |
| ExtractCreationInfo | Fixed(16) |
| DeserializeContext | PerItem(1, 10, 128) |
| DeserializeRegister | PerItem(1, 10, 128) |
| GetVar | Fixed(10) |
| OptionGet | Fixed(15) |
| OptionGetOrElse | Fixed(20) |
| OptionIsDefined | Fixed(10) |

**Context accessors:**

| Operation | CostKind |
|-----------|----------|
| MinerPubkey | Fixed(20) |
| Height | Fixed(26) |
| Inputs | Fixed(10) |
| Outputs | Fixed(10) |
| LastBlockUtxoRootHash | Fixed(15) |
| Self | Fixed(10) |
| Context | Fixed(1) |
| Global | Fixed(5) |

**Method costs** (added on top of MethodCall/PropertyCall's Fixed(4)):

| Type.method | CostKind |
|-------------|----------|
| SGroupElement.getEncoded | Fixed(250) |
| SGroupElement.negate | Fixed(45) |
| SBox.tokens | Fixed(15) |
| SBox.getReg / R0-R9 | Fixed(50) |
| SHeader.checkPow | Fixed(700) |
| SOption.map / filter | Fixed(20) |
| SCollection.indices | PerItem(20, 2, 16) |
| SCollection.flatMap | PerItem(60, 10, 8) |
| SCollection.patch | PerItem(30, 2, 10) |
| SCollection.updated | PerItem(20, 1, 10) |
| SCollection.zip | PerItem(10, 1, 10) |
| SCollection.indexOf | PerItem(20, 10, 2) |
| SAvlTree.digest / enabledOperations / keyLength / etc | Fixed(15) |
| SAvlTree.updateOperations | Fixed(45) |
| SAvlTree.updateDigest | Fixed(40) |
| SAvlTree contains/get/insert/update/remove | Dynamic |
| SGlobal.deserializeTo | PerItem(100, 32, 32) |
| SHeader field accessors | Fixed(10) |
| SPreHeader field accessors | Fixed(10) |
| SContext.dataInputs / headers / preHeader | Fixed(15) |
| SContext.selfBoxIndex | Fixed(20) |

## Implementation Components

### 1. Cost Types (`ergotree-interpreter/src/eval/costs.rs`)

Replace the existing stub with:

- `JitCost(u32)` — newtype with `to_block_cost() -> u64` (divides by 10)
- `CostKind` enum — `Fixed`, `PerItem`, `TypeBased`, `Dynamic`
- `PerItemCost::total(n_items: u32) -> u32` — computes `base + ceil(nItems / chunkSize) * perChunk`
- Lookup functions for expr-level and method-level costs

The `Cost` and `Costs` types are replaced entirely; `CostAccumulator` in `cost_accum.rs` becomes dead code (superseded by Context fields).

### 2. Context Changes (`ergotree-ir/src/chain/context.rs`)

New fields on `Context`:

```rust
pub jit_cost: Cell<u64>,       // accumulated JIT cost
pub jit_cost_limit: Option<u64>, // limit in JitCost scale (None = unlimited)
```

New error type in ergotree-ir:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostLimitExceeded(pub u64);
```

New methods on `Context`:

```rust
pub fn add_jit_cost(&self, amount: u32) -> Result<(), CostLimitExceeded>
pub fn add_per_item_jit_cost(&self, base: u32, per_chunk: u32, chunk_size: u32, n_items: u32) -> Result<(), CostLimitExceeded>
pub fn jit_cost(&self) -> u64
pub fn reset_jit_cost(&self)
```

`EvalError` in ergotree-interpreter already has a `CostError` variant; update it to accept `From<CostLimitExceeded>`.

Construction sites (Arbitrary impl, `make_context()`, `update_context()`) get default values: `jit_cost: Cell::new(0), jit_cost_limit: None`.

### 3. Eval Integration (~70 files)

Three patterns:

**Fixed cost** — call `ctx.add_jit_cost(N)?` before the operation:
```rust
fn eval<'ctx>(&self, env: &mut Env<'ctx>, ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
    ctx.add_jit_cost(8)?;  // ExtractAmount = Fixed(8)
    // ... existing logic unchanged
}
```

**Per-item cost** — call after computing item count:
```rust
fn eval<'ctx>(&self, ...) -> Result<Value<'ctx>, EvalError> {
    let input_v = self.input.eval(env, ctx)?;
    let n_items = bytes.len() as u32;
    ctx.add_per_item_jit_cost(20, 7, 128, n_items)?;  // CalcBlake2b256
    // ... existing logic
}
```

**Dynamic cost** — no additional charge at the node; sub-operations accumulate their own costs.

### 4. Method Costs

Method eval functions (in `savltree.rs`, `sbox.rs`, `scoll.rs`, etc.) add their method-specific cost at the start. The `MethodCall`/`PropertyCall` Fixed(4) is added in `method_call.rs` and `property_call.rs`.

### 5. Cost Propagation

**reduce_to_crypto**: After eval completes, reads `ctx.jit_cost()`, converts to block cost (`/ 10`), stores in `ReductionResult.cost`.

**Verifier**: `verify()` already reads `reduction_result.cost` and stores it in `VerificationResult.cost`. Currently gets 0 — once reduce_to_crypto populates it, it flows automatically.

**TransactionContext::validate()**: Sum `VerificationResult.cost` across all inputs. The `// TODO: costing` comment gets addressed.

### 6. Cost Limit

For validation, `make_context()` receives `max_block_cost` from `ErgoStateContext.parameters` and sets `jit_cost_limit = Some(max_block_cost as u64 * 10)`.

For signing (wallet), `jit_cost_limit` stays `None`.

`update_context()` resets `jit_cost` to 0 for each new input but preserves the limit.

## What Does NOT Change

- The `Expr` AST types in `ergotree-ir` — costs are runtime, not representation
- The `Evaluable` trait signature — cost accumulation is via Context, not a new parameter
- Existing test behavior — default cost limit is None, so no existing test can fail from cost limits

## Verification

- All existing tests pass unchanged
- New test: `{ true }` (Constant) has JitCost 5, block cost 0 (5/10 rounds down)
- New test: `SELF.value` has JitCost 18 (Self=10 + ExtractAmount=8), block cost 1
- New test: cost limit enforcement — accumulator exceeds limit, returns CostLimitExceeded error
