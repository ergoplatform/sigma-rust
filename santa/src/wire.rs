//! Wire-tier round-trip: parse a `kind`'s canonical bytes with sigma-rust's serializer and
//! reserialize for byte-identity comparison downstream. The wire analog of [`crate::eval::run_entry`]
//! — a single round-trip outcome, no value/cost (docs/specs/wire-tier.md). A parse/serialize failure
//! is `errored` (sigma-rust rejected bytes the reference accepted — a real divergence); a `kind` with no
//! sigma serializer wired here is `not-implemented`.

use ergo_lib::chain::transaction::Transaction;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::serialization::sigma_byte_reader;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;

/// One wire entry's outcome — the round-trip analog of [`crate::eval::Outcome`]: `bytes_hex`
/// replaces value+cost (the wire tier has no cost dimension).
pub enum WireOutcome {
    RoundTrip { bytes_hex: String },
    Errored,
    NotImplemented,
    Panicked { note: String },
}

impl WireOutcome {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            WireOutcome::RoundTrip { bytes_hex } => {
                serde_json::json!({"bytes_hex": bytes_hex, "error": serde_json::Value::Null})
            }
            WireOutcome::Errored => serde_json::json!({"bytes_hex": null, "error": "errored"}),
            WireOutcome::NotImplemented => {
                serde_json::json!({"bytes_hex": null, "error": "not-implemented"})
            }
            WireOutcome::Panicked { note } => {
                serde_json::json!({"bytes_hex": null, "error": "panicked", "note": note})
            }
        }
    }
}

/// Round-trip `bytes` through `T`'s sigma serializer: parse then reserialize. A parse or serialize
/// failure is `errored` — sigma-rust couldn't reproduce bytes the reference accepted (a real divergence).
fn roundtrip<T: SigmaSerializable>(bytes: &[u8]) -> WireOutcome {
    match T::sigma_parse_bytes(bytes) {
        Ok(obj) => match obj.sigma_serialize_bytes() {
            Ok(out) => WireOutcome::RoundTrip {
                bytes_hex: bytes_to_hex(&out),
            },
            Err(_) => WireOutcome::Errored,
        },
        Err(_) => WireOutcome::Errored,
    }
}

/// ErgoTree round-trip through `sigma_parse_bytes_lenient`, the arbitrary-root parse (the JVM's `checkType = false`
/// reader). The wire vectors carry trees whose root is not a SigmaProp (an Int, a type-variable witness), which the
/// strict `sigma_parse_bytes` keeps as unparsed bytes (an echo of the input) instead of reading; the lenient parse
/// reads them to structure, so reserializing re-encodes them: the structural round-trip the ErgoTree kind asks for
/// (runner-contract-wire §5).
fn roundtrip_ergotree(bytes: &[u8]) -> WireOutcome {
    match ErgoTree::sigma_parse_bytes_lenient(bytes) {
        Ok(tree) => match tree.sigma_serialize_bytes() {
            Ok(out) => WireOutcome::RoundTrip {
                bytes_hex: bytes_to_hex(&out),
            },
            Err(_) => WireOutcome::Errored,
        },
        Err(_) => WireOutcome::Errored,
    }
}

/// Round-trip one wire entry. `kind` selects the serializer. Header isn't wired here — it parses
/// via ScorexSerializable, not SigmaSerializable → not-implemented.
pub fn run_entry(kind: &str, bytes_hex: &str) -> WireOutcome {
    let bytes = match crate::hex_to_bytes(bytes_hex) {
        Ok(b) => b,
        Err(e) => {
            return WireOutcome::Panicked {
                note: format!("bad bytes_hex: {e}"),
            }
        }
    };
    match kind {
        "Box" => roundtrip::<ErgoBox>(&bytes),
        "Constant" => roundtrip::<Constant>(&bytes),
        "SigmaBoolean" => roundtrip::<SigmaBoolean>(&bytes),
        "Transaction" => roundtrip::<Transaction>(&bytes),
        "BlockTransactions" => roundtrip_block_transactions(&bytes),
        "ErgoTree" => roundtrip_ergotree(&bytes),
        _ => WireOutcome::NotImplemented,
    }
}

/// A block's transactions section, parsed the way ergo-node-rust parses one (`validation/src/sections.rs`
/// `parse_block_transactions`). The section is the 32-byte header id, then a VLQ that is either the tx count or
/// `10_000_000 + block version` (followed by the count), then the transactions. Every transaction is read by
/// `Transaction::sigma_parse` from ONE reader over the rest, so sigma-rust decides what reader state a transaction
/// starts with; the JVM starts each on a fresh reader (`ErgoTransactionSerializer.parse`). Reserialized with the same
/// framing.
fn roundtrip_block_transactions(bytes: &[u8]) -> WireOutcome {
    match block_transactions_reserialized(bytes) {
        Some(out) => WireOutcome::RoundTrip {
            bytes_hex: bytes_to_hex(&out),
        },
        None => WireOutcome::Errored,
    }
}

fn block_transactions_reserialized(bytes: &[u8]) -> Option<Vec<u8>> {
    const BLOCK_VERSION_SENTINEL: u32 = 10_000_000;
    let header_id = bytes.get(..32)?;
    let mut pos = 32;
    let ver_or_count = read_vlq_u32(bytes, &mut pos)?;
    let (version_marker, count) = if ver_or_count > BLOCK_VERSION_SENTINEL {
        (Some(ver_or_count), read_vlq_u32(bytes, &mut pos)?)
    } else {
        (None, ver_or_count)
    };
    let mut r = sigma_byte_reader::from_bytes(&bytes[pos..]);
    let mut out = header_id.to_vec();
    if let Some(marker) = version_marker {
        write_vlq_u32(marker, &mut out);
    }
    write_vlq_u32(count, &mut out);
    for _ in 0..count {
        let tx = Transaction::sigma_parse(&mut r).ok()?;
        out.extend(tx.sigma_serialize_bytes().ok()?);
    }
    Some(out)
}

/// Unsigned LEB128: the JVM's `getUInt` / `putUInt` section framing.
fn read_vlq_u32(bytes: &[u8], pos: &mut usize) -> Option<u32> {
    let mut value: u64 = 0;
    for shift in (0..35).step_by(7) {
        let b = *bytes.get(*pos)?;
        *pos += 1;
        value |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return u32::try_from(value).ok();
        }
    }
    None
}

fn write_vlq_u32(mut v: u32, out: &mut Vec<u8>) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::run_entry;
    use serde_json::Value as J;

    /// SANTA `wire/v6/authored/BlockTransactions.reader_scope` #3: a block section (header id, the version-4
    /// marker `84 ad e2 04`, count 2) holding two txs whose output trees each define ValDef(1) and use it. Shallow on
    /// purpose: a debug build takes tens of KB of stack per expression level, so #1's 109-deep tree would overflow
    /// the 2 MB test thread (the release runner parses it fine).
    const TWO_TX_SECTION: &str = "bfbc8c5cb61b59dd3cd2918450c7272600312b3b8c929808b09e76253d1bfecf84ade2040201456971d1f52b5bafce6e5c94848a0892d936cfe52a65d6a75d48d1fcc586f9f60000000001c0843d00d801d60108d37201010000015dea0e5b89dbbc08eb741989221dda7428575bb695d2a75bb15e434e379af43b0000000001c0843d00d801d60108d37201010000";

    #[test]
    fn block_transactions_round_trips_the_section_framing() {
        let j = run_entry("BlockTransactions", TWO_TX_SECTION).to_json();
        assert_eq!(j["error"], J::Null);
        assert_eq!(j["bytes_hex"], TWO_TX_SECTION);
    }

    #[test]
    fn block_transactions_rejects_a_truncated_section() {
        // the header id alone: no version marker, no count
        let j = run_entry("BlockTransactions", &TWO_TX_SECTION[..64]).to_json();
        assert_eq!(j["error"], "errored");
    }

    #[test]
    fn box_round_trips_to_its_own_bytes() {
        // sbox_minimal from vectors/wire/v5/authored/Box.json
        let hex = "c0843d09020101000000000000000000000000000000000000000000000000000000000000000000000000";
        let j = run_entry("Box", hex).to_json();
        assert_eq!(j["error"], J::Null);
        assert_eq!(j["bytes_hex"], hex);
    }

    #[test]
    fn sigma_boolean_round_trips_to_its_own_bytes() {
        let hex = "d3"; // TrivialProp(true) from vectors/wire/v5/authored/SigmaBoolean.json
        let j = run_entry("SigmaBoolean", hex).to_json();
        assert_eq!(j["error"], J::Null);
        assert_eq!(j["bytes_hex"], hex);
    }

    #[test]
    fn constant_round_trips_to_its_own_bytes() {
        // bool_0 from vectors/wire/v5/vendored/Constant.json (Fleet)
        let hex = "0101";
        let j = run_entry("Constant", hex).to_json();
        assert_eq!(j["error"], J::Null);
        assert_eq!(j["bytes_hex"], hex);
    }

    #[test]
    fn transaction_round_trips_to_its_own_bytes() {
        // tx_466f1aef from vectors/wire/v5/vendored/Transaction.json (Fleet signed tx)
        let hex = "010c7a0f145994fa15b02eca8189b454eeff9eec5ca33fa135b4474d4ee8ed35c70000000220fa2bf23962cdf51b07722d6237c0c7b8a44f78856c0f7ec308dc1ef1a92a51d9a2cc8a09abfaed87afacfbb7daee79a6b26f10c6613fc13d3f3953e5521d1a0280cc9497e9d1870f101004020e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a7017300730110010204020404040004c0fd4f05808c82f5f6030580b8c9e5ae040580f882ad16040204c0944004c0f407040004000580f882ad16d19683030191a38cc7a7019683020193c2b2a57300007473017302830108cdeeac93a38cc7b2a573030001978302019683040193b1a5730493c2a7c2b2a573050093958fa3730673079973089c73097e9a730a9d99a3730b730c0599c1a7c1b2a5730d00938cc7b2a5730e0001a390c1a7730fb3825c0200010180b0abe9c1c7fa1300809ccdca64100204a00b08cd0274e729bb6615cbda94d9d176a2f1525068f12b330e38bbbf387232797dfd891fea02d192a39a8cc7a70173007301b3825c010180f085da2c00";
        let j = run_entry("Transaction", hex).to_json();
        assert_eq!(j["error"], J::Null);
        assert_eq!(j["bytes_hex"], hex);
    }

    /// An ErgoTree whose root is not a SigmaProp (an Int constant) round-trips through the lenient parse: the
    /// arbitrary-root parse the JVM's `checkType = false` reader is, which the wire ErgoTree kind needs (the strict
    /// parse refuses or keeps such a tree as bytes). Header v0, no size: `00`, then the root `04 02` (Int 1).
    #[test]
    fn ergotree_with_a_non_sigmaprop_root_round_trips() {
        let j = run_entry("ErgoTree", "000402").to_json();
        assert_eq!(j["error"], J::Null);
        assert_eq!(j["bytes_hex"], "000402");
    }

    #[test]
    fn ergotree_that_does_not_parse_is_errored() {
        // header v0, then an opcode with no node behind it
        let j = run_entry("ErgoTree", "00ff").to_json();
        assert_eq!(j["error"], "errored");
    }

    #[test]
    fn unwired_kind_is_not_implemented() {
        // Header still parses via ScorexSerializable, not SigmaSerializable -> not wired here.
        let j = run_entry("Header", "00").to_json();
        assert_eq!(j["error"], "not-implemented");
        assert_eq!(j["bytes_hex"], J::Null);
    }
}
