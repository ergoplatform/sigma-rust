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
