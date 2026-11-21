//! The eval tier: evaluate an entry's tree in the canonical evaluation context, with the entry's
//! input bound where its envelope puts it, under the entry's `(activated, ergoTree)` versions.
//!
//! The canonical context is the one the reference evaluator (sigma-state) builds for a bare
//! evaluation. SELF is a box holding the entry's tree, with value 1,000,000, no tokens, no
//! additional registers, creation height 0, a zero transaction id and index 0. INPUTS is exactly
//! `[SELF]`, OUTPUTS and the data inputs are empty, HEIGHT is 0, the pre-header is fixed (see
//! [`ContextParts::context`]) and no spending input carries a context extension. The entry's input
//! goes, by envelope ([`Binding`]): to variable 1 of SELF's extension (v1/v2); into one extension
//! per spending input, read by `getVarFromInput` (v3); into SELF's registers R4-R9 plus variable 1
//! (v4); or it is SELF's whole extension (v5).
//!
//! `Context` holds exactly ten headers, where the canonical context has none, so it carries ten
//! copies of one fixed header ([`fixed_header`]). An entry that reads `CONTEXT.headers` therefore
//! sees ten headers where the reference sees an empty collection: a difference in this `Context`'s
//! model, not in the binding.
//!
//! Trees are read with `ErgoTree::sigma_parse_bytes_lenient`, because the vectors' roots are of any
//! type, not only SigmaProp (the reference's `deserializeErgoTree(checkType = false)`). They are
//! evaluated with `eval::test_util::try_eval_with_deserialize`, the public entry (behind the
//! `arbitrary` feature) that evaluates a root of any type: it substitutes `DeserializeContext`
//! nodes against the context, then evaluates. This sigma-rust has no JIT-cost model, so no cost is
//! measured.

use std::cell::Cell;

use ergo_chain_types::{
    ec_point, ADDigest, AutolykosSolution, BlockId, Digest, Digest32, Header, PreHeader, Votes,
};
use ergotree_interpreter::eval::test_util::try_eval_with_deserialize;
use ergotree_ir::chain::context::{Context, ContextExtensionProvider, TxIoVec};
use ergotree_ir::chain::context_extension::ContextExtension;
use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::{ErgoBox, NonMandatoryRegisterId, NonMandatoryRegisters};
use ergotree_ir::chain::tx_id::TxId;
use ergotree_ir::ergo_tree::{ErgoTree, ErgoTreeVersion};
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::value::Value;

use crate::binding;
use crate::sval;

/// One entry's outcome.
pub enum Outcome {
    /// Evaluated to a value.
    Success {
        value: serde_json::Value,
    },
    Errored,
    /// An entry the runner cannot evaluate faithfully on this sigma-rust: emitted for the
    /// full-context form, whose context (the 9 headers a script sees) `Context` cannot hold.
    NotImplemented,
    /// A failure that is not the library's verdict: a panic caught by `main`'s net, or a failure
    /// to decode the entry's input or encode its result. The note says which.
    Panicked {
        note: String,
    },
}

impl Outcome {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            // No cost is measured (the runner declares `cost: false`, so the grader ignores the
            // field), but the actuals schema wants a number on a success: 0.
            Outcome::Success { value } => {
                serde_json::json!({"value": value, "cost": 0, "error": serde_json::Value::Null})
            }
            Outcome::Errored => {
                serde_json::json!({"value": null, "cost": null, "error": "errored"})
            }
            Outcome::NotImplemented => {
                serde_json::json!({"value": null, "cost": null, "error": "not-implemented"})
            }
            Outcome::Panicked { note } => {
                serde_json::json!({"value": null, "cost": null, "error": "panicked", "note": note})
            }
        }
    }
}

/// What an entry binds into the canonical context, by envelope.
enum Binding {
    /// v1/v2: the entry's input, if it has one, at variable 1 of SELF's extension.
    Var1(Option<Constant>),
    /// v3: one context extension per spending input (`getVarFromInput`); SELF's own is empty.
    PerInput(Vec<ContextExtension>),
    /// v4: SELF's registers R4-R9, and the entry's input at variable 1.
    SelfRegisters(NonMandatoryRegisters, Constant),
    /// v5: SELF's whole extension, verbatim (keys 0..255).
    Extension(ContextExtension),
}

/// The spending inputs' context extensions, by input index.
struct InputExtensions(Vec<ContextExtension>);

impl ContextExtensionProvider for InputExtensions {
    fn context_extension(&self, input_index: usize) -> Option<&ContextExtension> {
        self.0.get(input_index)
    }
}

/// What the canonical context borrows.
struct ContextParts {
    /// SELF, and INPUTS' one box.
    self_box: ErgoBox,
    extension: ContextExtension,
    input_extensions: InputExtensions,
}

impl ContextParts {
    fn new(binding: Binding, tree: &ErgoTree) -> Result<ContextParts, String> {
        let with_var1 = |c: Option<Constant>| {
            let mut ext = ContextExtension::empty();
            if let Some(c) = c {
                ext.values.insert(1u8, c);
            }
            ext
        };
        let none = || InputExtensions(Vec::new());
        let no_registers = NonMandatoryRegisters::empty;
        let (registers, extension, input_extensions) = match binding {
            Binding::Var1(input) => (no_registers(), with_var1(input), none()),
            Binding::PerInput(exts) => (
                no_registers(),
                ContextExtension::empty(),
                InputExtensions(exts),
            ),
            Binding::SelfRegisters(regs, input) => (regs, with_var1(Some(input)), none()),
            Binding::Extension(ext) => (no_registers(), ext, none()),
        };
        // SELF is built with its registers, never given them afterwards: a box's id is the hash of
        // its bytes, registers included.
        let self_box = ErgoBox::new(
            BoxValue::new(1_000_000).map_err(|e| format!("SELF value: {e:?}"))?,
            tree.clone(),
            None, // no tokens
            registers,
            0, // creation height
            TxId::zero(),
            0, // index
        )
        .map_err(|e| format!("the canonical SELF box: {e:?}"))?;
        Ok(ContextParts {
            self_box,
            extension,
            input_extensions,
        })
    }

    /// The canonical context over these parts. The pre-header is the reference's: version
    /// `activated + 1` (the block version the activated script version derives from), a zero
    /// parent id, timestamp 3, nBits 0, height 0, the group generator as miner key, zero votes.
    fn context(&self, tree_version: u8, activated_version: u8) -> Context<'_> {
        Context {
            height: 0,
            self_box: &self.self_box,
            outputs: &[],
            data_inputs: None,
            inputs: TxIoVec::from_vec(vec![&self.self_box])
                .expect("one input is within TxIoVec's bounds"),
            pre_header: PreHeader {
                version: activated_version + 1,
                parent_id: BlockId(Digest::zero()),
                timestamp: 3,
                n_bits: 0,
                height: 0,
                miner_pk: Box::new(ec_point::generator()),
                votes: Votes([0u8; 3]),
            },
            headers: std::array::from_fn(|_| fixed_header()),
            extension: &self.extension,
            tree_version: Cell::new(ErgoTreeVersion::from(tree_version)),
            extension_provider: &self.input_extensions,
        }
    }
}

/// The header `Context`'s ten header slots hold: version 2, every field zero, the generator as miner
/// key (the canonical context has no headers; see the module docs).
fn fixed_header() -> Header {
    Header {
        version: 2,
        id: BlockId(Digest32::zero()),
        parent_id: BlockId(Digest32::zero()),
        ad_proofs_root: Digest32::zero(),
        state_root: ADDigest::zero(),
        transaction_root: Digest32::zero(),
        timestamp: 0,
        n_bits: 0,
        height: 0,
        extension_root: Digest32::zero(),
        autolykos_solution: AutolykosSolution {
            miner_pk: Box::new(ec_point::generator()),
            pow_onetime_pk: None,
            nonce: vec![0u8; 8],
            pow_distance: None,
        },
        votes: Votes([0u8; 3]),
        unparsed_bytes: Box::new([]),
    }
}

/// An input-decode failure's outcome: the library refusing the bytes (its verdict on material
/// the reference accepted) is `errored`, a divergence; any other failure (malformed vector JSON,
/// an unsupported kind) is the runner's own, `panicked` with the cause.
fn decode_failure_outcome(e: sval::BridgeError, site: &str) -> Outcome {
    match e {
        sval::BridgeError::Refused(_) => Outcome::Errored,
        other => Outcome::Panicked {
            note: format!("{}: {:?}", site, other),
        },
    }
}

/// One extension map (`{"<var id>": <binding>}`), each value read by sigma-rust's own reader. A
/// key that is not a byte is `errored`.
fn extension_from(
    map: &serde_json::Map<String, serde_json::Value>,
    activated_version: u8,
    site: &str,
) -> Result<ContextExtension, Outcome> {
    let mut ext = ContextExtension::empty();
    for (k, v) in map {
        let id: u8 = k.parse().map_err(|_| Outcome::Errored)?;
        let c = binding::extension_binding(v, activated_version)
            .map_err(|e| decode_failure_outcome(e, site))?;
        ext.values.insert(id, c);
    }
    Ok(ext)
}

/// SELF's registers (`{"4": <binding>, …, "9": …}`), read as one register map, in id order.
fn registers_from(
    map: &serde_json::Map<String, serde_json::Value>,
    activated_version: u8,
) -> Result<NonMandatoryRegisters, Outcome> {
    let mut pairs: Vec<(NonMandatoryRegisterId, &serde_json::Value)> =
        Vec::with_capacity(map.len());
    for (k, v) in map {
        let id: u8 = k.parse().map_err(|_| Outcome::Panicked {
            note: format!("v4 selfRegisters: bad key {:?}", k),
        })?;
        let reg_id = match id {
            4 => NonMandatoryRegisterId::R4,
            5 => NonMandatoryRegisterId::R5,
            6 => NonMandatoryRegisterId::R6,
            7 => NonMandatoryRegisterId::R7,
            8 => NonMandatoryRegisterId::R8,
            9 => NonMandatoryRegisterId::R9,
            _ => {
                return Err(Outcome::Panicked {
                    note: format!("v4 selfRegisters: key {} out of R4-R9 range", id),
                })
            }
        };
        pairs.push((reg_id, v));
    }
    pairs.sort_by_key(|(rid, _)| *rid as u8);
    binding::register_map(&pairs, activated_version)
        .map_err(|e| decode_failure_outcome(e, "v4 selfRegisters decode"))
}

/// The entry's binding, by envelope: an `extension` makes it v5, else `selfRegisters` v4, else
/// `inputs` v3, else v1/v2 (its `input`, if any). A v3 entry's `input` is still decoded, first.
fn binding_of(
    input: Option<&serde_json::Value>,
    inputs: Option<&Vec<serde_json::Value>>,
    self_registers: Option<&serde_json::Map<String, serde_json::Value>>,
    extension: Option<&serde_json::Map<String, serde_json::Value>>,
    activated_version: u8,
) -> Result<Binding, Outcome> {
    // A key >= 0x80 stays in: sigma-rust's own guard decides (the reference refuses it), so an
    // implementation that wrongly accepts it shows.
    if let Some(map) = extension {
        return extension_from(map, activated_version, "v5 extension decode")
            .map(Binding::Extension);
    }
    if let Some(map) = self_registers {
        let input = input.ok_or_else(|| Outcome::Panicked {
            note: "v4 entry missing input".into(),
        })?;
        let var1 = binding::extension_binding(input, activated_version)
            .map_err(|e| decode_failure_outcome(e, "v4 input decode"))?;
        let regs = registers_from(map, activated_version)?;
        return Ok(Binding::SelfRegisters(regs, var1));
    }
    let var1 = match input {
        Some(j) => Some(
            binding::extension_binding(j, activated_version)
                .map_err(|e| decode_failure_outcome(e, "input decode"))?,
        ),
        None => None,
    };
    match inputs {
        Some(arr) => {
            let mut exts = Vec::with_capacity(arr.len());
            for inp in arr {
                exts.push(match inp.get("extension").and_then(|e| e.as_object()) {
                    Some(map) => extension_from(map, activated_version, "input decode (v3)")?,
                    None => ContextExtension::empty(),
                });
            }
            Ok(Binding::PerInput(exts))
        }
        None => Ok(Binding::Var1(var1)),
    }
}

/// Evaluate one entry. Produces exactly one [`Outcome`].
pub fn run_entry(
    tree_bytes: &[u8],
    input: Option<&serde_json::Value>,
    inputs: Option<&Vec<serde_json::Value>>,
    self_registers: Option<&serde_json::Map<String, serde_json::Value>>,
    extension: Option<&serde_json::Map<String, serde_json::Value>>,
    tree_version: u8,
    activated_version: u8,
) -> Outcome {
    match binding_of(input, inputs, self_registers, extension, activated_version) {
        Ok(binding) => evaluate(tree_bytes, binding, tree_version, activated_version),
        Err(outcome) => outcome,
    }
}

/// Parse the tree, build the canonical context around it and the binding, evaluate, encode. A tree
/// the reference accepted that sigma-rust cannot parse, or an evaluation error, is `errored`.
fn evaluate(
    tree_bytes: &[u8],
    binding: Binding,
    tree_version: u8,
    activated_version: u8,
) -> Outcome {
    let tree = match ErgoTree::sigma_parse_bytes_lenient(tree_bytes) {
        Ok(t) => t,
        Err(_) => return Outcome::Errored,
    };
    let root = match tree.proposition() {
        Ok(r) => r,
        Err(_) => return Outcome::Errored,
    };
    let parts = match ContextParts::new(binding, &tree) {
        Ok(p) => p,
        Err(note) => return Outcome::Panicked { note },
    };
    let ctx = parts.context(tree_version, activated_version);
    match try_eval_with_deserialize::<Value<'static>>(&root, &ctx) {
        Ok(v) => match sval::encode_value(&v) {
            Ok(value) => Outcome::Success { value },
            Err(e) => Outcome::Panicked {
                note: format!("result encode: {:?}", e),
            },
        },
        Err(_) => Outcome::Errored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A library refusal is `errored`; any other bridge failure is `panicked`, the site in the note.
    #[test]
    fn decode_failure_outcome_split() {
        assert!(matches!(
            decode_failure_outcome(sval::BridgeError::Refused("x".into()), "input decode"),
            Outcome::Errored
        ));
        match decode_failure_outcome(sval::BridgeError::Decode("x".into()), "input decode") {
            Outcome::Panicked { note } => {
                assert!(note.starts_with("input decode:"), "note: {}", note)
            }
            other => panic!("expected Panicked, got {:?}", outcome_name(&other)),
        }
    }

    fn fixture_tree() -> ErgoTree {
        // built from an Expr, not parsed, so these tests do not depend on the lenient entry
        ErgoTree::new(
            ergotree_ir::ergo_tree::ErgoTreeHeader::v1(false),
            &ergotree_ir::mir::expr::Expr::Const(1i32.into()),
        )
        .expect("the fixture tree is constructible")
    }

    /// The canonical context's box surfaces. Each is asserted: a vector that reads one would
    /// otherwise grade against whatever the runner happened to put there.
    #[test]
    fn canonical_context_pins_the_box_surfaces() {
        let tree = fixture_tree();
        let parts = ContextParts::new(Binding::Var1(None), &tree).expect("the parts build");
        let ctx = parts.context(3, 2);
        assert_eq!(ctx.self_box.value.as_u64(), &1_000_000u64, "SELF.value");
        assert_eq!(
            ctx.self_box.ergo_tree, tree,
            "SELF.ergoTree is the entry's tree"
        );
        assert!(ctx.self_box.tokens.is_none(), "SELF carries no tokens");
        assert_eq!(ctx.self_box.creation_height, 0, "SELF.creationHeight");
        assert_eq!(
            ctx.self_box.transaction_id,
            TxId::zero(),
            "SELF.transactionId"
        );
        assert_eq!(ctx.self_box.index, 0, "SELF.index");
        // get_constant, not get: get indexes the registers by the id's discriminant (R4 = 4)
        assert!(
            matches!(
                ctx.self_box
                    .additional_registers
                    .get_constant(NonMandatoryRegisterId::R4),
                Ok(None)
            ),
            "SELF.R4 is absent"
        );
        assert_eq!(ctx.inputs.as_vec().len(), 1, "INPUTS is exactly [SELF]");
        assert_eq!(
            **ctx.inputs.first(),
            *ctx.self_box,
            "INPUTS[0] is SELF by value (selfBoxIndex is a value search)"
        );
        assert!(ctx.outputs.is_empty(), "OUTPUTS is empty");
        assert!(ctx.data_inputs.is_none(), "the data inputs are empty");
        assert_eq!(ctx.height, 0, "HEIGHT");
        assert_eq!(
            ctx.pre_header.version, 3,
            "preHeader.version is activated + 1"
        );
        assert_eq!(
            *ctx.pre_header.miner_pk,
            ec_point::generator(),
            "preHeader.minerPk"
        );
        assert_eq!(
            ctx.tree_version.get(),
            ErgoTreeVersion::from(3u8),
            "the tree version"
        );
    }

    /// No spending input carries a context extension, except in v3, which carries one per entry
    /// extension; v1/v2, v4 and v5 bind into SELF's own.
    #[test]
    fn only_v3_binds_per_input_extensions() {
        let tree = fixture_tree();
        let var1: Constant = 7i32.into();
        let no_input_extension = |parts: &ContextParts, what: &str| {
            for i in 0..3usize {
                assert!(
                    parts
                        .context(3, 2)
                        .extension_provider
                        .context_extension(i)
                        .is_none(),
                    "{what}: input {i} carries no extension"
                );
            }
        };

        let v2 = ContextParts::new(Binding::Var1(Some(var1.clone())), &tree).unwrap();
        assert!(
            v2.context(3, 2).extension.values.contains_key(&1u8),
            "v2: SELF's var 1"
        );
        no_input_extension(&v2, "v2");

        let regs = NonMandatoryRegisters::new(vec![(NonMandatoryRegisterId::R4, 1i32.into())])
            .expect("one register");
        let v4 = ContextParts::new(Binding::SelfRegisters(regs, var1.clone()), &tree).unwrap();
        let ctx = v4.context(3, 2);
        assert!(ctx.extension.values.contains_key(&1u8), "v4: SELF's var 1");
        assert!(
            matches!(
                ctx.self_box
                    .additional_registers
                    .get_constant(NonMandatoryRegisterId::R4),
                Ok(Some(_))
            ),
            "v4: SELF carries the entry's R4"
        );
        no_input_extension(&v4, "v4");

        let mut ext = ContextExtension::empty();
        ext.values.insert(0u8, var1.clone());
        let v5 = ContextParts::new(Binding::Extension(ext), &tree).unwrap();
        assert!(
            v5.context(3, 2).extension.values.contains_key(&0u8),
            "v5: verbatim"
        );
        no_input_extension(&v5, "v5");

        let mut inp = ContextExtension::empty();
        inp.values.insert(1u8, var1);
        let v3 = ContextParts::new(Binding::PerInput(vec![inp]), &tree).unwrap();
        let ctx = v3.context(3, 2);
        assert!(
            ctx.extension.values.is_empty(),
            "v3: SELF's own extension is empty"
        );
        assert!(
            ctx.extension_provider.context_extension(0).is_some(),
            "v3: input 0's"
        );
        assert!(
            ctx.extension_provider.context_extension(1).is_none(),
            "v3: only input 0"
        );
    }

    /// A v4 entry's SELF is ONE box: built with the entry's registers, and INPUTS' only box (the
    /// reference's `boxesToSpend = [selfBox]`). The five answers are the reference's (sigma-state
    /// 6.0.7) for a SELF holding R4 = Int 1: its id is the id of the box WITH the register.
    #[test]
    fn a_v4_self_is_the_one_input_box_with_its_registers() {
        let registers = serde_json::json!({"4": {"kind": "Int", "value": 1}});
        let input = serde_json::json!({"kind": "Int", "value": 0});
        let eval = |tree_hex: &str| {
            let tree = crate::hex_to_bytes(tree_hex).expect("hex");
            match run_entry(&tree, Some(&input), None, registers.as_object(), None, 3, 3) {
                Outcome::Success { value } => value,
                other => panic!("{tree_hex}: expected a value, got {}", outcome_name(&other)),
            }
        };
        let bytes = |hex: &str| {
            let items: Vec<serde_json::Value> = crate::hex_to_bytes(hex)
                .expect("hex")
                .into_iter()
                .map(|b| serde_json::json!({"kind": "Byte", "value": b as i8}))
                .collect();
            serde_json::json!({"kind": "Coll", "elem": {"tag": "SByte"}, "items": items})
        };
        let boolean = |b: bool| serde_json::json!({"kind": "Boolean", "value": b});
        assert_eq!(
            eval("0b0a959304020402c5a7c5a7"),
            bytes("12e4ec7a6bdd88b9932716ccae39a7607feb57b435346aeaf658b13d525bb19d"),
            "SELF.id"
        );
        assert_eq!(
            eval("0b12959304020402c5b2a4040000c5b2a4040000"),
            bytes("0feec40512b0c36c2da6a89645985b2a88a36d3f1b16e94b5ff1a301a303a8a1"),
            "INPUTS(0).id"
        );
        assert_eq!(
            eval("0b1495930402040293b2a4040000a793b2a4040000a7"),
            boolean(true),
            "INPUTS(0) == SELF"
        );
        assert_eq!(
            eval("0b0e959304020402db6508fedb6508fe"),
            serde_json::json!({"kind": "Int", "value": 0}),
            "CONTEXT.selfBoxIndex"
        );
        assert_eq!(
            eval("0b18959304020402e6c6b2a40400000404e6c6b2a40400000404"),
            boolean(true),
            "INPUTS(0).R4[Int].isDefined"
        );
    }

    fn outcome_name(o: &Outcome) -> &'static str {
        match o {
            Outcome::Success { .. } => "Success",
            Outcome::Errored => "Errored",
            Outcome::NotImplemented => "NotImplemented",
            Outcome::Panicked { .. } => "Panicked",
        }
    }
}
