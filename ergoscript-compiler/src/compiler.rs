//! ErgoScript compiler

use super::binder::BinderError;
use super::hir::HirLoweringError;
use crate::ast;
use crate::binder::Binder;
use crate::hir;
use crate::mir;
use crate::parser::parse_error::ParseError;
use crate::script_env::ScriptEnv;
use crate::type_infer::assign_type;
use crate::type_infer::TypeInferenceError;
use std::convert::TryInto;

extern crate derive_more;
use derive_more::From;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::ergo_tree::ErgoTreeError;
use ergotree_ir::type_check::TypeCheckError;
use mir::lower::MirLoweringError;

/// Compilation errors
#[derive(Debug, PartialEq, Eq, From)]
pub enum CompileError {
    /// Parser error
    ParseError(Vec<ParseError>),
    /// Error on AST to HIR lowering
    HirLoweringError(HirLoweringError),
    /// Error on binder pass
    BinderError(BinderError),
    /// Error on type inference pass
    TypeInferenceError(TypeInferenceError),
    /// Error on HIT to MIR lowering
    MirLoweringError(MirLoweringError),
    /// Error on type checking
    TypeCheckError(TypeCheckError),
    /// ErgoTree error
    ErgoTreeError(ErgoTreeError),
}

impl CompileError {
    /// Pretty formatted error with CST/AST/IR, etc.
    pub fn pretty_desc(&self, source: &str) -> String {
        match self {
            CompileError::ParseError(errors) => {
                errors.iter().map(|e| e.pretty_desc(source)).collect()
            }
            CompileError::HirLoweringError(e) => e.pretty_desc(source),
            CompileError::BinderError(e) => e.pretty_desc(source),
            CompileError::TypeInferenceError(e) => e.pretty_desc(source),
            CompileError::MirLoweringError(e) => e.pretty_desc(source),
            CompileError::TypeCheckError(e) => e.pretty_desc(),
            CompileError::ErgoTreeError(e) => format!("{:?}", e),
        }
    }
}

/// Compiles given source code to [`ergotree_ir::mir::expr::Expr`], or returns an error
pub fn compile_expr(
    source: &str,
    env: ScriptEnv,
) -> Result<ergotree_ir::mir::expr::Expr, CompileError> {
    let hir = compile_hir(source)?;
    compile_from_hir(hir, env)
}

/// Inner pipeline: bind → type → optimize → lower → typecheck.
/// Runs on a dedicated thread with 16MB stack so large contracts (500+ lines)
/// don't overflow when optimization passes triple the recursion depth.
fn compile_from_hir(
    hir: hir::Expr,
    env: ScriptEnv,
) -> Result<ergotree_ir::mir::expr::Expr, CompileError> {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let binder = Binder::new(env);
            let bind = binder.bind(hir)?;
            let typed = assign_type(bind)?;
            let optimized = hir::optimize::optimize(typed);
            let mir = mir::lower::lower(optimized)?;
            let cse_mir = mir::cse::apply_cse(mir);
            let res = ergotree_ir::type_check::type_check(cse_mir)?;
            Ok(res)
        })
        .expect("failed to spawn compiler thread")
        .join()
        .expect("compiler thread panicked")
}

/// Compiles given source code to [`ErgoTree`], or returns an error
pub fn compile(source: &str, env: ScriptEnv) -> Result<ErgoTree, CompileError> {
    let expr = compile_expr(source, env)?;
    Ok(expr.try_into()?)
}

/// Result of canonical compilation, indicating whether the node was used.
#[derive(Debug)]
pub struct CanonicalCompileResult {
    /// The canonical ErgoTree bytes (from node if available, otherwise local)
    pub tree: ErgoTree,
    /// Whether the local Rust compilation matched the Scala node
    pub matched: Option<bool>,
}

/// Compile ErgoScript to canonical ErgoTree bytes, verifying against an Ergo node.
///
/// If `node_url` is provided (e.g. `"http://localhost:9053"`), the source is also
/// compiled through the Scala compiler via the node's REST API. If the bytes differ,
/// the node's bytes are used (they are the canonical reference for P2S addresses).
///
/// If the node is unreachable or returns an error, falls back to local compilation.
///
/// # Arguments
/// * `source` - ErgoScript source code (without outer braces)
/// * `env` - Script environment bindings
/// * `node_url` - Optional Ergo node URL (e.g. `"http://localhost:9053"`)
/// * `api_key` - Optional API key for the node
pub fn compile_canonical(
    source: &str,
    env: ScriptEnv,
    node_url: &str,
    api_key: &str,
) -> Result<CanonicalCompileResult, CompileError> {
    use ergotree_ir::serialization::SigmaSerializable;

    // Step 1: Compile locally
    let local_tree = compile(source, env)?;
    let local_bytes = local_tree
        .sigma_serialize_bytes()
        .map_err(|e| CompileError::ErgoTreeError(ErgoTreeError::RootSerializationError(e)))?;

    // Step 2: Try the node
    let node_result = compile_via_node(source, node_url, api_key);

    match node_result {
        Ok(node_bytes) => {
            if local_bytes == node_bytes {
                Ok(CanonicalCompileResult {
                    tree: local_tree,
                    matched: Some(true),
                })
            } else {
                // Node bytes differ — use them (canonical reference)
                let canonical_tree = ErgoTree::sigma_parse_bytes(&node_bytes).map_err(|e| {
                    CompileError::ErgoTreeError(ErgoTreeError::SigmaParsingError(e))
                })?;
                Ok(CanonicalCompileResult {
                    tree: canonical_tree,
                    matched: Some(false),
                })
            }
        }
        Err(_) => {
            // Node unreachable — use local bytes
            Ok(CanonicalCompileResult {
                tree: local_tree,
                matched: None,
            })
        }
    }
}

/// Compile source code via the Ergo node's REST API using curl.
/// Returns the ErgoTree bytes or an error string.
fn compile_via_node(source: &str, node_url: &str, api_key: &str) -> Result<Vec<u8>, String> {
    use std::process::Command;

    // Wrap source in braces if not already wrapped
    let wrapped = if source.trim().starts_with('{') {
        source.to_string()
    } else {
        format!("{{ {} }}", source)
    };

    // Step 1: POST /script/p2sAddress to get the P2S address
    let p2s_url = format!("{}/script/p2sAddress", node_url);
    let body = format!(
        r#"{{"source": {}, "treeVersion": 0}}"#,
        serde_json_escape(&wrapped)
    );

    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            &p2s_url,
            "-H",
            "Content-Type: application/json",
            "-H",
            &format!("api_key: {}", api_key),
            "-d",
            &body,
            "--connect-timeout",
            "5",
            "--max-time",
            "10",
        ])
        .output()
        .map_err(|e| format!("curl not found or failed to execute: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "p2sAddress curl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let resp_text = String::from_utf8_lossy(&output.stdout).to_string();
    let address = extract_json_string(&resp_text, "address")
        .ok_or_else(|| format!("p2sAddress response missing 'address': {}", resp_text))?;

    // Step 2: GET /script/addressToTree/{address} to get the hex tree
    let tree_url = format!("{}/script/addressToTree/{}", node_url, address);
    let output = Command::new("curl")
        .args([
            "-s",
            &tree_url,
            "-H",
            &format!("api_key: {}", api_key),
            "--connect-timeout",
            "5",
            "--max-time",
            "10",
        ])
        .output()
        .map_err(|e| format!("curl failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "addressToTree curl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let resp_text = String::from_utf8_lossy(&output.stdout).to_string();
    let hex = extract_json_string(&resp_text, "tree")
        .ok_or_else(|| format!("addressToTree response missing 'tree': {}", resp_text))?;

    // Decode hex to bytes
    hex_to_bytes(&hex).map_err(|e| format!("hex decode failed: {}", e))
}

/// Escape a string for JSON (simple implementation avoiding serde dependency)
fn serde_json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Extract a string value from a JSON object (minimal parser)
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = json.find(&pattern)?;
    let after_key = &json[idx + pattern.len()..];
    // Skip `: ` or `:`
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let after_ws = after_colon.trim_start();
    if let Some(content) = after_ws.strip_prefix('"') {
        let end = content.find('"')?;
        Some(content[..end].to_string())
    } else {
        None
    }
}

/// Decode hex string to bytes
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err("odd-length hex string".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("invalid hex at {}: {}", i, e))
        })
        .collect()
}

pub(crate) fn compile_hir(source: &str) -> Result<hir::Expr, CompileError> {
    let parse = super::parser::parse(source);
    if !parse.errors.is_empty() {
        return Err(CompileError::ParseError(parse.errors));
    }
    let syntax = parse.syntax();
    let root = ast::Root::cast(syntax).unwrap();
    let hir = hir::lower(root)?;
    Ok(hir)
}

#[cfg(test)]
fn check(input: &str, expected_tree: expect_test::Expect) {
    let res = compile_expr(input, ScriptEnv::new());

    let expected_out = res
        .map(|tree| tree.debug_tree())
        .unwrap_or_else(|e| e.pretty_desc(input));
    expected_tree.assert_eq(&expected_out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_height() {
        check(
            "HEIGHT",
            expect![[r#"
                GlobalVars(
                    Height,
                )"#]],
        );
    }

    #[test]
    fn test_field_access_unresolved() {
        // HSB.HEIGHT now parses as a valid field access
        // but HSB is unresolved, so MIR lowering fails
        assert!(compile_expr("HSB.HEIGHT", ScriptEnv::new()).is_err());
    }

    #[test]
    fn test_sigmaprop_height_gt() {
        check(
            "sigmaProp(HEIGHT > 0)",
            expect![[r#"
                BoolToSigmaProp(
                    BoolToSigmaProp {
                        input: BinOp(
                            Spanned {
                                source_span: SourceSpan {
                                    offset: 0,
                                    length: 0,
                                },
                                expr: BinOp {
                                    kind: Relation(
                                        Gt,
                                    ),
                                    left: GlobalVars(
                                        Height,
                                    ),
                                    right: Const(
                                        "0: SInt",
                                    ),
                                },
                            },
                        ),
                    },
                )"#]],
        );
    }

    #[test]
    fn test_session1_target() {
        // { sigmaProp(HEIGHT > 0 && HEIGHT < 100) }
        // Block unwraps to single expression, so we get sigmaProp directly
        let result = compile_expr(
            "{ sigmaProp(HEIGHT > 0 && HEIGHT < 100) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session1_ergotree_hex() {
        use ergotree_ir::serialization::SigmaSerializable;
        // Compare against Ergo node output for:
        // { sigmaProp(HEIGHT > 0 && HEIGHT < 100) }
        let tree = compile(
            "{ sigmaProp(HEIGHT > 0 && HEIGHT < 100) }",
            ScriptEnv::new(),
        )
        .unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        // Expected from Ergo node: "1002040004c801d1ed91a373008fa37301"
        assert_eq!(hex, "1002040004c801d1ed91a373008fa37301");
    }

    #[test]
    fn test_session2_val_binding() {
        let result = compile_expr("{ val x: Long = 5L; sigmaProp(x > 0L) }", ScriptEnv::new());
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session3_self_value() {
        let result = compile_expr("{ sigmaProp(SELF.value > 0L) }", ScriptEnv::new());
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session3_ergotree_hex() {
        use ergotree_ir::serialization::SigmaSerializable;
        let tree = compile("{ sigmaProp(SELF.value > 0L) }", ScriptEnv::new()).unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        // Expected from Ergo node
        assert_eq!(hex, "10010500d191c1a77300");
    }

    #[test]
    fn test_session2_val_binding_no_annotation() {
        let result = compile_expr("{ val x = 42; sigmaProp(x > 0) }", ScriptEnv::new());
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session2_ergotree_roundtrip() {
        use ergotree_ir::serialization::SigmaSerializable;
        // Verify our output serializes and deserializes correctly.
        // Note: Scala compiler constant-folds this to sigmaProp(true),
        // so byte-for-byte match is not expected (no optimization passes yet).
        let tree = compile("{ val x: Long = 5L; sigmaProp(x > 0L) }", ScriptEnv::new()).unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        assert!(!bytes.is_empty());
        // Verify it round-trips through serialization
        let tree2 = ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        let bytes2 = tree2.sigma_serialize_bytes().unwrap();
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn test_session4_tokens_index_tuple() {
        let result = compile_expr(
            "{ sigmaProp(SELF.tokens.size > 0 && SELF.tokens(0)._2 == 1L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session4_ergotree_roundtrip() {
        use ergotree_ir::serialization::SigmaSerializable;
        // Note: Scala compiler does CSE (common subexpression elimination) for SELF.tokens,
        // introducing a val binding. Our output is semantically correct but not byte-identical.
        let tree = compile(
            "{ sigmaProp(SELF.tokens.size > 0 && SELF.tokens(0)._2 == 1L) }",
            ScriptEnv::new(),
        )
        .unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        assert!(!bytes.is_empty());
        let tree2 = ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        let bytes2 = tree2.sigma_serialize_bytes().unwrap();
        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn test_session5_exists_lambda() {
        let result = compile_expr(
            "{ sigmaProp(INPUTS.exists { (b: Box) => b.value > 0L }) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session5_ergotree_hex() {
        use ergotree_ir::serialization::SigmaSerializable;
        let tree = compile(
            "{ sigmaProp(INPUTS.exists { (b: Box) => b.value > 0L }) }",
            ScriptEnv::new(),
        )
        .unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "10010500d1aea4d901016391c172017300");
    }

    #[test]
    fn test_session6_from_base16() {
        let _result = compile_expr(
            r#"{ val x: Coll[Byte] = fromBase16("deadbeef"); sigmaProp(x.size > 0) }"#,
            ScriptEnv::new(),
        );
        // Note: Coll[Byte] type annotation won't parse (needs generic syntax)
        // Test without annotation instead
        let result = compile_expr(
            r#"{ val x = fromBase16("deadbeef"); sigmaProp(x.size > 0) }"#,
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session7_if_else() {
        let result = compile_expr(
            "{ if (HEIGHT > 100) sigmaProp(true) else sigmaProp(false) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session7_if_else_hex() {
        use ergotree_ir::serialization::SigmaSerializable;
        let tree = compile(
            "{ if (HEIGHT > 100) sigmaProp(true) else sigmaProp(false) }",
            ScriptEnv::new(),
        )
        .unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, "100304c801010101009591a37300d17301d17302");
    }

    #[test]
    fn test_session7_if_else_block_branches() {
        let result = compile_expr(
            "{ val x: Long = 5L; val y: Long = if (x > 0L) { x + 1L } else { 0L }; sigmaProp(y > 0L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session8_context_datainputs() {
        let result = compile_expr(
            "{ val di = CONTEXT.dataInputs(0); sigmaProp(di.value > 0L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session8_register_access() {
        let result = compile_expr(
            "{ val x = SELF.R4[Long].get; sigmaProp(x > 0L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_session8_time_validator_contract() {
        let source = r#"{
  val CounterNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val ValidVoteId = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val VYoloId = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val UserVoteScriptHash = fromBase16("0000000000000000000000000000000000000000000000000000000000000004")
  val cancellationCooldown = 4320L
  val counterBox = CONTEXT.dataInputs(0)
  val counterValid = counterBox.tokens.size >= 1 && counterBox.tokens(0)._1 == CounterNftId
  val voteDeadline = counterBox.R4[Long].get
  val votingWindow = 12960L
  val votingStart = voteDeadline - votingWindow
  val withinVotingWindow = HEIGHT >= votingStart && HEIGHT < voteDeadline
  val voteBoxValid = {
    val voteBox = OUTPUTS(0)
    val correctScript = blake2b256(voteBox.propositionBytes) == UserVoteScriptHash
    val correctTokens = voteBox.tokens.size >= 2 && voteBox.tokens(0)._1 == ValidVoteId && voteBox.tokens(0)._2 == 1L && voteBox.tokens(1)._1 == VYoloId && voteBox.tokens(1)._2 > 0L
    val validDirection = { val dir = voteBox.R4[Long].get; dir == 0L || dir == 1L }
    val validCancelHeight = voteBox.R7[Long].get >= HEIGHT + cancellationCooldown
    val validSubmissionDeadline = voteBox.R8[Long].get <= voteDeadline
    correctScript && correctTokens && validDirection && validCancelHeight && validSubmissionDeadline
  }
  sigmaProp(counterValid && withinVotingWindow && voteBoxValid)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "TimeValidator failed: {:?}", result.err());
    }

    #[test]
    fn test_session7_reserve_contract() {
        let source = r#"{
  val ReserveNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val StateNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val VYoloId = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val selfValid = SELF.tokens.size == 2 && SELF.tokens(0)._1 == ReserveNftId && SELF.tokens(0)._2 == 1L && SELF.tokens(1)._1 == VYoloId
  val out = OUTPUTS(1)
  val outValid = out.propositionBytes == SELF.propositionBytes && out.tokens.size == 2 && out.tokens(0)._1 == ReserveNftId && out.tokens(0)._2 == 1L && out.tokens(1)._1 == VYoloId && out.value == SELF.value
  val vaultIn = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == StateNftId }
  val vaultOut = OUTPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == StateNftId }
  val pairingValid = vaultIn.size == 1 && vaultOut.size == 1
  val deltaReserveVYolo = out.tokens(1)._2 - SELF.tokens(1)._2
  val deltaVaultYolo = vaultOut(0).value - vaultIn(0).value
  val conservation = deltaVaultYolo + deltaReserveVYolo == 0L
  val nonTrivial = deltaReserveVYolo != 0L
  val singleReserveInput = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }.size == 1
  sigmaProp(selfValid && outValid && pairingValid && conservation && nonTrivial && singleReserveInput)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "Reserve contract failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session6_vault_contract() {
        // Vault contract from 06-governance/contracts/vault.es
        // Replace placeholders with valid 32-byte hex strings
        let source = r#"{
  val StateNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val ReserveNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val selfValid = SELF.tokens.size == 1 && SELF.tokens(0)._1 == StateNftId && SELF.tokens(0)._2 == 1L
  val out = OUTPUTS(0)
  val outValid = out.propositionBytes == SELF.propositionBytes && out.tokens.size == 1 && out.tokens(0)._1 == StateNftId && out.tokens(0)._2 == 1L
  val reserveIn = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }
  val reserveOut = OUTPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }
  val pairingValid = reserveIn.size == 1 && reserveOut.size == 1
  val deltaVaultYolo = out.value - SELF.value
  val deltaReserveVYolo = reserveOut(0).tokens(1)._2 - reserveIn(0).tokens(1)._2
  val conservation = deltaVaultYolo + deltaReserveVYolo == 0L
  val nonTrivial = deltaVaultYolo != 0L
  val singleVaultInput = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == StateNftId }.size == 1
  sigmaProp(selfValid && outValid && pairingValid && conservation && nonTrivial && singleVaultInput)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "Vault contract failed: {:?}", result.err());
    }

    #[test]
    fn test_session9_tuple_register() {
        let result = compile_expr(
            "{ val t = SELF.R4[(Long, Long)].get; sigmaProp(t._1 > 0L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Tuple register failed: {:?}", result.err());
    }

    #[test]
    fn test_session9_coll_byte_register() {
        let result = compile_expr(
            "{ val x = SELF.R5[Coll[Byte]].get; sigmaProp(x.size > 0) }",
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "Coll[Byte] register failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session9_tuple_literal() {
        let result = compile_expr(
            "{ val t = (1L, 2L); sigmaProp(t._1 > 0L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Tuple literal failed: {:?}", result.err());
    }

    #[test]
    fn test_session9_sigma_and_or() {
        let result = compile_expr("{ sigmaProp(true) && sigmaProp(false) }", ScriptEnv::new());
        assert!(result.is_ok(), "SigmaAnd failed: {:?}", result.err());

        let result = compile_expr("{ sigmaProp(true) || sigmaProp(false) }", ScriptEnv::new());
        assert!(result.is_ok(), "SigmaOr failed: {:?}", result.err());
    }

    #[test]
    fn test_session9_modulo() {
        let result = compile_expr("{ val x = 42L % 10L; sigmaProp(x > 0L) }", ScriptEnv::new());
        assert!(result.is_ok(), "Modulo failed: {:?}", result.err());
    }

    #[test]
    fn test_session9_tuple_lambda_param() {
        let result = compile_expr(
            "{ sigmaProp(SELF.tokens.forall { (t: (Coll[Byte], Long)) => t._2 > 0L }) }",
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "Tuple lambda param failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session9_slice() {
        let result = compile_expr(
            "{ val boxes = INPUTS.slice(1, INPUTS.size); sigmaProp(boxes.size > 0) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Slice failed: {:?}", result.err());
    }

    #[test]
    fn test_session9_fold_with_inner_vals() {
        // Minimal fold body that has val bindings inside the fold lambda
        let result = compile_expr(
            r#"{ val boxes = INPUTS.slice(1, INPUTS.size); val result = boxes.fold((0L, 0L), { (acc: (Long, Long), b: Box) => if (b.value > 0L) { val x = b.value; (acc._1 + x, acc._2 + 1L) } else { acc } }); sigmaProp(result._1 > 0L) }"#,
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "Fold with inner vals failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session9_fold_multiline() {
        let source = r#"{ val ValidVoteId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val VYoloId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val voterBoxes: Coll[Box] = INPUTS.slice(1, INPUTS.size)
  val voteCounts = voterBoxes.fold((0L, 0L), { (acc: (Long, Long), voter: Box) =>
    if (voter.tokens.size >= 2 &&
        voter.tokens(0)._1 == ValidVoteId &&
        voter.tokens(1)._1 == VYoloId) {
      val power: Long = voter.tokens(1)._2
      val yesAdd: Long = if (voter.R4[Long].get == 1L) power else 0L
      (acc._1 + power, acc._2 + yesAdd)
    } else {
      acc
    }
  })
  sigmaProp(voteCounts._1 > 0L)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "Fold multiline failed: {:?}", result.err());
    }

    #[test]
    fn test_session9_counting_contract() {
        let source = r#"{
  val CounterNftId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val VYoloId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val ValidVoteId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val votingWindow: Long = 12960L
  val countingPhase: Long = 1080L
  val executionGrace: Long = 4320L
  val initiationHurdle: Long = 100000000000000L
  val quorumFloor: Long = 1000000000000000L
  val elevatedProportion: Long = 1000000L
  val minimumSupport: Long = 500L
  val elevatedSupport: Long = 900L
  val voteDeadline: Long = SELF.R4[Long].get
  val currentTally = SELF.R5[(Long, Long)].get
  val currentProportion: Long = currentTally._1
  val currentVotesFor: Long = currentTally._2
  val recipientHash: Coll[Byte] = SELF.R6[Coll[Byte]].get
  val totalVotes: Long = SELF.R7[Long].get
  val initiationStake: Long = SELF.R8[Long].get
  val validationVotes: Long = SELF.R9[Long].get
  val out0: Box = OUTPUTS(0)
  val countingEnd: Long = voteDeadline + countingPhase
  val validationEnd: Long = countingEnd + executionGrace
  val isBeforeCounting: Boolean = HEIGHT < voteDeadline
  val isCountingPeriod: Boolean = HEIGHT >= voteDeadline && HEIGHT < countingEnd
  val isVoteValidationPeriod: Boolean = HEIGHT >= countingEnd && HEIGHT < validationEnd
  val isNewProposalPeriod: Boolean = HEIGHT >= validationEnd
  val counterPreserved: Boolean =
    out0.propositionBytes == SELF.propositionBytes &&
    out0.tokens.size >= 1 &&
    out0.tokens(0)._1 == CounterNftId &&
    out0.tokens(0)._2 == 1L
  val phase1: Boolean = {
    val noActiveProposal: Boolean = totalVotes == 0L && validationVotes == 0L
    val initiationBox = CONTEXT.dataInputs(0)
    val hasEnoughStake: Boolean = initiationBox.tokens.size >= 1 &&
      initiationBox.tokens(0)._1 == VYoloId &&
      initiationBox.tokens(0)._2 >= initiationHurdle
    val talliesReset: Boolean = {
      out0.R7[Long].get == 0L &&
      out0.R9[Long].get == 0L &&
      out0.R4[Long].get == HEIGHT + votingWindow &&
      out0.R8[Long].get >= initiationHurdle
    }
    val valuePreserved: Boolean = out0.value >= SELF.value
    isBeforeCounting && noActiveProposal && hasEnoughStake &&
    talliesReset && counterPreserved && valuePreserved
  }
  val phase2: Boolean = {
    val voterBoxes: Coll[Box] = INPUTS.slice(1, INPUTS.size)
    val voteCounts = voterBoxes.fold((0L, 0L), { (acc: (Long, Long), voter: Box) =>
      if (voter.tokens.size >= 2 &&
          voter.tokens(0)._1 == ValidVoteId &&
          voter.tokens(1)._1 == VYoloId) {
        val power: Long = voter.tokens(1)._2
        val yesAdd: Long = if (voter.R4[Long].get == 1L) power else 0L
        (acc._1 + power, acc._2 + yesAdd)
      } else {
        acc
      }
    })
    val votesThisRound: Long = voteCounts._1
    val yesVotesThisRound: Long = voteCounts._2
    val talliesUpdated: Boolean = {
      val newTally = out0.R5[(Long, Long)].get
      newTally._1 == currentProportion &&
      newTally._2 == currentVotesFor + yesVotesThisRound &&
      out0.R7[Long].get == totalVotes + votesThisRound &&
      out0.R9[Long].get == validationVotes + yesVotesThisRound
    }
    val fieldsPreserved: Boolean = {
      out0.R4[Long].get == voteDeadline &&
      out0.R6[Coll[Byte]].get == recipientHash &&
      out0.R8[Long].get == initiationStake
    }
    val voteNftsBurnedFromOutputs: Boolean = OUTPUTS.forall { (o: Box) =>
      o.tokens.forall { (t: (Coll[Byte], Long)) => t._1 != ValidVoteId }
    }
    val valuePreserved: Boolean = out0.value >= SELF.value
    isCountingPeriod && votesThisRound > 0L && talliesUpdated &&
    fieldsPreserved && voteNftsBurnedFromOutputs && counterPreserved && valuePreserved
  }
  val phase3: Boolean = {
    val proposalBox: Box = INPUTS(1)
    val meetsQuorum: Boolean = totalVotes >= quorumFloor
    val requiredSupport: Long = if (currentProportion > elevatedProportion) {
      elevatedSupport
    } else {
      minimumSupport
    }
    val actualSupport: Long = if (totalVotes > 0L) {
      validationVotes * 1000L / totalVotes
    } else {
      0L
    }
    val meetsSupport: Boolean = actualSupport >= requiredSupport
    val proposalPassed: Boolean = meetsQuorum && meetsSupport
    val proposalAdvanced: Boolean = if (proposalPassed) {
      val proposalTokens = proposalBox.tokens(0)
      proposalTokens._2 == 1L &&
      OUTPUTS(1).tokens.size >= 1 &&
      OUTPUTS(1).tokens(0)._1 == proposalTokens._1 &&
      OUTPUTS(1).tokens(0)._2 == 2L &&
      OUTPUTS(1).propositionBytes == proposalBox.propositionBytes &&
      OUTPUTS(1).R4[(Long, Long)].get._1 == currentProportion &&
      blake2b256(OUTPUTS(1).R5[Coll[Byte]].get) == recipientHash
    } else {
      true
    }
    val counterReset: Boolean = {
      out0.R7[Long].get == 0L &&
      out0.R9[Long].get == 0L
    }
    val valuePreserved: Boolean = out0.value >= SELF.value
    isVoteValidationPeriod && proposalAdvanced && counterReset &&
    counterPreserved && valuePreserved
  }
  val phase4: Boolean = {
    val valuePreserved: Boolean = out0.value >= SELF.value
    val talliesCleared: Boolean =
      out0.R7[Long].get == 0L &&
      out0.R9[Long].get == 0L
    isNewProposalPeriod && counterPreserved && valuePreserved && talliesCleared
  }
  sigmaProp(phase1 || phase2 || phase3 || phase4)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "Counting contract failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session9_proposal_contract() {
        let source = r#"{
  val TreasuryNftId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val Denom: Long = 10000000L
  val currentTokens = SELF.tokens(0)
  val proposalTuple = SELF.R4[(Long, Long)].get
  val proportion: Long = proposalTuple._1
  val recipient: Coll[Byte] = SELF.R5[Coll[Byte]].get
  val validationHeight: Long = SELF.R6[Long].get
  val isFirstUpdate: Boolean = currentTokens._2 == 1L
  val isAdvancement: Boolean = {
    val out1: Box = OUTPUTS(1)
    val countingBox: Box = INPUTS(0)
    val countingValid: Boolean =
      countingBox.tokens.size >= 1 &&
      countingBox.tokens(0)._2 == validationHeight
    val scriptPreserved: Boolean = out1.propositionBytes == SELF.propositionBytes
    val proportionPreserved: Boolean = out1.R4[(Long, Long)].get._1 == proportion
    val recipientPreserved: Boolean = out1.R5[Coll[Byte]].get == recipient
    val tokenAdvanced: Boolean =
      out1.tokens.size >= 1 &&
      out1.tokens(0)._1 == currentTokens._1 &&
      out1.tokens(0)._2 == 2L
    val validTransition: Boolean = isFirstUpdate
    countingValid && scriptPreserved && proportionPreserved &&
    recipientPreserved && tokenAdvanced && validTransition
  }
  val isExecution: Boolean = {
    val isExecutionReady: Boolean = currentTokens._2 == 2L
    val treasuryPresent: Boolean = INPUTS(1).tokens.size >= 1 &&
      INPUTS(1).tokens(0)._1 == TreasuryNftId
    val tokenBurned: Boolean = {
      val stateTokenId: Coll[Byte] = currentTokens._1
      OUTPUTS.forall { (o: Box) =>
        o.tokens.forall { (t: (Coll[Byte], Long)) => t._1 != stateTokenId }
      }
    }
    isExecutionReady && treasuryPresent && tokenBurned
  }
  sigmaProp(isAdvancement || isExecution)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "Proposal contract failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session9_user_vote_contract() {
        let source = r#"{
  val ValidVoteId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val VYoloId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val CounterNftId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val cancellationCooldown: Long = 4320L
  val voteDirection: Long = SELF.R4[Long].get
  val proposalId: Coll[Byte] = SELF.R5[Coll[Byte]].get
  val voterPk: SigmaProp = SELF.R6[SigmaProp].get
  val cancelUnlockHeight: Long = SELF.R7[Long].get
  val submissionDeadline: Long = SELF.R8[Long].get
  val selfValid: Boolean =
    SELF.tokens.size >= 2 &&
    SELF.tokens(0)._1 == ValidVoteId &&
    SELF.tokens(1)._1 == VYoloId
  val voterVYolo: Long = SELF.tokens(1)._2
  val isCancel: Boolean = {
    val cooldownPassed: Boolean = HEIGHT >= cancelUnlockHeight
    val counterBox: Box = CONTEXT.dataInputs(0)
    val counterValid: Boolean =
      counterBox.tokens.size >= 1 &&
      counterBox.tokens(0)._1 == CounterNftId
    val counterDeadline: Long = counterBox.R4[Long].get
    val beforeDeadline: Boolean = HEIGHT < counterDeadline
    val vyoloReturned: Boolean =
      OUTPUTS(0).tokens.size >= 1 &&
      OUTPUTS(0).tokens(0)._1 == VYoloId &&
      OUTPUTS(0).tokens(0)._2 >= voterVYolo
    cooldownPassed && counterValid && beforeDeadline && vyoloReturned
  }
  val isSubmit: Boolean = {
    val counterInInputs: Boolean = INPUTS.exists { (b: Box) =>
      b.tokens.size >= 1 && b.tokens(0)._1 == CounterNftId
    }
    val withinDeadline: Boolean = HEIGHT <= submissionDeadline
    val voteNftBurned: Boolean = OUTPUTS.forall { (o: Box) =>
      o.tokens.forall { (t: (Coll[Byte], Long)) => t._1 != ValidVoteId }
    }
    val vyoloReturned: Boolean = OUTPUTS.exists { (o: Box) =>
      o.tokens.size >= 1 &&
      o.tokens(0)._1 == VYoloId &&
      o.tokens(0)._2 >= voterVYolo
    }
    counterInInputs && withinDeadline && voteNftBurned && vyoloReturned
  }
  sigmaProp(selfValid) && (
    (voterPk && sigmaProp(isCancel)) ||
    sigmaProp(isSubmit)
  )
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "UserVote contract failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session9_treasury_contract() {
        let source = r#"{
  val TreasuryNftId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val ProposalTokenId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val Denom: Long = 10000000L
  val out0: Box = OUTPUTS(0)
  val selfValid: Boolean =
    SELF.tokens.size >= 1 &&
    SELF.tokens(0)._1 == TreasuryNftId &&
    SELF.tokens(0)._2 == 1L
  val isDeposit: Boolean = {
    val selfIsFirstInput: Boolean = INPUTS(0).id == SELF.id
    val scriptPreserved: Boolean = out0.propositionBytes == SELF.propositionBytes
    val nftPreserved: Boolean =
      out0.tokens.size >= 1 &&
      out0.tokens(0)._1 == TreasuryNftId &&
      out0.tokens(0)._2 == 1L
    val valueGrown: Boolean = out0.value >= SELF.value
    selfIsFirstInput && scriptPreserved && nftPreserved && valueGrown
  }
  val proposalBox: Box = INPUTS(0)
  val hasProposalToken: Boolean =
    proposalBox.tokens.size >= 1 &&
    proposalBox.tokens(0)._1 == ProposalTokenId &&
    proposalBox.tokens(0)._2 == 2L
  val isWithdrawal: Boolean = {
    if (hasProposalToken) {
      val proposalTuple = proposalBox.R4[(Long, Long)].get
      val proportion: Long = proposalTuple._1
      val recipientHash: Coll[Byte] = proposalBox.R5[Coll[Byte]].get
      val isPartialWithdrawal: Boolean = proportion < Denom && proportion > 0L
      val wholePart: Long = (SELF.value / Denom) * proportion
      val remainderPart: Long = ((SELF.value % Denom) * proportion) / Denom
      val awarded: Long = wholePart + remainderPart
      val treasuryPreserved: Boolean =
        out0.propositionBytes == SELF.propositionBytes &&
        out0.tokens.size >= 1 &&
        out0.tokens(0)._1 == TreasuryNftId &&
        out0.tokens(0)._2 == 1L &&
        out0.value >= SELF.value - awarded
      val recipientValid: Boolean =
        blake2b256(OUTPUTS(1).propositionBytes) == recipientHash &&
        OUTPUTS(1).value >= awarded
      isPartialWithdrawal && treasuryPreserved && recipientValid
    } else {
      false
    }
  }
  val isNewTreasury: Boolean = {
    if (hasProposalToken) {
      val proposalTuple = proposalBox.R4[(Long, Long)].get
      val proportion: Long = proposalTuple._1
      val recipientScript: Coll[Byte] = proposalBox.R5[Coll[Byte]].get
      val isFullTransfer: Boolean = proportion == Denom
      val newTreasuryValid: Boolean =
        out0.value >= SELF.value &&
        out0.tokens.size >= 1 &&
        out0.tokens(0)._1 == TreasuryNftId &&
        out0.tokens(0)._2 == 1L &&
        blake2b256(out0.propositionBytes) == recipientScript
      isFullTransfer && newTreasuryValid
    } else {
      false
    }
  }
  sigmaProp(selfValid && (isDeposit || isWithdrawal || isNewTreasury))
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "Treasury contract failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_session5_filter_full() {
        let result = compile_expr(
            "{ val found = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == SELF.tokens(0)._1 }; sigmaProp(found.size == 1) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    // ================================================================
    // P2P OPTIONS CONTRACTS
    // ================================================================

    #[test]
    fn test_p2p_proxy_deposit() {
        // ProxyDeposit.es — simple proxy with SigmaProp &&/|| and register access
        // Requires: R4[SigmaProp], R5[Long], R6[Long], R7[Coll[Byte]], SigmaProp &&/||
        let source = r#"{
  val POOL_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val depositorPK = SELF.R4[SigmaProp].get
  val refundHeight = SELF.R5[Long].get
  val validFulfillment = {
    val poolBox = INPUTS(0)
    val depositorOutput = OUTPUTS(1)
    poolBox.tokens(0)._1 == POOL_NFT_ID &&
    depositorOutput.R7[Coll[Byte]].get == SELF.id &&
    depositorOutput.tokens(0)._2 >= SELF.R6[Long].get
  }
  val validRefund = HEIGHT > refundHeight
  sigmaProp(validFulfillment) || (sigmaProp(validRefund) && depositorPK)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "ProxyDeposit failed: {:?}", result.err());
    }

    #[test]
    fn test_p2p_proxy_withdraw() {
        // ProxyWithdraw.es — mirror of ProxyDeposit
        let source = r#"{
  val POOL_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val withdrawerPK = SELF.R4[SigmaProp].get
  val refundHeight = SELF.R5[Long].get
  val validFulfillment = {
    val poolBox = INPUTS(0)
    val withdrawerOutput = OUTPUTS(1)
    poolBox.tokens(0)._1 == POOL_NFT_ID &&
    withdrawerOutput.R7[Coll[Byte]].get == SELF.id &&
    withdrawerOutput.tokens(0)._2 >= SELF.R6[Long].get
  }
  val validRefund = HEIGHT > refundHeight
  sigmaProp(validFulfillment) || (sigmaProp(validRefund) && withdrawerPK)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "ProxyWithdraw failed: {:?}", result.err());
    }

    #[test]
    fn test_p2p_option_escrow() {
        // OptionEscrow.es — auto-exercise escrow with SigmaProp ||
        let source = r#"{
  val POOL_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val buyerPropBytes = SELF.R4[Coll[Byte]].get
  val buyerPK = SELF.R5[SigmaProp].get
  val validExercise = {
    val poolBox = INPUTS(0)
    poolBox.tokens(0)._1 == POOL_NFT_ID &&
    OUTPUTS(2).propositionBytes == buyerPropBytes
  }
  val validCancel = buyerPK
  sigmaProp(validExercise) || validCancel
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "OptionEscrow failed: {:?}", result.err());
    }

    #[test]
    fn test_p2p_token_registry() {
        // TokenRegistry.es — governance guarded registry
        // Requires: R4[Coll[Coll[Byte]]], R5[Coll[Long]], R6[SigmaProp], .isDefined
        // NOTE: Coll[Coll[Byte]] needs nested generic parsing, .isDefined is new
        let source = r#"{
  val REGISTRY_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val registryNFT = SELF.tokens(0)._1 == REGISTRY_NFT_ID
  val successor = OUTPUTS(0)
  val nftPreserved = successor.tokens(0)._1 == REGISTRY_NFT_ID
  val scriptPreserved = successor.propositionBytes == SELF.propositionBytes
  val valuePreserved = successor.value >= SELF.value
  val governancePK = SELF.R6[SigmaProp].get
  sigmaProp(
    registryNFT && nftPreserved && scriptPreserved && valuePreserved
  ) && governancePK
}"#;
        // Simplified version without Coll[Coll[Byte]] size checks and .isDefined
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "TokenRegistry (simplified) failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_buy_token_request() {
        // BuyTokenRequest.es — requires .propBytes on SigmaProp, getOrElse
        // Simplified: skip getOrElse and .propBytes (not yet implemented)
        let source = r#"{
  val PAYMENT_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val userPKIn: SigmaProp = SELF.R4[SigmaProp].get
  val validBuyToken = if (OUTPUTS(1).tokens.size >= 1) {
    val tokenTokenId = SELF.R5[Coll[Byte]].get
    val tokenAmountRequested = SELF.R6[Long].get
    val validPayment = SELF.tokens(0)._1 == PAYMENT_TOKEN_ID
    OUTPUTS(1).tokens(0)._1 == tokenTokenId &&
    OUTPUTS(1).tokens(0)._2 >= tokenAmountRequested &&
    validPayment
  } else {
    false
  }
  userPKIn || sigmaProp(validBuyToken)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "BuyTokenRequest (simplified) failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_fixed_price_sell() {
        // FixedPriceSell.es — requires .propBytes, .exists with tuple lambda, getOrElse
        // Testing the core logic with simplifications
        let source = r#"{
  val PAYMENT_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val sellerPK = SELF.R4[SigmaProp].get
  val pricingParams = SELF.R5[Coll[Long]].get
  val premiumPerToken = pricingParams(0)
  val dAppUIFeePer1000 = pricingParams(1)
  val txFee = pricingParams(2)
  val dAppUIFeeTree = SELF.R6[Coll[Byte]].get
  val optionTokenId = SELF.tokens(0)._1
  val inputTokens = SELF.tokens(0)._2
  val validSale = {
    val successor = OUTPUTS(0)
    val outputTokens = if (successor.tokens.size > 0 &&
                           successor.tokens(0)._1 == optionTokenId) {
      successor.tokens(0)._2
    } else { 0L }
    val tokensSold = inputTokens - outputTokens
    val totalPrice = tokensSold * premiumPerToken
    val uiFee = totalPrice * dAppUIFeePer1000 / 1000L
    val sellerPaid = totalPrice - uiFee
    val validSuccessor = if (outputTokens > 0L) {
      successor.propositionBytes == SELF.propositionBytes &&
      successor.R5[Coll[Long]].get == pricingParams &&
      successor.R6[Coll[Byte]].get == dAppUIFeeTree
    } else { true }
    val sellerBox = OUTPUTS(1)
    val sellerReceived = sellerBox.tokens.exists { (t: (Coll[Byte], Long)) =>
      t._1 == PAYMENT_TOKEN_ID && t._2 >= sellerPaid
    }
    val uiFeeValid = if (uiFee > 0L) {
      val feeBox = OUTPUTS(2)
      feeBox.propositionBytes == dAppUIFeeTree &&
      feeBox.tokens.exists { (t: (Coll[Byte], Long)) =>
        t._1 == PAYMENT_TOKEN_ID && t._2 >= uiFee
      }
    } else { true }
    tokensSold > 0L && validSuccessor && sellerReceived && uiFeeValid
  }
  sigmaProp(validSale) || sellerPK
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "FixedPriceSell failed: {:?}", result.err());
    }

    #[test]
    fn test_p2p_option_reserve_v2_core_logic() {
        // OptionReserveV2 core logic — tests the most complex patterns we CAN compile:
        // - Nested tuple type annotations: R8[Coll[Long]], R9[Coll[Coll[Byte]]]
        // - Collection indexing: params(0), params(1)
        // - Complex boolean chains with if/else
        // - Multiple execution paths with nested blocks
        // - Register preservation checks
        // NOTE: Excludes getOrElse, propBytes, decodePoint, toInt/toLong
        let source = r#"{
  val REGISTRY_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val COMPANION_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val USE_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val SIGUSD_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000004")
  val ORACLE_DECIMAL: Long = 1000000L
  val MIN_BOX_VALUE: Long = 1000000L
  val EXERCISE_WINDOW: Long = 720L

  val optionTokenId: Coll[Byte] = SELF.R7[Coll[Byte]].get
  val isMinted: Boolean = SELF.tokens(0)._1 == optionTokenId

  val optionName: Coll[Byte] = SELF.R4[Coll[Byte]].get
  val underlyingAssetTokenId: Coll[Byte] = SELF.R5[Coll[Byte]].get
  val optionDecimals: Coll[Byte] = SELF.R6[Coll[Byte]].get
  val params: Coll[Long] = SELF.R8[Coll[Long]].get
  val optionType = params(0)
  val style = params(1)
  val shareSize = params(2)
  val maturityDate = params(3)
  val strikePrice = params(4)
  val dAppUIMintFee = params(5)
  val txFee = params(6)
  val settlementType = params(8)
  val collateralCap = params(9)
  val stablecoinDecimal = params(10)

  val MinReserveValue: Long = txFee + MIN_BOX_VALUE
  val isOptionDelivered: Boolean = isMinted && SELF.tokens(0)._2 == 1L
  val isExpired: Boolean = HEIGHT > maturityDate + EXERCISE_WINDOW

  val isInExerciseWindow: Boolean = if (style == 0L) {
    isOptionDelivered && HEIGHT >= maturityDate && HEIGHT <= maturityDate + EXERCISE_WINDOW
  } else {
    isOptionDelivered && HEIGHT <= maturityDate + EXERCISE_WINDOW
  }

  val validBasicReplicatedOutput0: Boolean = if (OUTPUTS(0).propositionBytes == SELF.propositionBytes) {
    OUTPUTS(0).value >= MinReserveValue &&
    OUTPUTS(0).R4[Coll[Byte]].get == optionName &&
    OUTPUTS(0).R5[Coll[Byte]].get == underlyingAssetTokenId &&
    OUTPUTS(0).R6[Coll[Byte]].get == optionDecimals &&
    OUTPUTS(0).R8[Coll[Long]].get == params
  } else {
    false
  }

  val validExerciseSuccessor: Boolean = validBasicReplicatedOutput0 &&
    OUTPUTS(0).R7[Coll[Byte]].get == optionTokenId &&
    OUTPUTS(0).tokens(0)._1 == SELF.tokens(0)._1 && OUTPUTS(0).tokens(0)._2 == 1L

  val optionTokensExercised: Long = INPUTS.fold(0L, { (acc: Long, box: Box) =>
    if (box.id != SELF.id) {
      box.tokens.fold(acc, { (innerAcc: Long, t: (Coll[Byte], Long)) =>
        if (t._1 == optionTokenId) innerAcc + t._2
        else innerAcc
      })
    } else acc
  })

  val validMint: Boolean = if (!isMinted && INPUTS.size == 1 && OUTPUTS.size == 3) {
    val strikePerContract = strikePrice * stablecoinDecimal / ORACLE_DECIMAL
    val validStrikeConversion = strikePerContract > 0L
    validStrikeConversion &&
    validBasicReplicatedOutput0 &&
    OUTPUTS(0).R7[Coll[Byte]].get == SELF.id &&
    OUTPUTS(0).value == SELF.value - txFee - dAppUIMintFee &&
    OUTPUTS(0).value >= 2L * MinReserveValue &&
    OUTPUTS(0).tokens(0)._1 == SELF.id
  } else {
    false
  }

  val validDelivery: Boolean = if (isMinted && !isOptionDelivered && INPUTS.size == 1 && OUTPUTS.size == 3) {
    OUTPUTS(0).value == SELF.value - txFee - MIN_BOX_VALUE &&
    validBasicReplicatedOutput0 &&
    OUTPUTS(0).R7[Coll[Byte]].get == optionTokenId &&
    OUTPUTS(0).tokens(0)._1 == SELF.tokens(0)._1 &&
    OUTPUTS(0).tokens(0)._2 == 1L
  } else {
    false
  }

  val validCashExercise: Boolean = if (isInExerciseWindow && settlementType == 1L && optionTokensExercised > 0L) {
    val collateralTokenId = SELF.tokens(1)._1
    val validStablecoin = collateralTokenId == USE_TOKEN_ID || collateralTokenId == SIGUSD_TOKEN_ID
    val oracleBox = CONTEXT.dataInputs(0)
    val validOracle = oracleBox.tokens(0)._1 == COMPANION_NFT_ID
    val prices = oracleBox.R8[Coll[Long]].get
    val spotPrice = prices(0)
    val profitPerContractOracle: Long = if (optionType == 0L) {
      if (spotPrice > strikePrice) {
        val raw = spotPrice - strikePrice
        if (raw > collateralCap) collateralCap else raw
      } else { 0L }
    } else {
      if (spotPrice < strikePrice) {
        val raw = strikePrice - spotPrice
        if (raw > collateralCap) collateralCap else raw
      } else { 0L }
    }
    val inTheMoney = profitPerContractOracle > 0L
    val payoutPerContract = profitPerContractOracle * stablecoinDecimal / ORACLE_DECIMAL
    val totalPayout = payoutPerContract * optionTokensExercised
    val validPayout = payoutPerContract > 0L
    val selfBalance = SELF.tokens(1)._2
    val exerciserBox = OUTPUTS(1)
    val exerciserPaid = exerciserBox.tokens.exists { (t: (Coll[Byte], Long)) =>
      t._1 == collateralTokenId && t._2 >= totalPayout
    }
    validStablecoin && validOracle && inTheMoney && validPayout &&
    exerciserPaid && validExerciseSuccessor &&
    OUTPUTS(0).value >= SELF.value - txFee
  } else {
    false
  }

  val validCloseOptionContract: Boolean = if (isExpired) {
    OUTPUTS.size == 2 &&
    OUTPUTS(0).value >= SELF.value - txFee
  } else {
    false
  }

  sigmaProp(
    validMint ||
    validDelivery ||
    validCashExercise ||
    validCloseOptionContract
  )
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "OptionReserveV2 core logic failed: {:?}",
            result.err()
        );
    }

    fn compile_to_hex(source: &str) -> String {
        use ergotree_ir::serialization::SigmaSerializable;
        let tree = compile(source, ScriptEnv::new()).unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    // ================================================================
    // BYTE-FOR-BYTE NODE COMPARISON TESTS
    // These expressions have NO val bindings (avoiding constant-folding
    // differences between Scala compiler and our compiler).
    // All hex values verified against Ergo node localhost:9053.
    // ================================================================

    #[test]
    fn test_node_match_sigma_and() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(true) && sigmaProp(false) }"),
            "100201010100ea02d17300d17301"
        );
    }

    #[test]
    fn test_node_match_sigma_or() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(true) || sigmaProp(false) }"),
            "100201010100eb02d17300d17301"
        );
    }

    #[test]
    fn test_node_match_tuple_register_direct() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(SELF.R4[(Long, Long)].get._1 > 0L) }"),
            "10010500d1918ce4c6a70459017300"
        );
    }

    #[test]
    fn test_node_match_coll_byte_register_direct() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(SELF.R5[Coll[Byte]].get.size > 0) }"),
            "10010400d191b1e4c6a7050e7300"
        );
    }

    #[test]
    fn test_node_match_is_defined_direct() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(SELF.R4[Long].isDefined) }"),
            "1000d1e6c6a70405"
        );
    }

    #[test]
    fn test_node_match_prop_bytes_direct() {
        assert_eq!(
            compile_to_hex(
                "{ sigmaProp(SELF.R4[SigmaProp].get.propBytes == SELF.propositionBytes) }"
            ),
            "1000d193d0e4c6a70408c2a7"
        );
    }

    #[test]
    fn test_node_match_preheader_direct() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(CONTEXT.preHeader.timestamp > 0L) }"),
            "10010500d191db6903db6503fe7300"
        );
    }

    #[test]
    fn test_node_match_modulo_direct() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(SELF.value % 10L > 0L) }"),
            "100205140500d1919ec1a773007301"
        );
    }

    #[test]
    fn test_node_match_to_long_direct() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(SELF.tokens.size.toLong > 0L) }"),
            "10010500d1917eb1db6308a7057300"
        );
    }

    /// Compare our compiler output size vs node output for contracts.
    /// The Scala compiler optimizes (CSE, constant folding, negation elimination)
    /// so our output is larger but semantically equivalent.
    /// This test verifies our output round-trips and reports the size ratio.
    #[test]
    fn test_node_size_comparison_report() {
        use ergotree_ir::serialization::SigmaSerializable;

        let contracts: Vec<(&str, &str)> = vec![
            (
                "sigmaProp(HEIGHT > 0 && HEIGHT < 100)",
                "1002040004c801d1ed91a373008fa37301",
            ),
            ("sigmaProp(SELF.value > 0L)", "10010500d191c1a77300"),
            (
                "sigmaProp(INPUTS.exists { (b: Box) => b.value > 0L })",
                "10010500d1aea4d901016391c172017300",
            ),
            (
                "if (HEIGHT > 100) sigmaProp(true) else sigmaProp(false)",
                "100304c801010101009591a37300d17301d17302",
            ),
            (
                "sigmaProp(true) && sigmaProp(false)",
                "100201010100ea02d17300d17301",
            ),
            (
                "sigmaProp(true) || sigmaProp(false)",
                "100201010100eb02d17300d17301",
            ),
            (
                "sigmaProp(SELF.R4[(Long, Long)].get._1 > 0L)",
                "10010500d1918ce4c6a70459017300",
            ),
            (
                "sigmaProp(SELF.R5[Coll[Byte]].get.size > 0)",
                "10010400d191b1e4c6a7050e7300",
            ),
            ("sigmaProp(SELF.R4[Long].isDefined)", "1000d1e6c6a70405"),
            (
                "sigmaProp(SELF.R4[SigmaProp].get.propBytes == SELF.propositionBytes)",
                "1000d193d0e4c6a70408c2a7",
            ),
            (
                "sigmaProp(CONTEXT.preHeader.timestamp > 0L)",
                "10010500d191db6903db6503fe7300",
            ),
            (
                "sigmaProp(SELF.value % 10L > 0L)",
                "100205140500d1919ec1a773007301",
            ),
            (
                "sigmaProp(SELF.tokens.size.toLong > 0L)",
                "10010500d1917eb1db6308a7057300",
            ),
        ];

        let mut matches = 0;
        let mut total = 0;
        for (source, node_hex) in &contracts {
            total += 1;
            let full_source = format!("{{ {} }}", source);
            let tree = compile(&full_source, ScriptEnv::new()).unwrap();
            let bytes = tree.sigma_serialize_bytes().unwrap();
            let our_hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

            // Verify round-trip
            let tree2 = ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(&bytes).unwrap();
            assert_eq!(
                bytes,
                tree2.sigma_serialize_bytes().unwrap(),
                "Round-trip failed for: {}",
                source
            );

            if our_hex == *node_hex {
                matches += 1;
            }
        }
        // At least the simple expressions should match
        assert!(
            matches >= 13,
            "Only {}/{} byte-matched the node (expected >= 13)",
            matches,
            total
        );
    }

    #[test]
    fn test_node_match_not_direct() {
        // Node optimizes !(HEIGHT > 100) → HEIGHT <= 100 (negation elimination).
        // With optimization passes, our compiler now matches the node byte-for-byte.
        assert_eq!(
            compile_to_hex("{ sigmaProp(!(HEIGHT > 100)) }"),
            "100104c801d190a37300"
        );
    }

    #[test]
    fn test_node_match_get_or_else_direct() {
        assert_eq!(
            compile_to_hex(
                r#"{ sigmaProp(SELF.tokens.getOrElse(0, (fromBase16("00"), 0L))._2 > 0L) }"#
            ),
            "100404000e010005000500d1918cb2db6308a7730001860273017302027303"
        );
    }

    #[test]
    fn test_node_match_slice_direct() {
        assert_eq!(
            compile_to_hex("{ sigmaProp(INPUTS.slice(0, 1).size > 0) }"),
            "1003040004020400d191b1b4a4730073017302"
        );
    }

    #[test]
    fn test_feature_is_defined() {
        let result = compile_expr(
            "{ val x = SELF.R4[Long]; sigmaProp(x.isDefined) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "isDefined failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_to_long_to_int() {
        let result = compile_expr(
            "{ val x: Int = 42; val y: Long = x.toLong; sigmaProp(y > 0L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "toLong failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_subst_constants() {
        let result = compile_expr(
            "{ val script = SELF.propositionBytes; val positions = Coll[Int](0); val values = Coll[Int](1); val result = substConstants(script, positions, values); sigmaProp(result.size > 0) }",
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "substConstants failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_feature_byte_array_to_long() {
        let result = compile_expr(
            "{ val bytes = longToByteArray(1L); sigmaProp(byteArrayToLong(bytes) == 1L) }",
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "byteArrayToLong failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_feature_byte_array_to_bigint() {
        let result = compile_expr(
            "{ val bytes = longToByteArray(42L); val bi = byteArrayToBigInt(bytes); sigmaProp(bi == bi) }",
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "byteArrayToBigInt failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_feature_xor() {
        let result = compile_expr(
            r#"{ val a = SELF.propositionBytes; val b = SELF.propositionBytes; val result = xor(a, b); sigmaProp(result.size > 0) }"#,
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "xor failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_xor_of() {
        let result = compile_expr(
            "{ val bools = Coll[Boolean](true, false, true); sigmaProp(xorOf(bools)) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "xorOf failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_decode_point() {
        let result = compile_expr(
            r#"{ val ecPoint = decodePoint(fromBase16("02d04baf1e643c82e9e25f35a8636e1c4ae9bfc12944af9c8dd9b6a47fd7f8b700")); sigmaProp(proveDlog(ecPoint)) }"#,
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "decodePoint failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_prop_bytes() {
        let result = compile_expr(
            "{ val pk = SELF.R4[SigmaProp].get; sigmaProp(pk.propBytes == SELF.propositionBytes) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "propBytes failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_get_or_else() {
        let result = compile_expr(
            "{ val t = SELF.tokens.getOrElse(0, (fromBase16(\"00\"), 0L)); sigmaProp(t._2 > 0L) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "getOrElse failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_preheader_timestamp() {
        let result = compile_expr(
            "{ val ts = CONTEXT.preHeader.timestamp; sigmaProp(ts > 0L) }",
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "preHeader.timestamp failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_feature_logical_not() {
        let result = compile_expr("{ val x = true; sigmaProp(!x) }", ScriptEnv::new());
        assert!(result.is_ok(), "LogicalNot failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_coll_literal() {
        let result = compile_expr(
            "{ val x = Coll(1, 2, 3); sigmaProp(x.size > 0) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Coll literal failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_coll_typed_empty() {
        // Coll[Byte]() — typed empty collection (used in getOrElse defaults)
        let result = compile_expr(
            r#"{ val empty: Coll[Byte] = Coll[Byte](); sigmaProp(empty.size == 0) }"#,
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "Coll[Byte]() failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_get_var() {
        // Test getVar without .get first
        let result = compile_expr(
            "{ val proof = getVar[Coll[Byte]](0); sigmaProp(proof.isDefined) }",
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "getVar isDefined failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_feature_get_var_get() {
        let result = compile_expr(
            "{ val proof = getVar[Coll[Byte]](0).get; sigmaProp(proof.size > 0) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "getVar.get failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_avl_tree_insert_update() {
        let result = compile_expr(
            r#"{ val tree = SELF.R5[AvlTree].get
  val proof = getVar[Coll[Byte]](0).get
  val key = getVar[Coll[Byte]](2).get
  val value = getVar[Coll[Byte]](3).get
  val opType = getVar[Int](1).get
  val newTree = if (opType == 0) {
    tree.insert(Coll((key, value)), proof).get
  } else {
    tree.update(Coll((key, value)), proof).get
  }
  val outputTree = OUTPUTS(0).R5[AvlTree].get
  sigmaProp(outputTree.digest == newTree.digest)
}"#,
            ScriptEnv::new(),
        );
        assert!(
            result.is_ok(),
            "AvlTree insert/update failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_feature_avl_tree_get() {
        let result = compile_expr(
            r#"{ val tree = SELF.R5[AvlTree].get
  val key = getVar[Coll[Byte]](2).get
  val proof = getVar[Coll[Byte]](11).get
  val value = tree.get(key, proof).get
  sigmaProp(value.size > 0)
}"#,
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "AvlTree.get failed: {:?}", result.err());
    }

    #[test]
    fn test_feature_avl_tree_digest() {
        let result = compile_expr(
            "{ val tree = SELF.R5[AvlTree].get; sigmaProp(tree.digest.size > 0) }",
            ScriptEnv::new(),
        );
        assert!(result.is_ok(), "AvlTree digest failed: {:?}", result.err());
    }

    // ============================================================
    // FULL CONTRACT TESTS (no simplifications)
    // ============================================================

    #[test]
    fn test_p2p_proxy_deposit_full() {
        let source = r#"{
  val POOL_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val depositorPK = SELF.R4[SigmaProp].get
  val refundHeight = SELF.R5[Long].get
  val validFulfillment = {
    val poolBox = INPUTS(0)
    val depositorOutput = OUTPUTS(1)
    poolBox.tokens(0)._1 == POOL_NFT_ID &&
    depositorOutput.R7[Coll[Byte]].get == SELF.id &&
    depositorOutput.tokens(0)._2 >= SELF.R6[Long].get
  }
  val validRefund = HEIGHT > refundHeight
  sigmaProp(validFulfillment) || (sigmaProp(validRefund) && depositorPK)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "ProxyDeposit full failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_option_escrow_full() {
        let source = r#"{
  val POOL_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val buyerPropBytes = SELF.R4[Coll[Byte]].get
  val buyerPK = SELF.R5[SigmaProp].get
  val validExercise = {
    val poolBox = INPUTS(0)
    poolBox.tokens(0)._1 == POOL_NFT_ID &&
    OUTPUTS(2).propositionBytes == buyerPropBytes
  }
  val validCancel = buyerPK
  sigmaProp(validExercise) || validCancel
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "OptionEscrow full failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_token_registry_full() {
        // Full contract with Coll[Coll[Byte]] register type and .isDefined
        let source = r#"{
  val REGISTRY_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val registryNFT = SELF.tokens(0)._1 == REGISTRY_NFT_ID
  val successor = OUTPUTS(0)
  val nftPreserved = successor.tokens(0)._1 == REGISTRY_NFT_ID
  val scriptPreserved = successor.propositionBytes == SELF.propositionBytes
  val r4SizeOk = successor.R4[Coll[Coll[Byte]]].get.size >= SELF.R4[Coll[Coll[Byte]]].get.size
  val r5SizeOk = successor.R5[Coll[Long]].get.size >= SELF.R5[Coll[Long]].get.size
  val arraysSynced = successor.R4[Coll[Coll[Byte]]].get.size == successor.R5[Coll[Long]].get.size
  val valuePreserved = successor.value >= SELF.value
  val r6Valid = successor.R6[SigmaProp].isDefined
  val governancePK = SELF.R6[SigmaProp].get
  sigmaProp(
    registryNFT && nftPreserved && scriptPreserved &&
    r4SizeOk && r5SizeOk && arraysSynced &&
    valuePreserved && r6Valid
  ) && governancePK
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "TokenRegistry full failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_buy_token_request_full() {
        // Full contract with .propBytes and getOrElse
        let source = r#"{
  val PAYMENT_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val userPKIn: SigmaProp = SELF.R4[SigmaProp].get
  val validBuyToken = if (OUTPUTS(1).tokens.size >= 1) {
    val tokenTokenId = SELF.R5[Coll[Byte]].get
    val tokenAmountRequested = SELF.R6[Long].get
    val output1Token0: (Coll[Byte], Long) = OUTPUTS(1).tokens.getOrElse(0, (Coll(0.toByte), 0L))
    val validPayment = SELF.tokens(0)._1 == PAYMENT_TOKEN_ID
    OUTPUTS(1).propositionBytes == userPKIn.propBytes &&
    output1Token0._1 == tokenTokenId &&
    output1Token0._2 >= tokenAmountRequested &&
    validPayment
  } else {
    false
  }
  userPKIn || sigmaProp(validBuyToken)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "BuyTokenRequest full failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_fixed_price_sell_full() {
        // Full FixedPriceSell with .propBytes, .exists tuple lambda, getOrElse
        let source = r#"{
  val PAYMENT_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val sellerPK = SELF.R4[SigmaProp].get
  val pricingParams = SELF.R5[Coll[Long]].get
  val premiumPerToken = pricingParams(0)
  val dAppUIFeePer1000 = pricingParams(1)
  val txFee = pricingParams(2)
  val dAppUIFeeTree = SELF.R6[Coll[Byte]].get
  val optionTokenId = SELF.tokens(0)._1
  val inputTokens = SELF.tokens(0)._2
  val validSale = {
    val successor = OUTPUTS(0)
    val outputTokens = if (successor.tokens.size > 0 &&
                           successor.tokens(0)._1 == optionTokenId) {
      successor.tokens(0)._2
    } else { 0L }
    val tokensSold = inputTokens - outputTokens
    val totalPrice = tokensSold * premiumPerToken
    val uiFee = totalPrice * dAppUIFeePer1000 / 1000L
    val sellerPaid = totalPrice - uiFee
    val validSuccessor = if (outputTokens > 0L) {
      successor.propositionBytes == SELF.propositionBytes &&
      successor.R4[SigmaProp].get == sellerPK &&
      successor.R5[Coll[Long]].get == pricingParams &&
      successor.R6[Coll[Byte]].get == dAppUIFeeTree
    } else { true }
    val sellerBox = OUTPUTS(1)
    val sellerReceived = sellerBox.propositionBytes == sellerPK.propBytes &&
                         sellerBox.tokens.exists { (t: (Coll[Byte], Long)) =>
                             t._1 == PAYMENT_TOKEN_ID && t._2 >= sellerPaid
                         }
    val uiFeeValid = if (uiFee > 0L) {
      val feeBox = OUTPUTS(2)
      feeBox.propositionBytes == dAppUIFeeTree &&
      feeBox.tokens.exists { (t: (Coll[Byte], Long)) =>
        t._1 == PAYMENT_TOKEN_ID && t._2 >= uiFee
      }
    } else { true }
    tokensSold > 0L && validSuccessor && sellerReceived && uiFeeValid
  }
  sigmaProp(validSale) || sellerPK
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "FixedPriceSell full failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_option_reserve_v2_full() {
        // Full OptionReserveV2.es — 483 lines, 5 execution paths
        // Replaced compile-time constants with fromBase16 placeholders
        let source = r#"{
    val ORACLE_POOL_NFT: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000010")
    val COMPANION_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000011")
    val REGISTRY_NFT_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000012")
    val USE_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000013")
    val SIGUSD_TOKEN_ID: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000014")
    val ORACLE_DECIMAL: Long = 1000000L
    val MIN_BOX_VALUE: Long = 1000000L
    val EXERCISE_WINDOW: Long = 720L
    val ERG_ORACLE_INDEX: Long = 17L

    val selfToken0: (Coll[Byte], Long) = SELF.tokens.getOrElse(0, (Coll[Byte](), 0L))
    val selfToken1: (Coll[Byte], Long) = SELF.tokens.getOrElse(1, (Coll[Byte](), 0L))
    val output0Token0: (Coll[Byte], Long) = OUTPUTS(0).tokens.getOrElse(0, (Coll[Byte](), 0L))
    val output0Token1: (Coll[Byte], Long) = OUTPUTS(0).tokens.getOrElse(1, (Coll[Byte](), 0L))
    val output1Token0: (Coll[Byte], Long) = OUTPUTS(1).tokens.getOrElse(0, (Coll[Byte](), 0L))

    val optionTokenId: Coll[Byte] = SELF.R7[Coll[Byte]].get
    val isMinted: Boolean = selfToken0._1 == optionTokenId

    val optionName: Coll[Byte] = SELF.R4[Coll[Byte]].get
    val underlyingAssetTokenId: Coll[Byte] = SELF.R5[Coll[Byte]].get
    val optionDecimals: Coll[Byte] = SELF.R6[Coll[Byte]].get
    val params: Coll[Long] = SELF.R8[Coll[Long]].get
    val optionType = params(0)
    val style = params(1)
    val shareSize = params(2)
    val maturityDate = params(3)
    val strikePrice = params(4)
    val dAppUIMintFee = params(5)
    val txFee = params(6)
    val oracleIndex = params(7)
    val settlementType = params(8)
    val collateralCap = params(9)
    val stablecoinDecimal = params(10)

    val issuerData: Coll[Coll[Byte]] = SELF.R9[Coll[Coll[Byte]]].get
    val issuerECPoint = decodePoint(issuerData(0))
    val dAppUIFeeTree: Coll[Byte] = issuerData(1)
    val issuerPropBytes: Coll[Byte] = proveDlog(issuerECPoint).propBytes

    val MinReserveValue: Long = txFee + MIN_BOX_VALUE

    val isOptionDelivered: Boolean = isMinted && selfToken0._2 == 1L
    val isExpired: Boolean = HEIGHT > maturityDate + EXERCISE_WINDOW

    val isInExerciseWindow: Boolean = if (style == 0L) {
        isOptionDelivered && HEIGHT >= maturityDate && HEIGHT <= maturityDate + EXERCISE_WINDOW
    } else {
        isOptionDelivered && HEIGHT <= maturityDate + EXERCISE_WINDOW
    }

    val validBasicReplicatedOutput0: Boolean = if (OUTPUTS(0).propositionBytes == SELF.propositionBytes) {
        OUTPUTS(0).value >= MinReserveValue &&
        OUTPUTS(0).R4[Coll[Byte]].get == optionName &&
        OUTPUTS(0).R5[Coll[Byte]].get == underlyingAssetTokenId &&
        OUTPUTS(0).R6[Coll[Byte]].get == optionDecimals &&
        OUTPUTS(0).R8[Coll[Long]].get == params &&
        OUTPUTS(0).R9[Coll[Coll[Byte]]].get == issuerData
    } else {
        false
    }

    val validExerciseSuccessor: Boolean = validBasicReplicatedOutput0 &&
        OUTPUTS(0).R7[Coll[Byte]].get == optionTokenId &&
        output0Token0._1 == selfToken0._1 && output0Token0._2 == 1L

    val optionTokensExercised: Long = INPUTS.fold(0L, { (acc: Long, box: Box) =>
        if (box.id != SELF.id) {
            box.tokens.fold(acc, { (innerAcc: Long, t: (Coll[Byte], Long)) =>
                if (t._1 == optionTokenId) innerAcc + t._2
                else innerAcc
            })
        } else acc
    })

    val validMint: Boolean = if (!isMinted && INPUTS.size == 1 && OUTPUTS.size == 3) {
        val registryValid: Boolean = if (settlementType == 0L) {
            val registryBox = CONTEXT.dataInputs(0)
            val validRegistryNFT = registryBox.tokens(0)._1 == REGISTRY_NFT_ID
            val registryTokens = registryBox.R4[Coll[Coll[Byte]]].get
            val registryRates = registryBox.R5[Coll[Long]].get
            val expectedTokenId = registryTokens(oracleIndex.toInt)
            val tokensPerUnit = registryRates(oracleIndex.toInt)
            if (oracleIndex == ERG_ORACLE_INDEX) {
                validRegistryNFT && tokensPerUnit > 0L
            } else if (optionType == 0L) {
                validRegistryNFT &&
                expectedTokenId.size > 0 && tokensPerUnit > 0L &&
                SELF.tokens(0)._1 == expectedTokenId
            } else {
                validRegistryNFT &&
                expectedTokenId.size > 0 && tokensPerUnit > 0L
            }
        } else {
            val collateralTokenId = SELF.tokens(0)._1
            collateralTokenId == USE_TOKEN_ID || collateralTokenId == SIGUSD_TOKEN_ID
        }

        val validStablecoinDecimal: Boolean = if (settlementType == 0L) {
            stablecoinDecimal == 1000L || stablecoinDecimal == 100L
        } else {
            (stablecoinDecimal == 1000L && SELF.tokens(0)._1 == USE_TOKEN_ID) ||
            (stablecoinDecimal == 100L && SELF.tokens(0)._1 == SIGUSD_TOKEN_ID)
        }

        val strikePerContract = strikePrice * stablecoinDecimal / ORACLE_DECIMAL
        val validStrikeConversion = strikePerContract > 0L

        registryValid && validStablecoinDecimal && validStrikeConversion &&
        validBasicReplicatedOutput0 &&
        OUTPUTS(0).R7[Coll[Byte]].get == SELF.id &&
        OUTPUTS(0).value == SELF.value - txFee - dAppUIMintFee &&
        OUTPUTS(0).value >= 2L * MinReserveValue &&
        output0Token0._1 == SELF.id &&
        (
            (
                optionType == 0L && settlementType == 0L &&
                oracleIndex != ERG_ORACLE_INDEX &&
                output0Token1._1 == underlyingAssetTokenId &&
                output0Token1 == selfToken0 &&
                output0Token0._2 == selfToken0._2 / shareSize + 1L &&
                OUTPUTS(0).tokens.size == 2
            )
            ||
            (
                optionType == 0L && settlementType == 0L &&
                oracleIndex == ERG_ORACLE_INDEX && {
                    val registryBox = CONTEXT.dataInputs(0)
                    val tokensPerUnit = registryBox.R5[Coll[Long]].get(ERG_ORACLE_INDEX.toInt)
                    val nanoErgPerContract = shareSize * tokensPerUnit / ORACLE_DECIMAL
                    val availableCollateral = SELF.value - 3L * txFee - dAppUIMintFee - 2L * MIN_BOX_VALUE
                    nanoErgPerContract > 0L &&
                    output0Token0._2 == availableCollateral / nanoErgPerContract + 1L &&
                    OUTPUTS(0).tokens.size == 1 &&
                    OUTPUTS(0).value >= SELF.value - txFee - dAppUIMintFee
                }
            )
            ||
            (
                optionType == 1L && settlementType == 0L &&
                output0Token1._1 == underlyingAssetTokenId &&
                output0Token1 == selfToken0 &&
                strikePerContract > 0L &&
                output0Token0._2 == selfToken0._2 / strikePerContract + 1L &&
                OUTPUTS(0).tokens.size == 2
            )
            ||
            (
                settlementType == 1L && {
                    val capPerContract = collateralCap * stablecoinDecimal / ORACLE_DECIMAL
                    capPerContract > 0L &&
                    output0Token1._1 == underlyingAssetTokenId &&
                    output0Token1 == selfToken0 &&
                    output0Token0._2 == selfToken0._2 / capPerContract + 1L &&
                    OUTPUTS(0).tokens.size == 2
                }
            )
        ) &&
        OUTPUTS(1).propositionBytes == dAppUIFeeTree &&
        OUTPUTS(1).tokens.size == 0 &&
        OUTPUTS(1).value >= dAppUIMintFee
    } else {
        false
    }

    val validDelivery: Boolean = if (isMinted && !isOptionDelivered && INPUTS.size == 1 && OUTPUTS.size == 3) {
        OUTPUTS(0).value == SELF.value - txFee - MIN_BOX_VALUE &&
        validBasicReplicatedOutput0 &&
        OUTPUTS(0).R7[Coll[Byte]].get == optionTokenId &&
        output0Token0._1 == selfToken0._1 &&
        output0Token0._2 == 1L &&
        output0Token1 == selfToken1 &&
        OUTPUTS(1).propositionBytes == issuerPropBytes &&
        OUTPUTS(1).value == MIN_BOX_VALUE &&
        OUTPUTS(1).tokens.size == 1 &&
        output1Token0._1 == selfToken0._1 &&
        output1Token0._2 == selfToken0._2 - 1L
    } else {
        false
    }

    val validCashExercise: Boolean = if (isInExerciseWindow && settlementType == 1L && optionTokensExercised > 0L) {
        val collateralTokenId = SELF.tokens(1)._1
        val validStablecoin = collateralTokenId == USE_TOKEN_ID || collateralTokenId == SIGUSD_TOKEN_ID
        val oracleBox = CONTEXT.dataInputs(0)
        val validOracle = oracleBox.tokens(0)._1 == COMPANION_NFT_ID
        val prices = oracleBox.R8[Coll[Long]].get
        val spotPrice = prices(oracleIndex.toInt)
        val profitPerContractOracle: Long = if (optionType == 0L) {
            if (spotPrice > strikePrice) {
                val raw = spotPrice - strikePrice
                if (raw > collateralCap) collateralCap else raw
            } else { 0L }
        } else {
            if (spotPrice < strikePrice) {
                val raw = strikePrice - spotPrice
                if (raw > collateralCap) collateralCap else raw
            } else { 0L }
        }
        val inTheMoney = profitPerContractOracle > 0L
        val payoutPerContract = profitPerContractOracle * stablecoinDecimal / ORACLE_DECIMAL
        val totalPayout = payoutPerContract * optionTokensExercised
        val validPayout = payoutPerContract > 0L
        val selfBalance = SELF.tokens(1)._2
        val succBalance = output0Token1._2
        val exerciserBox = OUTPUTS(1)
        val exerciserPaid = exerciserBox.tokens.exists { (t: (Coll[Byte], Long)) =>
            t._1 == collateralTokenId && t._2 >= totalPayout
        }
        val tokenPreserved = if (succBalance > 0L) {
            output0Token1._1 == collateralTokenId
        } else { true }
        validStablecoin && validOracle && inTheMoney && validPayout &&
        exerciserPaid && tokenPreserved && validExerciseSuccessor &&
        OUTPUTS(0).value >= SELF.value - txFee &&
        succBalance >= selfBalance - totalPayout
    } else {
        false
    }

    val validCloseOptionContract: Boolean = if (isExpired) {
        OUTPUTS.size == 2 &&
        OUTPUTS(0).propositionBytes == issuerPropBytes &&
        OUTPUTS(0).value >= SELF.value - txFee &&
        output0Token0._1 == selfToken1._1 &&
        output0Token0._2 == selfToken1._2
    } else {
        false
    }

    (
        (
            proveDlog(issuerECPoint) &&
            sigmaProp(!isMinted &&
                      OUTPUTS.size == 2 &&
                      OUTPUTS(0).propositionBytes == issuerPropBytes
            )
        )
        ||
        sigmaProp(
            validMint ||
            validDelivery ||
            validCashExercise ||
            validCloseOptionContract
        )
    )
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "OptionReserveV2 FULL failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_sigma_o_option_full() {
        // Option-SigmaO.es — 253-line original SigmaO option contract
        // Uses: preHeader.timestamp, R7[Box].get, getOrElse, decodePoint, proveDlog, propBytes
        // Replaced compile-time BoxMinValue with literal
        let source = r#"{
    val HourInMilli = 3600000L
    val BoxMinValue = 1000000L
    val valueIn: Long = SELF.value
    val selfToken0: (Coll[Byte], Long) = SELF.tokens.getOrElse(0, (Coll[Byte](),0L))
    val selfToken1: (Coll[Byte], Long) = SELF.tokens.getOrElse(1, (Coll[Byte](),0L))
    val output0Token0: (Coll[Byte], Long) = OUTPUTS(0).tokens.getOrElse(0, (Coll[Byte](),0L))
    val output0Token1: (Coll[Byte], Long) = OUTPUTS(0).tokens.getOrElse(1, (Coll[Byte](),0L))
    val output1Token0: (Coll[Byte], Long) = OUTPUTS(1).tokens.getOrElse(0, (Coll[Byte](),0L))

    val isMinted: Boolean = selfToken0._1 == SELF.R7[Box].get.id &&
                            SELF.propositionBytes == SELF.R7[Box].get.propositionBytes
    val optionCreationBox: Box = if (isMinted) {
        SELF.R7[Box].get
    } else {
        SELF
    }

    val optionName: Coll[Byte] = optionCreationBox.R4[Coll[Byte]].get
    val underlyingAssetTokenId: Coll[Byte] = optionCreationBox.R5[Coll[Byte]].get
    val optionDecimals: Coll[Byte] = optionCreationBox.R6[Coll[Byte]].get
    val isCall: Boolean = optionCreationBox.R8[Coll[Long]].get(0) == 0L
    val isEuropean: Boolean = optionCreationBox.R8[Coll[Long]].get(1) == 0L
    val shareSize: Long = optionCreationBox.R8[Coll[Long]].get(2)
    val maturityDate: Long = optionCreationBox.R8[Coll[Long]].get(3)
    val strikePrice: Long = optionCreationBox.R8[Coll[Long]].get(4)
    val dAppUIMintFee: Long = optionCreationBox.R8[Coll[Long]].get(5)
    val TxFee: Long = optionCreationBox.R8[Coll[Long]].get(6)
    val issuerECPoint: Coll[Byte] = optionCreationBox.R9[Coll[Coll[Byte]]].get(0)
    val issuerErgoTree: Coll[Byte] = proveDlog(decodePoint(issuerECPoint)).propBytes
    val dAppUIFeeErgoTree: Coll[Byte] = optionCreationBox.R9[Coll[Coll[Byte]]].get(1)
    val optionTokenIDIn: Coll[Byte] = optionCreationBox.id

    val MinOptionReserveValue: Long = TxFee + BoxMinValue
    val currentTimestamp: Long = CONTEXT.preHeader.timestamp
    val remainingDuration: Long = maturityDate - currentTimestamp
    val isOptionDelivered: Boolean = isMinted && selfToken0._2 == 1L
    val isExpired: Boolean = currentTimestamp > maturityDate
    val isExercible: Boolean = if (isEuropean) {
        isOptionDelivered && isExpired && currentTimestamp < maturityDate + 24L * HourInMilli
    } else {
        isOptionDelivered && !isExpired
    }
    val isEmpty: Boolean = if (isCall) {
        isOptionDelivered && selfToken1._2 == 0L
    } else {
        isOptionDelivered && valueIn == MinOptionReserveValue
    }

    val validBasicReplicatedOutput0: Boolean = if (OUTPUTS(0).propositionBytes == SELF.propositionBytes) {
        OUTPUTS(0).value >= MinOptionReserveValue &&
        OUTPUTS(0).R4[Coll[Byte]].get == optionName &&
        OUTPUTS(0).R5[Coll[Byte]].get == underlyingAssetTokenId &&
        OUTPUTS(0).R6[Coll[Byte]].get == optionDecimals &&
        OUTPUTS(0).R7[Box].get == optionCreationBox
    } else {
        false
    }

    val validMintOption: Boolean = if (!isMinted && INPUTS.size == 1 && OUTPUTS.size == 3) {
        validBasicReplicatedOutput0 &&
        OUTPUTS(0).value == valueIn - TxFee - dAppUIMintFee &&
        OUTPUTS(0).value >= 2L * MinOptionReserveValue &&
        output0Token0._1 == SELF.id &&
        (
            (
                isCall &&
                output0Token1._1 == underlyingAssetTokenId &&
                output0Token1 == selfToken0 &&
                output0Token0._2 == selfToken0._2 / shareSize + 1L &&
                OUTPUTS(0).tokens.size == 2
            ) ||
            (
                !isCall &&
                OUTPUTS(0).tokens.size == 1 &&
                output0Token0._2 == (valueIn - 3L * TxFee - dAppUIMintFee - 2L * BoxMinValue) / (strikePrice * shareSize) + 1L
            )
        ) &&
        OUTPUTS(1).propositionBytes == dAppUIFeeErgoTree &&
        OUTPUTS(1).tokens.size == 0 &&
        OUTPUTS(1).value >= dAppUIMintFee
    } else {
        false
    }

    val validDeliverOption: Boolean = if (isMinted && !isOptionDelivered && INPUTS.size == 1 && OUTPUTS.size == 3) {
        OUTPUTS(0).value == valueIn - TxFee - BoxMinValue &&
        validBasicReplicatedOutput0 &&
        output0Token0._1 == selfToken0._1 &&
        output0Token0._2 == 1L &&
        output0Token1 == selfToken1 &&
        OUTPUTS(1).propositionBytes == issuerErgoTree &&
        OUTPUTS(1).value == BoxMinValue &&
        OUTPUTS(1).tokens.size == 1 &&
        output1Token0._1 == selfToken0._1 &&
        output1Token0._2 == selfToken0._2 - 1L
    } else {
        false
    }

    val validCloseOptionContract: Boolean = if ((isExpired && !isExercible) || isEmpty) {
        OUTPUTS.size == 2 &&
        OUTPUTS(0).propositionBytes == issuerErgoTree &&
        OUTPUTS(0).value >= valueIn - TxFee &&
        output0Token0._1 == selfToken1._1 &&
        output0Token0._2 == selfToken1._2
    } else {
        false
    }

    val validExerciseOption: Boolean = if (isExercible && INPUTS.size == 2 && OUTPUTS.size == 4) {
        val output2Token0: (Coll[Byte], Long) = OUTPUTS(2).tokens.getOrElse(0, (Coll[Byte](),0L))
        val input1Token0: (Coll[Byte], Long) = INPUTS(1).tokens.getOrElse(0, (Coll[Byte](),0L))
        val exercisedAmountReserve: Long = if (isCall) {
            selfToken1._2 - output0Token1._2
        } else {
            valueIn - OUTPUTS(0).value
        }
        val numberOptionExpected: Long = if (isCall) {
            exercisedAmountReserve / shareSize
        } else {
            exercisedAmountReserve / (strikePrice * shareSize)
        }
        val numberOptionProvided = if (input1Token0._1 == optionTokenIDIn) {
            input1Token0._2
        } else {
            0L
        }
        numberOptionExpected == numberOptionProvided &&
        validBasicReplicatedOutput0 &&
        (
            (
                isCall &&
                selfToken0 == output0Token0 &&
                (
                    output0Token1._1 == underlyingAssetTokenId ||
                    output0Token1._2 == 0L
                ) &&
                output1Token0._1 == underlyingAssetTokenId &&
                output1Token0._2 == exercisedAmountReserve &&
                OUTPUTS(1).tokens.size == 1 &&
                OUTPUTS(2).value >= numberOptionExpected * strikePrice * shareSize &&
                OUTPUTS(2).tokens.size == 0
            )
            ||
            (
                !isCall &&
                OUTPUTS(1).value >= exercisedAmountReserve &&
                OUTPUTS(1).tokens.size == 0 &&
                output2Token0._1 == underlyingAssetTokenId &&
                output2Token0._2 >= numberOptionExpected * shareSize &&
                OUTPUTS(2).tokens.size == 1
            )
        ) &&
        OUTPUTS(2).propositionBytes == issuerErgoTree &&
        OUTPUTS(3).tokens.size == 0
    } else {
        false
    }

    (
        (
            proveDlog(decodePoint(issuerECPoint)) &&
            sigmaProp(!isMinted &&
                      OUTPUTS.size == 2 &&
                      OUTPUTS(0).propositionBytes == issuerErgoTree
                     )
        )
                                        ||
        sigmaProp(
            validMintOption ||
            validExerciseOption ||
            validDeliverOption ||
            validCloseOptionContract
            )
    )
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "SigmaO Option full failed: {:?}",
            result.err()
        );
    }

    fn replace_compile_constants(source: &str) -> String {
        source
            .replace(
                "ORACLE_POOL_NFT",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000010\")",
            )
            .replace(
                "COMPANION_NFT_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000011\")",
            )
            .replace(
                "REGISTRY_NFT_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000012\")",
            )
            .replace(
                "USE_TOKEN_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000013\")",
            )
            .replace(
                "SIGUSD_TOKEN_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000014\")",
            )
            .replace(
                "SELL_CONTRACT_USE_BYTES",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000015\")",
            )
            .replace(
                "SELL_CONTRACT_SIGUSD_BYTES",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000016\")",
            )
            .replace(
                "PAYMENT_TOKEN_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000017\")",
            )
            .replace(
                "POOL_NFT_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000018\")",
            )
            .replace("ORACLE_DECIMAL", "1000000L")
            .replace("MIN_BOX_VALUE", "1000000L")
            .replace("EXERCISE_WINDOW", "720L")
            .replace("ERG_ORACLE_INDEX", "17")
            .replace(
                "POOL_OPTION_RESERVE_HASH",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000019\")",
            )
            .replace(
                "LP_TOKEN_ID",
                "fromBase16(\"000000000000000000000000000000000000000000000000000000000000001a\")",
            )
            .replace("TWAP_BLEND_SPOT", "70L")
            .replace("TWAP_BLEND_TWAP", "30L")
            .replace("BLOCKS_PER_YEAR", "262800L")
            .replace("PROTOCOL_FEE_BPS", "200L")
            .replace("TOTAL_LP_MINTED", "1000000000L")
            .replace(
                "POOL_STATE_CONTRACT",
                "fromBase16(\"000000000000000000000000000000000000000000000000000000000000001b\")",
            )
    }

    fn test_contract_file(path: &str, name: &str) {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("SKIP {}: file not found at {}", name, path);
                return;
            }
        };
        let source = replace_compile_constants(&source);
        let result = compile_expr(&source, ScriptEnv::new());
        assert!(result.is_ok(), "{} FAILED: {:?}", name, result.err());
    }

    #[test]
    fn test_p2p_option_reserve_v1() {
        test_contract_file(
            "/home/cq/working-files/p2p-options-contracts/contracts/OptionReserve.es",
            "OptionReserve V1 (478 lines)",
        );
    }

    #[test]
    fn test_p2p_pool_option_reserve() {
        test_contract_file(
            "/home/cq/working-files/p2p-options-contracts/contracts/PoolOptionReserve.es",
            "PoolOptionReserve (490 lines)",
        );
    }

    #[test]
    fn test_p2p_option_reserve_v3() {
        test_contract_file(
            "/home/cq/working-files/p2p-options-contracts/contracts/OptionReserveV3.es",
            "OptionReserveV3 (502 lines)",
        );
    }

    #[test]
    fn test_p2p_option_reserve_v4() {
        test_contract_file(
            "/home/cq/working-files/p2p-options-contracts/contracts/OptionReserveV4.es",
            "OptionReserveV4 (515 lines)",
        );
    }

    #[test]
    fn test_p2p_option_reserve_v5() {
        test_contract_file(
            "/home/cq/working-files/p2p-options-contracts/contracts/OptionReserveV5.es",
            "OptionReserveV5 (570 lines)",
        );
    }

    #[test]
    fn test_p2p_option_reserve_v6() {
        test_contract_file(
            "/home/cq/working-files/p2p-options-contracts/contracts/OptionReserveV6.es",
            "OptionReserveV6 (575 lines)",
        );
    }

    #[test]
    fn test_p2p_option_reserve_v7() {
        test_contract_file(
            "/home/cq/working-files/p2p-options-contracts/contracts/OptionReserveV7.es",
            "OptionReserveV7 (582 lines)",
        );
    }

    #[test]
    fn test_p2p_option_reserve_v8_full() {
        // Full OptionReserveV8.es — 682 lines, 6 execution paths
        // The most complex production contract. V8 adds pre-expiry reclaim.
        let path = "/home/cq/working-files/p2p-options-contracts/contracts/OptionReserveV8.es";
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("SKIP OptionReserveV8: file not found at {}", path);
                return;
            }
        };
        // Replace compile-time constants
        let source = source
            .replace(
                "ORACLE_POOL_NFT",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000010\")",
            )
            .replace(
                "COMPANION_NFT_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000011\")",
            )
            .replace(
                "REGISTRY_NFT_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000012\")",
            )
            .replace(
                "USE_TOKEN_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000013\")",
            )
            .replace(
                "SIGUSD_TOKEN_ID",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000014\")",
            )
            .replace(
                "SELL_CONTRACT_USE_BYTES",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000015\")",
            )
            .replace(
                "SELL_CONTRACT_SIGUSD_BYTES",
                "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000016\")",
            )
            .replace("ORACLE_DECIMAL", "1000000L")
            .replace("MIN_BOX_VALUE", "1000000L")
            .replace("EXERCISE_WINDOW", "720L")
            .replace("ERG_ORACLE_INDEX", "17");
        let result = compile_expr(&source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "OptionReserveV8 FULL failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_sell_token_request_full() {
        // SellTokenRequest.es — full contract with getOrElse, propBytes, max(), nested if/else
        // BoxMinValue replaced with literal (compile-time constant not wired through ScriptEnv)
        let source = r#"{
    val BoxMinValue: Long = 1000000L
    val inputValue: Long = SELF.value
    val inputTokenReserve: (Coll[Byte], Long) = SELF.tokens.getOrElse(0, (Coll[Byte](),0L))
    val userPKIn: SigmaProp = SELF.R4[SigmaProp].get
    val sellParams: Coll[Long] = SELF.R5[Coll[Long]].get
    val txFee: Long = sellParams(2)
    val validCloseEmpty: Boolean = inputTokenReserve._2 == 0L &&
                                   OUTPUTS.size == 2 &&
                                   OUTPUTS(0).propositionBytes == userPKIn.propBytes &&
                                   OUTPUTS(1).value == txFee
    val validSellToken: Boolean = if (OUTPUTS(0).propositionBytes == SELF.propositionBytes) {
        val tokenPrice: Long = sellParams(0)
        val dAppUIFeePerThousand: Long = sellParams(1)
        val dAppUIFeeErgoTree: Coll[Byte] = SELF.R6[Coll[Byte]].get
        val outputReserveTokens: (Coll[Byte], Long) = OUTPUTS(0).tokens.getOrElse(0, (Coll[Byte](),0L))
        val deliveredTokens: Long = if (inputTokenReserve._2 > 0 && outputReserveTokens._2 > 0) {
            if (inputTokenReserve._1 == outputReserveTokens._1) {
                inputTokenReserve._2 - outputReserveTokens._2
            } else {
                0L
            }
        } else {
            if (inputTokenReserve._2 > 0 && outputReserveTokens._2 == 0) {
                inputTokenReserve._2
            } else {
                0L
            }
        }
        val minPrice: Long = max(BoxMinValue, deliveredTokens * tokenPrice)
        val dAppUIFee: Long = max(BoxMinValue, dAppUIFeePerThousand * minPrice / 1000L)
        deliveredTokens > 0L &&
        OUTPUTS.size == 5 &&
        (
            (
                inputTokenReserve._1 == outputReserveTokens._1 &&
                inputTokenReserve._2 > 0 &&
                outputReserveTokens._2 > 0
            )
            ||
            (
                inputTokenReserve._2 > 0 &&
                outputReserveTokens._2 == 0
            )
        ) &&
        OUTPUTS(0).value == inputValue &&
        OUTPUTS(0).R4[SigmaProp].get == userPKIn &&
        OUTPUTS(0).R5[Coll[Long]].get == sellParams &&
        OUTPUTS(0).R6[Coll[Byte]].get == dAppUIFeeErgoTree &&
        OUTPUTS(1).tokens(0)._1 == inputTokenReserve._1 &&
        OUTPUTS(1).tokens(0)._2 == deliveredTokens &&
        OUTPUTS(2).propositionBytes == userPKIn.propBytes &&
        OUTPUTS(2).value >= minPrice &&
        OUTPUTS(3).propositionBytes == dAppUIFeeErgoTree &&
        OUTPUTS(3).value >= dAppUIFee &&
        OUTPUTS(4).value == txFee
    } else {
        false
    }
    userPKIn ||
    sigmaProp(validSellToken || validCloseEmpty)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(
            result.is_ok(),
            "SellTokenRequest full failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_p2p_option_reserve_v2_nested_fold() {
        // Test the nested fold pattern from OptionReserveV2 (optionTokensExercised)
        // This is the most complex fold: outer fold over INPUTS, inner fold over box.tokens
        let source = r#"{
  val optionTokenId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val optionTokensExercised: Long = INPUTS.fold(0L, { (acc: Long, box: Box) =>
    if (box.id != SELF.id) {
      box.tokens.fold(acc, { (innerAcc: Long, t: (Coll[Byte], Long)) =>
        if (t._1 == optionTokenId) innerAcc + t._2
        else innerAcc
      })
    } else acc
  })
  sigmaProp(optionTokensExercised > 0L)
}"#;
        let result = compile_expr(source, ScriptEnv::new());
        assert!(result.is_ok(), "Nested fold failed: {:?}", result.err());
    }

    #[test]
    fn test_vault_byte_match() {
        let source = r#"{
  val StateNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val ReserveNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val selfValid = SELF.tokens.size == 1 && SELF.tokens(0)._1 == StateNftId && SELF.tokens(0)._2 == 1L
  val out = OUTPUTS(0)
  val outValid = out.propositionBytes == SELF.propositionBytes && out.tokens.size == 1 && out.tokens(0)._1 == StateNftId && out.tokens(0)._2 == 1L
  val reserveIn = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }
  val reserveOut = OUTPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }
  val pairingValid = reserveIn.size == 1 && reserveOut.size == 1
  val deltaVaultYolo = out.value - SELF.value
  val deltaReserveVYolo = reserveOut(0).tokens(1)._2 - reserveIn(0).tokens(1)._2
  val conservation = deltaVaultYolo + deltaReserveVYolo == 0L
  val nonTrivial = deltaVaultYolo != 0L
  val singleVaultInput = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == StateNftId }.size == 1
  sigmaProp(selfValid && outValid && pairingValid && conservation && nonTrivial && singleVaultInput)
}"#;
        // Vault with CSE: verify compilation + round-trip serialization
        use ergotree_ir::serialization::SigmaSerializable;
        let tree = compile(source, ScriptEnv::new()).unwrap();
        let bytes = tree.sigma_serialize_bytes().unwrap();
        let tree2 = ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(&bytes).unwrap();
        let bytes2 = tree2.sigma_serialize_bytes().unwrap();
        assert_eq!(bytes, bytes2, "Vault should round-trip serialize");
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        // Exact byte-match against the Ergo node (Scala compiler) output
        let node_hex = "101a0e20000000000000000000000000000000000000000000000000000000000000000104000e20000000000000000000000000000000000000000000000000000000000000000204000400040004000402040004000502040204000400050204020402040004020400040205000500040004000402d807d601db6308a7d6027300d603b2a5730100d6047302d605b5a4d9010563d801d607db63087205ed91b172077303938cb27207730400017204d606b5a5d9010663d801d608db63087206ed91b172087305938cb27208730600017204d60799c17203c1a7d1ededededededed93b172017307938cb27201730800017202938cb2720173090002730aededed93c27203c2a793b1db63087203730b938cb2db63087203730c00017202938cb2db63087203730d0002730eed93b17205730f93b172067310939a7207998cb2db6308b27206731100731200028cb2db6308b27205731300731400027315947207731693b1b5a4d9010863d801d60adb63087208ed91b1720a7317938cb2720a7318000172027319";
        assert_eq!(hex, node_hex, "Vault must byte-match the Ergo node");
    }

    /// Batch byte-match test: 13 real-world contracts compiled by the Ergo node.
    /// Covers governance, DeFi (LP, options), nested forall, fold, map, if-else,
    /// registers, blake2b256, and multi-filter patterns.
    #[test]
    fn test_batch_node_byte_match() {
        use ergotree_ir::serialization::SigmaSerializable;

        // (name, source, node_hex)
        let contracts: Vec<(&str, &str, &str)> = vec![
            // -- Governance contracts --
            ("governance reserve (467 bytes)",
             r#"{
  val ReserveNftId: Coll[Byte] = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val StateNftId: Coll[Byte]   = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val VYoloId: Coll[Byte]      = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val selfValid: Boolean = SELF.tokens.size == 2 && SELF.tokens(0)._1 == ReserveNftId && SELF.tokens(0)._2 == 1L && SELF.tokens(1)._1 == VYoloId
  val out: Box = OUTPUTS(1)
  val outValid: Boolean = out.propositionBytes == SELF.propositionBytes && out.tokens.size == 2 && out.tokens(0)._1 == ReserveNftId && out.tokens(0)._2 == 1L && out.tokens(1)._1 == VYoloId && out.value == SELF.value
  val vaultIn: Coll[Box] = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == StateNftId }
  val vaultOut: Coll[Box] = OUTPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == StateNftId }
  val pairingValid: Boolean = vaultIn.size == 1 && vaultOut.size == 1
  val deltaReserveVYolo: Long = out.tokens(1)._2 - SELF.tokens(1)._2
  val deltaVaultYolo: Long    = vaultOut(0).value - vaultIn(0).value
  val conservation: Boolean   = deltaVaultYolo + deltaReserveVYolo == 0L
  val nonTrivial: Boolean = deltaReserveVYolo != 0L
  val singleReserveInput: Boolean = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }.size == 1
  sigmaProp(selfValid && outValid && pairingValid && conservation && nonTrivial && singleReserveInput)
}"#,
             "101d0e2000000000000000000000000000000000000000000000000000000000000000020e20000000000000000000000000000000000000000000000000000000000000000304020e2000000000000000000000000000000000000000000000000000000000000000010400040004000400040204020404040004000502040204040400040005020402040204020400040005000500040004000402d808d601db6308a7d6027300d6037301d604b2a5730200d6057303d606b5a4d9010663d801d608db63087206ed91b172087304938cb27208730500017205d607b5a5d9010763d801d609db63087207ed91b172097306938cb27209730700017205d608998cb2db63087204730800028cb2720173090002d1edededededededed93b17201730a938cb27201730b00017202938cb27201730c0002730d938cb27201730e00017203ededededed93c27204c2a793b1db63087204730f938cb2db63087204731000017202938cb2db63087204731100027312938cb2db6308720473130001720393c17204c1a7ed93b17206731493b172077315939a99c1b27207731600c1b2720673170072087318947208731993b1b5a4d9010963d801d60bdb63087209ed91b1720b731a938cb2720b731b00017202731c"),
            // -- Dexy-style LP swap --
            ("dexy-style LP (146 bytes)",
             r#"{
  val lpNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val validLP = SELF.tokens(0)._1 == lpNFT && SELF.tokens(0)._2 == 1L
  val out = OUTPUTS(0)
  val outValid = out.propositionBytes == SELF.propositionBytes && out.tokens(0)._1 == lpNFT
  val deltaX = out.value - SELF.value
  val deltaY = out.tokens(1)._2 - SELF.tokens(1)._2
  val validSwap = deltaX * deltaY < 0L
  sigmaProp(validLP && outValid && validSwap)
}"#,
             "100804000e200000000000000000000000000000000000000000000000000000000000000001040005020400040204020500d804d601db6308a7d602b27201730000d6037301d604b2a5730200d1ededed938c7202017203938c7202027303ed93c27204c2a7938cb2db630872047304000172038f9c99c17204c1a7998cb2db63087204730500028cb27201730600027307"),
            // -- Nested forall (token burn verification) --
            ("nested forall (61 bytes)",
             r#"{
  val stateToken = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val tokenBurned = OUTPUTS.forall { (o: Box) =>
    o.tokens.forall { (t: (Coll[Byte], Long)) => t._1 != stateToken }
  }
  sigmaProp(tokenBurned)
}"#,
             "10010e200000000000000000000000000000000000000000000000000000000000000001d1afa5d9010163afdb63087201d901034d0e948c7203017300"),
            // -- Multi-filter + arithmetic --
            ("multi filter + arith (219 bytes)",
             r#"{
  val nftA = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val nftB = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val boxesA = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == nftA }
  val boxesB = OUTPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == nftB }
  val deltaA = boxesA(0).value - SELF.value
  val deltaB = boxesB(0).tokens(1)._2 - SELF.tokens(1)._2
  sigmaProp(boxesA.size == 1 && boxesB.size == 1 && deltaA + deltaB == 0L)
}"#,
             "100d040004000e200000000000000000000000000000000000000000000000000000000000000001040004000e2000000000000000000000000000000000000000000000000000000000000000020402040204000400040204020500d802d601b5a4d9010163d801d603db63087201ed91b172037300938cb27203730100017302d602b5a5d9010263d801d604db63087202ed91b172047303938cb27204730400017305d1eded93b17201730693b172027307939a99c1b27201730800c1a7998cb2db6308b27202730900730a00028cb2db6308a7730b0002730c"),
            // -- Register tuples + blake2b256 --
            ("register tuple + blake (96 bytes)",
             r#"{
  val proposalTuple = SELF.R4[(Long, Long)].get
  val proportion = proposalTuple._1
  val recipient = SELF.R5[Coll[Byte]].get
  val denom = 10000000L
  val wholePart = (SELF.value / denom) * proportion
  val remainderPart = ((SELF.value % denom) * proportion) / denom
  val awarded = wholePart + remainderPart
  val recipientValid = blake2b256(OUTPUTS(1).propositionBytes) == recipient && OUTPUTS(1).value >= awarded
  sigmaProp(proportion > 0L && proportion < denom && recipientValid)
}"#,
             "1006040205000580dac4090580dac4090580dac4090580dac409d803d6018ce4c6a7045901d602b2a5730000d603c1a7d1eded91720173018f72017302ed93cbc27202e4c6a7050e92c172029a9c9d7203730372019d9c9e7203730472017305"),
            // -- Exists + forall + size checks --
            ("exists + size checks (49 bytes)",
             r#"{
  val minBoxValue = 1000000L
  val allOutputsValid = OUTPUTS.forall { (o: Box) => o.value >= minBoxValue }
  val hasTokenInput = INPUTS.exists { (b: Box) => b.tokens.size > 0 }
  val selfPreserved = OUTPUTS(0).propositionBytes == SELF.propositionBytes
  sigmaProp(allOutputsValid && hasTokenInput && selfPreserved)
}"#,
             "10030580897a04000400d1ededafa5d901016392c172017300aea4d901016391b1db63087201730193c2b2a5730200c2a7"),
            // -- If-else paths --
            ("if-else paths (118 bytes)",
             r#"{
  val out = OUTPUTS(0)
  val scriptPreserved = out.propositionBytes == SELF.propositionBytes
  val isDeposit = INPUTS(0).id == SELF.id && out.value >= SELF.value && scriptPreserved
  val isWithdraw = if (INPUTS(0).tokens.size >= 1) {
    val authToken = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
    INPUTS(0).tokens(0)._1 == authToken && scriptPreserved
  } else { false }
  sigmaProp(isDeposit || isWithdraw)
}"#,
             "100604000400040204000e2000000000000000000000000000000000000000000000000000000000000000010100d804d601b2a4730000d602b2a5730100d60393c27202c2a7d604db63087201d1eceded93c57201c5a792c17202c1a772039592b172047302ed938cb2720473030001730472037305"),
            // -- Fold sum --
            ("fold sum (46 bytes)",
             r#"{
  val totalInput = INPUTS.fold(0L, { (acc: Long, b: Box) => acc + b.value })
  val totalOutput = OUTPUTS.fold(0L, { (acc: Long, b: Box) => acc + b.value })
  sigmaProp(totalInput >= totalOutput)
}"#,
             "100205000500d192b0a47300d9010141639a8c720101c18c720102b0a57301d9010141639a8c720101c18c720102"),
            // -- Map + size --
            ("map + size (31 bytes)",
             r#"{
  val inputValues = INPUTS.map { (b: Box) => b.value }
  sigmaProp(inputValues.size > 0 && inputValues(0) > 0L)
}"#,
             "1003040004000500d1ed91b1a4730091b2ada4d9010163c172017301007302"),
            // -- Triple CSE --
            // -- Real-world production contracts (from EKB) --
            ("Oracle Pool v2 - Pool (104B)",
             r#"{
  val otherTokenId = INPUTS(1).tokens(0)._1
  val refreshNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val updateNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  sigmaProp(otherTokenId == refreshNFT || otherTokenId == updateNFT)
}"#,
             "1004040204000e2000000000000000000000000000000000000000000000000000000000000000010e200000000000000000000000000000000000000000000000000000000000000002d801d6018cb2db6308b2a473000073010001d1ec93720173029372017303"),
            ("Dexy Bank (291B)",
             r#"{
  val bankNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val freeMintNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val arbMintNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000003")
  val interventionNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000004")
  val payoutNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000005")
  val updateNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000006")
  val successor = OUTPUTS(1)
  val validSuccessor = successor.tokens(0)._1 == bankNFT &&
                       successor.propositionBytes == SELF.propositionBytes
  val validMint = INPUTS(0).tokens(0)._1 == freeMintNFT || INPUTS(0).tokens(0)._1 == arbMintNFT
  val validIntervention = INPUTS(0).tokens(0)._1 == interventionNFT
  val validPayout = INPUTS(0).tokens(0)._1 == payoutNFT
  val validUpdate = INPUTS(0).tokens(0)._1 == updateNFT
  sigmaProp((validSuccessor && (validMint || validIntervention || validPayout)) || validUpdate)
}"#,
             "100a04020400040004000e2000000000000000000000000000000000000000000000000000000000000000010e2000000000000000000000000000000000000000000000000000000000000000020e2000000000000000000000000000000000000000000000000000000000000000030e2000000000000000000000000000000000000000000000000000000000000000040e2000000000000000000000000000000000000000000000000000000000000000050e200000000000000000000000000000000000000000000000000000000000000006d802d601b2a5730000d6028cb2db6308b2a473010073020001d1eceded938cb2db6308720173030001730493c27201c2a7ececec93720273059372027306937202730793720273089372027309"),
            ("DuckPools Lending Pool (275B)",
             r#"{
  val poolNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val selfValid = SELF.tokens(0)._1 == poolNFT && SELF.tokens(0)._2 == 1L
  val out = OUTPUTS(0)
  val outValid = out.tokens(0)._1 == poolNFT &&
                 out.propositionBytes == SELF.propositionBytes &&
                 out.tokens(0)._2 == 1L
  val lendTokensIn = SELF.tokens(1)._2
  val lendTokensOut = out.tokens(1)._2
  val borrowTokensIn = SELF.tokens(2)._2
  val borrowTokensOut = out.tokens(2)._2
  val ergsDelta = out.value - SELF.value
  val lendDelta = lendTokensOut - lendTokensIn
  val borrowDelta = borrowTokensOut - borrowTokensIn
  val isDeposit = ergsDelta > 0L && lendDelta < 0L && borrowDelta == 0L
  val isWithdraw = ergsDelta < 0L && lendDelta > 0L && borrowDelta == 0L
  val isBorrow = ergsDelta < 0L && borrowDelta < 0L && lendDelta == 0L
  val isRepay = ergsDelta > 0L && borrowDelta > 0L && lendDelta == 0L
  sigmaProp(selfValid && outValid && (isDeposit || isWithdraw || isBorrow || isRepay))
}"#,
             "101404000e200000000000000000000000000000000000000000000000000000000000000001040004000500040204020404040405000502050205000500050005000500050005000500d80bd601db6308a7d602b27201730000d6037301d604b2a5730200d605db63087204d606b27205730300d60799c17204c1a7d6088f72077304d609998cb27205730500028cb2720173060002d60a998cb27205730700028cb2720173080002d60b9172077309d1ededed938c7202017203938c720202730aeded938c720601720393c27204c2a7938c720602730becececeded720b8f7209730c93720a730deded7208917209730e93720a730feded72088f720a73109372097311eded720b91720a73129372097313"),
            ("Spectrum AMM Swap (177B)",
             r#"{
  val feeNum = 997L
  val feeDenom = 1000L
  val poolIn = INPUTS(0)
  val poolOut = OUTPUTS(0)
  val selfOut = OUTPUTS(1)
  val reservesXIn = poolIn.value
  val reservesYIn = poolIn.tokens(2)._2
  val reservesXOut = poolOut.value
  val reservesYOut = poolOut.tokens(2)._2
  val deltaReservesX = reservesXOut - reservesXIn
  val deltaReservesY = reservesYOut - reservesYIn
  val validSwap = if (deltaReservesX > 0L) {
    reservesYIn * deltaReservesX * feeNum >= -deltaReservesY * (reservesXIn * feeDenom + deltaReservesX * feeNum)
  } else {
    reservesXIn * deltaReservesY * feeNum >= -deltaReservesX * (reservesYIn * feeDenom + deltaReservesY * feeNum)
  }
  val selfPreserved = selfOut.propositionBytes == SELF.propositionBytes &&
                      selfOut.value >= SELF.value &&
                      selfOut.tokens == SELF.tokens
  sigmaProp(validSwap && selfPreserved)
}"#,
             "100c04000400040404040402050005ca0f05d00f05ca0f05ca0f05d00f05ca0fd807d601b2a5730000d602b2a4730100d603c17202d60499c172017203d6058cb2db6308720273020002d606998cb2db63087201730300027205d607b2a5730400d1ed959172047305929c9c7205720473069cf072069a9c720373079c72047308929c9c7203720673099cf072049a9c7205730a9c7206730beded93c27207c2a792c17207c1a793db63087207db6308a7"),
            ("Oracle Pool v2 - Oracle (209B)",
             r#"{
  val poolNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val otherTokenId = INPUTS(0).tokens(0)._1
  val minStorageRent = 10000000L
  val selfPubKey = SELF.R4[GroupElement].get
  val outIndex = getVar[Int](0).get
  val output = OUTPUTS(outIndex)
  val isSimpleCopy = output.tokens(0) == SELF.tokens(0) &&
                     output.propositionBytes == SELF.propositionBytes &&
                     output.R4[GroupElement].isDefined &&
                     output.value >= minStorageRent
  val collection = otherTokenId == poolNFT &&
                   output.tokens(1)._1 == SELF.tokens(1)._1 &&
                   output.tokens(1)._2 > SELF.tokens(1)._2 &&
                   output.R4[GroupElement].get == selfPubKey &&
                   output.value >= SELF.value &&
                   ! (output.R5[Any].isDefined)
  val owner = proveDlog(selfPubKey)
  isSimpleCopy && (owner || collection)
}"#,
             "100a040004000580dac409040004000e2000000000000000000000000000000000000000000000000000000000000000010402040204020402d804d601b2a5e4e3000400d602db63087201d603db6308a7d604e4c6a70407ea02d1ededed93b27202730000b2720373010093c27201c2a7e6c67201040792c172017302eb02cd7204d1ededededed938cb2db6308b2a4730300730400017305938cb27202730600018cb2720373070001918cb27202730800028cb272037309000293e4c672010407720492c17201c1a7efe6c672010561"),
            // -- Synthetic edge cases (known structural diffs from Scala) --
            ("triple CSE (117 bytes)",
             r#"{
  val nft = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val selfValid = SELF.tokens.size > 0 && SELF.tokens(0)._1 == nft
  val out = OUTPUTS(0)
  val outValid = out.tokens.size > 0 && out.tokens(0)._1 == nft
  val preserved = out.propositionBytes == SELF.propositionBytes
  sigmaProp(selfValid && outValid && preserved)
}"#,
             "10060e20000000000000000000000000000000000000000000000000000000000000000104000400040004000400d804d601db6308a7d6027300d603b2a5730100d604db63087203d1ededed91b172017302938cb27201730300017202ed91b172047304938cb2720473050001720293c27203c2a7"),
        ];

        let mut matched = 0;
        let mut failed = Vec::new();
        for (name, source, node_hex) in &contracts {
            let tree = match compile(source, ScriptEnv::new()) {
                Ok(t) => t,
                Err(e) => {
                    failed.push(format!("{}: COMPILE ERROR: {:?}", name, e));
                    continue;
                }
            };
            let bytes = tree.sigma_serialize_bytes().unwrap();
            let our_hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

            // Verify round-trip
            let tree2 = ergotree_ir::ergo_tree::ErgoTree::sigma_parse_bytes(&bytes).unwrap();
            assert_eq!(
                bytes,
                tree2.sigma_serialize_bytes().unwrap(),
                "Round-trip failed for: {}",
                name
            );

            if our_hex == *node_hex {
                matched += 1;
            } else {
                let diff_count = our_hex
                    .as_bytes()
                    .chunks(2)
                    .zip(node_hex.as_bytes().chunks(2))
                    .filter(|(a, b)| a != b)
                    .count();
                failed.push(format!(
                    "{}: {} differing bytes (ours {} vs node {} bytes)",
                    name,
                    diff_count,
                    our_hex.len() / 2,
                    node_hex.len() / 2
                ));
            }
        }

        // 15/15 contracts byte-match the Scala node output.
        assert!(
            matched >= 15,
            "Batch byte-match: {}/{} matched (expected 15).\nFailures:\n  {}",
            matched,
            matched + failed.len(),
            failed.join("\n  ")
        );
    }

    #[test]
    fn test_oracle_contract_compiles() {
        // Oracle Pool v2 - Oracle contract (EIP-0023)
        // Tests: getVar, proveDlog, R5[Any], variable OUTPUTS index, tuple equality, SigmaProp mixing
        let _tree = compile(
            r#"{
  val poolNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val otherTokenId = INPUTS(0).tokens(0)._1
  val minStorageRent = 10000000L
  val selfPubKey = SELF.R4[GroupElement].get
  val outIndex = getVar[Int](0).get
  val output = OUTPUTS(outIndex)
  val isSimpleCopy = output.tokens(0) == SELF.tokens(0) &&
                     output.propositionBytes == SELF.propositionBytes &&
                     output.R4[GroupElement].isDefined &&
                     output.value >= minStorageRent
  val collection = otherTokenId == poolNFT &&
                   output.tokens(1)._1 == SELF.tokens(1)._1 &&
                   output.tokens(1)._2 > SELF.tokens(1)._2 &&
                   output.R4[GroupElement].get == selfPubKey &&
                   output.value >= SELF.value &&
                   ! (output.R5[Any].isDefined)
  val owner = proveDlog(selfPubKey)
  isSimpleCopy && (owner || collection)
}"#,
            ScriptEnv::new(),
        )
        .expect("Oracle Pool v2 Oracle contract should compile");
    }

    /// Test compile_canonical against the Ergo node.
    /// Requires a running node at localhost:9053 with API key from ~/.secrets.
    /// Run with: cargo test -p ergoscript-compiler test_canonical -- --ignored --nocapture
    #[test]
    #[ignore] // requires running Ergo node
    fn test_canonical_compilation() {
        use ergotree_ir::serialization::SigmaSerializable;

        // Read API key from environment
        let api_key = std::env::var("API_KEY").unwrap_or_default();
        let node_url = "http://localhost:9053";

        let contracts: Vec<(&str, &str)> = vec![
            ("simple", "{ sigmaProp(SELF.value > 0L) }"),
            (
                "vault (with lambdas)",
                r#"{
  val StateNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val ReserveNftId = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val selfValid = SELF.tokens.size == 1 && SELF.tokens(0)._1 == StateNftId && SELF.tokens(0)._2 == 1L
  val out = OUTPUTS(0)
  val outValid = out.propositionBytes == SELF.propositionBytes && out.tokens.size == 1 && out.tokens(0)._1 == StateNftId && out.tokens(0)._2 == 1L
  val reserveIn = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }
  val reserveOut = OUTPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == ReserveNftId }
  val pairingValid = reserveIn.size == 1 && reserveOut.size == 1
  val deltaVaultYolo = out.value - SELF.value
  val deltaReserveVYolo = reserveOut(0).tokens(1)._2 - reserveIn(0).tokens(1)._2
  val conservation = deltaVaultYolo + deltaReserveVYolo == 0L
  val nonTrivial = deltaVaultYolo != 0L
  val singleVaultInput = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == StateNftId }.size == 1
  sigmaProp(selfValid && outValid && pairingValid && conservation && nonTrivial && singleVaultInput)
}"#,
            ),
            (
                "dexy LP (canonical needed)",
                r#"{
  val lpNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val validLP = SELF.tokens(0)._1 == lpNFT && SELF.tokens(0)._2 == 1L
  val out = OUTPUTS(0)
  val outValid = out.propositionBytes == SELF.propositionBytes && out.tokens(0)._1 == lpNFT
  val deltaX = out.value - SELF.value
  val deltaY = out.tokens(1)._2 - SELF.tokens(1)._2
  val validSwap = deltaX * deltaY < 0L
  sigmaProp(validLP && outValid && validSwap)
}"#,
            ),
        ];

        for (name, source) in &contracts {
            let result = compile_canonical(source, ScriptEnv::new(), node_url, &api_key)
                .unwrap_or_else(|e| panic!("{}: compile failed: {:?}", name, e));

            let bytes = result.tree.sigma_serialize_bytes().unwrap();
            let _hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

            match result.matched {
                Some(true) => eprintln!("  {} ({} bytes): LOCAL MATCH", name, bytes.len()),
                Some(false) => eprintln!(
                    "  {} ({} bytes): USED NODE (local differed)",
                    name,
                    bytes.len()
                ),
                None => eprintln!(
                    "  {} ({} bytes): NODE UNAVAILABLE (local only)",
                    name,
                    bytes.len()
                ),
            }
        }
    }

    /// Test compilation of real-world contract patterns from SigmaUSD, Rosen Bridge,
    /// Spectrum DEX, etc. Verifies against the Ergo node via compile_canonical.
    /// Run with: cargo test -p ergoscript-compiler test_real_world -- --ignored --nocapture
    #[test]
    #[ignore] // requires running Ergo node
    fn test_real_world_contracts() {
        use ergotree_ir::serialization::SigmaSerializable;

        let api_key = std::env::var("API_KEY").unwrap_or_default();
        let node_url = "http://localhost:9053";

        let contracts: Vec<(&str, &str)> = vec![
            (
                "SigmaUSD bank (reserve ratio)",
                r#"{
  val bankNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val oracleNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000002")
  val selfValid = SELF.tokens(0)._1 == bankNFT
  val successor = OUTPUTS(0)
  val successorValid = successor.tokens(0)._1 == bankNFT && successor.propositionBytes == SELF.propositionBytes
  val oracleBox = CONTEXT.dataInputs(0)
  val oracleValid = oracleBox.tokens(0)._1 == oracleNFT
  val rate = oracleBox.R4[Long].get
  val reserveIn = SELF.value
  val reserveOut = successor.value
  val scIn = SELF.tokens(1)._2
  val scOut = successor.tokens(1)._2
  val deltaErg = reserveOut - reserveIn
  val deltaSc = scOut - scIn
  val reserveRatio = reserveOut * 100L / (scOut * rate / 1000000L)
  sigmaProp(selfValid && successorValid && oracleValid && reserveRatio >= 400L)
}"#,
            ),
            (
                "Rosen GuardSign (atLeast+proveDlog)",
                r#"{
  val guardNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val guardBoxes = INPUTS.filter { (b: Box) => b.tokens.size > 0 && b.tokens(0)._1 == guardNFT }
  val singleGuard = guardBoxes.size == 1
  val tokenPreserved = OUTPUTS(0).tokens(0)._1 == SELF.tokens(0)._1
  val qtyPreserved = OUTPUTS(0).tokens(0)._2 == SELF.tokens(0)._2
  val pks = SELF.R4[Coll[GroupElement]].get
  val threshold = SELF.R5[Coll[Int]].get(1)
  val sigmas = pks.map { (pk: GroupElement) => proveDlog(pk) }
  sigmaProp(singleGuard && tokenPreserved && qtyPreserved) && atLeast(threshold, sigmas)
}"#,
            ),
            (
                "DEX swap order",
                r#"{
  val quoteId = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val minOutput = SELF.R4[Long].get
  val rewardPk = SELF.R5[GroupElement].get
  val deadline = SELF.R6[Int].get
  val validSwap = OUTPUTS(0).tokens.size > 0 && OUTPUTS(0).tokens(0)._1 == quoteId && OUTPUTS(0).tokens(0)._2 >= minOutput && OUTPUTS(0).propositionBytes == proveDlog(rewardPk).propBytes
  val cancel = HEIGHT > deadline && proveDlog(rewardPk)
  sigmaProp(validSwap) || cancel
}"#,
            ),
            (
                "multi-sig treasury",
                r#"{
  val pks = SELF.R4[Coll[GroupElement]].get
  val threshold = SELF.R5[Int].get
  val sigmas = pks.map { (pk: GroupElement) => proveDlog(pk) }
  val preserved = OUTPUTS(0).propositionBytes == SELF.propositionBytes && OUTPUTS(0).value >= SELF.value - 1000000L
  sigmaProp(preserved) && atLeast(threshold, sigmas)
}"#,
            ),
            (
                "token emission",
                r#"{
  val emissionNFT = fromBase16("0000000000000000000000000000000000000000000000000000000000000001")
  val selfValid = SELF.tokens(0)._1 == emissionNFT && SELF.tokens(0)._2 == 1L
  val successor = OUTPUTS(0)
  val successorValid = successor.tokens(0)._1 == emissionNFT && successor.tokens(0)._2 == 1L && successor.propositionBytes == SELF.propositionBytes
  val tokensEmitted = SELF.tokens(1)._2 - successor.tokens(1)._2
  val validEmission = tokensEmitted > 0L && tokensEmitted <= 1000L
  sigmaProp(selfValid && successorValid && validEmission)
}"#,
            ),
            (
                "time-locked vesting",
                r#"{
  val beneficiary = SELF.R4[GroupElement].get
  val unlockHeight = SELF.R5[Int].get
  val vestingComplete = HEIGHT >= unlockHeight
  val beneficiarySpend = proveDlog(beneficiary)
  sigmaProp(vestingComplete) && beneficiarySpend
}"#,
            ),
        ];

        let mut matched = 0;
        let mut node_fallback = 0;
        let mut errors = 0;
        for (name, source) in &contracts {
            match compile_canonical(source, ScriptEnv::new(), node_url, &api_key) {
                Ok(result) => {
                    let bytes = result.tree.sigma_serialize_bytes().unwrap();
                    match result.matched {
                        Some(true) => {
                            eprintln!("  {} ({} bytes): LOCAL MATCH", name, bytes.len());
                            matched += 1;
                        }
                        Some(false) => {
                            eprintln!("  {} ({} bytes): USED NODE", name, bytes.len());
                            node_fallback += 1;
                        }
                        None => {
                            eprintln!("  {} ({} bytes): NODE UNAVAILABLE", name, bytes.len());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  {}: COMPILE ERROR: {:?}", name, e);
                    errors += 1;
                }
            }
        }
        eprintln!(
            "\nResults: {} local match, {} node fallback, {} errors out of {}",
            matched,
            node_fallback,
            errors,
            contracts.len()
        );
        assert_eq!(errors, 0, "Some contracts failed to compile");
    }
}
