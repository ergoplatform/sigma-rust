//! Context bindings (runner contract §4). A binding is an SValue — decoded by
//! [`sval::decode_constant`] — or `{"kind":"Value","bytes_hex":…}`: the bytes a node itself reads
//! for that position, read here by sigma-rust's OWN reader for it and never rebuilt from an SValue.
//! The reader refusing them (a parse error, rule 1019's `check_v6_type`, bytes left over) is
//! [`BridgeError::Refused`], which every caller maps to `errored`: a node would reject the
//! transaction.

use ergotree_ir::chain::context_extension::ContextExtension;
use ergotree_ir::chain::ergo_box::{NonMandatoryRegisterId, NonMandatoryRegisters};
use ergotree_ir::ergo_tree::ErgoTreeVersion;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::serialization::sigma_byte_reader::{self, SigmaByteRead};
use ergotree_ir::serialization::{SigmaParsingError, SigmaSerializable};
use serde_json::Value as J;

use crate::sval::{self, BridgeError};

/// A `Value` binding's bytes, or `None` for an SValue.
fn value_bytes(j: &J) -> Option<Result<Vec<u8>, BridgeError>> {
    if j.get("kind").and_then(|k| k.as_str()) != Some("Value") {
        return None;
    }
    Some(match j.get("bytes_hex").and_then(|h| h.as_str()) {
        Some(h) => crate::hex_to_bytes(h)
            .map_err(|_| BridgeError::Decode(format!("Value bytes_hex: bad hex {h:?}"))),
        None => Err(BridgeError::Decode(
            "Value binding without bytes_hex".into(),
        )),
    })
}

/// Read a frame of `len` bytes whole with `parse`, at the entry's activated version (stock
/// sigma-rust's reader carries one version, set with `with_tree_version`). A reader error, or bytes
/// left after the value, is the reader refusing the binding.
fn read_whole<R: SigmaByteRead, T>(
    mut r: R,
    len: usize,
    activated: u8,
    parse: impl FnOnce(&mut R) -> Result<T, SigmaParsingError>,
) -> Result<T, BridgeError> {
    let out = r
        .with_tree_version(ErgoTreeVersion::from(activated), parse)
        .map_err(|e| BridgeError::Refused(format!("the reader refused the Value: {e}")))?;
    let pos =
        r.position()
            .map_err(|e| BridgeError::Decode(format!("reader position: {e:?}")))? as usize;
    if pos != len {
        return Err(BridgeError::Refused(format!(
            "{} byte(s) left after the value",
            len - pos
        )));
    }
    Ok(out)
}

/// A binding at an extension position: var 1 (v2/v4 `input`), a v3 / v5 extension value. A
/// `Value` is read by sigma-rust's `ContextExtension` reader, framed as the one-variable extension
/// `01 01 <bytes>` — the count, then a dummy id: the binding's key is the vector's and the caller
/// applies it, so a v5 key >= 0x80 still reaches the context the way the key-domain vectors need.
/// This reader holds a `Constant`, so it refuses an evaluated expression the JVM's `getValue` takes.
pub fn extension_binding(j: &J, activated: u8) -> Result<Constant, BridgeError> {
    let Some(bytes) = value_bytes(j) else {
        return sval::decode_constant(j);
    };
    let frame = [&[0x01u8, 0x01][..], &bytes?].concat();
    let ext = read_whole(
        sigma_byte_reader::from_bytes(&frame[..]),
        frame.len(),
        activated,
        ContextExtension::sigma_parse,
    )?;
    ext.values
        .get(&1u8)
        .cloned()
        .ok_or_else(|| BridgeError::Refused("the extension reader returned no variable".into()))
}

/// SELF's registers (`selfRegisters`, as `(id, binding)` pairs). With no `Value` among them they
/// are built from their constants as before (`NonMandatoryRegisters::new`). With one, the whole
/// map is read by sigma-rust's `NonMandatoryRegisters` reader in one frame — the count, then each
/// register's bytes: a `Value`'s as given, an SValue's as sigma-rust's writer encodes its constant
/// — so every register is the reader's own `RegisterValue` (a constant, or a parsed Tuple
/// expression). One frame rather than one register at a time because stock sigma-rust offers no
/// way to take a single parsed register out as a `RegisterValue`: its `NonMandatoryRegisters::get`
/// indexes by the id itself (4 for R4) where the values are stored from 0 (the fork fixed it), and
/// its one correct conversion out is into a `hashbrown` map this crate could only name by pinning
/// the impl's version.
pub fn register_map(
    regs: &[(NonMandatoryRegisterId, &J)],
    activated: u8,
) -> Result<NonMandatoryRegisters, BridgeError> {
    // Densely packed from R4, as `NonMandatoryRegisters` requires — a gap is the vector's defect.
    for (i, (id, _)) in regs.iter().enumerate() {
        if *id as usize != NonMandatoryRegisterId::START_INDEX + i {
            return Err(BridgeError::Decode(format!(
                "selfRegisters are not densely packed from R4 (found {id:?} at position {i})"
            )));
        }
    }
    if !regs.iter().any(|(_, j)| value_bytes(j).is_some()) {
        let consts = regs
            .iter()
            .map(|(id, j)| sval::decode_constant(j).map(|c| (*id, c)))
            .collect::<Result<Vec<_>, _>>()?;
        return NonMandatoryRegisters::new(consts)
            .map_err(|e| BridgeError::Decode(format!("selfRegisters: {e}")));
    }
    let mut frame = vec![regs.len() as u8];
    for (_, j) in regs {
        match value_bytes(j) {
            Some(bytes) => frame.extend(bytes?),
            None => frame.extend(
                sval::decode_constant(j)?
                    .sigma_serialize_bytes()
                    .map_err(|e| BridgeError::Decode(format!("selfRegisters constant: {e:?}")))?,
            ),
        }
    }
    read_whole(
        sigma_byte_reader::from_bytes(&frame[..]),
        frame.len(),
        activated,
        NonMandatoryRegisters::sigma_parse,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
    fn value_json(bytes_hex: &str) -> J {
        serde_json::json!({ "kind": "Value", "bytes_hex": bytes_hex })
    }
    fn const_hex(c: &Constant) -> String {
        hex(&c.sigma_serialize_bytes().unwrap())
    }

    #[test]
    fn an_extension_value_equals_the_json_form_of_the_same_value() {
        let json = serde_json::json!({ "kind": "Long", "value": "7" });
        let from_json = extension_binding(&json, 3).unwrap();
        let from_bytes =
            extension_binding(&value_json(&const_hex(&Constant::from(7i64))), 3).unwrap();
        assert_eq!(from_bytes, from_json);
    }

    fn r4(regs: &NonMandatoryRegisters) -> Option<Constant> {
        regs.get_constant(NonMandatoryRegisterId::R4).unwrap()
    }

    #[test]
    fn a_register_value_equals_the_json_form_of_the_same_value() {
        let json = serde_json::json!({ "kind": "Long", "value": "7" });
        let value = value_json(&const_hex(&Constant::from(7i64)));
        let from_json = register_map(&[(NonMandatoryRegisterId::R4, &json)], 3).unwrap();
        let from_bytes = register_map(&[(NonMandatoryRegisterId::R4, &value)], 3).unwrap();
        assert_eq!(from_bytes, from_json);
        assert_eq!(r4(&from_bytes), Some(Constant::from(7i64)));
    }

    #[test]
    fn a_value_register_beside_json_registers_reads_each_one_in_place() {
        let json4 = serde_json::json!({ "kind": "Int", "value": 4 });
        let value5 = value_json(&const_hex(&Constant::from(7i64)));
        let json6 = serde_json::json!({ "kind": "Boolean", "value": true });
        let regs = register_map(
            &[
                (NonMandatoryRegisterId::R4, &json4),
                (NonMandatoryRegisterId::R5, &value5),
                (NonMandatoryRegisterId::R6, &json6),
            ],
            3,
        )
        .unwrap();
        assert_eq!(regs.len(), 3);
        assert_eq!(r4(&regs), Some(Constant::from(4i32)));
        assert_eq!(
            regs.get_constant(NonMandatoryRegisterId::R5).unwrap(),
            Some(Constant::from(7i64))
        );
        assert_eq!(
            regs.get_constant(NonMandatoryRegisterId::R6).unwrap(),
            Some(Constant::from(true))
        );
    }

    #[test]
    fn a_gap_in_the_registers_is_the_vectors_defect_not_a_refusal() {
        let json = serde_json::json!({ "kind": "Int", "value": 4 });
        let value = value_json(&const_hex(&Constant::from(7i64)));
        let got = register_map(
            &[
                (NonMandatoryRegisterId::R4, &json),
                (NonMandatoryRegisterId::R6, &value),
            ],
            3,
        );
        assert!(matches!(got, Err(BridgeError::Decode(_))), "got {got:?}");
    }

    #[test]
    fn bytes_left_after_the_value_are_refused_at_both_positions() {
        let trailing = format!("{}00", const_hex(&Constant::from(7i64)));
        for got in [
            extension_binding(&value_json(&trailing), 3).map(|_| ()),
            register_map(&[(NonMandatoryRegisterId::R4, &value_json(&trailing))], 3).map(|_| ()),
        ] {
            match got {
                Err(BridgeError::Refused(m)) => assert!(m.contains("left after the value"), "{m}"),
                other => panic!("expected the leftover-bytes refusal, got {other:?}"),
            }
        }
    }

    // Rule 1019: an Option-typed value (`Option[Int]` Some(1): type 0x24|0x04 = 0x28, then
    // 01 = Some, then Int 1 zig-zag = 02) is refused by both of a node's readers at every version.
    #[test]
    fn rule_1019_refuses_an_option_at_both_positions() {
        // The bytes ARE an Option[Int] constant — a plain constant read at v3 takes them — so the
        // refusals below are rule 1019's, not a parse error's.
        let c = sigma_byte_reader::from_bytes(&[0x28u8, 0x01, 0x02][..])
            .with_tree_version(ErgoTreeVersion::V3, Constant::sigma_parse)
            .expect("an Option[Int] constant");
        assert_eq!(
            c.tpe,
            ergotree_ir::types::stype::SType::SOption(
                Box::new(ergotree_ir::types::stype::SType::SInt).into()
            )
        );
        let some_int = value_json("280102");
        assert!(matches!(
            extension_binding(&some_int, 3),
            Err(BridgeError::Refused(_))
        ));
        assert!(matches!(
            register_map(&[(NonMandatoryRegisterId::R4, &some_int)], 3),
            Err(BridgeError::Refused(_))
        ));
    }

    // A Tuple EXPRESSION (opcode 0x86, two items: Int 1, Int 2): this reader keeps a register as a
    // parsed Tuple expression, and refuses one as an extension value (it reads a constant there) —
    // the reader's own answer, surfaced as it is.
    #[test]
    fn a_tuple_expression_is_a_register_and_not_an_extension_value_here() {
        let tuple_expr = value_json("860204020404");
        let regs = register_map(&[(NonMandatoryRegisterId::R4, &tuple_expr)], 3).unwrap();
        // The reader kept the Tuple EXPRESSION: it re-serializes as one (opcode 0x86), where an
        // STuple constant would write its type byte instead.
        let bytes = regs.sigma_serialize_bytes().unwrap();
        assert_eq!(hex(&bytes), "01860204020404");
        assert!(matches!(
            extension_binding(&tuple_expr, 3),
            Err(BridgeError::Refused(_))
        ));
    }
}
