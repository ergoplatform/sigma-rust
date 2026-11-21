//! SValue ⇄ canonical SANTA JSON bridge.
//!
//! Implements the encoding pinned in `santa/docs/contract/runner-contract.md` §4.
//! Two directions:
//!   - [`encode_value`]: a sigma-rust eval-result `Value` → canonical SValue JSON.
//!   - [`decode_constant`]: a canonical SValue JSON (an entry's `input`) → a
//!     sigma-rust `Constant`, to bind at ContextExtension var 1.
//!
//! Frictions (per the contract): Long/BigInt/UnsignedBigInt are decimal **strings**;
//! GroupElement/SigmaProp/Box/Header/AvlTree are **lower-case** hex; cost is handled by the
//! caller (it is not an SValue). Kinds with no canonical encoding (Unit,
//! Context, PreHeader, Global, Lambda, String) surface as [`BridgeError::Encode`], which the
//! runner records faithfully as a `panicked` note (runner contract §3) — a SANTA-side gap to wire,
//! never an "excuse" outcome.

use ergotree_ir::mir::constant::{Constant, Literal};
use ergotree_ir::mir::value::{CollKind, Value};
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::types::stype::SType;
use serde_json::{json, Value as J};

/// A bridge failure. The payload strings are diagnostic context, read only via
/// `Debug` (e.g. a test `.expect`); they are intentionally NOT surfaced in the
/// contract actuals (runner contract §7 defers the error-reason taxonomy), so the dead-code lint
/// reads the fields as unused in a non-test build.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BridgeError {
    /// The bridge cannot encode this sigma-rust Value to canonical JSON (a SANTA-side gap;
    /// surfaced to the runner as a `panicked` note, not a contract outcome of its own).
    Encode(String),
    /// Malformed SANTA JSON on the decode side.
    Decode(String),
    /// The underlying library REFUSED structurally-valid input bytes — its own
    /// parse/`try_from` verdict on material the reference accepted (e.g. a Box value outside
    /// the impl's bounds). The library speaking, not the bridge: runners surface this
    /// as `errored` (a real divergence on an accept vector), never `panicked`.
    Refused(String),
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, BridgeError> {
    if !s.len().is_multiple_of(2) {
        return Err(BridgeError::Decode(format!("odd-length hex: {}", s)));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| BridgeError::Decode(format!("bad hex {}: {}", s, e)))
        })
        .collect()
}

fn ser_bytes<T: SigmaSerializable>(t: &T, what: &str) -> Result<String, BridgeError> {
    t.sigma_serialize_bytes()
        .map(|b| hex_lower(&b))
        .map_err(|e| BridgeError::Encode(format!("{} serialize: {:?}", what, e)))
}

// ---------------------------------------------------------------------------
// SType bridge (the `{tag}` forms used in Coll.elem and nested positions)
// ---------------------------------------------------------------------------

pub fn encode_stype(t: &SType) -> J {
    match t {
        SType::SAny => json!({"tag": "SAny"}),
        SType::SUnit => json!({"tag": "SUnit"}),
        SType::SBoolean => json!({"tag": "SBoolean"}),
        SType::SByte => json!({"tag": "SByte"}),
        SType::SShort => json!({"tag": "SShort"}),
        SType::SInt => json!({"tag": "SInt"}),
        SType::SLong => json!({"tag": "SLong"}),
        SType::SBigInt => json!({"tag": "SBigInt"}),
        SType::SUnsignedBigInt => json!({"tag": "SUnsignedBigInt"}),
        SType::SGroupElement => json!({"tag": "SGroupElement"}),
        SType::SSigmaProp => json!({"tag": "SSigmaProp"}),
        SType::SBox => json!({"tag": "SBox"}),
        SType::SHeader => json!({"tag": "SHeader"}),
        SType::SPreHeader => json!({"tag": "SPreHeader"}),
        SType::SAvlTree => json!({"tag": "SAvlTree"}),
        SType::SColl(elem) => json!({"tag": "SColl", "elem": encode_stype(elem)}),
        SType::SOption(elem) => json!({"tag": "SOption", "elem": encode_stype(elem)}),
        SType::STuple(stuple) => {
            let items: Vec<J> = stuple.items.iter().map(encode_stype).collect();
            json!({"tag": "STuple", "items": items})
        }
        other => json!({"tag": format!("{:?}", other)}),
    }
}

pub fn decode_stype(j: &J) -> Result<SType, BridgeError> {
    let tag = j["tag"]
        .as_str()
        .ok_or_else(|| BridgeError::Decode(format!("SType missing tag: {}", j)))?;
    Ok(match tag {
        "SAny" => SType::SAny,
        "SUnit" => SType::SUnit,
        "SBoolean" => SType::SBoolean,
        "SByte" => SType::SByte,
        "SShort" => SType::SShort,
        "SInt" => SType::SInt,
        "SLong" => SType::SLong,
        "SBigInt" => SType::SBigInt,
        "SUnsignedBigInt" => SType::SUnsignedBigInt,
        "SGroupElement" => SType::SGroupElement,
        "SSigmaProp" => SType::SSigmaProp,
        "SBox" => SType::SBox,
        "SHeader" => SType::SHeader,
        "SPreHeader" => SType::SPreHeader,
        "SAvlTree" => SType::SAvlTree,
        "SColl" => SType::SColl(Box::new(decode_stype(&j["elem"])?).into()),
        "SOption" => SType::SOption(Box::new(decode_stype(&j["elem"])?).into()),
        "STuple" => {
            use ergotree_ir::types::stuple::STuple;
            let items_json = j["items"]
                .as_array()
                .ok_or_else(|| BridgeError::Decode(format!("STuple items not an array: {}", j)))?;
            let items: Vec<SType> = items_json
                .iter()
                .map(decode_stype)
                .collect::<Result<_, _>>()?;
            SType::STuple(
                STuple::try_from(items)
                    .map_err(|e| BridgeError::Decode(format!("STuple arity: {:?}", e)))?,
            )
        }
        other => {
            return Err(BridgeError::Decode(format!(
                "unsupported SType tag: {}",
                other
            )))
        }
    })
}

// ---------------------------------------------------------------------------
// Value → canonical JSON
// ---------------------------------------------------------------------------

pub fn encode_value(v: &Value) -> Result<J, BridgeError> {
    Ok(match v {
        Value::Boolean(b) => json!({"kind": "Boolean", "value": b}),
        Value::Byte(x) => json!({"kind": "Byte", "value": x}),
        Value::Short(x) => json!({"kind": "Short", "value": x}),
        Value::Int(x) => json!({"kind": "Int", "value": x}),
        Value::Long(x) => json!({"kind": "Long", "value": x.to_string()}),
        Value::BigInt(x) => json!({"kind": "BigInt", "value": x.to_string()}),
        Value::UnsignedBigInt(x) => json!({"kind": "UnsignedBigInt", "value": x.to_string()}),
        Value::GroupElement(ge) => {
            json!({"kind": "GroupElement", "bytes_hex": ser_bytes(&**ge, "GroupElement")?})
        }
        Value::SigmaProp(sp) => {
            // Canonical encoding (runner contract §4) is the bare serialized `SigmaBoolean`,
            // NOT `prop_bytes()` — which wraps it in an ErgoTree (spurious `0008` header).
            json!({"kind": "SigmaProp", "raw_hex": ser_bytes(sp.value(), "SigmaProp")?})
        }
        Value::CBox(b) => json!({"kind": "Box", "bytes_hex": ser_bytes(&**b, "Box")?}),
        Value::AvlTree(a) => json!({"kind": "AvlTree", "bytes_hex": ser_bytes(&**a, "AvlTree")?}),
        Value::Header(h) => {
            // Mirror of the decode arm: Header rides Scorex serialization (not
            // SigmaSerializable) — {kind:"Header", bytes_hex} like the JVM encoder.
            use sigma_ser::ScorexSerializable;
            let bytes = h
                .scorex_serialize_bytes()
                .map_err(|e| BridgeError::Encode(format!("Header serialize: {:?}", e)))?;
            json!({"kind": "Header", "bytes_hex": hex_lower(&bytes)})
        }
        Value::Coll(coll) => encode_coll(coll)?,
        Value::Tup(items) => {
            let arr: Result<Vec<J>, _> = items.iter().map(encode_value).collect();
            json!({"kind": "Tuple", "items": arr?})
        }
        Value::Opt(opt) => match opt {
            Some(inner) => json!({"kind": "Option", "value": encode_value(inner)?}),
            None => json!({"kind": "Option", "value": J::Null}),
        },
        other => {
            return Err(BridgeError::Encode(format!(
                "no canonical SValue encoding for {:?}",
                core::mem::discriminant(other)
            )))
        }
    })
}

// A runtime collection, whatever class it is held as: its item type and its items, the same JSON
// for every form. The canonical encoding (runner contract §4) is representation-BLIND by design — a
// specialised and a generic collection of the same items must encode identically, or the runner
// would leak sigma-rust's internal representation into the actuals and false-red the corpus.
// Written against the accessors rather than the variants so the representation can grow a case
// (the shape of the sigma-rust session's forthcoming `CollValue`) without touching this bridge.
fn encode_coll(coll: &CollKind<Value>) -> Result<J, BridgeError> {
    let arr: Result<Vec<J>, _> = coll.as_vec().iter().map(encode_value).collect();
    Ok(json!({"kind": "Coll", "elem": encode_stype(coll.elem_tpe()), "items": arr?}))
}

// ---------------------------------------------------------------------------
// canonical JSON → Constant  (for binding an entry's `input` at ext var 1)
// ---------------------------------------------------------------------------

pub fn decode_constant(j: &J) -> Result<Constant, BridgeError> {
    let kind = j["kind"]
        .as_str()
        .ok_or_else(|| BridgeError::Decode(format!("SValue missing kind: {}", j)))?;
    let num_i64 = |label: &str| -> Result<i64, BridgeError> {
        j["value"]
            .as_i64()
            .ok_or_else(|| BridgeError::Decode(format!("{} value not an int: {}", label, j)))
    };
    let str_val = |label: &str| -> Result<String, BridgeError> {
        j["value"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| BridgeError::Decode(format!("{} value not a string: {}", label, j)))
    };
    let hex_field = |field: &str| -> Result<Vec<u8>, BridgeError> {
        let s = j[field]
            .as_str()
            .ok_or_else(|| BridgeError::Decode(format!("missing {}: {}", field, j)))?;
        hex_decode(s)
    };

    Ok(match kind {
        "Boolean" => {
            lit(
                SType::SBoolean,
                Literal::Boolean(j["value"].as_bool().ok_or_else(|| {
                    BridgeError::Decode(format!("Boolean value not a bool: {}", j))
                })?),
            )
        }
        "Byte" => lit(SType::SByte, Literal::Byte(num_i64("Byte")? as i8)),
        "Short" => lit(SType::SShort, Literal::Short(num_i64("Short")? as i16)),
        "Int" => lit(SType::SInt, Literal::Int(num_i64("Int")? as i32)),
        "Long" => {
            let n: i64 = str_val("Long")?
                .parse()
                .map_err(|e| BridgeError::Decode(format!("Long parse: {:?}", e)))?;
            lit(SType::SLong, Literal::Long(n))
        }
        "BigInt" => {
            use core::str::FromStr;
            let n = ergotree_ir::bigint256::BigInt256::from_str(&str_val("BigInt")?)
                .map_err(|e| BridgeError::Decode(format!("BigInt parse: {:?}", e)))?;
            lit(SType::SBigInt, Literal::BigInt(n))
        }
        "UnsignedBigInt" => {
            use core::str::FromStr;
            // Decimal string (runner contract §4), like Long/BigInt. Parse straight to
            // UnsignedBigInt — NOT via BigInt256, which would reject values in
            // the 2^255..2^256 range that are valid unsigned.
            let n = ergotree_ir::unsignedbigint256::UnsignedBigInt::from_str(&str_val(
                "UnsignedBigInt",
            )?)
            .map_err(|e| BridgeError::Decode(format!("UnsignedBigInt parse: {:?}", e)))?;
            lit(SType::SUnsignedBigInt, Literal::UnsignedBigInt(n))
        }
        "GroupElement" => {
            use ergo_chain_types::EcPoint;
            let ge = EcPoint::sigma_parse_bytes(&hex_field("bytes_hex")?)
                .map_err(|e| BridgeError::Refused(format!("GroupElement parse: {:?}", e)))?;
            lit(SType::SGroupElement, Literal::GroupElement(alloc_arc(ge)))
        }
        "Box" => {
            use ergotree_ir::chain::ergo_box::ErgoBox;
            let b = ErgoBox::sigma_parse_bytes(&hex_field("bytes_hex")?)
                .map_err(|e| BridgeError::Refused(format!("Box parse: {:?}", e)))?;
            lit(
                SType::SBox,
                Literal::CBox(ergotree_ir::reference::Ref::from(b)),
            )
        }
        "AvlTree" => {
            use ergotree_ir::mir::avl_tree_data::AvlTreeData;
            let a = AvlTreeData::sigma_parse_bytes(&hex_field("bytes_hex")?)
                .map_err(|e| BridgeError::Refused(format!("AvlTree parse: {:?}", e)))?;
            lit(SType::SAvlTree, Literal::AvlTree(Box::new(a)))
        }
        "Header" => {
            // Header serializes via Scorex (not SigmaSerializable); parse the input bytes the
            // same way sigma-rust's DataSerializer does (Header::scorex_parse).
            use ergo_chain_types::Header;
            use sigma_ser::ScorexSerializable;
            let h = Header::scorex_parse_bytes(&hex_field("bytes_hex")?)
                .map_err(|e| BridgeError::Refused(format!("Header parse: {:?}", e)))?;
            lit(SType::SHeader, Literal::Header(Box::new(h)))
        }
        "Coll" => {
            let elem = decode_stype(&j["elem"])?;
            let items_json = j["items"]
                .as_array()
                .ok_or_else(|| BridgeError::Decode(format!("Coll items not an array: {}", j)))?;
            let items: Vec<Constant> = items_json
                .iter()
                .map(decode_constant)
                .collect::<Result<_, _>>()?;
            let lits: Vec<Literal> = items.into_iter().map(|c| c.v).collect();
            let coll = CollKind::from_collection(elem.clone(), lits)
                .map_err(|e| BridgeError::Decode(format!("Coll build: {:?}", e)))?;
            lit(SType::SColl(Box::new(elem).into()), Literal::Coll(coll))
        }
        // Compact byte-collection form (runner contract §2): value_hex, semantically
        // identical to Coll/SByte per-item — exists for large byte payloads (the SBox
        // token-window family carries >4KB box bytes as context input).
        "Coll[Byte]" => {
            let bytes = hex_field("value_hex")?;
            let coll = CollKind::from_collection(
                SType::SByte,
                bytes
                    .iter()
                    .map(|b| Literal::Byte(*b as i8))
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| BridgeError::Decode(format!("Coll[Byte] build: {:?}", e)))?;
            lit(
                SType::SColl(Box::new(SType::SByte).into()),
                Literal::Coll(coll),
            )
        }
        "Tuple" => {
            let items_json = j["items"]
                .as_array()
                .ok_or_else(|| BridgeError::Decode(format!("Tuple items not an array: {}", j)))?;
            let consts: Vec<Constant> = items_json
                .iter()
                .map(decode_constant)
                .collect::<Result<_, _>>()?;
            use ergotree_ir::types::stuple::STuple;
            use ergotree_ir::types::stuple::TupleItems;
            let tpes: Vec<SType> = consts.iter().map(|c| c.tpe.clone()).collect();
            let lits: Vec<Literal> = consts.into_iter().map(|c| c.v).collect();
            let lit_items = TupleItems::try_from(lits)
                .map_err(|e| BridgeError::Decode(format!("Tuple arity: {:?}", e)))?;
            let tpe_items = TupleItems::try_from(tpes)
                .map_err(|e| BridgeError::Decode(format!("Tuple type arity: {:?}", e)))?;
            lit(
                SType::STuple(STuple { items: tpe_items }),
                Literal::Tup(lit_items),
            )
        }
        "Option" => {
            if j["value"].is_null() {
                return Err(BridgeError::Decode(
                    "cannot infer element type of Option.None input".to_string(),
                ));
            }
            let inner = decode_constant(&j["value"])?;
            lit(
                SType::SOption(Box::new(inner.tpe.clone()).into()),
                Literal::Opt(Some(Box::new(inner.v))),
            )
        }
        "SigmaProp" => {
            use ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProp};
            // Mirror of `encode_value`: `raw_hex` is the bare serialized `SigmaBoolean`.
            let sb = SigmaBoolean::sigma_parse_bytes(&hex_field("raw_hex")?)
                .map_err(|e| BridgeError::Refused(format!("SigmaProp parse: {:?}", e)))?;
            lit(
                SType::SSigmaProp,
                Literal::SigmaProp(Box::new(SigmaProp::new(sb))),
            )
        }
        other => {
            return Err(BridgeError::Decode(format!(
                "unsupported input SValue kind: {}",
                other
            )))
        }
    })
}

fn lit(tpe: SType, v: Literal) -> Constant {
    Constant { tpe, v }
}

fn alloc_arc(ge: ergo_chain_types::EcPoint) -> alloc_sync::Arc<ergo_chain_types::EcPoint> {
    alloc_sync::Arc::new(ge)
}

mod alloc_sync {
    pub use std::sync::Arc;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip a value-preserving SValue: JSON → Constant → Value → JSON.
    fn roundtrip(j: J) {
        let c = decode_constant(&j).expect("decode");
        let v = Value::from(c.v);
        let back = encode_value(&v).expect("encode");
        assert_eq!(back, j, "round-trip mismatch");
    }

    #[test]
    fn rt_bool() {
        roundtrip(json!({"kind": "Boolean", "value": true}));
        roundtrip(json!({"kind": "Boolean", "value": false}));
    }

    #[test]
    fn rt_byte_short_int() {
        roundtrip(json!({"kind": "Byte", "value": -128}));
        roundtrip(json!({"kind": "Short", "value": 32767}));
        roundtrip(json!({"kind": "Int", "value": -2147483648i64}));
    }

    #[test]
    fn rt_long_is_string() {
        roundtrip(json!({"kind": "Long", "value": "9223372036854775807"}));
        roundtrip(json!({"kind": "Long", "value": "-9223372036854775808"}));
    }

    #[test]
    fn rt_bigint_is_string() {
        roundtrip(json!({"kind": "BigInt", "value": "-45"}));
        roundtrip(json!({"kind": "BigInt", "value": "123456789012345678901234567890"}));
    }

    #[test]
    fn rt_unsigned_bigint_is_string() {
        roundtrip(json!({"kind": "UnsignedBigInt", "value": "0"}));
        roundtrip(json!({"kind": "UnsignedBigInt", "value": "123456789012345678901234567890"}));
        // 2^256 - 1: above the signed BigInt256 ceiling, so this fails if the
        // decode routes through BigInt256 instead of parsing unsigned directly.
        roundtrip(json!({
            "kind": "UnsignedBigInt",
            "value": "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        }));
    }

    #[test]
    fn rt_coll_bytes() {
        roundtrip(json!({
            "kind": "Coll",
            "elem": {"tag": "SByte"},
            "items": [
                {"kind": "Byte", "value": 0},
                {"kind": "Byte", "value": 8},
                {"kind": "Byte", "value": -45}
            ]
        }));
    }

    #[test]
    fn rt_coll_int() {
        roundtrip(json!({
            "kind": "Coll",
            "elem": {"tag": "SInt"},
            "items": [{"kind": "Int", "value": 1}, {"kind": "Int", "value": 2}]
        }));
    }

    // The three forms below pin that the canonical encoding is representation-BLIND, which is
    // what `encode_coll` going through the accessors has to preserve: an empty collection still
    // reports its item type (it has no items to infer one from), a nested one encodes its inner
    // collections as collections (not flattened — `from_vec_vec`'s concatenation is flatMap's, not
    // the bridge's), and a collection of pairs keeps the pair shape the specialisation census made
    // load-bearing (a `PairOfCols` and a plain collection over Tuple2 must encode alike).
    #[test]
    fn rt_coll_bytes_empty() {
        roundtrip(json!({"kind": "Coll", "elem": {"tag": "SByte"}, "items": []}));
    }

    #[test]
    fn rt_coll_nested() {
        roundtrip(json!({
            "kind": "Coll",
            "elem": {"tag": "SColl", "elem": {"tag": "SByte"}},
            "items": [
                {"kind": "Coll", "elem": {"tag": "SByte"}, "items": [{"kind": "Byte", "value": 1}]},
                {"kind": "Coll", "elem": {"tag": "SByte"}, "items": []}
            ]
        }));
    }

    #[test]
    fn rt_coll_of_pairs() {
        roundtrip(json!({
            "kind": "Coll",
            "elem": {"tag": "STuple", "items": [{"tag": "SInt"}, {"tag": "SLong"}]},
            "items": [
                {"kind": "Tuple", "items": [
                    {"kind": "Int", "value": 1}, {"kind": "Long", "value": "2"}
                ]},
                {"kind": "Tuple", "items": [
                    {"kind": "Int", "value": -3}, {"kind": "Long", "value": "-4"}
                ]}
            ]
        }));
    }

    #[test]
    fn rt_tuple() {
        roundtrip(json!({
            "kind": "Tuple",
            "items": [
                {"kind": "Coll", "elem": {"tag": "SByte"}, "items": [{"kind": "Byte", "value": 1}]},
                {"kind": "Int", "value": 0}
            ]
        }));
    }

    #[test]
    fn rt_option_some() {
        roundtrip(json!({"kind": "Option", "value": {"kind": "Int", "value": 2}}));
    }

    #[test]
    fn rt_sigmaprop_trivial() {
        // `d2`/`d3` are the bare serialized SigmaBoolean for sigma false/true (TrivialProp).
        roundtrip(json!({"kind": "SigmaProp", "raw_hex": "d2"}));
        roundtrip(json!({"kind": "SigmaProp", "raw_hex": "d3"}));
    }

    #[test]
    fn rt_sigmaprop_provedlog() {
        // A real ProveDlog (bare SigmaBoolean), from the blessed proveDlog_equivalence vector.
        roundtrip(json!({"kind": "SigmaProp",
            "raw_hex": "cd02288f0e55610c3355c89ed6c5de43cf20da145b8c54f03a29f481e540d94e9a69"}));
    }

    #[test]
    fn rt_avl_tree() {
        // A real AvlTreeData (serialized), from the blessed
        // AvlTree_properties_equivalence vector (entry #0 input).
        roundtrip(json!({"kind": "AvlTree",
            "bytes_hex": "000183807f66b301530120ff7fc6bd6601ff01ff7f7d2bedbbffff00187fe8909406010101"}));
    }

    #[test]
    fn stype_tag_map_total_over_contract_s4() {
        // Every leaf tag in runner-contract §4 maps to its EXACT string both ways
        // (no Debug fallback on encode, no "unsupported" on decode).
        let leaves = [
            (SType::SBoolean, "SBoolean"),
            (SType::SByte, "SByte"),
            (SType::SShort, "SShort"),
            (SType::SInt, "SInt"),
            (SType::SLong, "SLong"),
            (SType::SBigInt, "SBigInt"),
            (SType::SUnsignedBigInt, "SUnsignedBigInt"),
            (SType::SGroupElement, "SGroupElement"),
            (SType::SSigmaProp, "SSigmaProp"),
            (SType::SBox, "SBox"),
            (SType::SHeader, "SHeader"),
            (SType::SPreHeader, "SPreHeader"),
            (SType::SAvlTree, "SAvlTree"),
            (SType::SUnit, "SUnit"),
            (SType::SAny, "SAny"),
        ];
        for (t, tag) in leaves {
            assert_eq!(encode_stype(&t), json!({"tag": tag}), "encode {}", tag);
            assert_eq!(
                decode_stype(&json!({"tag": tag})).expect(tag),
                t,
                "decode {}",
                tag
            );
        }
        // Recursive forms round-trip (incl. STuple, previously missing from decode).
        for j in [
            json!({"tag": "SColl", "elem": {"tag": "SAvlTree"}}),
            json!({"tag": "SOption", "elem": {"tag": "SAvlTree"}}),
            json!({"tag": "STuple", "items": [{"tag": "SInt"}, {"tag": "SAvlTree"}]}),
        ] {
            let t = decode_stype(&j).expect("recursive decode");
            assert_eq!(encode_stype(&t), j, "recursive round-trip");
        }
    }

    /// The Refused/Decode boundary: bytes the LIBRARY parses-and-rejects classify
    /// as `Refused` (its verdict -> runner `errored`); JSON the bridge cannot even
    /// dispatch classifies as `Decode` (bridge failure -> runner `panicked`).
    #[test]
    fn refused_vs_decode_classification() {
        // well-formed JSON, valid hex -- the library's Box parse refuses the bytes
        let r = decode_constant(&json!({"kind": "Box", "bytes_hex": "00"}));
        assert!(matches!(r, Err(BridgeError::Refused(_))), "got {:?}", r);
        // unsupported kind never reaches the library -- bridge malformation
        let d = decode_constant(&json!({"kind": "Nope", "value": 1}));
        assert!(matches!(d, Err(BridgeError::Decode(_))), "got {:?}", d);
    }
}
