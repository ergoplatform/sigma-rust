//! Cross-validation test harness for JIT costing parity between sigma-rust
//! and the Scala sigmastate-interpreter.
//!
//! Loads mainnet transaction data, Scala-computed reference costs, and block
//! headers from JSON files, then replays the Scala cost pipeline locally and
//! compares results with zero tolerance.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use ergo_chain_types::{Header, PreHeader};
use ergo_lib::chain::ergo_state_context::ErgoStateContext;
use ergo_lib::chain::parameters::Parameters;
use ergo_lib::chain::transaction::Transaction;
use ergo_lib::wallet::signing::make_context;
use ergo_lib::wallet::tx_context::TransactionContext;
use ergotree_interpreter::eval::reduce_to_crypto;
use ergotree_interpreter::sigma_protocol::crypto_cost::estimate_crypto_cost;
use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use ergotree_interpreter::sigma_protocol::verifier::verify_signature;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::token::TokenId;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::*;
use sigma_ser::ScorexSerializable;

// ---------------------------------------------------------------------------
// JSON serde structs
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct TxRecord {
    #[allow(dead_code)]
    id: String,
    bytes: String,
    #[allow(dead_code)]
    #[serde(rename = "bytesToSign")]
    bytes_to_sign: String,
    height: u32,
}

#[derive(serde::Deserialize)]
struct CostRecord {
    tx_id: String,
    #[allow(dead_code)]
    height: u32,
    block_cost: u64,
}

#[derive(serde::Deserialize)]
struct HeaderRecord {
    height: u32,
    #[allow(dead_code)]
    id: String,
    bytes: String,
    #[allow(dead_code)]
    #[serde(rename = "headerWithoutPow")]
    header_without_pow: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn decode_hex(hex: &str) -> Vec<u8> {
    base16::decode(hex.as_bytes()).expect("invalid hex in test vector")
}

/// Count token entries for a list of boxes (for init cost computation).
/// Returns (total_entries, distinct_ids).
fn count_tokens(boxes: &[ErgoBox]) -> (usize, usize) {
    let mut total = 0usize;
    let mut distinct = HashSet::<TokenId>::new();
    for b in boxes {
        for tok in b.tokens.iter().flatten() {
            total += 1;
            distinct.insert(tok.token_id);
        }
    }
    (total, distinct.len())
}

/// Build the 10-header window for a given block height from a map of
/// height -> Header. Returns None if any of the 10 preceding headers is missing.
fn build_header_window(header_map: &HashMap<u32, Header>, height: u32) -> Option<[Header; 10]> {
    // We need headers at heights (height-1), (height-2), ..., (height-10)
    // but the Scala code uses the "previous 10 headers" in the state context.
    // The pre_header is FROM the current block header (height).
    // The 10 headers are the 10 preceding ones: height-1 .. height-10.
    let mut arr: Vec<Header> = Vec::with_capacity(10);
    for i in 1..=10u32 {
        if height < i {
            return None;
        }
        match header_map.get(&(height - i)) {
            Some(h) => arr.push(h.clone()),
            None => return None,
        }
    }
    // Convert to [Header; 10]
    arr.try_into().ok()
}

/// Progressive UTXO tracker.
struct UtxoTracker {
    utxo: HashMap<ergotree_ir::chain::ergo_box::BoxId, ErgoBox>,
}

impl UtxoTracker {
    fn new() -> Self {
        Self {
            utxo: HashMap::new(),
        }
    }

    /// Snapshot the boxes needed for a transaction (inputs + data inputs) WITHOUT mutating UTXO.
    fn snapshot(&self, tx: &Transaction) -> (Vec<ErgoBox>, Vec<ErgoBox>, bool) {
        let mut input_boxes = Vec::new();
        let mut data_boxes = Vec::new();
        let mut ok = true;

        for inp in tx.inputs.iter() {
            match self.utxo.get(&inp.box_id) {
                Some(b) => input_boxes.push(b.clone()),
                None => {
                    ok = false;
                }
            }
        }
        if let Some(ref dis) = tx.data_inputs {
            for di in dis.iter() {
                match self.utxo.get(&di.box_id) {
                    Some(b) => data_boxes.push(b.clone()),
                    None => {
                        ok = false;
                    }
                }
            }
        }
        (input_boxes, data_boxes, ok)
    }

    /// Advance UTXO: remove consumed inputs, insert outputs.
    fn advance(&mut self, tx: &Transaction) {
        for inp in tx.inputs.iter() {
            self.utxo.remove(&inp.box_id);
        }
        for out in tx.outputs.iter() {
            self.utxo.insert(out.box_id(), out.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// Core parity check
// ---------------------------------------------------------------------------

const STORAGE_RENT_PERIOD: u32 = 1_051_200;

#[allow(dead_code)]
struct ParityResult {
    compared: usize,
    mismatches: usize,
    skipped: usize,
    errors: usize,
    script_errors: usize,
    max_height: u32,
    mismatch_details: Vec<String>,
}

fn run_parity_check_paths(tx_path: &Path, cost_path: &Path, hdr_path: &Path) -> ParityResult {
    let tx_records: Vec<TxRecord> =
        serde_json::from_str(&std::fs::read_to_string(tx_path).expect("read tx json"))
            .expect("parse tx json");
    let cost_records: Vec<CostRecord> =
        serde_json::from_str(&std::fs::read_to_string(cost_path).expect("read cost json"))
            .expect("parse cost json");
    let hdr_records: Vec<HeaderRecord> =
        serde_json::from_str(&std::fs::read_to_string(hdr_path).expect("read header json"))
            .expect("parse header json");

    // Build cost lookup: tx_id_hex -> block_cost
    let cost_map: HashMap<String, u64> = cost_records
        .iter()
        .map(|c| (c.tx_id.clone(), c.block_cost))
        .collect();

    // Parse and index headers
    let mut header_map: HashMap<u32, Header> = HashMap::new();
    for rec in &hdr_records {
        let bytes = decode_hex(&rec.bytes);
        let hdr = Header::scorex_parse_bytes(&bytes)
            .unwrap_or_else(|e| panic!("header parse at height {}: {:?}", rec.height, e));
        header_map.insert(rec.height, hdr);
    }

    let mut utxo = UtxoTracker::new();
    let mut compared = 0usize;
    let mut mismatches = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;
    let mut script_errors = 0usize;
    let mut max_height = 0u32;
    let mut mismatch_details: Vec<String> = Vec::new();

    for tx_rec in &tx_records {
        let tx_id_hex = &tx_rec.id;
        let height = tx_rec.height;
        if height > max_height {
            max_height = height;
        }

        // (a) Parse tx bytes
        let tx_bytes = decode_hex(&tx_rec.bytes);
        let tx = match Transaction::sigma_parse_bytes(&tx_bytes) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("ERROR: parse tx {}: {:?}", tx_id_hex, e);
                errors += 1;
                continue;
            }
        };

        // (b) Snapshot: look up spending and data input boxes BEFORE mutation
        let (input_boxes, data_boxes, resolved) = utxo.snapshot(&tx);

        // (c) Advance UTXO: remove consumed inputs, insert outputs
        utxo.advance(&tx);

        // (d) Check if fully resolvable and if we have the header window
        if !resolved {
            skipped += 1;
            continue;
        }

        let headers_10 = match build_header_window(&header_map, height) {
            Some(h) => h,
            None => {
                skipped += 1;
                continue;
            }
        };

        // Skip if no Scala reference cost for this tx
        let scala_cost = match cost_map.get(tx_id_hex) {
            Some(&c) => c,
            None => {
                skipped += 1;
                continue;
            }
        };

        // (e) Build ErgoStateContext and TransactionContext
        let current_header = match header_map.get(&height) {
            Some(h) => h.clone(),
            None => {
                skipped += 1;
                continue;
            }
        };
        let pre_header = PreHeader::from(current_header);
        let state_ctx = ErgoStateContext::new(pre_header, headers_10, Parameters::default());

        let tx_ctx =
            match TransactionContext::new(tx.clone(), input_boxes.clone(), data_boxes.clone()) {
                Ok(tc) => tc,
                Err(e) => {
                    eprintln!("ERROR: TransactionContext for {}: {:?}", tx_id_hex, e);
                    errors += 1;
                    continue;
                }
            };

        // (f) Init cost (block cost formula)
        let n_inputs = tx.inputs.len();
        let n_data_inputs = tx.data_inputs.as_ref().map_or(0, |d| d.len());
        let n_outputs = tx.outputs.len();

        let (in_entries, in_distinct) = count_tokens(&input_boxes);
        let (out_entries, out_distinct) = count_tokens(tx.outputs.as_slice());
        let token_cost = (in_entries + out_entries + in_distinct + out_distinct) as u64 * 100;

        let init_cost: u64 = 10000
            + n_inputs as u64 * 2000
            + n_data_inputs as u64 * 100
            + n_outputs as u64 * 100
            + token_cost;

        let mut running_jit: u64 = init_cost * 10;

        // (g) Per-input loop
        let mut tx_ok = true;
        let message = match tx.bytes_to_sign() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ERROR: bytes_to_sign for {}: {:?}", tx_id_hex, e);
                errors += 1;
                continue;
            }
        };

        #[allow(clippy::needless_range_loop)]
        for input_idx in 0..n_inputs {
            let pre_input = running_jit;

            // Build fresh context per input
            let ctx = match make_context(&state_ctx, &tx_ctx, input_idx) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "ERROR: make_context for {} input {}: {:?}",
                        tx_id_hex, input_idx, e
                    );
                    script_errors += 1;
                    tx_ok = false;
                    break;
                }
            };

            // Storage rent check: empty proof + age >= STORAGE_RENT_PERIOD -> cost=0
            let input_box = &input_boxes[input_idx];
            let proof = &tx.inputs.as_slice()[input_idx].spending_proof.proof;
            if matches!(proof, ProofBytes::Empty)
                && height >= input_box.creation_height + STORAGE_RENT_PERIOD
            {
                // Storage rent spending, no script evaluation needed
                continue;
            }

            // reduce_to_crypto
            let ergo_tree = &input_box.ergo_tree;
            let reduction = match reduce_to_crypto(ergo_tree, &ctx) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "ERROR: reduce_to_crypto for {} input {}: {:?}",
                        tx_id_hex, input_idx, e
                    );
                    script_errors += 1;
                    tx_ok = false;
                    break;
                }
            };

            // Accumulate eval cost. PR 846 stored JitCost directly in
            // `ReductionResult.cost`; sigma-rust stores block cost
            // (JitCost/10 floor). To avoid precision loss from ×10 scaling
            // a floored value, read the JIT delta straight from the fresh
            // per-input ctx (make_context initialises jit_cost to 0).
            running_jit += ctx.jit_cost_value();

            // Snap: running_jit -= (running_jit - pre_input) % 10
            let delta = running_jit - pre_input;
            running_jit -= delta % 10;

            // Crypto cost
            running_jit += estimate_crypto_cost(&reduction.sigma_prop);

            // Verify proof
            match &reduction.sigma_prop {
                SigmaBoolean::TrivialProp(false) => {
                    eprintln!(
                        "SCRIPT_ERROR: TrivialProp(false) for {} input {}",
                        tx_id_hex, input_idx
                    );
                    script_errors += 1;
                    tx_ok = false;
                    break;
                }
                SigmaBoolean::TrivialProp(true) => {
                    // No proof needed
                }
                sigma_prop => {
                    let proof_bytes: Vec<u8> = match proof {
                        ProofBytes::Empty => Vec::new(),
                        ProofBytes::Some(b) => b.clone(),
                    };
                    match verify_signature(sigma_prop.clone(), &message, &proof_bytes) {
                        Ok(true) => {}
                        Ok(false) => {
                            eprintln!(
                                "SCRIPT_ERROR: verify_signature false for {} input {}",
                                tx_id_hex, input_idx
                            );
                            script_errors += 1;
                            tx_ok = false;
                            break;
                        }
                        Err(e) => {
                            eprintln!(
                                "SCRIPT_ERROR: verify_signature error for {} input {}: {:?}",
                                tx_id_hex, input_idx, e
                            );
                            script_errors += 1;
                            tx_ok = false;
                            break;
                        }
                    }
                }
            }
        }

        // (h-i) Only if tx_ok: compare block_cost
        if !tx_ok {
            continue;
        }

        let block_cost = running_jit / 10;

        if block_cost != scala_cost {
            mismatches += 1;
            let detail = format!(
                "MISMATCH tx={} height={} rust={} scala={} diff={}",
                tx_id_hex,
                height,
                block_cost,
                scala_cost,
                block_cost as i64 - scala_cost as i64
            );
            eprintln!("{}", detail);
            mismatch_details.push(detail);
        } else {
            compared += 1;
        }
    }

    eprintln!("\n=== COST PARITY SUMMARY ===");
    eprintln!("compared (matched): {}", compared);
    eprintln!("mismatches:         {}", mismatches);
    eprintln!("skipped:            {}", skipped);
    eprintln!("parse errors:       {}", errors);
    eprintln!("script errors:      {}", script_errors);
    eprintln!("max height:         {}", max_height);
    eprintln!("===========================\n");

    ParityResult {
        compared,
        mismatches,
        skipped,
        errors,
        script_errors,
        max_height,
        mismatch_details,
    }
}

// ---------------------------------------------------------------------------
// Test entry points
// ---------------------------------------------------------------------------

fn run_parity_check(vectors_dir: &Path) -> ParityResult {
    run_parity_check_paths(
        &vectors_dir.join("transactions_700000_700050.json"),
        &vectors_dir.join("tx_costs_700000_700050.json"),
        &vectors_dir.join("headers_700000_700060.json"),
    )
}

/// Smoke test: always runs, uses bundled test vectors in ergo-lib/tests/test-vectors/
#[test]
fn smoke_cost_parity() {
    let vectors_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test-vectors");
    assert!(
        vectors_dir.exists(),
        "test-vectors directory not found at {:?}",
        vectors_dir
    );

    let result = run_parity_check(&vectors_dir);

    // Assertions
    assert_eq!(result.errors, 0, "no parse failures expected");
    assert_eq!(
        result.script_errors, 0,
        "no script evaluation failures expected"
    );
    assert!(
        result.compared + result.mismatches >= 50,
        "not vacuous: only {} txs compared+mismatched, expected >= 50",
        result.compared + result.mismatches
    );
    assert!(
        result.max_height < STORAGE_RENT_PERIOD,
        "smoke set guard: max_height {} >= storage rent period {}",
        result.max_height,
        STORAGE_RENT_PERIOD
    );
    assert_eq!(
        result.mismatches,
        0,
        "zero tolerance: {} mismatches\n{}",
        result.mismatches,
        result.mismatch_details.join("\n")
    );
}

/// Full corpus test: reads test vectors from a directory specified by
/// the ERGO_COST_VECTORS_DIR environment variable. Ignored by default.
/// Find a headers file in `dir` whose range covers `start..end`.
fn find_header_file(dir: &Path, start: u32, end: u32) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("headers_") || !name.ends_with(".json") {
                return None;
            }
            let range = name
                .trim_start_matches("headers_")
                .trim_end_matches(".json");
            let parts: Vec<&str> = range.split('_').collect();
            if parts.len() != 2 {
                return None;
            }
            let hs: u32 = parts[0].parse().ok()?;
            let he: u32 = parts[1].parse().ok()?;
            if hs <= start && he >= end {
                Some(e.path())
            } else {
                None
            }
        })
        .next()
}

#[test]
#[ignore]
fn full_corpus_cost_parity() {
    let vectors_dir = match std::env::var("ERGO_COST_VECTORS_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => {
            eprintln!("ERGO_COST_VECTORS_DIR not set, skipping full_corpus_cost_parity");
            return;
        }
    };
    assert!(
        vectors_dir.exists(),
        "ERGO_COST_VECTORS_DIR does not exist: {:?}",
        vectors_dir
    );

    let mut total_compared = 0usize;
    let mut total_mismatches = 0usize;
    let mut ranges_tested = 0usize;

    // Auto-discover tx_costs_START_END.json files
    let mut cost_files: Vec<_> = std::fs::read_dir(&vectors_dir)
        .expect("read vectors dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("tx_costs_") && n.ends_with(".json")
        })
        .collect();
    cost_files.sort_by_key(|e| e.file_name());

    for entry in &cost_files {
        let name = entry.file_name().to_string_lossy().to_string();
        let range = name
            .trim_start_matches("tx_costs_")
            .trim_end_matches(".json");

        let tx_file = vectors_dir.join(format!("transactions_{}.json", range));
        if !tx_file.exists() {
            eprintln!("SKIP range {}: no transactions file", range);
            continue;
        }

        let parts: Vec<&str> = range.split('_').collect();
        if parts.len() != 2 {
            continue;
        }
        let start: u32 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let end: u32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let hdr_file = match find_header_file(&vectors_dir, start, end) {
            Some(f) => f,
            None => {
                eprintln!("SKIP range {}: no matching headers file", range);
                continue;
            }
        };

        eprintln!("\n--- Range: {} ---", range);
        let result = run_validate_parity_check_paths(&tx_file, &entry.path(), &hdr_file);
        eprintln!(
            "  matched={} mismatches={} skipped={} errors={} script_errors={}",
            result.compared, result.mismatches, result.skipped, result.errors, result.script_errors
        );

        assert_eq!(result.errors, 0, "range {}: parse errors", range);
        assert_eq!(result.script_errors, 0, "range {}: script errors", range);

        total_compared += result.compared;
        total_mismatches += result.mismatches;
        ranges_tested += 1;

        if result.mismatches > 0 {
            for d in &result.mismatch_details {
                eprintln!("  {}", d);
            }
        }
    }

    eprintln!("\n=== FULL CORPUS TOTALS ===");
    eprintln!("ranges tested:   {}", ranges_tested);
    eprintln!("total compared:  {}", total_compared);
    eprintln!("total mismatches: {}", total_mismatches);
    eprintln!("==========================");

    assert!(ranges_tested > 0, "no ranges found in {:?}", vectors_dir);
    assert!(
        total_compared >= 200,
        "not vacuous: only {} txs compared, expected >= 200",
        total_compared
    );
    assert_eq!(
        total_mismatches, 0,
        "full corpus: {} mismatches across {} ranges",
        total_mismatches, ranges_tested
    );
}

// ---------------------------------------------------------------------------
// Phase 2 end-to-end parity: uses TransactionContext::validate() directly
// ---------------------------------------------------------------------------

/// Runs parity check using the shipped validate() path instead of the
/// external pipeline. This confirms that the wired validator produces
/// the same block costs as the Scala reference.
fn run_validate_parity_check_paths(
    tx_path: &Path,
    cost_path: &Path,
    hdr_path: &Path,
) -> ParityResult {
    let tx_records: Vec<TxRecord> =
        serde_json::from_str(&std::fs::read_to_string(tx_path).expect("read tx json"))
            .expect("parse tx json");
    let cost_records: Vec<CostRecord> =
        serde_json::from_str(&std::fs::read_to_string(cost_path).expect("read cost json"))
            .expect("parse cost json");
    let hdr_records: Vec<HeaderRecord> =
        serde_json::from_str(&std::fs::read_to_string(hdr_path).expect("read header json"))
            .expect("parse header json");

    let cost_map: HashMap<String, u64> = cost_records
        .iter()
        .map(|c| (c.tx_id.clone(), c.block_cost))
        .collect();

    let mut header_map: HashMap<u32, Header> = HashMap::new();
    for rec in &hdr_records {
        let bytes = decode_hex(&rec.bytes);
        let header = Header::scorex_parse_bytes(&bytes).expect("parse header");
        header_map.insert(rec.height, header);
    }

    let mut utxo: HashMap<ergotree_ir::chain::ergo_box::BoxId, ErgoBox> = HashMap::new();
    let mut compared = 0usize;
    let mut mismatches = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;
    let mut script_errors = 0usize;
    let mut max_height: u32 = 0;
    let mut mismatch_details: Vec<String> = Vec::new();

    for rec in &tx_records {
        let height = rec.height;
        if height > max_height {
            max_height = height;
        }
        let tx_bytes = decode_hex(&rec.bytes);
        let tx: Transaction = match Transaction::sigma_parse_bytes(&tx_bytes) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("PARSE ERROR for {}: {:?}", rec.id, e);
                errors += 1;
                continue;
            }
        };
        let tx_id_hex = rec.id.clone();

        // Snapshot + advance UTXO (same as external pipeline)
        let input_boxes: Vec<Option<ErgoBox>> = tx
            .inputs
            .iter()
            .map(|inp| utxo.get(&inp.box_id).cloned())
            .collect();
        let data_boxes: Vec<Option<ErgoBox>> =
            tx.data_inputs.as_ref().map_or_else(Vec::new, |dis| {
                dis.iter().map(|d| utxo.get(&d.box_id).cloned()).collect()
            });

        for inp in tx.inputs.iter() {
            utxo.remove(&inp.box_id);
        }
        for out in tx.outputs.iter() {
            utxo.insert(out.box_id(), out.clone());
        }

        if input_boxes.iter().any(|b| b.is_none()) {
            skipped += 1;
            continue;
        }
        let n_data = tx.data_inputs.as_ref().map_or(0, |d| d.len());
        if data_boxes.len() != n_data || data_boxes.iter().any(|b| b.is_none()) {
            skipped += 1;
            continue;
        }

        let can_build_headers =
            (1..=10).all(|i| height >= i && header_map.contains_key(&(height - i)));
        if !can_build_headers || !header_map.contains_key(&height) {
            skipped += 1;
            continue;
        }

        let current_header = header_map[&height].clone();
        let headers_10: [Header; 10] =
            std::array::from_fn(|i| header_map[&(height - 1 - i as u32)].clone());
        let pre_header = PreHeader::from(current_header);
        let state_ctx = ErgoStateContext::new(pre_header, headers_10, Parameters::default());

        let resolved_spending: Vec<ErgoBox> = input_boxes.into_iter().flatten().collect();
        let resolved_data: Vec<ErgoBox> = data_boxes.into_iter().flatten().collect();

        let tx_ctx = match TransactionContext::new(tx.clone(), resolved_spending, resolved_data) {
            Ok(tc) => tc,
            Err(e) => {
                eprintln!("ERROR: TransactionContext for {}: {:?}", tx_id_hex, e);
                errors += 1;
                continue;
            }
        };

        // THE KEY DIFFERENCE: call validate() directly
        let block_cost = match tx_ctx.validate(&state_ctx) {
            Ok(cost) => cost,
            Err(e) => {
                eprintln!("VALIDATE ERROR for {}: {:?}", tx_id_hex, e);
                script_errors += 1;
                continue;
            }
        };

        if let Some(&scala_cost) = cost_map.get(&tx_id_hex) {
            if block_cost != scala_cost {
                mismatches += 1;
                let detail = format!(
                    "VALIDATE_MISMATCH tx={} height={} rust={} scala={} diff={}",
                    tx_id_hex,
                    height,
                    block_cost,
                    scala_cost,
                    block_cost as i64 - scala_cost as i64
                );
                eprintln!("{}", detail);
                mismatch_details.push(detail);
            } else {
                compared += 1;
            }
        }
    }

    eprintln!("\n=== VALIDATE PARITY SUMMARY ===");
    eprintln!("compared (matched): {}", compared);
    eprintln!("mismatches:         {}", mismatches);
    eprintln!("skipped:            {}", skipped);
    eprintln!("errors:             {}", errors);
    eprintln!("script errors:      {}", script_errors);
    eprintln!("max height:         {}", max_height);
    eprintln!("===============================\n");

    ParityResult {
        compared,
        mismatches,
        skipped,
        errors,
        script_errors,
        max_height,
        mismatch_details,
    }
}

fn run_validate_parity_check(vectors_dir: &Path) -> ParityResult {
    run_validate_parity_check_paths(
        &vectors_dir.join("transactions_700000_700050.json"),
        &vectors_dir.join("tx_costs_700000_700050.json"),
        &vectors_dir.join("headers_700000_700060.json"),
    )
}

/// End-to-end parity: validate() produces correct block costs.
#[test]
fn smoke_validate_parity() {
    let vectors_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test-vectors");
    assert!(vectors_dir.exists());

    let result = run_validate_parity_check(&vectors_dir);

    assert_eq!(result.errors, 0, "no parse failures");
    assert_eq!(result.script_errors, 0, "no validate failures");
    assert!(
        result.compared + result.mismatches >= 50,
        "not vacuous: {} txs",
        result.compared + result.mismatches
    );
    assert!(result.max_height < STORAGE_RENT_PERIOD);
    assert_eq!(
        result.mismatches,
        0,
        "validate() parity: {} mismatches\n{}",
        result.mismatches,
        result.mismatch_details.join("\n")
    );
}
