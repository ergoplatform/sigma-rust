# JIT Costing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port per-opcode JIT cost table from JVM sigmastate-interpreter so that `ReductionResult.cost` and `VerificationResult.cost` carry real accumulated evaluation costs.

**Architecture:** Cost accumulator embedded in `Context` via `Cell<u64>` (no trait signature changes). Each eval impl calls `ctx.add_jit_cost(N)?` with the operation's JIT cost. Cost propagates through `reduce_to_crypto` → `ReductionResult` → `VerificationResult` → transaction validation.

**Tech Stack:** Rust, `no_std` compatible (ergotree-ir is `no_std`), `Cell<u64>` for interior mutability.

**Spec:** `docs/superpowers/specs/2026-04-09-jit-costing-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `ergotree-ir/src/chain/context.rs` | Modify | Add `jit_cost: Cell<u64>`, `jit_cost_limit: Option<u64>`, `CostLimitExceeded` error, cost methods |
| `ergotree-interpreter/src/eval/costs.rs` | Rewrite | JitCost type, per-item cost helper |
| `ergotree-interpreter/src/eval/cost_accum.rs` | Modify | Update `CostError` to wrap `CostLimitExceeded` |
| `ergotree-interpreter/src/eval/expr.rs` | Modify | Add cost calls for Const, Global, Context in the central dispatch |
| `ergotree-interpreter/src/eval/bin_op.rs` | Modify | Type-based costs for ArithOp, fixed for Bit/Logical/Relation ops |
| `ergotree-interpreter/src/eval/*.rs` (~50 files) | Modify | Add `ctx.add_jit_cost(N)?` call to each eval impl |
| `ergotree-interpreter/src/eval.rs` | Modify | Wire cost into `reduce_to_crypto`, remove TODO comment |
| `ergotree-interpreter/src/sigma_protocol/verifier.rs` | Modify | Propagate `reduction_result.cost` to `VerificationResult.cost` |
| `ergo-lib/src/wallet/signing.rs` | Modify | Add cost fields to `make_context()`, `update_context()` |
| `ergo-lib/src/wallet/tx_context.rs` | Modify | Sum costs in `validate()`, pass cost limit |

---

### Task 1: Cost Types and Context Infrastructure

**Files:**
- Rewrite: `ergotree-interpreter/src/eval/costs.rs`
- Modify: `ergotree-ir/src/chain/context.rs`
- Modify: `ergotree-interpreter/src/eval/cost_accum.rs`
- Modify: `ergotree-interpreter/src/eval/error.rs`

- [ ] **Step 1: Rewrite costs.rs with JitCost and per-item cost helper**

```rust
// ergotree-interpreter/src/eval/costs.rs

extern crate derive_more;
use derive_more::{From, Into};

/// JIT cost unit. Values are in 10x scale relative to block costs.
/// To convert to block cost: divide by 10.
#[derive(PartialEq, Eq, Debug, Clone, Copy, From, Into)]
pub struct JitCost(pub u32);

impl JitCost {
    /// Convert JIT cost to block cost (divides by 10, rounding down)
    pub fn to_block_cost(self) -> u64 {
        self.0 as u64 / 10
    }
}

/// Compute per-item cost: base + ceil(n_items / chunk_size) * per_chunk
pub fn per_item_cost(base: u32, per_chunk: u32, chunk_size: u32, n_items: u32) -> u32 {
    let chunks = (n_items + chunk_size - 1) / chunk_size; // ceiling division
    base + chunks * per_chunk
}
```

- [ ] **Step 2: Add CostLimitExceeded and cost fields to Context**

In `ergotree-ir/src/chain/context.rs`, add the error type and fields:

```rust
// Add at the top of the file, after existing imports:
use core::fmt;

/// Error returned when JIT cost limit is exceeded during evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostLimitExceeded(pub u64);

impl fmt::Display for CostLimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JIT cost limit ({}) exceeded", self.0)
    }
}
```

Add two fields to the `Context` struct (after `extension_provider`):

```rust
    /// Accumulated JIT cost of evaluation
    pub jit_cost: Cell<u64>,
    /// JIT cost limit (None = unlimited, e.g. during signing)
    pub jit_cost_limit: Option<u64>,
```

Add methods to `impl Context`:

```rust
    /// Add JIT cost and check limit. Returns Err if limit exceeded.
    pub fn add_jit_cost(&self, amount: u32) -> Result<(), CostLimitExceeded> {
        let new = self.jit_cost.get() + amount as u64;
        self.jit_cost.set(new);
        if let Some(limit) = self.jit_cost_limit {
            if new > limit {
                return Err(CostLimitExceeded(limit));
            }
        }
        Ok(())
    }

    /// Add per-item JIT cost: base + ceil(n_items / chunk_size) * per_chunk
    pub fn add_per_item_jit_cost(
        &self,
        base: u32,
        per_chunk: u32,
        chunk_size: u32,
        n_items: u32,
    ) -> Result<(), CostLimitExceeded> {
        let chunks = (n_items + chunk_size - 1) / chunk_size;
        let cost = base + chunks * per_chunk;
        self.add_jit_cost(cost)
    }

    /// Read the accumulated JIT cost
    pub fn jit_cost_value(&self) -> u64 {
        self.jit_cost.get()
    }

    /// Reset JIT cost accumulator (used between input evaluations)
    pub fn reset_jit_cost(&self) {
        self.jit_cost.set(0);
    }
```

- [ ] **Step 3: Add jit_cost fields to all Context construction sites**

In the `Arbitrary` impl in `context.rs`, add to the struct literal at line 105:

```rust
                            jit_cost: Cell::new(0),
                            jit_cost_limit: None,
```

In `ergo-lib/src/wallet/signing.rs` `make_context()`, add to the `Context` struct literal at line 104:

```rust
        jit_cost: Cell::new(0),
        jit_cost_limit: None,
```

- [ ] **Step 4: Wire CostLimitExceeded into EvalError**

In `ergotree-interpreter/src/eval/cost_accum.rs`, replace the `CostError` enum:

```rust
use ergotree_ir::chain::context::CostLimitExceeded;
use thiserror::Error;

#[derive(Error, PartialEq, Eq, Debug, Clone)]
pub enum CostError {
    #[error("Cost limit exceeded: {0}")]
    LimitExceeded(#[from] CostLimitExceeded),
}
```

Remove the old `CostAccumulator` struct and its `use` of `Costs`/`Cost`/`Expr`. The file shrinks to just the error type.

The existing `EvalError::CostError(#[from] CostError)` in `error.rs` continues to work since `CostError` still exists. Also add a direct `From<CostLimitExceeded>` impl for convenience:

In `ergotree-interpreter/src/eval/error.rs`, add after the existing `CostError` variant (no new variant needed — just impl `From`):

```rust
// Add this impl after the EvalError enum definition
impl From<CostLimitExceeded> for EvalError {
    fn from(e: CostLimitExceeded) -> Self {
        EvalError::CostError(CostError::from(e))
    }
}
```

And add the import at the top:
```rust
use ergotree_ir::chain::context::CostLimitExceeded;
```

- [ ] **Step 5: Verify compilation**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter`
Expected: compiles (warnings about dead code in costs.rs are fine)

- [ ] **Step 6: Run existing tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all existing tests pass

- [ ] **Step 7: Commit**

```bash
git add ergotree-ir/src/chain/context.rs ergotree-interpreter/src/eval/costs.rs ergotree-interpreter/src/eval/cost_accum.rs ergotree-interpreter/src/eval/error.rs ergo-lib/src/wallet/signing.rs
git commit -m "feat: add JIT cost infrastructure to Context and cost types"
```

---

### Task 2: Cost Calls for Central Dispatch and GlobalVars

**Files:**
- Modify: `ergotree-interpreter/src/eval/expr.rs`
- Modify: `ergotree-interpreter/src/eval/global_vars.rs`

These are operations handled directly in the `Expr::eval` match or in GlobalVars, not delegated to individual struct impls.

- [ ] **Step 1: Add costs in expr.rs central dispatch**

In `ergotree-interpreter/src/eval/expr.rs`, the `Expr::eval` match at line 21 handles `Const`, `Global`, and `Context` directly. Add cost calls:

```rust
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let res = match self {
            Expr::Const(c) => {
                ctx.add_jit_cost(5)?; // Constant = Fixed(5)
                Ok(Value::from(c.v.clone()))
            }
            // ... all other arms unchanged ...
            Expr::Global => {
                ctx.add_jit_cost(5)?; // Global = Fixed(5)
                Ok(Value::Global)
            }
            Expr::Context => {
                ctx.add_jit_cost(1)?; // Context = Fixed(1)
                Ok(Value::Context)
            }
            // ... rest unchanged ...
        };
        res.enrich_err(self.span(), env)
    }
```

Add import at top of `expr.rs` (if not already there — `Context` is already imported via `super::Context`):
No new imports needed since `add_jit_cost` is a method on `Context` which is already in scope.

- [ ] **Step 2: Add costs in global_vars.rs**

In `ergotree-interpreter/src/eval/global_vars.rs`, add cost calls at the start of each match arm in the `Evaluable` impl for `GlobalVars`:

```rust
impl Evaluable for GlobalVars {
    fn eval<'ctx>(&self, _env: &mut Env, ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        match self {
            GlobalVars::Height => {
                ctx.add_jit_cost(26)?; // Height = Fixed(26)
                Ok((ctx.height as i32).into())
            }
            GlobalVars::SelfBox => {
                ctx.add_jit_cost(10)?; // Self = Fixed(10)
                Ok(Value::CBox(Ref::from(ctx.self_box)))
            }
            GlobalVars::Outputs => {
                ctx.add_jit_cost(10)?; // Outputs = Fixed(10)
                Ok(ctx.outputs.iter().map(Ref::Borrowed).collect::<Vec<_>>().into())
            }
            GlobalVars::Inputs => {
                ctx.add_jit_cost(10)?; // Inputs = Fixed(10)
                Ok(ctx.inputs.iter().map(|&i| Ref::Borrowed(i)).collect::<Vec<_>>().into())
            }
            GlobalVars::MinerPubKey => {
                ctx.add_jit_cost(20)?; // MinerPubkey = Fixed(20)
                Ok(ctx.pre_header.miner_pk.sigma_serialize_bytes()?.into())
            }
            GlobalVars::GroupGenerator => {
                ctx.add_jit_cost(10)?; // GroupGenerator = Fixed(10)
                Ok(ergo_chain_types::ec_point::generator().into())
            }
        }
    }
}
```

- [ ] **Step 3: Verify compilation and tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add ergotree-interpreter/src/eval/expr.rs ergotree-interpreter/src/eval/global_vars.rs
git commit -m "feat: add JIT costs to Const, Context, Global, and GlobalVars"
```

---

### Task 3: Cost Calls for Fixed-Cost Operations (Part 1 — Box Extractors, Options, Simple Ops)

**Files:**
- Modify: `ergotree-interpreter/src/eval/extract_amount.rs`
- Modify: `ergotree-interpreter/src/eval/extract_script_bytes.rs`
- Modify: `ergotree-interpreter/src/eval/extract_bytes.rs`
- Modify: `ergotree-interpreter/src/eval/extract_bytes_with_no_ref.rs`
- Modify: `ergotree-interpreter/src/eval/extract_id.rs`
- Modify: `ergotree-interpreter/src/eval/extract_reg_as.rs`
- Modify: `ergotree-interpreter/src/eval/extract_creation_info.rs`
- Modify: `ergotree-interpreter/src/eval/option_get.rs`
- Modify: `ergotree-interpreter/src/eval/option_get_or_else.rs`
- Modify: `ergotree-interpreter/src/eval/option_is_defined.rs`
- Modify: `ergotree-interpreter/src/eval/get_var.rs`
- Modify: `ergotree-interpreter/src/eval/select_field.rs`
- Modify: `ergotree-interpreter/src/eval/coll_by_index.rs`
- Modify: `ergotree-interpreter/src/eval/coll_size.rs`

For each file, add a single `ctx.add_jit_cost(N)?;` line as the **first line** of the `eval` method body (before any sub-expression evaluation).

- [ ] **Step 1: Add cost calls to all files listed above**

The pattern is identical for each — add one line at the start of `fn eval`:

| File | Cost | Comment |
|------|------|---------|
| `extract_amount.rs` | `ctx.add_jit_cost(8)?;` | ExtractAmount = Fixed(8) |
| `extract_script_bytes.rs` | `ctx.add_jit_cost(10)?;` | ExtractScriptBytes = Fixed(10) |
| `extract_bytes.rs` | `ctx.add_jit_cost(12)?;` | ExtractBytes = Fixed(12) |
| `extract_bytes_with_no_ref.rs` | `ctx.add_jit_cost(12)?;` | ExtractBytesWithNoRef = Fixed(12) |
| `extract_id.rs` | `ctx.add_jit_cost(12)?;` | ExtractId = Fixed(12) |
| `extract_reg_as.rs` | `ctx.add_jit_cost(50)?;` | ExtractRegisterAs = Fixed(50) |
| `extract_creation_info.rs` | `ctx.add_jit_cost(16)?;` | ExtractCreationInfo = Fixed(16) |
| `option_get.rs` | `ctx.add_jit_cost(15)?;` | OptionGet = Fixed(15) |
| `option_get_or_else.rs` | `ctx.add_jit_cost(20)?;` | OptionGetOrElse = Fixed(20) |
| `option_is_defined.rs` | `ctx.add_jit_cost(10)?;` | OptionIsDefined = Fixed(10) |
| `get_var.rs` | `ctx.add_jit_cost(10)?;` | GetVar = Fixed(10) |
| `select_field.rs` | `ctx.add_jit_cost(10)?;` | SelectField = Fixed(10) |
| `coll_by_index.rs` | `ctx.add_jit_cost(30)?;` | ByIndex = Fixed(30) |
| `coll_size.rs` | `ctx.add_jit_cost(14)?;` | SizeOf = Fixed(14) |

Example — `extract_amount.rs` changes from:

```rust
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
```

to:

```rust
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        ctx.add_jit_cost(8)?; // ExtractAmount = Fixed(8)
        let input_v = self.input.eval(env, ctx)?;
```

Apply the same pattern to all 14 files.

- [ ] **Step 2: Verify compilation and tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all pass

- [ ] **Step 3: Commit**

```bash
git add ergotree-interpreter/src/eval/extract_amount.rs ergotree-interpreter/src/eval/extract_script_bytes.rs ergotree-interpreter/src/eval/extract_bytes.rs ergotree-interpreter/src/eval/extract_bytes_with_no_ref.rs ergotree-interpreter/src/eval/extract_id.rs ergotree-interpreter/src/eval/extract_reg_as.rs ergotree-interpreter/src/eval/extract_creation_info.rs ergotree-interpreter/src/eval/option_get.rs ergotree-interpreter/src/eval/option_get_or_else.rs ergotree-interpreter/src/eval/option_is_defined.rs ergotree-interpreter/src/eval/get_var.rs ergotree-interpreter/src/eval/select_field.rs ergotree-interpreter/src/eval/coll_by_index.rs ergotree-interpreter/src/eval/coll_size.rs
git commit -m "feat: add JIT costs to box extractors, options, and simple fixed-cost ops"
```

---

### Task 4: Cost Calls for Fixed-Cost Operations (Part 2 — Crypto, Logic, Control Flow)

**Files:**
- Modify: `ergotree-interpreter/src/eval/bool_to_sigma.rs`
- Modify: `ergotree-interpreter/src/eval/logical_not.rs`
- Modify: `ergotree-interpreter/src/eval/create_provedlog.rs`
- Modify: `ergotree-interpreter/src/eval/create_prove_dh_tuple.rs`
- Modify: `ergotree-interpreter/src/eval/if_op.rs`
- Modify: `ergotree-interpreter/src/eval/apply.rs`
- Modify: `ergotree-interpreter/src/eval/func_value.rs`
- Modify: `ergotree-interpreter/src/eval/val_use.rs`
- Modify: `ergotree-interpreter/src/eval/tuple.rs`
- Modify: `ergotree-interpreter/src/eval/collection.rs`
- Modify: `ergotree-interpreter/src/eval/negation.rs`
- Modify: `ergotree-interpreter/src/eval/bit_inversion.rs`
- Modify: `ergotree-interpreter/src/eval/decode_point.rs`
- Modify: `ergotree-interpreter/src/eval/long_to_byte_array.rs`
- Modify: `ergotree-interpreter/src/eval/byte_array_to_long.rs`
- Modify: `ergotree-interpreter/src/eval/byte_array_to_bigint.rs`
- Modify: `ergotree-interpreter/src/eval/exponentiate.rs`
- Modify: `ergotree-interpreter/src/eval/multiply_group.rs`
- Modify: `ergotree-interpreter/src/eval/method_call.rs`
- Modify: `ergotree-interpreter/src/eval/property_call.rs`

- [ ] **Step 1: Add cost calls to all files**

Same pattern as Task 3 — add `ctx.add_jit_cost(N)?;` as first line of `eval`:

| File | Cost | Comment |
|------|------|---------|
| `bool_to_sigma.rs` | `ctx.add_jit_cost(15)?;` | BoolToSigmaProp = Fixed(15) |
| `logical_not.rs` | `ctx.add_jit_cost(15)?;` | LogicalNot = Fixed(15) |
| `create_provedlog.rs` | `ctx.add_jit_cost(10)?;` | CreateProveDlog = Fixed(10) |
| `create_prove_dh_tuple.rs` | `ctx.add_jit_cost(20)?;` | CreateProveDHTuple = Fixed(20) |
| `if_op.rs` | `ctx.add_jit_cost(10)?;` | If = Fixed(10) |
| `apply.rs` | `ctx.add_jit_cost(30)?;` | Apply = Fixed(30) |
| `func_value.rs` | `ctx.add_jit_cost(5)?;` | FuncValue = Fixed(5) |
| `val_use.rs` | `ctx.add_jit_cost(5)?;` | ValUse = Fixed(5) |
| `tuple.rs` | `ctx.add_jit_cost(15)?;` | Tuple = Fixed(15) |
| `collection.rs` | `ctx.add_jit_cost(20)?;` | ConcreteCollection = Fixed(20) |
| `negation.rs` | `ctx.add_jit_cost(30)?;` | Negation = Fixed(30) |
| `bit_inversion.rs` | `ctx.add_jit_cost(1)?;` | BitOp = Fixed(1) |
| `decode_point.rs` | `ctx.add_jit_cost(300)?;` | DecodePoint = Fixed(300) |
| `long_to_byte_array.rs` | `ctx.add_jit_cost(17)?;` | LongToByteArray = Fixed(17) |
| `byte_array_to_long.rs` | `ctx.add_jit_cost(16)?;` | ByteArrayToLong = Fixed(16) |
| `byte_array_to_bigint.rs` | `ctx.add_jit_cost(30)?;` | ByteArrayToBigInt = Fixed(30) |
| `exponentiate.rs` | `ctx.add_jit_cost(900)?;` | Exponentiate = Fixed(900) |
| `multiply_group.rs` | `ctx.add_jit_cost(40)?;` | MultiplyGroup = Fixed(40) |
| `method_call.rs` | `ctx.add_jit_cost(4)?;` | MethodCall = Fixed(4) |
| `property_call.rs` | `ctx.add_jit_cost(4)?;` | PropertyCall = Fixed(4) |

- [ ] **Step 2: Verify compilation and tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all pass

- [ ] **Step 3: Commit**

```bash
git add ergotree-interpreter/src/eval/bool_to_sigma.rs ergotree-interpreter/src/eval/logical_not.rs ergotree-interpreter/src/eval/create_provedlog.rs ergotree-interpreter/src/eval/create_prove_dh_tuple.rs ergotree-interpreter/src/eval/if_op.rs ergotree-interpreter/src/eval/apply.rs ergotree-interpreter/src/eval/func_value.rs ergotree-interpreter/src/eval/val_use.rs ergotree-interpreter/src/eval/tuple.rs ergotree-interpreter/src/eval/collection.rs ergotree-interpreter/src/eval/negation.rs ergotree-interpreter/src/eval/bit_inversion.rs ergotree-interpreter/src/eval/decode_point.rs ergotree-interpreter/src/eval/long_to_byte_array.rs ergotree-interpreter/src/eval/byte_array_to_long.rs ergotree-interpreter/src/eval/byte_array_to_bigint.rs ergotree-interpreter/src/eval/exponentiate.rs ergotree-interpreter/src/eval/multiply_group.rs ergotree-interpreter/src/eval/method_call.rs ergotree-interpreter/src/eval/property_call.rs
git commit -m "feat: add JIT costs to crypto, logic, control flow, and method/property calls"
```

---

### Task 5: Cost Calls for Per-Item Operations

**Files:**
- Modify: `ergotree-interpreter/src/eval/and.rs`
- Modify: `ergotree-interpreter/src/eval/or.rs`
- Modify: `ergotree-interpreter/src/eval/xor_of.rs`
- Modify: `ergotree-interpreter/src/eval/xor.rs`
- Modify: `ergotree-interpreter/src/eval/atleast.rs`
- Modify: `ergotree-interpreter/src/eval/sigma_and.rs`
- Modify: `ergotree-interpreter/src/eval/sigma_or.rs`
- Modify: `ergotree-interpreter/src/eval/coll_map.rs`
- Modify: `ergotree-interpreter/src/eval/coll_append.rs`
- Modify: `ergotree-interpreter/src/eval/coll_slice.rs`
- Modify: `ergotree-interpreter/src/eval/coll_filter.rs`
- Modify: `ergotree-interpreter/src/eval/coll_exists.rs`
- Modify: `ergotree-interpreter/src/eval/coll_forall.rs`
- Modify: `ergotree-interpreter/src/eval/coll_fold.rs`
- Modify: `ergotree-interpreter/src/eval/sigma_prop_bytes.rs`
- Modify: `ergotree-interpreter/src/eval/calc_blake2b256.rs`
- Modify: `ergotree-interpreter/src/eval/calc_sha256.rs`
- Modify: `ergotree-interpreter/src/eval/subst_const.rs`
- Modify: `ergotree-interpreter/src/eval/block.rs`

Per-item costs must be added **after** the input is evaluated (so we know the item count), but **before** the main operation is performed.

- [ ] **Step 1: Add per-item costs to collection operations**

For each file, add `ctx.add_per_item_jit_cost(base, per_chunk, chunk_size, n_items)?;` after evaluating the input to determine item count.

**and.rs** — AND = PerItem(10, 5, 32):
```rust
    fn eval<'ctx>(&self, env: &mut Env<'ctx>, ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bools = input_v.try_extract_into::<Vec<bool>>()?;
        ctx.add_per_item_jit_cost(10, 5, 32, input_v_bools.len() as u32)?;
        Ok(input_v_bools.iter().all(|b| *b).into())
    }
```

**or.rs** — OR = PerItem(5, 5, 64):
```rust
    fn eval<'ctx>(&self, env: &mut Env<'ctx>, ctx: &Context<'ctx>) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        let input_v_bools = input_v.try_extract_into::<Vec<bool>>()?;
        ctx.add_per_item_jit_cost(5, 5, 64, input_v_bools.len() as u32)?;
        Ok(input_v_bools.iter().any(|b| *b).into())
    }
```

**xor_of.rs** — XorOf = PerItem(20, 5, 32):
Add `ctx.add_per_item_jit_cost(20, 5, 32, n_items)?;` after extracting the bool Vec, using `input_v_bools.len() as u32` as n_items.

**xor.rs** — Xor = PerItem(10, 2, 128):
Add `ctx.add_per_item_jit_cost(10, 2, 128, n_items)?;` after evaluating left/right, using the byte collection length as n_items.

**atleast.rs** — AtLeast = PerItem(20, 3, 5):
Add `ctx.add_per_item_jit_cost(20, 3, 5, n_items)?;` after evaluating the input collection, using its length as n_items.

**sigma_and.rs** — SigmaAnd = PerItem(10, 2, 1):
Add `ctx.add_per_item_jit_cost(10, 2, 1, self.items.len() as u32)?;` at the start of eval (item count is known from the AST node).

**sigma_or.rs** — SigmaOr = PerItem(10, 2, 1):
Add `ctx.add_per_item_jit_cost(10, 2, 1, self.items.len() as u32)?;` at the start of eval.

**coll_map.rs** — MapCollection = PerItem(20, 1, 10):
Add `ctx.add_per_item_jit_cost(20, 1, 10, n_items)?;` after evaluating the input collection, using its length as n_items.

**coll_append.rs** — Append = PerItem(20, 2, 100):
Add `ctx.add_per_item_jit_cost(20, 2, 100, n_items)?;` after evaluating both collections, using the sum of their lengths as n_items.

**coll_slice.rs** — Slice = PerItem(10, 2, 100):
Add `ctx.add_per_item_jit_cost(10, 2, 100, n_items)?;` after evaluating the input, using the input collection length as n_items.

**coll_filter.rs** — Filter = PerItem(20, 1, 10):
Add `ctx.add_per_item_jit_cost(20, 1, 10, n_items)?;` after evaluating the input collection.

**coll_exists.rs** — Exists = PerItem(3, 1, 10):
Add `ctx.add_per_item_jit_cost(3, 1, 10, n_items)?;` after evaluating the input collection.

**coll_forall.rs** — ForAll = PerItem(3, 1, 10):
Add `ctx.add_per_item_jit_cost(3, 1, 10, n_items)?;` after evaluating the input collection.

**coll_fold.rs** — Fold = PerItem(3, 1, 10):
Add `ctx.add_per_item_jit_cost(3, 1, 10, n_items)?;` after evaluating the input collection.

**sigma_prop_bytes.rs** — SigmaPropBytes = PerItem(35, 6, 1):
Add `ctx.add_per_item_jit_cost(35, 6, 1, 1)?;` at the start of eval (single item).

**calc_blake2b256.rs** — CalcBlake2b256 = PerItem(20, 7, 128):
Add `ctx.add_per_item_jit_cost(20, 7, 128, n_items)?;` after evaluating input, using byte array length as n_items.

**calc_sha256.rs** — CalcSha256 = PerItem(80, 8, 64):
Add `ctx.add_per_item_jit_cost(80, 8, 64, n_items)?;` after evaluating input, using byte array length as n_items.

**subst_const.rs** — SubstConstants = PerItem(100, 100, 1):
Add `ctx.add_per_item_jit_cost(100, 100, 1, n_items)?;` after evaluating positions, using positions length as n_items.

**block.rs** — BlockValue = PerItem(1, 1, 10):
Add `ctx.add_per_item_jit_cost(1, 1, 10, self.items.len() as u32)?;` at the start of eval (item count known from AST).

- [ ] **Step 2: Verify compilation and tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all pass

- [ ] **Step 3: Commit**

```bash
git add ergotree-interpreter/src/eval/and.rs ergotree-interpreter/src/eval/or.rs ergotree-interpreter/src/eval/xor_of.rs ergotree-interpreter/src/eval/xor.rs ergotree-interpreter/src/eval/atleast.rs ergotree-interpreter/src/eval/sigma_and.rs ergotree-interpreter/src/eval/sigma_or.rs ergotree-interpreter/src/eval/coll_map.rs ergotree-interpreter/src/eval/coll_append.rs ergotree-interpreter/src/eval/coll_slice.rs ergotree-interpreter/src/eval/coll_filter.rs ergotree-interpreter/src/eval/coll_exists.rs ergotree-interpreter/src/eval/coll_forall.rs ergotree-interpreter/src/eval/coll_fold.rs ergotree-interpreter/src/eval/sigma_prop_bytes.rs ergotree-interpreter/src/eval/calc_blake2b256.rs ergotree-interpreter/src/eval/calc_sha256.rs ergotree-interpreter/src/eval/subst_const.rs ergotree-interpreter/src/eval/block.rs
git commit -m "feat: add JIT per-item costs to collection, hash, and block operations"
```

---

### Task 6: Type-Based and Dynamic Costs (BinOp, Upcast, Downcast)

**Files:**
- Modify: `ergotree-interpreter/src/eval/bin_op.rs`
- Modify: `ergotree-interpreter/src/eval/upcast.rs`
- Modify: `ergotree-interpreter/src/eval/downcast.rs`

These operations have costs that depend on the argument type (BigInt costs more).

- [ ] **Step 1: Add type-based costs to bin_op.rs**

In `ergotree-interpreter/src/eval/bin_op.rs`, the `Evaluable` impl for `BinOp` at line 184. Add cost after evaluating `lv` (so we can check its type) but before the operation:

```rust
impl Evaluable for BinOp {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let lv = self.left.eval(env, ctx)?;

        // Add type-based cost based on operation kind and left value type
        let is_bigint = matches!(lv, Value::BigInt(_) | Value::UnsignedBigInt(_));
        match self.kind {
            BinOpKind::Arith(op) => match op {
                ArithOp::Plus | ArithOp::Minus => {
                    ctx.add_jit_cost(if is_bigint { 20 } else { 15 })?;
                }
                ArithOp::Multiply | ArithOp::Divide | ArithOp::Modulo => {
                    ctx.add_jit_cost(if is_bigint { 25 } else { 15 })?;
                }
                ArithOp::Max | ArithOp::Min => {
                    ctx.add_jit_cost(if is_bigint { 10 } else { 5 })?;
                }
            },
            BinOpKind::Relation(op) => match op {
                // EQ and NEQ are Dynamic — no cost at this node
                RelationOp::Eq | RelationOp::NEq => {}
                // LT, LE, GT, GE = Fixed(20) regardless of type
                _ => { ctx.add_jit_cost(20)?; }
            },
            BinOpKind::Logical(_) => {
                // BinOr, BinAnd, BinXor = Fixed(20)
                ctx.add_jit_cost(20)?;
            }
            BinOpKind::Bit(_) => {
                // BitOp (all 6) = Fixed(1)
                ctx.add_jit_cost(1)?;
            }
        }

        // existing logic unchanged from here:
        let mut rv = || self.right.eval(env, ctx);
        match self.kind {
            // ... rest unchanged
```

Remove the commented-out line `//ctx.cost_accum.add(Costs::DEFAULT.eq_const_size)?;` at line 190.

- [ ] **Step 2: Add type-based costs to upcast.rs**

In `ergotree-interpreter/src/eval/upcast.rs`, add cost after evaluating input but before the cast:

```rust
impl Evaluable for Upcast {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        // Upcast: TypeBased(bigint=30, other=10)
        ctx.add_jit_cost(if self.tpe == SType::SBigInt { 30 } else { 10 })?;
        match self.tpe {
            // ... existing match unchanged
```

- [ ] **Step 3: Add type-based costs to downcast.rs**

Same pattern as upcast. In `ergotree-interpreter/src/eval/downcast.rs`:

```rust
impl Evaluable for Downcast {
    fn eval<'ctx>(
        &self,
        env: &mut Env<'ctx>,
        ctx: &Context<'ctx>,
    ) -> Result<Value<'ctx>, EvalError> {
        let input_v = self.input.eval(env, ctx)?;
        // Downcast: TypeBased(bigint=30, other=10)
        ctx.add_jit_cost(if self.tpe == SType::SBigInt { 30 } else { 10 })?;
        match self.tpe {
            // ... existing match unchanged
```

- [ ] **Step 4: Verify compilation and tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all pass

- [ ] **Step 5: Commit**

```bash
git add ergotree-interpreter/src/eval/bin_op.rs ergotree-interpreter/src/eval/upcast.rs ergotree-interpreter/src/eval/downcast.rs
git commit -m "feat: add JIT type-based costs to BinOp, Upcast, and Downcast"
```

---

### Task 7: Method Costs

**Files:**
- Modify: `ergotree-interpreter/src/eval/sbox.rs`
- Modify: `ergotree-interpreter/src/eval/sgroup_elem.rs`
- Modify: `ergotree-interpreter/src/eval/savltree.rs`
- Modify: `ergotree-interpreter/src/eval/scoll.rs`
- Modify: `ergotree-interpreter/src/eval/scontext.rs`
- Modify: `ergotree-interpreter/src/eval/soption.rs`
- Modify: `ergotree-interpreter/src/eval/sheader.rs`
- Modify: `ergotree-interpreter/src/eval/spreheader.rs`
- Modify: `ergotree-interpreter/src/eval/sglobal.rs`

Method eval functions are `EvalFn` function pointers. They receive `ctx` as a parameter. Each method needs its cost added at the start of its function body. The MethodCall/PropertyCall Fixed(4) is already handled in Task 4.

- [ ] **Step 1: Add costs to sbox.rs methods**

Each eval fn const is a closure. Add `ctx.add_jit_cost(N)?;` as the first line of each function body:

| Eval fn | Cost | Comment |
|---------|------|---------|
| `VALUE_EVAL_FN` | `ctx.add_jit_cost(8)?;` | SBox.value = Fixed(8) (same as ExtractAmount) |
| `GET_REG_EVAL_FN` | `ctx.add_jit_cost(50)?;` | SBox.getReg = Fixed(50) |
| `TOKENS_EVAL_FN` | `ctx.add_jit_cost(15)?;` | SBox.tokens = Fixed(15) |

- [ ] **Step 2: Add costs to sgroup_elem.rs methods**

| Eval fn | Cost | Comment |
|---------|------|---------|
| `GET_ENCODED_EVAL_FN` | `ctx.add_jit_cost(250)?;` | SGroupElement.getEncoded = Fixed(250) |
| `NEGATE_EVAL_FN` | `ctx.add_jit_cost(45)?;` | SGroupElement.negate = Fixed(45) |
| `EXPONENTIATE_EVAL_FN` | `ctx.add_jit_cost(900)?;` | Same as Exponentiate = Fixed(900) |
| `MULTIPLY_EVAL_FN` | `ctx.add_jit_cost(40)?;` | Same as MultiplyGroup = Fixed(40) |
| `EXPONENTIATE_UNSIGNED_EVAL_FN` | `ctx.add_jit_cost(900)?;` | Same as Exponentiate |

- [ ] **Step 3: Add costs to savltree.rs methods**

| Eval fn | Cost | Comment |
|---------|------|---------|
| `DIGEST_EVAL_FN` | `ctx.add_jit_cost(15)?;` | SAvlTree.digest = Fixed(15) |
| `ENABLED_OPERATIONS_EVAL_FN` | `ctx.add_jit_cost(15)?;` | Fixed(15) |
| `KEY_LENGTH_EVAL_FN` | `ctx.add_jit_cost(15)?;` | Fixed(15) |
| `VALUE_LENGTH_OPT_EVAL_FN` | `ctx.add_jit_cost(15)?;` | Fixed(15) |
| `IS_INSERT_ALLOWED_EVAL_FN` | `ctx.add_jit_cost(15)?;` | Fixed(15) |
| `IS_UPDATE_ALLOWED_EVAL_FN` | `ctx.add_jit_cost(15)?;` | Fixed(15) |
| `IS_REMOVE_ALLOWED_EVAL_FN` | `ctx.add_jit_cost(15)?;` | Fixed(15) |
| `UPDATE_OPERATIONS_EVAL_FN` | `ctx.add_jit_cost(45)?;` | SAvlTree.updateOperations = Fixed(45) |
| `UPDATE_DIGEST_EVAL_FN` | `ctx.add_jit_cost(40)?;` | SAvlTree.updateDigest = Fixed(40) |
| `GET_EVAL_FN` | (none) | Dynamic — sub-ops handle cost |
| `GET_MANY_EVAL_FN` | (none) | Dynamic |
| `INSERT_EVAL_FN` | (none) | Dynamic |
| `CONTAINS_EVAL_FN` | (none) | Dynamic |
| `REMOVE_EVAL_FN` | (none) | Dynamic |
| `UPDATE_EVAL_FN` | (none) | Dynamic |
| `INSERT_OR_UPDATE_EVAL_FN` | (none) | Dynamic |

- [ ] **Step 4: Add costs to scoll.rs methods**

| Eval fn | Cost | Comment |
|---------|------|---------|
| `INDEX_OF_EVAL_FN` | Per-item: `ctx.add_per_item_jit_cost(20, 10, 2, n)?;` after extracting collection | SCollection.indexOf |
| `FLATMAP_EVAL_FN` (`flatmap_eval`) | Per-item: `ctx.add_per_item_jit_cost(60, 10, 8, n)?;` | SCollection.flatMap |
| `ZIP_EVAL_FN` | Per-item: `ctx.add_per_item_jit_cost(10, 1, 10, n)?;` | SCollection.zip |
| `INDICES_EVAL_FN` | Per-item: `ctx.add_per_item_jit_cost(20, 2, 16, n)?;` | SCollection.indices |
| `PATCH_EVAL_FN` | Per-item: `ctx.add_per_item_jit_cost(30, 2, 10, n)?;` | SCollection.patch |
| `UPDATED_EVAL_FN` | Per-item: `ctx.add_per_item_jit_cost(20, 1, 10, n)?;` | SCollection.updated |
| `UPDATE_MANY_EVAL_FN` | Per-item: same as `updated` | |
| `REVERSE_EVAL_FN` | `ctx.add_jit_cost(20)?;` | No specific cost in spec, use Fixed(20) |
| `STARTS_WITH_EVAL_FN` | `ctx.add_jit_cost(20)?;` | No specific cost in spec, use Fixed(20) |
| `ENDS_WITH_EVAL_FN` | `ctx.add_jit_cost(20)?;` | No specific cost in spec, use Fixed(20) |
| `GET_EVAL_FN` | `ctx.add_jit_cost(30)?;` | Same as ByIndex = Fixed(30) |

For each per-item method, the `n` value is the collection length extracted from the `obj` parameter. Add the cost call after extracting the collection from `obj`.

- [ ] **Step 5: Add costs to scontext.rs methods**

| Eval fn | Cost | Comment |
|---------|------|---------|
| `DATA_INPUTS_EVAL_FN` | `ctx.add_jit_cost(15)?;` | SContext.dataInputs = Fixed(15) |
| `SELF_BOX_INDEX_EVAL_FN` | `ctx.add_jit_cost(20)?;` | SContext.selfBoxIndex = Fixed(20) |
| `HEADERS_EVAL_FN` | `ctx.add_jit_cost(15)?;` | SContext.headers = Fixed(15) |
| `PRE_HEADER_EVAL_FN` | `ctx.add_jit_cost(15)?;` | SContext.preHeader = Fixed(15) |
| `LAST_BLOCK_UTXO_ROOT_HASH_EVAL_FN` | `ctx.add_jit_cost(15)?;` | Fixed(15) |
| `MINER_PUBKEY_EVAL_FN` | `ctx.add_jit_cost(20)?;` | Fixed(20) |
| `GET_VAR_FROM_INPUT_EVAL_FN` | `ctx.add_jit_cost(10)?;` | Fixed(10) |

- [ ] **Step 6: Add costs to soption.rs methods**

| Function | Cost | Comment |
|----------|------|---------|
| `map_eval` | `ctx.add_jit_cost(20)?;` | SOption.map = Fixed(20) |
| `filter_eval` | `ctx.add_jit_cost(20)?;` | SOption.filter = Fixed(20) |

- [ ] **Step 7: Add costs to sheader.rs methods**

All header field accessors get Fixed(10). `CHECK_POW_EVAL_FN` gets Fixed(700):

```
VERSION, ID, PARENT_ID, AD_PROOFS_ROOT, STATE_ROOT,
TRANSACTION_ROOT, EXTENSION_ROOT, TIMESTAMP, N_BITS,
HEIGHT, MINER_PK, POW_ONETIME_PK, POW_DISTANCE,
POW_NONCE, VOTES → ctx.add_jit_cost(10)?;

CHECK_POW → ctx.add_jit_cost(700)?;
```

- [ ] **Step 8: Add costs to spreheader.rs methods**

All pre-header field accessors get Fixed(10):

```
VERSION, PARENT_ID, TIMESTAMP, N_BITS, HEIGHT,
MINER_PK, VOTES → ctx.add_jit_cost(10)?;
```

- [ ] **Step 9: Add costs to sglobal.rs methods**

| Eval fn | Cost | Comment |
|---------|------|---------|
| `GROUP_GENERATOR_EVAL_FN` | `ctx.add_jit_cost(10)?;` | Fixed(10) |
| `XOR_EVAL_FN` | `ctx.add_jit_cost(10)?;` | Fixed(10) |
| `DESERIALIZE_EVAL_FN` | Per-item: `ctx.add_per_item_jit_cost(100, 32, 32, n)?;` | SGlobal.deserializeTo, n = byte length |
| `SERIALIZE_EVAL_FN` | `ctx.add_jit_cost(10)?;` | Fixed(10) |
| `SGLOBAL_FROM_BIGENDIAN_BYTES_EVAL_FN` | `ctx.add_jit_cost(10)?;` | Fixed(10) |
| `SGLOBAL_SOME_EVAL_FN` | `ctx.add_jit_cost(5)?;` | Fixed(5) |
| `SGLOBAL_NONE_EVAL_FN` | `ctx.add_jit_cost(5)?;` | Fixed(5) |
| `ENCODE_NBITS_EVAL_FN` | `ctx.add_jit_cost(10)?;` | Fixed(10) |
| `DECODE_NBITS_EVAL_FN` | `ctx.add_jit_cost(10)?;` | Fixed(10) |
| `POW_HIT_EVAL_FN` | `ctx.add_jit_cost(900)?;` | Fixed(900) — compute-intensive |

- [ ] **Step 10: Verify compilation and tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all pass

- [ ] **Step 11: Commit**

```bash
git add ergotree-interpreter/src/eval/sbox.rs ergotree-interpreter/src/eval/sgroup_elem.rs ergotree-interpreter/src/eval/savltree.rs ergotree-interpreter/src/eval/scoll.rs ergotree-interpreter/src/eval/scontext.rs ergotree-interpreter/src/eval/soption.rs ergotree-interpreter/src/eval/sheader.rs ergotree-interpreter/src/eval/spreheader.rs ergotree-interpreter/src/eval/sglobal.rs
git commit -m "feat: add JIT costs to all method dispatch eval functions"
```

---

### Task 8: Cost Propagation Through reduce_to_crypto and Verifier

**Files:**
- Modify: `ergotree-interpreter/src/eval.rs`
- Modify: `ergotree-interpreter/src/sigma_protocol/verifier.rs`

- [ ] **Step 1: Wire cost into reduce_to_crypto**

In `ergotree-interpreter/src/eval.rs`, modify the `inner` function at line 131 to read `ctx.jit_cost_value()` and convert to block cost:

```rust
    fn inner<'ctx>(expr: &'ctx Expr, ctx: &Context<'ctx>) -> Result<ReductionResult, EvalError> {
        let mut env_mut = Env::empty();
        ctx.reset_jit_cost(); // ensure clean start
        expr.eval(&mut env_mut, ctx)
            .and_then(|v| -> Result<ReductionResult, EvalError> {
                let cost = ctx.jit_cost_value() / 10; // convert JitCost to block cost
                match v {
                    Value::Boolean(b) => Ok(ReductionResult {
                        sigma_prop: SigmaBoolean::TrivialProp(b),
                        cost,
                        diag: ReductionDiagnosticInfo {
                            env: env_mut.to_static(),
                            pretty_printed_expr: None,
                        },
                    }),
                    Value::SigmaProp(sp) => Ok(ReductionResult {
                        sigma_prop: sp.value().clone(),
                        cost,
                        diag: ReductionDiagnosticInfo {
                            env: env_mut.to_static(),
                            pretty_printed_expr: None,
                        },
                    }),
                    _ => Err(EvalError::InvalidResultType),
                }
            })
    }
```

Also remove the TODO comment on line 204:
```rust
    // TODO for JIT costing: cost_accum: &mut CostAccumulator,
```
Replace with:
```rust
    // JIT costing is handled via ctx.add_jit_cost()
```

- [ ] **Step 2: Wire cost into verifier**

In `ergotree-interpreter/src/sigma_protocol/verifier.rs`, the `verify` method at line 82 already constructs `VerificationResult`. Change `cost: 0` to use `reduction_result.cost`:

```rust
        Ok(VerificationResult {
            result: res,
            cost: reduction_result.cost,
            diag: reduction_result.diag,
        })
```

- [ ] **Step 3: Verify compilation and tests**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check -p ergotree-interpreter && cargo test -p ergotree-interpreter -- --test-threads=4`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add ergotree-interpreter/src/eval.rs ergotree-interpreter/src/sigma_protocol/verifier.rs
git commit -m "feat: propagate JIT cost through reduce_to_crypto and verifier"
```

---

### Task 9: Transaction Validation Cost Limit and Accumulation

**Files:**
- Modify: `ergo-lib/src/wallet/signing.rs`
- Modify: `ergo-lib/src/wallet/tx_context.rs`

- [ ] **Step 1: Pass cost limit into make_context for validation**

In `ergo-lib/src/wallet/signing.rs`, modify `make_context` to accept an optional cost limit:

```rust
pub fn make_context<'ctx, T: ErgoTransaction>(
    state_ctx: &'ctx ErgoStateContext,
    tx_ctx: &'ctx TransactionContext<T>,
    self_index: usize,
) -> Result<Context<'ctx>, TransactionContextError> {
```

Add cost fields to the `Context` struct literal at line 104:

```rust
    Ok(Context {
        height,
        self_box,
        outputs,
        data_inputs: data_inputs_ir,
        inputs: inputs_ir,
        pre_header: state_ctx.pre_header.clone(),
        extension,
        headers: state_ctx.headers.clone(),
        tree_version: Default::default(),
        extension_provider: &tx_ctx.spending_tx,
        jit_cost: Cell::new(0),
        jit_cost_limit: None,
    })
```

Add a `use core::cell::Cell;` import at the top of signing.rs if not present.

- [ ] **Step 2: Reset cost and set limit in validate()**

In `ergo-lib/src/wallet/tx_context.rs`, modify the `validate` method. Before the input verification loop, set the cost limit on the context. After each input, reset the cost:

```rust
    pub fn validate(&self, state_context: &ErgoStateContext) -> Result<u64, TxValidationError> {
        // ... existing checks (input_sum, output_sum, assets, etc.) unchanged ...

        // Verify input proofs with cost tracking
        let bytes_to_sign = self.spending_tx.bytes_to_sign()?;
        let mut context = make_context(state_context, self, 0)?;
        // Set cost limit per-script: MaxBlockCost * 10 (convert block cost to JitCost scale)
        context.jit_cost_limit = Some(state_context.parameters.max_block_cost() as u64 * 10);
        let mut total_cost: u64 = 0;
        for input_idx in 0..self.spending_tx.inputs.len() {
            context.reset_jit_cost();
            match verify_tx_input_proof(self, &mut context, state_context, input_idx, &bytes_to_sign)? {
                res @ VerificationResult { result: false, .. } => {
                    return Err(TxValidationError::ReducedToFalse(input_idx, res));
                }
                VerificationResult { cost, .. } => {
                    total_cost += cost;
                }
            }
        }
        Ok(total_cost)
    }
```

Note: the return type changes from `Result<(), TxValidationError>` to `Result<u64, TxValidationError>`. Update callers if any exist — check for all call sites of `.validate()`.

- [ ] **Step 3: Update validate() callers**

Search for all call sites of `.validate()` and update them to handle the new `u64` return. Common patterns:
- `tx_ctx.validate(&state_ctx)?` → still works, just discards the cost
- `tx_ctx.validate(&state_ctx).unwrap()` → still works
- Tests that check `validate().is_ok()` → still works

- [ ] **Step 4: Verify compilation and tests across all crates**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check && cargo test -- --test-threads=4`
Expected: all pass (may need to fix callers if any match on `Ok(())`)

- [ ] **Step 5: Commit**

```bash
git add ergo-lib/src/wallet/signing.rs ergo-lib/src/wallet/tx_context.rs
git commit -m "feat: wire cost limit into transaction validation and accumulate total cost"
```

---

### Task 10: Tests

**Files:**
- Modify: `ergotree-interpreter/src/eval.rs` (test module at bottom)

- [ ] **Step 1: Write test for trivial prop cost**

Add to the `mod test` block at the bottom of `ergotree-interpreter/src/eval.rs`:

```rust
    #[test]
    fn jit_cost_trivial_prop() {
        // { true } → Constant(5) = JitCost(5) → block cost 0 (5/10 rounds down)
        let tree = ErgoTree::try_from(Expr::Const(true.into())).unwrap();
        let ctx = force_any_val::<Context>();
        let res = reduce_to_crypto(&tree, &ctx).unwrap();
        assert_eq!(res.sigma_prop, SigmaBoolean::TrivialProp(true));
        assert_eq!(res.cost, 0); // JitCost 5 / 10 = 0
    }
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo test -p ergotree-interpreter -- jit_cost_trivial_prop -v`
Expected: PASS

- [ ] **Step 3: Write test for SELF.value cost**

```rust
    #[test]
    fn jit_cost_self_value() {
        // SELF.value → Self(10) + ExtractAmount(8) = JitCost(18) → block cost 1
        use ergotree_ir::mir::extract_amount::ExtractAmount;
        use ergotree_ir::mir::global_vars::GlobalVars;

        let expr: Expr = ExtractAmount {
            input: Box::new(GlobalVars::SelfBox.into()),
        }.into();
        let tree = ErgoTree::try_from(
            Expr::BoolToSigmaProp(
                ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp {
                    input: Box::new(
                        ergotree_ir::mir::bin_op::BinOp {
                            kind: ergotree_ir::mir::bin_op::BinOpKind::Relation(
                                ergotree_ir::mir::bin_op::RelationOp::Gt,
                            ),
                            left: Box::new(expr),
                            right: Box::new(Expr::Const(0i64.into())),
                        }.into(),
                    ),
                }.into(),
            )
        ).unwrap();
        let ctx = force_any_val::<Context>();
        let res = reduce_to_crypto(&tree, &ctx).unwrap();
        // Self(10) + ExtractAmount(8) + Constant(5) + GT(20) + BoolToSigmaProp(15) = 58
        // block cost = 58 / 10 = 5
        assert_eq!(res.cost, 5);
    }
```

- [ ] **Step 4: Run test**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo test -p ergotree-interpreter -- jit_cost_self_value -v`
Expected: PASS

- [ ] **Step 5: Write test for cost limit enforcement**

```rust
    #[test]
    fn jit_cost_limit_exceeded() {
        // Set a very low cost limit and verify that evaluation returns CostError
        let tree = ErgoTree::try_from(Expr::Const(true.into())).unwrap();
        let mut ctx = force_any_val::<Context>();
        ctx.jit_cost_limit = Some(1); // limit of 1 JitCost unit — Constant(5) will exceed it
        let res = reduce_to_crypto(&tree, &ctx);
        assert!(res.is_err());
        match res.unwrap_err() {
            EvalError::CostError(_) => {} // expected
            EvalError::Spanned(e) => {
                match *e.error {
                    EvalError::CostError(_) => {} // may be wrapped
                    other => panic!("expected CostError, got {:?}", other),
                }
            }
            other => panic!("expected CostError, got {:?}", other),
        }
    }
```

- [ ] **Step 6: Run test**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo test -p ergotree-interpreter -- jit_cost_limit_exceeded -v`
Expected: PASS

- [ ] **Step 7: Run full test suite**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo test -- --test-threads=4`
Expected: all pass

- [ ] **Step 8: Commit**

```bash
git add ergotree-interpreter/src/eval.rs
git commit -m "test: add JIT cost accumulation and limit enforcement tests"
```

---

### Task 11: Cleanup

**Files:**
- Modify: `ergotree-interpreter/src/eval/costs.rs` (remove dead `Cost`/`Costs` types)
- Modify: `ergotree-interpreter/src/eval/cost_accum.rs` (remove dead `CostAccumulator`)

- [ ] **Step 1: Remove dead code**

In `costs.rs`, remove the old `Cost` and `Costs` types entirely. The file should contain only `JitCost` and `per_item_cost`.

In `cost_accum.rs`, remove the old `CostAccumulator` struct. The file should contain only `CostError`.

- [ ] **Step 2: Fix any remaining compilation issues**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo check`
Fix any remaining references to removed types.

- [ ] **Step 3: Run full test suite**

Run: `cd /home/mwaddip/projects/sigma-rust/sigma-rust && cargo test -- --test-threads=4`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add ergotree-interpreter/src/eval/costs.rs ergotree-interpreter/src/eval/cost_accum.rs
git commit -m "chore: remove dead Cost, Costs, and CostAccumulator types"
```
