//! WS-F.1 — Differential fuzzer harness skeleton.
//!
//! Four-way classification (`MATCH` / `DIFF` / `RUST_FAIL` / `SCALA_FAIL` /
//! `BOTH_FAIL`) comparing the local Rust pipeline (`compile`) against the
//! Scala oracle (`compile_via_node` → `/script/p2sAddress` +
//! `/script/addressToTree`).
//!
//! Run with:
//!   source ~/.secrets &&
//!   cargo test -p ergoscript-compiler --test diff_fuzz -- --ignored --nocapture
//!
//! Inputs (priority order):
//!   1. `target/diff_fuzz/corpus/*.es` — populated by F.2 (empty in F.1).
//!   2. Fallback: 14 ecosystem (`ecosystem_corpus`) + 15 sig-15 (.es files
//!      under `tests/fixtures/significant_15/` with prelude injection).
//!
//! Outputs (everything under `target/`, gitignored):
//!   - `target/diff_fuzz/scala_cache/<info_hash>_<src_hash>.{bin,err}`
//!   - `target/diff_fuzz/diff/<name>.txt`
//!   - `target/diff_fuzz/fail/<name>_<label>.txt`
//!   - `target/diff_fuzz/summary.txt`
//!
//! Diagnostic, not gate: never panics on `DIFF` / `*_FAIL`. Only panics if
//! the Ergo node is unreachable at startup (`/info` fetch fails) — running
//! uncached against a possibly-broken oracle would poison the cache.

use ergoscript_compiler::compiler::{compile, compile_via_node, ecosystem_corpus};
use ergoscript_compiler::script_env::ScriptEnv;
use ergotree_ir::serialization::SigmaSerializable;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const NODE_URL: &str = "http://localhost:9053";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Match,
    Diff,
    RustFail,
    ScalaFail,
    BothFail,
}

impl Outcome {
    fn label(self) -> &'static str {
        match self {
            Outcome::Match => "MATCH",
            Outcome::Diff => "DIFF",
            Outcome::RustFail => "RUST_FAIL",
            Outcome::ScalaFail => "SCALA_FAIL",
            Outcome::BothFail => "BOTH_FAIL",
        }
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Extract a stable cache-key from `/info`. Only fields that affect compiler
/// output are included; volatile fields (timestamps, heights, peer counts)
/// are deliberately excluded so the cache survives across runs.
///
/// Fields hashed:
///   - `appVersion` (node software version → ErgoScript compiler revision)
///   - `network` (mainnet/testnet differs in some predef constants)
///   - `parameters.blockVersion` (governs script semantics)
///   - `eip27Supported`, `eip37Supported` (EIP feature gates)
fn fetch_node_info_hash(api_key: &str) -> Result<String, String> {
    use std::process::Command;
    let url = format!("{}/info", NODE_URL);
    let output = Command::new("curl")
        .args([
            "-s",
            &url,
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
            "/info curl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout.is_empty() {
        return Err("/info returned empty body".into());
    }
    let body = String::from_utf8_lossy(&output.stdout);
    let stable = stable_info_fingerprint(&body);
    Ok(sha256_hex(stable.as_bytes())[..16].to_string())
}

/// Build a deterministic string from `/info` containing only fields that
/// influence compiler output. Format: `key=value\n` lines, sorted by key.
fn stable_info_fingerprint(body: &str) -> String {
    let keys = [
        "appVersion",
        "network",
        "blockVersion",
        "eip27Supported",
        "eip37Supported",
    ];
    let mut out = String::new();
    for k in keys {
        let v = extract_json_value(body, k).unwrap_or_else(|| "<missing>".into());
        out.push_str(&format!("{}={}\n", k, v));
    }
    out
}

/// Minimal JSON value extractor: finds the first `"key"` substring and
/// returns the next scalar (string in quotes, or unquoted bool/number) up
/// to the next `,` / `}` / `\n`. Tolerant to nested `parameters.blockVersion`
/// since `blockVersion` is unique inside `/info` even nested.
fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let pat = format!("\"{}\"", key);
    let idx = json.find(&pat)?;
    let after = &json[idx + pat.len()..];
    let after_colon = after.trim_start().strip_prefix(':')?.trim_start();
    if let Some(rest) = after_colon.strip_prefix('"') {
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    let end = after_colon
        .find([',', '\n', '}', ' '])
        .unwrap_or(after_colon.len());
    Some(after_colon[..end].trim().to_string())
}

fn ensure_dir(p: &Path) {
    fs::create_dir_all(p).unwrap_or_else(|e| panic!("mkdir {}: {}", p.display(), e));
}

fn cache_stem(info_hash: &str, source: &str) -> String {
    let src_hash = sha256_hex(source.as_bytes());
    format!("{}_{}", info_hash, &src_hash[..32])
}

fn cache_lookup(
    cache_dir: &Path,
    info_hash: &str,
    source: &str,
) -> Option<Result<Vec<u8>, String>> {
    let stem = cache_stem(info_hash, source);
    let bin = cache_dir.join(format!("{}.bin", stem));
    if bin.exists() {
        return fs::read(&bin).ok().map(Ok);
    }
    let err = cache_dir.join(format!("{}.err", stem));
    if err.exists() {
        return fs::read_to_string(&err).ok().map(Err);
    }
    None
}

fn cache_store(
    cache_dir: &Path,
    info_hash: &str,
    source: &str,
    result: &Result<Vec<u8>, String>,
) {
    let stem = cache_stem(info_hash, source);
    let (path, tmp, payload): (PathBuf, PathBuf, Vec<u8>) = match result {
        Ok(bytes) => (
            cache_dir.join(format!("{}.bin", stem)),
            cache_dir.join(format!("{}.bin.tmp", stem)),
            bytes.clone(),
        ),
        Err(msg) => (
            cache_dir.join(format!("{}.err", stem)),
            cache_dir.join(format!("{}.err.tmp", stem)),
            msg.as_bytes().to_vec(),
        ),
    };
    if fs::write(&tmp, &payload).is_ok() {
        let _ = fs::rename(&tmp, &path);
    }
}

fn scala_compile_cached(
    source: &str,
    cache_dir: &Path,
    info_hash: &str,
    api_key: &str,
) -> Result<Vec<u8>, String> {
    if let Some(hit) = cache_lookup(cache_dir, info_hash, source) {
        return hit;
    }
    let res = compile_via_node(source, NODE_URL, api_key);
    cache_store(cache_dir, info_hash, source, &res);
    res
}

fn rust_compile(source: &str) -> Result<Vec<u8>, String> {
    let tree = compile(source, ScriptEnv::new()).map_err(|e| format!("{:?}", e))?;
    tree.sigma_serialize_bytes()
        .map_err(|e| format!("serialize: {:?}", e))
}

fn classify(rust: &Result<Vec<u8>, String>, scala: &Result<Vec<u8>, String>) -> Outcome {
    match (rust, scala) {
        (Ok(rb), Ok(sb)) => {
            if rb == sb {
                Outcome::Match
            } else {
                Outcome::Diff
            }
        }
        (Ok(_), Err(_)) => Outcome::ScalaFail,
        (Err(_), Ok(_)) => Outcome::RustFail,
        (Err(_), Err(_)) => Outcome::BothFail,
    }
}

fn safe_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn first_diff_offset(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

fn agreement_prefix_hex(a: &[u8], b: &[u8], len: usize) -> String {
    let n = first_diff_offset(a, b).min(len);
    a[..n].iter().map(|x| format!("{:02x}", x)).collect()
}

fn error_prefix(msg: &str, n: usize) -> String {
    msg.chars()
        .take(n)
        .collect::<String>()
        .replace(['\n', '\r'], " ")
}

fn write_diff_artifact(
    diff_dir: &Path,
    name: &str,
    source: &str,
    rust_bytes: &[u8],
    scala_bytes: &[u8],
) {
    let path = diff_dir.join(format!("{}.txt", safe_filename(name)));
    let off = first_diff_offset(rust_bytes, scala_bytes);
    let mut s = String::new();
    s.push_str(&format!("=== name ===\n{}\n\n", name));
    s.push_str(&format!(
        "rust_bytes={} scala_bytes={} first_diff_offset={}\n\n",
        rust_bytes.len(),
        scala_bytes.len(),
        off
    ));
    s.push_str("=== source ===\n");
    s.push_str(source);
    s.push_str("\n\n=== rust hex ===\n");
    for b in rust_bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s.push_str("\n\n=== scala hex ===\n");
    for b in scala_bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s.push('\n');
    let _ = fs::write(&path, s);
}

fn write_fail_artifact(fail_dir: &Path, name: &str, label: &str, source: &str, msg: &str) {
    let path = fail_dir.join(format!("{}_{}.txt", safe_filename(name), label));
    let mut s = String::new();
    s.push_str(&format!("=== name ===\n{}\n\n=== source ===\n", name));
    s.push_str(source);
    s.push_str(&format!("\n\n=== {} ===\n{}\n", label, msg));
    let _ = fs::write(&path, s);
}

fn build_corpus(crate_root: &Path) -> Vec<(String, String)> {
    let corpus_dir = crate_root.join("target").join("diff_fuzz").join("corpus");
    if let Ok(rd) = fs::read_dir(&corpus_dir) {
        let mut paths: Vec<PathBuf> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("es"))
            .collect();
        if !paths.is_empty() {
            paths.sort();
            return paths
                .into_iter()
                .map(|p| {
                    let name = p
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| p.display().to_string());
                    let src = fs::read_to_string(&p)
                        .unwrap_or_else(|e| panic!("read {}: {}", p.display(), e));
                    (name, src)
                })
                .collect();
        }
    }

    let mut out: Vec<(String, String)> = Vec::new();
    for (name, source) in ecosystem_corpus() {
        out.push((name.to_string(), source.to_string()));
    }
    let sig15_dir = crate_root
        .join("tests")
        .join("fixtures")
        .join("significant_15");
    for (fname, prelude) in significant_15_preludes() {
        let path = sig15_dir.join(fname);
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        let source = if prelude.is_empty() {
            raw
        } else if let Some(idx) = raw.find('{') {
            let mut s = String::with_capacity(raw.len() + prelude.len());
            s.push_str(&raw[..=idx]);
            s.push('\n');
            s.push_str(&prelude);
            s.push_str(&raw[idx + 1..]);
            s
        } else {
            raw
        };
        out.push((fname.to_string(), source));
    }
    out
}

/// Significant-15 (filename, prelude) pairs — mirrors the inline definition
/// inside `test_significant_15` in `compiler.rs`. Duplicated here because the
/// canonical version is private test state; if the upstream prelude config
/// drifts, `test_significant_15` is the source of truth.
fn significant_15_preludes() -> Vec<(&'static str, String)> {
    let dummy_token =
        "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000001\")";
    let dummy_token2 =
        "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000002\")";
    let dummy_token3 =
        "fromBase16(\"0000000000000000000000000000000000000000000000000000000000000003\")";
    let dummy_addr = "fromBase16(\"00aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\")";
    let dummy_pk = "proveDlog(decodePoint(fromBase16(\"02d04baf1e643c82e9e25f35a8636e1c4ae9bfc12944af9c8dd9b6a47fd7f8b700\")))";
    let _ = dummy_pk;
    vec![
        ("chaincash_reserve.es", String::new()),
        ("dexy_bank_full.es", String::new()),
        ("duckpools_child_interest.es", String::new()),
        ("oracle_refresh.es", String::new()),
        ("rosen_event_trigger.es", String::new()),
        ("sigmao_option.es", String::new()),
        ("skyharbor_v1_erg.es", String::new()),
        ("spectrum_n2t_pool.es", String::new()),
        (
            "ergomixer_fullmix.es",
            format!(
                "val tokenId: Coll[Byte] = {dummy_token};\n\
                 val feeEmissionScriptHash: Coll[Byte] = {dummy_token2};\n",
            ),
        ),
        (
            "ergoraffle_active.es",
            format!(
                "val ticketScriptHash: Coll[Byte] = {dummy_token};\n\
                 val winnerScriptHash: Coll[Byte] = {dummy_token2};\n\
                 val redeemScriptHash: Coll[Byte] = {dummy_token3};\n\
                 val randomBoxToken: Coll[Byte] = {dummy_token};\n\
                 val fee: Long = 1000000L;\n",
            ),
        ),
        (
            "gluon_box_guard.es",
            format!(
                "val _MinFee: Long = 1000000L;\n\
                 val _GluonWNFTId: Coll[Byte] = {dummy_token};\n\
                 val _OracleBuybackNFT: Coll[Byte] = {dummy_token2};\n\
                 val _OraclePoolNFT: Coll[Byte] = {dummy_token3};\n\
                 val _GLUONW_BOX: Coll[Byte] = {dummy_token};\n\
                 val _GLUONW_NEUTRONS_TOKEN: Coll[Byte] = {dummy_token2};\n\
                 val _GLUONW_PROTONS_TOKEN: Coll[Byte] = {dummy_token3};\n\
                 val _BOX: Coll[Byte] = {dummy_token};\n\
                 val _OracleFeePk: Coll[Byte] = {dummy_addr};\n\
                 val _MULTISIG: SigmaProp = {dummy_pk};\n\
                 val _TOTAL_SUPPLY: Long = 1000000000000000L;\n\
                 val _TOTAL_SUPPLY_REGISTER: Long = 1000000000000000L;\n\
                 val _DEV_FEE_THRESHOLD: Long = 1000000L;\n\
                 val _MAX_DEV_FEE_THRESHOLD: Long = 100000000L;\n\
                 val _ASSET_MAX_DEV_FEE_THRESHOLD: Long = 100000000L;\n\
                 val _DEV_FEE_REPAID: Long = 0L;\n\
                 val _FEE_REPAID: Long = 0L;\n\
                 val _Per_volume_bucket: Long = 720L;\n\
                 val _PER_VOLUME_BUCKET: Long = 720L;\n",
            ),
        ),
        (
            "phoenix_hodlerg_bank_full.es",
            format!("val phoenixFeeContractBytesHash: Coll[Byte] = {dummy_token};\n"),
        ),
        (
            "paideia_stake_state.es",
            format!(
                "val _stakedTokenID: Coll[Byte] = {dummy_token};\n\
                 val _stakePoolNFT: Coll[Byte] = {dummy_token2};\n\
                 val _emissionNFT: Coll[Byte] = {dummy_token3};\n\
                 val _stakeContractHash: Coll[Byte] = {dummy_addr};\n",
            ),
        ),
        (
            "sigmausd_bank.es",
            format!(
                "val oraclePoolNFT: Coll[Byte] = {dummy_token};\n\
                 val updateNFT: Coll[Byte] = {dummy_token2};\n\
                 val minReserveRatioPercent: Long = 400L;\n\
                 val defaultMaxReserveRatioPercent: Long = 800L;\n",
            ),
        ),
        (
            "spectrum_t2t_pool.es",
            String::from("val InitiallyLockedLP: Long = 9223372036854775807L;\n"),
        ),
    ]
}

#[test]
#[ignore] // requires running Ergo node + API_KEY for write endpoints
fn test_diff_fuzz() {
    let api_key = std::env::var("API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        eprintln!(
            "WARN: API_KEY not set — proceeding (read-only endpoints may not require auth)"
        );
    }
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_root = crate_root.join("target").join("diff_fuzz");
    let cache_dir = out_root.join("scala_cache");
    let diff_dir = out_root.join("diff");
    let fail_dir = out_root.join("fail");
    ensure_dir(&out_root);
    ensure_dir(&cache_dir);
    ensure_dir(&diff_dir);
    ensure_dir(&fail_dir);

    // Per-run artifact dirs are recomputed fresh; cache survives.
    for d in [&diff_dir, &fail_dir] {
        if let Ok(entries) = fs::read_dir(d) {
            for e in entries.flatten() {
                let _ = fs::remove_file(e.path());
            }
        }
    }

    let info_hash = match fetch_node_info_hash(&api_key) {
        Ok(h) => h,
        Err(e) => panic!(
            "node not running at {} (/info unreachable): {}\n\
             Start the local Ergo node before running diff_fuzz.",
            NODE_URL, e
        ),
    };
    eprintln!("=== diff_fuzz ===");
    eprintln!("node_info_hash: {}", info_hash);

    let corpus = build_corpus(&crate_root);
    eprintln!("corpus size: {}", corpus.len());

    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for k in ["MATCH", "DIFF", "RUST_FAIL", "SCALA_FAIL", "BOTH_FAIL"] {
        counts.insert(k, 0);
    }
    let mut diff_fingerprints: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut rust_fail_prefixes: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut scala_fail_prefixes: BTreeMap<String, (usize, String)> = BTreeMap::new();

    for (name, source) in &corpus {
        let rust = rust_compile(source);
        let scala = scala_compile_cached(source, &cache_dir, &info_hash, &api_key);
        let outcome = classify(&rust, &scala);
        match outcome {
            Outcome::Match => {
                let n = rust.as_ref().unwrap().len();
                eprintln!("  [MATCH]      {} ({} bytes)", name, n);
            }
            Outcome::Diff => {
                let rb = rust.as_ref().unwrap();
                let sb = scala.as_ref().unwrap();
                let off = first_diff_offset(rb, sb);
                eprintln!(
                    "  [DIFF]       {} (rust {}B / scala {}B; first diff @ byte {})",
                    name,
                    rb.len(),
                    sb.len(),
                    off
                );
                write_diff_artifact(&diff_dir, name, source, rb, sb);
                let prefix = agreement_prefix_hex(rb, sb, 16);
                let entry = diff_fingerprints
                    .entry(prefix)
                    .or_insert_with(|| (0, name.clone()));
                entry.0 += 1;
            }
            Outcome::RustFail => {
                let msg = rust.as_ref().err().unwrap();
                eprintln!("  [RUST_FAIL]  {}: {}", name, error_prefix(msg, 120));
                write_fail_artifact(&fail_dir, name, "rust_fail", source, msg);
                let entry = rust_fail_prefixes
                    .entry(error_prefix(msg, 80))
                    .or_insert_with(|| (0, name.clone()));
                entry.0 += 1;
            }
            Outcome::ScalaFail => {
                let msg = scala.as_ref().err().unwrap();
                eprintln!("  [SCALA_FAIL] {}: {}", name, error_prefix(msg, 120));
                write_fail_artifact(&fail_dir, name, "scala_fail", source, msg);
                let entry = scala_fail_prefixes
                    .entry(error_prefix(msg, 80))
                    .or_insert_with(|| (0, name.clone()));
                entry.0 += 1;
            }
            Outcome::BothFail => {
                let r = rust.as_ref().err().unwrap();
                let s = scala.as_ref().err().unwrap();
                eprintln!(
                    "  [BOTH_FAIL]  {}: rust={} scala={}",
                    name,
                    error_prefix(r, 60),
                    error_prefix(s, 60)
                );
                write_fail_artifact(
                    &fail_dir,
                    name,
                    "both_fail",
                    source,
                    &format!("=== rust ===\n{}\n=== scala ===\n{}", r, s),
                );
            }
        }
        *counts.get_mut(outcome.label()).unwrap() += 1;
    }

    let mut summary = String::new();
    summary.push_str("=== diff_fuzz summary ===\n");
    summary.push_str(&format!("node_info_hash: {}\n", info_hash));
    summary.push_str(&format!("corpus size:    {}\n\n", corpus.len()));
    for k in ["MATCH", "DIFF", "RUST_FAIL", "SCALA_FAIL", "BOTH_FAIL"] {
        summary.push_str(&format!("{:11}: {}\n", k, counts[k]));
    }

    fn dump_top(buf: &mut String, header: &str, m: &BTreeMap<String, (usize, String)>, n: usize) {
        if m.is_empty() {
            return;
        }
        buf.push('\n');
        buf.push_str(header);
        buf.push('\n');
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then(a.0.cmp(b.0)));
        for (key, (count, example)) in v.iter().take(n) {
            buf.push_str(&format!("  {:3}  {}  example: {}\n", count, key, example));
        }
    }

    dump_top(
        &mut summary,
        "Top DIFF agreement prefixes (first 16 bytes of agreement before divergence):",
        &diff_fingerprints,
        8,
    );
    dump_top(
        &mut summary,
        "Top RUST_FAIL error prefixes:",
        &rust_fail_prefixes,
        8,
    );
    dump_top(
        &mut summary,
        "Top SCALA_FAIL error prefixes:",
        &scala_fail_prefixes,
        8,
    );

    let cluster_eligible = counts["DIFF"] + counts["RUST_FAIL"];
    summary.push_str(&format!(
        "\nCluster-eligible: {} programs in DIFF + RUST_FAIL (input for F.3)\n",
        cluster_eligible
    ));

    eprintln!("\n{}", summary);
    let _ = fs::write(out_root.join("summary.txt"), &summary);
}

// ===========================================================================
// WS-F.3 — triage clustering
// ---------------------------------------------------------------------------
// Reads `target/diff_fuzz/{diff,fail}/*.txt` (populated by `test_diff_fuzz`)
// and groups failing programs into actionable per-cluster fix candidates.
//
// Cluster keying — deterministic, no randomness:
//   - DIFF       → agreement-prefix hex (rust[..first_diff_offset]) capped
//                  at 8 bytes / 16 hex chars. Same prefix ⇒ same divergence
//                  point in the lowering pipeline.
//   - RUST_FAIL  → normalized error fingerprint (strip `span: N..M`, drop
//                  trailing path, take ≤100 chars). Identical errors ⇒ same
//                  bug.
//   - SCALA_FAIL → same shape as RUST_FAIL on the Scala-side error body.
//   - BOTH_FAIL  → not expected (F.2 surface had 0); supported for safety.
//
// Calibration target (must reproduce within ±10% before trusting the rest):
//   - No-segregation fallback DIFF — ~209 programs, prefix "10".
//   - getOrElse-needs-default RUST_FAIL — 20 programs.
//   - MIR missing-tpe RUST_FAIL — 6 programs.
//
// Output: `target/diff_fuzz/clusters/<NNN>_<slug>.md` per cluster, plus
// `clusters/INDEX.md` ordered by priority (new bug surface first, known
// CSE-segregation issue last).
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ClusterOutcome {
    Diff,
    RustFail,
    ScalaFail,
    BothFail,
}

impl ClusterOutcome {
    fn label(self) -> &'static str {
        match self {
            ClusterOutcome::Diff => "DIFF",
            ClusterOutcome::RustFail => "RUST_FAIL",
            ClusterOutcome::ScalaFail => "SCALA_FAIL",
            ClusterOutcome::BothFail => "BOTH_FAIL",
        }
    }
}

#[derive(Debug, Clone)]
struct ClusterMember {
    name: String,
    source: String,
    outcome: ClusterOutcome,
    /// rust hex (DIFF only)
    rust_hex: String,
    /// scala hex (DIFF only)
    scala_hex: String,
    /// agreement prefix length in BYTES (DIFF only)
    first_diff_offset: usize,
    /// first ≤16-byte rust hex prefix where rust == scala (DIFF only)
    agreement_prefix_hex: String,
    /// raw error message (FAIL only)
    error: String,
    /// normalized error fingerprint (FAIL only)
    error_fingerprint: String,
    /// sorted multi-set of constructs extracted from source
    constructs: Vec<String>,
    /// number of non-empty source lines
    line_count: usize,
}

/// Tag for INDEX.md priority ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ClusterTag {
    NewSurface,
    ParserBinder,
    KnownIssue,
}

impl ClusterTag {
    fn label(self) -> &'static str {
        match self {
            ClusterTag::NewSurface => "new-surface",
            ClusterTag::ParserBinder => "parser-binder",
            ClusterTag::KnownIssue => "known-issue",
        }
    }
}

#[derive(Debug, Clone)]
struct Cluster {
    key: String,
    outcome: ClusterOutcome,
    tag: ClusterTag,
    members: Vec<ClusterMember>,
}

/// Parse a DIFF artifact written by `write_diff_artifact`.
fn parse_diff_artifact(path: &Path) -> Option<ClusterMember> {
    let body = fs::read_to_string(path).ok()?;
    let name = extract_section(&body, "=== name ===")?
        .trim()
        .lines()
        .next()?
        .to_string();
    let stats_line = body.lines().find(|l| l.contains("first_diff_offset="))?;
    let first_diff_offset = stats_line
        .split_whitespace()
        .find_map(|t| t.strip_prefix("first_diff_offset="))
        .and_then(|s| s.parse::<usize>().ok())?;
    let source = extract_section(&body, "=== source ===")?
        .trim_end()
        .to_string();
    let rust_hex = extract_section(&body, "=== rust hex ===")?
        .trim()
        .to_string();
    let scala_hex = extract_section(&body, "=== scala hex ===")?
        .trim()
        .to_string();
    let cap_bytes = first_diff_offset.min(8);
    let agreement_prefix_hex = rust_hex.chars().take(cap_bytes * 2).collect::<String>();
    let constructs = construct_fingerprint(&source);
    let line_count = source.lines().filter(|l| !l.trim().is_empty()).count();
    Some(ClusterMember {
        name,
        source,
        outcome: ClusterOutcome::Diff,
        rust_hex,
        scala_hex,
        first_diff_offset,
        agreement_prefix_hex,
        error: String::new(),
        error_fingerprint: String::new(),
        constructs,
        line_count,
    })
}

/// Parse a FAIL artifact written by `write_fail_artifact`.
fn parse_fail_artifact(path: &Path) -> Option<ClusterMember> {
    let fname = path.file_stem()?.to_string_lossy().to_string();
    let (outcome, label_marker) = if fname.ends_with("_rust_fail") {
        (ClusterOutcome::RustFail, "=== rust_fail ===")
    } else if fname.ends_with("_scala_fail") {
        (ClusterOutcome::ScalaFail, "=== scala_fail ===")
    } else if fname.ends_with("_both_fail") {
        (ClusterOutcome::BothFail, "=== both_fail ===")
    } else {
        return None;
    };
    let body = fs::read_to_string(path).ok()?;
    let name = extract_section(&body, "=== name ===")?
        .trim()
        .lines()
        .next()?
        .to_string();
    let source = extract_section(&body, "=== source ===")?
        .trim_end()
        .to_string();
    let error = extract_section(&body, label_marker)?.trim().to_string();
    let error_fingerprint = normalize_error(&error);
    let constructs = construct_fingerprint(&source);
    let line_count = source.lines().filter(|l| !l.trim().is_empty()).count();
    Some(ClusterMember {
        name,
        source,
        outcome,
        rust_hex: String::new(),
        scala_hex: String::new(),
        first_diff_offset: 0,
        agreement_prefix_hex: String::new(),
        error,
        error_fingerprint,
        constructs,
        line_count,
    })
}

/// Pull the body between `header` and the next `=== ` line (or EOF).
fn extract_section(body: &str, header: &str) -> Option<String> {
    let start = body.find(header)?;
    let after = &body[start + header.len()..];
    let after = after.strip_prefix('\n').unwrap_or(after);
    let end = after.find("\n=== ").unwrap_or(after.len());
    Some(after[..end].to_string())
}

/// Normalize an error message into a fingerprint suitable for clustering.
///
/// Steps:
///   1. Take the first line.
///   2. Replace `span: N..M` with `span: _`.
///   3. Strip absolute file paths (`/...`) up to a whitespace.
///   4. Truncate to 100 chars.
fn normalize_error(err: &str) -> String {
    let first = err.lines().next().unwrap_or("").trim();
    // Strip span numbers
    let mut out = String::with_capacity(first.len());
    let bytes = first.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if first[i..].starts_with("span: ") {
            out.push_str("span: _");
            i += "span: ".len();
            // skip digits, dots, digits
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out.chars().take(100).collect()
}

/// Lex-extract a sorted multi-set of constructs from an ErgoScript source.
///
/// Captures: predef names, `.method` calls, operators, special forms.
/// The output is intentionally coarse — used both as a per-cluster summary
/// and as a hypothesis-text seed.
fn construct_fingerprint(source: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let known_predefs: &[&str] = &[
        "sigmaProp", "anyOf", "allOf", "atLeast", "blake2b256", "sha256",
        "byteArrayToBigInt", "byteArrayToLong", "longToByteArray",
        "decodePoint", "groupGenerator", "fromBase16", "fromBase58",
        "proveDlog", "proveDHTuple", "getVar", "OUTPUTS", "INPUTS",
        "SELF", "CONTEXT", "HEIGHT", "MIN_VALUE", "MAX_VALUE",
        "Coll", "Some", "None", "Option", "min", "max", "abs",
        "executeFromVar", "substConstants", "xorOf", "logicalNot",
        "outerJoin", "place_holder",
    ];
    let known_methods: &[&str] = &[
        "exp", "multiply", "negate", "getEncoded", "get", "getOrElse",
        "isDefined", "isEmpty", "size", "filter", "map", "fold", "forall",
        "exists", "indices", "indexOf", "slice", "append", "flatMap",
        "patch", "updated", "updateMany", "zip", "toBigInt", "toByte",
        "toShort", "toInt", "toLong", "toBytes", "toBits", "value",
        "propositionBytes", "id", "bytes", "bytesWithoutRef",
        "tokens", "creationInfo", "register", "R0", "R1", "R2", "R3",
        "R4", "R5", "R6", "R7", "R8", "R9",
    ];
    let known_forms: &[&str] = &[
        "if", "else", "val", "fun", "true", "false",
    ];

    // Identifier scan
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident = &source[start..i];
            if known_predefs.contains(&ident) {
                tokens.push(ident.to_string());
            } else if known_forms.contains(&ident) {
                tokens.push(ident.to_string());
            }
        } else if c == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let start = i + 1;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident = &source[start..i];
            if known_methods.contains(&ident) {
                tokens.push(format!(".{}", ident));
            } else if ident.starts_with('R') && ident.len() == 2 {
                // Treat any `.R0`..`.R9` (and others matching pattern) as register
                tokens.push(format!(".{}", ident));
            }
        } else {
            // Operator scan
            let two = if i + 1 < bytes.len() {
                std::str::from_utf8(&bytes[i..i + 2]).unwrap_or("")
            } else {
                ""
            };
            let op2 = matches!(two, "==" | "!=" | "<=" | ">=" | "&&" | "||");
            if op2 {
                tokens.push(two.to_string());
                i += 2;
                continue;
            }
            i += 1;
        }
    }
    tokens.sort();
    tokens
}

/// Determine cluster key + tag for a member.
fn cluster_key_and_tag(m: &ClusterMember) -> (String, ClusterTag) {
    match m.outcome {
        ClusterOutcome::Diff => {
            // No-seg fallback: agreement prefix is exactly "10" (1 byte).
            let tag = if m.agreement_prefix_hex == "10" {
                ClusterTag::KnownIssue
            } else {
                ClusterTag::NewSurface
            };
            (format!("D:{}", m.agreement_prefix_hex), tag)
        }
        ClusterOutcome::RustFail => (
            format!("RF:{}", m.error_fingerprint),
            ClusterTag::NewSurface,
        ),
        ClusterOutcome::ScalaFail => (
            format!("SF:{}", m.error_fingerprint),
            ClusterTag::ParserBinder,
        ),
        ClusterOutcome::BothFail => (
            format!("BF:{}", m.error_fingerprint),
            ClusterTag::NewSurface,
        ),
    }
}

/// Priority rank — lower comes first in INDEX.md.
///
/// Order (per F.3 handoff §"Handoff to per-cluster fix sessions"):
///   0. RUST_FAIL with "missing tpe" (smallest, tightest)
///   1. RUST_FAIL with "getOrElse"
///   2. Other RUST_FAIL
///   3. DIFF (new-surface, i.e. not no-seg-fallback)
///   4. SCALA_FAIL (parser-binder)
///   5. DIFF no-seg-fallback (known-issue)
///   6. BOTH_FAIL
fn priority_rank(c: &Cluster) -> u8 {
    match (c.outcome, c.tag) {
        (ClusterOutcome::RustFail, _) => {
            let any = c.members.first();
            let fp = any.map(|m| m.error_fingerprint.as_str()).unwrap_or("");
            if fp.contains("missing tpe") {
                0
            } else if fp.contains("getOrElse") {
                1
            } else {
                2
            }
        }
        (ClusterOutcome::Diff, ClusterTag::NewSurface) => 3,
        (ClusterOutcome::ScalaFail, _) => 4,
        (ClusterOutcome::Diff, ClusterTag::KnownIssue) => 5,
        (ClusterOutcome::BothFail, _) => 6,
        _ => 7,
    }
}

/// Short summary string for a cluster (used in filename + INDEX.md row).
fn cluster_summary(c: &Cluster) -> String {
    match c.outcome {
        ClusterOutcome::Diff => {
            if c.members.first().map(|m| m.agreement_prefix_hex.as_str()) == Some("10") {
                "no-seg-fallback".to_string()
            } else {
                let prefix = c
                    .members
                    .first()
                    .map(|m| m.agreement_prefix_hex.as_str())
                    .unwrap_or("");
                format!("DIFF agreement-prefix {}", prefix)
            }
        }
        ClusterOutcome::RustFail => {
            let fp = c
                .members
                .first()
                .map(|m| m.error_fingerprint.as_str())
                .unwrap_or("");
            // Drop the wrapping `MirLoweringError(MirLoweringError { msg: "..."` if present
            if let Some(idx) = fp.find("msg: \"") {
                let after = &fp[idx + 6..];
                let end = after.find('"').unwrap_or(after.len().min(60));
                return after[..end].to_string();
            }
            fp.chars().take(60).collect()
        }
        ClusterOutcome::ScalaFail => {
            let fp = c
                .members
                .first()
                .map(|m| m.error_fingerprint.as_str())
                .unwrap_or("");
            fp.chars().take(60).collect()
        }
        ClusterOutcome::BothFail => "BOTH_FAIL".to_string(),
    }
}

/// Slug for filenames — short, ascii-safe.
fn cluster_slug(c: &Cluster) -> String {
    let s = cluster_summary(c);
    let mut out = String::new();
    for ch in s.chars().take(40) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// Top-N most common constructs in a cluster, sorted by frequency desc then
/// alphabetically.
fn top_constructs(c: &Cluster, n: usize) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for m in &c.members {
        // Use a per-member set so a construct repeated N times in one source
        // doesn't dominate the cluster summary.
        let mut seen: std::collections::BTreeSet<&str> =
            std::collections::BTreeSet::new();
        for t in &m.constructs {
            seen.insert(t.as_str());
        }
        for t in seen {
            *counts.entry(t.to_string()).or_insert(0) += 1;
        }
    }
    let mut v: Vec<(String, usize)> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.into_iter().take(n).collect()
}

/// Heuristic hypothesis text — a hint, not a diagnosis. Per the F.3 handoff,
/// the per-cluster fix session still needs Metals confirmation before any
/// code change.
fn hypothesis(c: &Cluster) -> String {
    match c.outcome {
        ClusterOutcome::RustFail => {
            let fp = c
                .members
                .first()
                .map(|m| m.error_fingerprint.as_str())
                .unwrap_or("");
            if fp.contains("getOrElse requires default") {
                "Generator emits `Option.getOrElse(default)` shapes that the \
MIR lowering rejects with `getOrElse requires default argument`. Likely a \
match-arm gap in the `getOrElse` lowering — the Rust pipeline probably loses \
the default-arg slot at HIR→MIR for a specific Option-source variant (e.g. \
`SELF.R4[Long].getOrElse(0L)`). Confirm Scala-side via Metals on \
`MethodCall(Option.GetOrElseMethod)` in `sigmastate`, then locate the missing \
arm in `mir_lowering` getOrElse handling."
                    .to_string()
            } else if fp.contains("missing tpe") {
                "MIR lowering encountered an HIR Expr without a type annotation. \
Likely a `propagate_val_types` coverage gap for the specific Expr variant the \
generator surfaces. Inspect the representative source for the inner construct \
that lacks a type — common candidates: numeric coercion, register access \
without explicit `[T]`. Cross-check the WS-E.1 IR-PASS-COVERAGE-MATRIX for the \
unproduced variants list."
                    .to_string()
            } else {
                format!(
                    "RUST_FAIL with normalized error `{}`. New surface: \
representative source likely contains a construct the Rust pipeline rejects \
where Scala accepts. Start from the smallest representative and grep the \
error site in `compiler.rs` / `mir_lowering`.",
                    fp.chars().take(80).collect::<String>()
                )
            }
        }
        ClusterOutcome::Diff => {
            let constructs: Vec<String> =
                top_constructs(c, 5).into_iter().map(|(s, _)| s).collect();
            if c.members.first().map(|m| m.agreement_prefix_hex.as_str()) == Some("10") {
                format!(
                    "All members diverge at the ErgoTree header byte (offset 1) \
— the Scala oracle emits a segregated tree with N constants while the Rust \
pipeline falls back to a non-segregated emission. This is the documented \
**CSE-segregation-roundtrip** known issue (ERGOSCRIPT-COMPILER-STATUS.md \
§Known issues): CSE-extracted vals fail `ErgoTree::new` segregation, so \
`schedule.rs` falls back to non-segregated output. Same family as SigmaFi \
OpenOrderERG / OpenOrderToken / SkyHarbor SigUSDV1 and chaincash pre-S76. \
**Not a quick categorical fix** — closing it is a structural rewrite of the \
post-CSE schedule pipeline. Top constructs in this cluster: {}.",
                    constructs.join(", ")
                )
            } else {
                let prefix = c
                    .members
                    .first()
                    .map(|m| m.agreement_prefix_hex.as_str())
                    .unwrap_or("");
                format!(
                    "All members agree on tree-shape up to prefix `{}` then \
diverge. Same divergence point ⇒ likely the same lowering arm. Top constructs: \
{}. Per-cluster fix template: pick the smallest representative, run \
`compile_via_node` to capture Scala's exact bytes, decode both trees with \
`ergotree-ir` and Metals goto-definition on the first diverging op to confirm \
the Scala-emitted shape, then narrow the Rust-side lowering arm.",
                    prefix,
                    constructs.join(", ")
                )
            }
        }
        ClusterOutcome::ScalaFail => format!(
            "Scala parser/binder rejects what the Rust pipeline accepts. \
Likely a generator-emitted construct that Scala's stricter typing or runtime \
arithmetic check refuses (the F.2 surface includes `Byte` overflow on \
literal addition, which Scala validates at compile time). Scope is \
documentation / surface understanding rather than a code fix on the Rust \
side. Top constructs: {}.",
            top_constructs(c, 5)
                .into_iter()
                .map(|(s, _)| s)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ClusterOutcome::BothFail => "Both compilers failed — likely an \
ill-typed source. Should be 0 if F.2's typed-AST discipline is holding."
            .to_string(),
    }
}

fn write_cluster_file(
    cluster_dir: &Path,
    cluster_id: usize,
    c: &Cluster,
) -> String {
    let slug = cluster_slug(c);
    let filename = format!("{:03}_{}.md", cluster_id, slug);
    let path = cluster_dir.join(&filename);

    // Sort members by line_count asc, then name asc, for deterministic
    // representative selection.
    let mut sorted = c.members.clone();
    sorted.sort_by(|a, b| a.line_count.cmp(&b.line_count).then(a.name.cmp(&b.name)));

    let representative = &sorted[0];
    let constructs_str: Vec<String> = top_constructs(c, 8)
        .into_iter()
        .map(|(s, n)| format!("{} ({}×)", s, n))
        .collect();

    let mut s = String::new();
    s.push_str(&format!("# Cluster {:03} — {}\n\n", cluster_id, cluster_summary(c)));
    s.push_str(&format!("**Outcome:** {}\n", c.outcome.label()));
    s.push_str(&format!("**Tag:** {}\n", c.tag.label()));
    s.push_str(&format!("**Programs in cluster:** {}\n", c.members.len()));
    s.push_str(&format!(
        "**Top constructs:** {}\n",
        constructs_str.join(", ")
    ));
    if c.outcome == ClusterOutcome::Diff {
        let m0 = &c.members[0];
        s.push_str(&format!(
            "**Agreement prefix:** {} ({} bytes agreed before divergence)\n",
            m0.agreement_prefix_hex, m0.first_diff_offset
        ));
    }
    if matches!(
        c.outcome,
        ClusterOutcome::RustFail | ClusterOutcome::ScalaFail | ClusterOutcome::BothFail
    ) {
        s.push_str(&format!(
            "**Error fingerprint:** `{}`\n",
            c.members
                .first()
                .map(|m| m.error_fingerprint.as_str())
                .unwrap_or("")
        ));
    }
    s.push_str(&format!("**Cluster key:** `{}`\n\n", c.key));

    s.push_str("## Smallest representative\n\n");
    s.push_str(&format!("Source: `{}` ({} non-empty lines)\n\n", representative.name, representative.line_count));
    s.push_str("```ergoscript\n");
    s.push_str(representative.source.trim_end());
    s.push_str("\n```\n\n");
    if c.outcome == ClusterOutcome::Diff {
        s.push_str(&format!(
            "Rust hex: `{}`  \nScala hex: `{}`  \nfirst_diff_offset: {}\n\n",
            representative.rust_hex, representative.scala_hex, representative.first_diff_offset
        ));
    } else if !representative.error.is_empty() {
        s.push_str("Error:\n```\n");
        s.push_str(representative.error.trim_end());
        s.push_str("\n```\n\n");
    }

    s.push_str("## Five smallest\n\n");
    for m in sorted.iter().take(5) {
        s.push_str(&format!(
            "1. `{}` — {} lines\n",
            m.name, m.line_count
        ));
    }
    s.push('\n');

    if c.members.len() > 5 {
        s.push_str("## All member names\n\n");
        for m in &sorted {
            s.push_str(&format!("- `{}` ({} lines)\n", m.name, m.line_count));
        }
        s.push('\n');
    }

    s.push_str("## Hypothesis\n\n");
    s.push_str(&hypothesis(c));
    s.push_str("\n\n");

    s.push_str("## Suggested next session\n\n");
    s.push_str(
        "Per-cluster fix using the S68/S71/S76 template:\n\n\
1. Read the smallest representative above.\n\
2. Use Metals MCP `goto-definition` on the relevant Scala primitive to \
   confirm Scala's emitted shape (anchors in \
   `06c-ergoraffle-inner-block-HANDOFF.md` §\"Reference: Scala-side semantics\").\n\
3. Narrow the Rust-side fix to one arm.\n\
4. Run full regression suite + re-run `diff_fuzz` + `cluster`; verify this \
   cluster vanishes (or shrinks measurably for the no-seg known-issue).\n\
5. Commit `fix(ergoscript-compiler): WS-F cluster ",
    );
    s.push_str(&format!("{:03}", cluster_id));
    s.push_str(" — <root cause>`.\n");

    let _ = fs::write(&path, s);
    filename
}

#[test]
#[ignore] // requires F.1/F.2 outputs in target/diff_fuzz/{diff,fail}/
fn cluster() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_root = crate_root.join("target").join("diff_fuzz");
    let diff_dir = out_root.join("diff");
    let fail_dir = out_root.join("fail");
    let cluster_dir = out_root.join("clusters");

    if !diff_dir.exists() && !fail_dir.exists() {
        panic!(
            "no diff_fuzz artifacts at {} — run `test_diff_fuzz` first",
            out_root.display()
        );
    }

    // Wipe + recreate clusters dir each run so removed clusters don't linger.
    if cluster_dir.exists() {
        if let Ok(entries) = fs::read_dir(&cluster_dir) {
            for e in entries.flatten() {
                let _ = fs::remove_file(e.path());
            }
        }
    }
    ensure_dir(&cluster_dir);

    // Read all artifacts. Sort by file name for deterministic ingestion order.
    let mut members: Vec<ClusterMember> = Vec::new();
    let mut diff_paths: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = fs::read_dir(&diff_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("txt") {
                diff_paths.push(p);
            }
        }
    }
    diff_paths.sort();
    for p in &diff_paths {
        if let Some(m) = parse_diff_artifact(p) {
            members.push(m);
        }
    }

    let mut fail_paths: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = fs::read_dir(&fail_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("txt") {
                fail_paths.push(p);
            }
        }
    }
    fail_paths.sort();
    for p in &fail_paths {
        if let Some(m) = parse_fail_artifact(p) {
            members.push(m);
        }
    }

    let total = members.len();
    eprintln!("=== cluster ===");
    eprintln!("input artifacts: {}", total);

    // Group by (outcome, key) — use a sorted map for determinism.
    let mut buckets: BTreeMap<(ClusterOutcome, String), (ClusterTag, Vec<ClusterMember>)> =
        BTreeMap::new();
    for m in members {
        let (key, tag) = cluster_key_and_tag(&m);
        buckets
            .entry((m.outcome, key))
            .or_insert_with(|| (tag, Vec::new()))
            .1
            .push(m);
    }

    let mut clusters: Vec<Cluster> = buckets
        .into_iter()
        .map(|((outcome, key), (tag, mut members))| {
            // Sort cluster members by name for stable per-file output.
            members.sort_by(|a, b| a.name.cmp(&b.name));
            Cluster {
                key,
                outcome,
                tag,
                members,
            }
        })
        .collect();

    // Sort by priority rank, then by size desc, then by key for determinism.
    clusters.sort_by(|a, b| {
        priority_rank(a)
            .cmp(&priority_rank(b))
            .then_with(|| b.members.len().cmp(&a.members.len()))
            .then_with(|| a.key.cmp(&b.key))
    });

    eprintln!("distinct clusters: {}", clusters.len());

    // ===== Calibration check =====
    // Find the 3 pre-known clusters and emit a warning if any is off by >10%.
    let mut calib: Vec<(String, usize, usize)> = Vec::new(); // (label, expected, actual)
    let no_seg = clusters
        .iter()
        .find(|c| {
            c.outcome == ClusterOutcome::Diff
                && c.members.first().map(|m| m.agreement_prefix_hex.as_str()) == Some("10")
        })
        .map(|c| c.members.len())
        .unwrap_or(0);
    calib.push(("no-seg-fallback".to_string(), 209, no_seg));
    let get_or_else = clusters
        .iter()
        .find(|c| {
            c.outcome == ClusterOutcome::RustFail
                && c.members
                    .first()
                    .map(|m| m.error_fingerprint.contains("getOrElse"))
                    .unwrap_or(false)
        })
        .map(|c| c.members.len())
        .unwrap_or(0);
    calib.push(("getOrElse-needs-default".to_string(), 20, get_or_else));
    let missing_tpe = clusters
        .iter()
        .find(|c| {
            c.outcome == ClusterOutcome::RustFail
                && c.members
                    .first()
                    .map(|m| m.error_fingerprint.contains("missing tpe"))
                    .unwrap_or(false)
        })
        .map(|c| c.members.len())
        .unwrap_or(0);
    calib.push(("MIR-missing-tpe".to_string(), 6, missing_tpe));

    eprintln!("calibration:");
    for (label, expected, actual) in &calib {
        let ratio = if *expected > 0 {
            (*actual as f64 / *expected as f64 - 1.0).abs()
        } else {
            0.0
        };
        let status = if *actual == 0 {
            "MISSING"
        } else if ratio > 0.10 {
            "OFF (>10%)"
        } else {
            "OK"
        };
        eprintln!(
            "  {:<25} expected ~{:<4} actual {:<4} [{}]",
            label, expected, actual, status
        );
    }

    // Write per-cluster files.
    let mut index_rows: Vec<(usize, String, String)> = Vec::new();
    for (idx, c) in clusters.iter().enumerate() {
        let cluster_id = idx + 1;
        let filename = write_cluster_file(&cluster_dir, cluster_id, c);
        let summary = cluster_summary(c);
        index_rows.push((cluster_id, filename, summary));
        eprintln!(
            "  cluster {:03} [{}] [{}] {}: {} programs",
            cluster_id,
            c.outcome.label(),
            c.tag.label(),
            c.key,
            c.members.len()
        );
    }

    // Write INDEX.md
    let mut index = String::new();
    index.push_str("# Cluster index\n\n");
    index.push_str("Generated by `cargo test -p ergoscript-compiler --test diff_fuzz cluster -- --ignored`.\n\n");
    index.push_str(&format!("**Total programs analyzed:** {}\n", total));
    index.push_str(&format!("**Distinct clusters:** {}\n\n", clusters.len()));
    index.push_str("## Calibration vs pre-known cluster shapes\n\n");
    index.push_str("| Pre-known cluster | Expected | Actual | Status |\n");
    index.push_str("|---|---|---|---|\n");
    for (label, expected, actual) in &calib {
        let ratio = if *expected > 0 {
            (*actual as f64 / *expected as f64 - 1.0).abs()
        } else {
            0.0
        };
        let status = if *actual == 0 {
            "MISSING"
        } else if ratio > 0.10 {
            "OFF (>10%)"
        } else {
            "OK"
        };
        index.push_str(&format!(
            "| {} | ~{} | {} | {} |\n",
            label, expected, actual, status
        ));
    }
    index.push('\n');

    index.push_str("## Clusters by priority\n\n");
    index.push_str(
        "Order: new-surface RUST_FAIL (smallest/tightest first), new-surface DIFF, \
parser-binder SCALA_FAIL, known-issue no-seg-fallback last.\n\n",
    );
    index.push_str("| # | Outcome | Tag | Programs | Smallest (lines) | Summary | File |\n");
    index.push_str("|---|---|---|---|---|---|---|\n");
    for (idx, c) in clusters.iter().enumerate() {
        let cluster_id = idx + 1;
        let smallest_lines = c.members.iter().map(|m| m.line_count).min().unwrap_or(0);
        let filename = &index_rows[idx].1;
        let summary = cluster_summary(c);
        index.push_str(&format!(
            "| {:03} | {} | {} | {} | {} | {} | [{}](./{}) |\n",
            cluster_id,
            c.outcome.label(),
            c.tag.label(),
            c.members.len(),
            smallest_lines,
            summary,
            filename,
            filename
        ));
    }
    index.push('\n');
    index.push_str("## Handoff notes\n\n");
    index.push_str(
        "Per the F.3 handoff, each cluster is its own downstream session. \
Recommended priority follows the table above (top → bottom). For each cluster:\n\n\
1. Read `clusters/<NNN>_<slug>.md` and the smallest representative.\n\
2. Use Metals MCP to confirm Scala-side behavior for the constructs involved \
(anchors in `06c-ergoraffle-inner-block-HANDOFF.md` §\"Reference: Scala-side semantics\").\n\
3. Narrow Rust-side fix.\n\
4. Run full regression suite + re-run `diff_fuzz` + `cluster`; verify the \
cluster vanishes or shrinks measurably (no-seg-fallback only).\n\
5. Commit with `fix(ergoscript-compiler): WS-F cluster <id> — <root cause>`.\n\n\
Do NOT batch fixes — bisecting becomes impossible.\n",
    );

    let _ = fs::write(cluster_dir.join("INDEX.md"), &index);
    eprintln!("wrote {} cluster files + INDEX.md", clusters.len());
}
