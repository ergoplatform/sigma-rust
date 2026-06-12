//! Language conformance smoke tests (entry point).
//!
//! One compile case per predef function and per language construct documented
//! in the EKB ErgoScript built-in functions reference. These tests do NOT
//! validate byte-for-byte parity against Scala — they only assert that the
//! ergoscript-compiler accepts the source, infers a sensible type, and reaches
//! a valid MIR / ErgoTree without panicking.
//!
//! Byte-match parity against Scala is covered by the ecosystem batch
//! (`test_ecosystem_batch`) and `test_batch_node_byte_match` in the lib tests.
//! This file is the *language coverage* axis: every predef gets a smoke test
//! so a future regression that drops a predef is caught immediately.
//!
//! Layout (matches `language-conformance.md` plan):
//!   * `predef_funcs` — one test per SigmaPredef.PredefinedFunc entry
//!   * `methods/<registry>` — one file per type-method registry
//!     (e.g. `methods::scoll` for SCollectionMethods)

use ergoscript_compiler::compiler::{compile, compile_expr};
use ergoscript_compiler::script_env::ScriptEnv;

/// Compile a snippet via `compile_expr` (raw IR, no ErgoTree serialization).
/// Use this for predefs that require ErgoTree V3+ (v6.0+) since the default
/// `compile()` path emits a V0 tree which would reject v6 ops at serialization.
pub fn compile_ok(src: &str) -> ergotree_ir::mir::expr::Expr {
    compile_expr(src, ScriptEnv::new())
        .unwrap_or_else(|e| panic!("compile_expr failed for {:?}: {:?}", src, e))
}

/// Compile a snippet all the way to ErgoTree (V0). Use this only for predefs
/// that work at V0 — anything v6+ will fail in serialization.
pub fn compile_tree(src: &str) {
    compile(src, ScriptEnv::new())
        .unwrap_or_else(|e| panic!("compile (full pipeline) failed for {:?}: {:?}", src, e));
}

// Integration-test crate roots resolve `mod foo;` against the siblings of
// the entry file, so the submodule files in `tests/conformance/` need an
// explicit `#[path]` to be picked up.
#[path = "conformance/bitwise_infix.rs"]
mod bitwise_infix;
#[path = "conformance/methods/mod.rs"]
mod methods;
#[path = "conformance/predef_funcs.rs"]
mod predef_funcs;
