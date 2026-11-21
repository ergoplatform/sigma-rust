# Blitzen

sigma-rust's runner for [SANTA](https://github.com/mwaddip/santa), the Ergo consensus conformance
suite. SANTA holds test vectors whose expected answers were recorded ("blessed") from the
reference implementation (sigma-state and the JVM node); Blitzen runs them through sigma-rust,
grades what sigma-rust answers by SANTA's grading rules, and writes down the answers and the
verdicts; SANTA grades the answers again as a cross-check. Where sigma-rust answers
differently, that is a divergence to look at, not something this runner smooths over.

It covers four tiers:

| tier | what an entry is | what Blitzen answers |
|---|---|---|
| eval | an ErgoTree, an input, the `(activated, ergoTree)` versions | `{value, cost, error}` |
| wire | bytes of a `Constant`, `Box`, `Transaction`, `SigmaBoolean`, `ErgoTree` or block-transactions section | the bytes sigma-rust writes back |
| transaction | a transaction, its input boxes, the headers, pre-header and parameters it is validated under | `{valid, cost, error}` |
| nipopow | a header chain, and an interlinks or proof request over it | the interlinks, or the proof's bytes |

The formats are SANTA's runner contract
([`docs/contract/runner-contract.md`](https://github.com/mwaddip/santa/blob/main/docs/contract/runner-contract.md)
and its `-wire`, `-transaction` and `-nipopow` companions). This sigma-rust has no JIT-cost
model, so the runner declares `cost: false` and only values and verdicts are graded.

## Run it

```bash
cargo test                                            # the runner's own tests
cargo run --release -- <vector.json | dir>            # run entries, compare with `expected`, print nice/coal
cargo run --release -- run runner.json <santa-checkout> <out-dir>   # grade this checkout
./santa-run - <santa-checkout> <out-dir>              # the same, through the entry SANTA calls
```

`run` grades this checkout against a checkout of SANTA (`git clone
https://github.com/mwaddip/santa`). `santa-run <impl-path> <santa-dir> <out-dir>` is the entry
SANTA calls for it: it provisions the repository's toolchain through
[mise](https://mise.jdx.dev), which must be on `PATH`, builds the runner and runs
`blitzen run runner.json <santa-dir> <out-dir>`. The run


- checks its grader and its selection of files against that checkout's `oracle/`, and refuses to
  run if either is not reproduced;
- selects the vector files `runner.json` covers (`version`, `tiers`; `cost` says whether the
  cost dimension is graded);
- runs every entry, a file that carries a memory and time budget (`limits`) in a child process of
  its own (`blitzen one`), which is stopped when it passes the budget;
- grades every answer by the checkout's `vectors/grading.json`;
- writes `<out-dir>/results.json` (per slice what was graded, and every red, with a short
  rendering of its expected and actual answer) and the raw answers under `<out-dir>/actuals/`.

It exits 0 when it ran, whatever is red, and non-zero when it could not run. `santa-run` does not
use `<impl-path>`: the runner is built against the checkout it sits in.

## The harness

Everything in that list but running an entry is `harness/`, the crate `santa-harness`: SANTA's
code, copied unchanged from [santa-blitzen](https://github.com/mwaddip/santa-blitzen) (its
`harness/` at commit `5d63a3b`), and the same in every Rust runner SANTA maintains. It is not
edited here: a fix to it goes to SANTA and comes back as a new copy. It is a workspace of its own
(the runner's `Cargo.toml` excludes it) with its own tests, which read a checkout of SANTA and
fail without one:

```bash
SANTA_DIR=<santa-checkout> cargo test --manifest-path harness/Cargo.toml
```

The runner is built and run on Linux; a bounded file needs Linux, where the harness watches the
child's memory in `/proc`.

## How it works

- **`eval.rs`**: evaluates an entry's tree in the canonical evaluation context the reference
  builds for a bare evaluation (SELF holding the entry's tree, `INPUTS = [SELF]`, no outputs,
  height 0, a fixed pre-header), with the entry's input bound where its envelope puts it. Trees
  are read with `ErgoTree::sigma_parse_bytes_lenient`, since the vectors' roots are of any type,
  and evaluated with `eval::test_util::try_eval_with_deserialize`, the public entry (behind the
  `arbitrary` feature) that evaluates a root of any type. `Context` holds exactly ten headers
  where the canonical context has none, so it carries ten copies of one fixed header.
- **`wire.rs`**: parses a kind's bytes with sigma-rust's serializer and writes them back. The
  `ErgoTree` kind goes through the lenient parse, so the tree is re-encoded from its structure.
- **`transaction.rs`**: decodes the transaction and its boxes from their bytes, builds the
  `ErgoStateContext` from the entry's own headers, pre-header and parameters, and runs ergo-lib's
  stateful validation (`TransactionContext::validate`). A validation error is a clean
  `valid: false` with sigma-rust's reason; bytes sigma-rust cannot read are `errored`.
- **`nipopow.rs`**: rebuilds the chain as `PoPowHeader`s and answers interlinks or
  `NipopowAlgos::prove`. The proof's bytes end with its `continuous` byte (0), which the node's
  wire encoding adds and sigma-rust's proof serializer leaves out.
- **`sval.rs`** and **`binding.rs`**: the bridge between SANTA's JSON values and sigma-rust's.
  `Long` and `BigInt` are decimal strings, bytes lower-case hex, a `SigmaProp` its bare
  `SigmaBoolean` bytes; an input given as `Value` bytes is read with sigma-rust's own reader for
  that position.
- **`main.rs`**: the CLI, and the dispatch by vector schema. Every entry gets exactly one outcome:
  a value or verdict, `errored` (sigma-rust's own refusal, a divergence where the reference
  accepted), `not-implemented`, or `panicked` (a panic, caught, or a failure of the runner itself,
  with a note).
