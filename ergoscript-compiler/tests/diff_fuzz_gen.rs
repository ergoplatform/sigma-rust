#![allow(dead_code)]
//! WS-F.2 — Programmatic ErgoScript generator (typed AST builder).
//!
//! Emits ~600 type-correct ErgoScript programs covering the predef + method
//! dispatch surface catalogued in
//! `tests/fixtures/significant_15/parity-handoffs/LOWERING-SHAPE-AUDIT.md`
//! and the produced-Expr-variant surface from
//! `tests/fixtures/significant_15/parity-handoffs/IR-PASS-COVERAGE-MATRIX.md`.
//!
//! Run with:
//!   cargo test -p ergoscript-compiler --test diff_fuzz_gen \
//!       gen_corpus -- --ignored --nocapture
//!
//! Produces files at `target/diff_fuzz/corpus/gen_<seed_hex>_<id>.es`.
//! `tests/diff_fuzz.rs` reads the same directory; running the harness after
//! `gen_corpus` is what surfaces F.3's cluster-eligible bug surface.
//!
//! ## Type-correctness by construction
//!
//! Every emitter returns a `Term { src, ty }`. We never compose terms whose
//! types disagree with what the consuming construct demands. If `pick_or_lit`
//! cannot satisfy a slot, the emitter skips that variation rather than
//! emitting ill-typed source. A generator that emits binder errors produces
//! noise that drowns the actual parity bugs.
//!
//! ## Tables vs audit
//!
//! `PREDEF_NAMES` and `METHOD_TABLE` are hand-mirrored from the audits — if
//! `LOWERING-SHAPE-AUDIT.md` adds a builtin, this module must be updated.
//! See module-top comments in those files; both directions are linked.
//!
//! ## Determinism
//!
//! SplitMix64-based PRNG is platform-independent. Sub-seeds for each pass
//! derive from `mix(seed, pass_idx)`. Same seed → byte-identical corpus.
//! F.3's cluster verification depends on this property.

use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_SEED: u64 = 0xF000_0000_F000_0001;

// ---- PRNG (SplitMix64) ------------------------------------------------------

/// Deterministic PRNG. SplitMix64 — same algorithm everywhere, no float ops,
/// no platform sensitivity, no external dep. Fixture `state` evolves
/// monotonically; output bytes are pure functions of `state` alone.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn gen_range(&mut self, lo: u64, hi: u64) -> u64 {
        debug_assert!(hi > lo);
        lo + self.next_u64() % (hi - lo)
    }

    fn pick_idx(&mut self, n: usize) -> usize {
        (self.next_u64() % (n as u64)) as usize
    }

    fn flip(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }
}

fn mix_seed(seed: u64, salt: u64) -> u64 {
    let mut r = Rng::new(seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    r.next_u64()
}

// ---- Type system ------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum Ty {
    Boolean,
    Byte,
    Short,
    Int,
    Long,
    BigInt,
    GroupElement,
    SigmaProp,
    Box_,
    Coll(Box<Ty>),
    Option_(Box<Ty>),
}

impl Ty {
    fn ergo_str(&self) -> String {
        match self {
            Ty::Boolean => "Boolean".into(),
            Ty::Byte => "Byte".into(),
            Ty::Short => "Short".into(),
            Ty::Int => "Int".into(),
            Ty::Long => "Long".into(),
            Ty::BigInt => "BigInt".into(),
            Ty::GroupElement => "GroupElement".into(),
            Ty::SigmaProp => "SigmaProp".into(),
            Ty::Box_ => "Box".into(),
            Ty::Coll(t) => format!("Coll[{}]", t.ergo_str()),
            Ty::Option_(t) => format!("Option[{}]", t.ergo_str()),
        }
    }

    fn is_numeric(&self) -> bool {
        matches!(self, Ty::Byte | Ty::Short | Ty::Int | Ty::Long | Ty::BigInt)
    }
}

#[derive(Clone, Debug)]
struct Term {
    src: String,
    ty: Ty,
}

impl Term {
    fn new(src: impl Into<String>, ty: Ty) -> Self {
        Self {
            src: src.into(),
            ty,
        }
    }
}

// ---- Environment ------------------------------------------------------------

/// In-scope bindings — globals + locally-bound vals. Adding a binding never
/// invalidates earlier ones; `pick_typed` filters by exact type equality.
struct Env {
    vals: Vec<Term>,
}

impl Env {
    fn fresh() -> Self {
        Self {
            vals: vec![
                Term::new("HEIGHT", Ty::Int),
                Term::new("SELF", Ty::Box_),
                Term::new("INPUTS", Ty::Coll(Box::new(Ty::Box_))),
                Term::new("OUTPUTS", Ty::Coll(Box::new(Ty::Box_))),
                Term::new("groupGenerator", Ty::GroupElement),
            ],
        }
    }

    fn bind(&mut self, t: Term) {
        self.vals.push(t);
    }

    fn pick_typed(&self, ty: &Ty, rng: &mut Rng) -> Option<Term> {
        let cands: Vec<&Term> = self.vals.iter().filter(|t| &t.ty == ty).collect();
        if cands.is_empty() {
            None
        } else {
            Some(cands[rng.pick_idx(cands.len())].clone())
        }
    }

    fn pick_or_lit(&self, ty: &Ty, rng: &mut Rng) -> Option<Term> {
        if rng.flip() {
            if let Some(t) = self.pick_typed(ty, rng) {
                return Some(t);
            }
        }
        lit(ty, rng).or_else(|| self.pick_typed(ty, rng))
    }
}

/// Safe-known literal forms. Returns `None` for types whose literal form
/// would require additional context (Box, Coll[Box], Option, GroupElement).
fn lit(ty: &Ty, rng: &mut Rng) -> Option<Term> {
    match ty {
        Ty::Boolean => Some(Term::new(
            if rng.flip() { "true" } else { "false" },
            ty.clone(),
        )),
        Ty::Byte => Some(Term::new(
            format!("{}.toByte", rng.gen_range(0, 100)),
            ty.clone(),
        )),
        Ty::Short => Some(Term::new(
            format!("{}.toShort", rng.gen_range(0, 1000)),
            ty.clone(),
        )),
        Ty::Int => Some(Term::new(
            format!("{}", rng.gen_range(0, 100_000)),
            ty.clone(),
        )),
        Ty::Long => Some(Term::new(
            format!("{}L", rng.gen_range(0, 1_000_000)),
            ty.clone(),
        )),
        Ty::BigInt => Some(Term::new(
            // `<int-literal>.toBigInt` is the Scala-parseable form; the
            // earlier `byteArrayToBigInt(fromBase16("XX"))` form tripped
            // Scala's binder on small constant Coll[Byte] inputs.
            format!("({}).toBigInt", rng.gen_range(1, 1_000_000)),
            ty.clone(),
        )),
        Ty::Coll(inner) if **inner == Ty::Byte => {
            // 1–4 bytes hex literal
            let n = rng.gen_range(1, 5) as usize;
            let mut s = String::from("fromBase16(\"");
            for _ in 0..n {
                s.push_str(&format!("{:02x}", rng.gen_range(0, 256)));
            }
            s.push_str("\")");
            Some(Term::new(s, ty.clone()))
        }
        Ty::GroupElement => {
            // groupGenerator is in env; literal form via decodePoint requires
            // a valid encoded point — just defer to env.
            None
        }
        _ => None,
    }
}

// ---- Predef + method tables (hand-mirrored from LOWERING-SHAPE-AUDIT.md) ---
//
// Source of truth: `tests/fixtures/significant_15/parity-handoffs/LOWERING-SHAPE-AUDIT.md`.
// If that audit changes, update PREDEF_BUILDERS and METHOD_BUILDERS below.

/// A predef call emitter — builds a Boolean predicate that exercises one
/// predef. `body` returns `(decls, terminal_bool)` where `decls` are
/// extra `val ...` lines and `terminal_bool` is a Boolean expression.
type PredefBuilder = fn(&mut Env, &mut Rng) -> Option<(Vec<String>, Term)>;

fn p_blake2b256(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let arg = env.pick_or_lit(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    let bound = format!("val h = blake2b256({})", arg.src);
    Some((vec![bound], Term::new("h.size > 0", Ty::Boolean)))
}

fn p_sha256(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let arg = env.pick_or_lit(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    Some((
        vec![format!("val h = sha256({})", arg.src)],
        Term::new("h.size == 32", Ty::Boolean),
    ))
}

fn p_long_to_byte_array(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let arg = env.pick_or_lit(&Ty::Long, rng)?;
    Some((
        vec![format!("val ba = longToByteArray({})", arg.src)],
        Term::new("ba.size == 8", Ty::Boolean),
    ))
}

fn p_byte_array_to_long(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let arg = env.pick_or_lit(&Ty::Long, rng)?;
    Some((
        vec![
            format!("val ba = longToByteArray({})", arg.src),
            "val l = byteArrayToLong(ba)".into(),
        ],
        Term::new(format!("l == {}", arg.src), Ty::Boolean),
    ))
}

fn p_byte_array_to_big_int(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let arg = env.pick_or_lit(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    Some((
        vec![format!("val bi = byteArrayToBigInt({})", arg.src)],
        Term::new("bi >= bi", Ty::Boolean),
    ))
}

fn p_min(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Long, rng)?;
    let b = env.pick_or_lit(&Ty::Long, rng)?;
    Some((
        vec![format!("val m = min({}, {})", a.src, b.src)],
        Term::new(format!("m <= {}", a.src), Ty::Boolean),
    ))
}

fn p_max(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Long, rng)?;
    let b = env.pick_or_lit(&Ty::Long, rng)?;
    Some((
        vec![format!("val m = max({}, {})", a.src, b.src)],
        Term::new(format!("m >= {}", a.src), Ty::Boolean),
    ))
}

fn p_min_int(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Int, rng)?;
    let b = env.pick_or_lit(&Ty::Int, rng)?;
    Some((
        vec![format!("val m = min({}, {})", a.src, b.src)],
        Term::new(format!("m <= {}", a.src), Ty::Boolean),
    ))
}

fn p_all_of(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Boolean, rng)?;
    let b = env.pick_or_lit(&Ty::Boolean, rng)?;
    Some((
        vec![format!("val ok = allOf(Coll({}, {}))", a.src, b.src)],
        Term::new("ok || true", Ty::Boolean),
    ))
}

fn p_any_of(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Boolean, rng)?;
    let b = env.pick_or_lit(&Ty::Boolean, rng)?;
    Some((
        vec![format!("val ok = anyOf(Coll({}, {}))", a.src, b.src)],
        Term::new("ok || true", Ty::Boolean),
    ))
}

fn p_xor_of(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Boolean, rng)?;
    let b = env.pick_or_lit(&Ty::Boolean, rng)?;
    Some((
        vec![format!("val ok = xorOf(Coll({}, {}))", a.src, b.src)],
        Term::new("ok || true", Ty::Boolean),
    ))
}

fn p_at_least(_env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let n = rng.gen_range(1, 3);
    Some((
        vec![format!(
            "val sp = atLeast({}, Coll(proveDlog(groupGenerator), proveDlog(groupGenerator)))",
            n
        )],
        Term::new("sp.propBytes.size > 0", Ty::Boolean),
    ))
}

fn p_sigma_prop(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let arg = env.pick_or_lit(&Ty::Boolean, rng)?;
    Some((
        vec![format!("val sp = sigmaProp({})", arg.src)],
        Term::new("sp.propBytes.size > 0", Ty::Boolean),
    ))
}

fn p_prove_dlog(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val sp = proveDlog(groupGenerator)".into()],
        Term::new("sp.propBytes.size > 0", Ty::Boolean),
    ))
}

fn p_prove_dh_tuple(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec![
            "val g = groupGenerator".into(),
            "val sp = proveDHTuple(g, g, g, g)".into(),
        ],
        Term::new("sp.propBytes.size > 0", Ty::Boolean),
    ))
}

fn p_decode_point(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let bytes = env.pick_typed(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    Some((
        vec![format!("val gE = decodePoint({})", bytes.src)],
        Term::new("gE != gE", Ty::Boolean),
    ))
}

// `some[T]`, `none[T]`, `upcast[T]`, `downcast[T]`, `serialize`, `getVar`
// were dropped from the predef-source table after the F.2 first run because
// Scala's source-language binder doesn't expose them (the audit lists them
// as Rust-side lowering arms, not user syntax). Including them produced
// SCALA_FAIL noise that drowned the DIFF surface. The lowering arms are
// still exercised via implicit numeric promotion + Option methods.

fn p_coll_int(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Int, rng)?;
    let b = env.pick_or_lit(&Ty::Int, rng)?;
    Some((
        vec![format!("val c = Coll({}, {})", a.src, b.src)],
        Term::new("c.size == 2", Ty::Boolean),
    ))
}

fn p_coll_long(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let a = env.pick_or_lit(&Ty::Long, rng)?;
    Some((
        vec![format!("val c = Coll[Long]({})", a.src)],
        Term::new("c.size == 1", Ty::Boolean),
    ))
}

const PREDEF_BUILDERS: &[(&str, PredefBuilder)] = &[
    ("blake2b256", p_blake2b256),
    ("sha256", p_sha256),
    ("longToByteArray", p_long_to_byte_array),
    ("byteArrayToLong", p_byte_array_to_long),
    ("byteArrayToBigInt", p_byte_array_to_big_int),
    ("min_long", p_min),
    ("max_long", p_max),
    ("min_int", p_min_int),
    ("allOf", p_all_of),
    ("anyOf", p_any_of),
    ("xorOf", p_xor_of),
    ("atLeast", p_at_least),
    ("sigmaProp", p_sigma_prop),
    ("proveDlog", p_prove_dlog),
    ("proveDHTuple", p_prove_dh_tuple),
    ("decodePoint", p_decode_point),
    ("Coll_int", p_coll_int),
    ("Coll_long", p_coll_long),
];

// ---- Method builders (hand-mirrored from LOWERING-SHAPE-AUDIT.md) ----------

type MethodBuilder = fn(&mut Env, &mut Rng) -> Option<(Vec<String>, Term)>;

fn m_box_value(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((vec![], Term::new("SELF.value > 0L", Ty::Boolean)))
}

fn m_box_id(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((vec![], Term::new("SELF.id.size == 32", Ty::Boolean)))
}

fn m_box_proposition_bytes(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec![],
        Term::new("SELF.propositionBytes.size > 0", Ty::Boolean),
    ))
}

fn m_box_bytes(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((vec![], Term::new("SELF.bytes.size > 0", Ty::Boolean)))
}

fn m_box_creation_info(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val ci = SELF.creationInfo".into()],
        Term::new("ci._1 >= 0", Ty::Boolean),
    ))
}

fn m_box_tokens(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((vec![], Term::new("SELF.tokens.size >= 0", Ty::Boolean)))
}

fn m_coll_size(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((vec![], Term::new("INPUTS.size > 0", Ty::Boolean)))
}

fn m_coll_byte_size(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let bs = env.pick_typed(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    Some((
        vec![],
        Term::new(format!("{}.size > 0", bs.src), Ty::Boolean),
    ))
}

fn m_coll_indexof(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let bs = env.pick_typed(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    let needle = env.pick_or_lit(&Ty::Byte, rng)?;
    Some((
        vec![format!("val idx = {}.indexOf({}, 0)", bs.src, needle.src)],
        Term::new("idx >= -1", Ty::Boolean),
    ))
}

fn m_coll_slice(env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let bs = env.pick_typed(&Ty::Coll(Box::new(Ty::Byte)), _rng)?;
    Some((
        vec![format!("val s = {}.slice(0, 1)", bs.src)],
        Term::new("s.size <= 1", Ty::Boolean),
    ))
}

fn m_coll_append(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let bs = env.pick_typed(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    let other = lit(&Ty::Coll(Box::new(Ty::Byte)), rng)?;
    Some((
        vec![format!("val a = {}.append({})", bs.src, other.src)],
        Term::new("a.size > 0", Ty::Boolean),
    ))
}

fn m_coll_exists(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val anyValid = INPUTS.exists({(b: Box) => b.value > 0L})".into()],
        Term::new("anyValid || true", Ty::Boolean),
    ))
}

fn m_coll_forall(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val allValid = INPUTS.forall({(b: Box) => b.value >= 0L})".into()],
        Term::new("allValid", Ty::Boolean),
    ))
}

fn m_coll_map(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val vs = INPUTS.map({(b: Box) => b.value})".into()],
        Term::new("vs.size == INPUTS.size", Ty::Boolean),
    ))
}

fn m_coll_filter(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val pos = INPUTS.filter({(b: Box) => b.value > 0L})".into()],
        Term::new("pos.size <= INPUTS.size", Ty::Boolean),
    ))
}

fn m_coll_fold(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val total = INPUTS.fold(0L, {(z: Long, b: Box) => z + b.value})".into()],
        Term::new("total >= 0L", Ty::Boolean),
    ))
}

fn m_int_to_long(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let i = env.pick_or_lit(&Ty::Int, rng)?;
    Some((
        vec![format!("val l = {}.toLong", i.src)],
        Term::new("l >= 0L", Ty::Boolean),
    ))
}

fn m_int_to_bigint(env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let i = env.pick_or_lit(&Ty::Int, rng)?;
    Some((
        vec![format!("val bi = {}.toBigInt", i.src)],
        Term::new("bi >= bi", Ty::Boolean),
    ))
}

fn m_long_to_int(_env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    // Use a small literal to avoid Scala's compile-time overflow rejection.
    let v = rng.gen_range(0, 100);
    Some((
        vec![format!("val i = {}L.toInt", v)],
        Term::new("i >= 0", Ty::Boolean),
    ))
}

fn m_long_to_byte(_env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let v = rng.gen_range(0, 100);
    Some((
        vec![format!("val b = {}L.toByte", v)],
        Term::new("b >= 0.toByte", Ty::Boolean),
    ))
}

fn m_long_to_short(_env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let v = rng.gen_range(0, 100);
    Some((
        vec![format!("val s = {}L.toShort", v)],
        Term::new("s >= 0.toShort", Ty::Boolean),
    ))
}

fn m_group_multiply(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val gm = groupGenerator.multiply(groupGenerator)".into()],
        Term::new("gm != groupGenerator || gm == groupGenerator", Ty::Boolean),
    ))
}

fn m_group_exp(_env: &mut Env, rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    let scalar = lit(&Ty::BigInt, rng)?;
    Some((
        vec![format!("val ge = groupGenerator.exp({})", scalar.src)],
        Term::new("ge != groupGenerator || ge == groupGenerator", Ty::Boolean),
    ))
}

fn m_option_get_or_else(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec![
            "val o = SELF.R4[Long]".into(),
            "val v = o.getOrElse(0L)".into(),
        ],
        Term::new("v >= 0L", Ty::Boolean),
    ))
}

fn m_option_is_defined(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val o = SELF.R4[Long]".into()],
        Term::new("o.isDefined || o.isDefined == false", Ty::Boolean),
    ))
}

fn m_sigma_prop_prop_bytes(_env: &mut Env, _rng: &mut Rng) -> Option<(Vec<String>, Term)> {
    Some((
        vec!["val sp = proveDlog(groupGenerator)".into()],
        Term::new("sp.propBytes.size > 0", Ty::Boolean),
    ))
}

const METHOD_BUILDERS: &[(&str, MethodBuilder)] = &[
    ("box.value", m_box_value),
    ("box.id", m_box_id),
    ("box.propositionBytes", m_box_proposition_bytes),
    ("box.bytes", m_box_bytes),
    ("box.creationInfo", m_box_creation_info),
    ("box.tokens", m_box_tokens),
    ("coll.size", m_coll_size),
    ("coll_byte.size", m_coll_byte_size),
    ("coll_byte.indexOf", m_coll_indexof),
    ("coll_byte.slice", m_coll_slice),
    ("coll_byte.append", m_coll_append),
    ("coll.exists", m_coll_exists),
    ("coll.forall", m_coll_forall),
    ("coll.map", m_coll_map),
    ("coll.filter", m_coll_filter),
    ("coll.fold", m_coll_fold),
    ("int.toLong", m_int_to_long),
    ("int.toBigInt", m_int_to_bigint),
    ("long.toInt", m_long_to_int),
    ("long.toByte", m_long_to_byte),
    ("long.toShort", m_long_to_short),
    ("group.multiply", m_group_multiply),
    ("group.exp", m_group_exp),
    ("option.getOrElse", m_option_get_or_else),
    ("option.isDefined", m_option_is_defined),
    ("sigmaProp.propBytes", m_sigma_prop_prop_bytes),
];

// ---- Program assembly ------------------------------------------------------

/// Wrap (decls, terminal_bool) → a `{ ... ; sigmaProp(...) }` source string.
fn assemble(decls: &[String], term: &Term) -> String {
    let mut out = String::from("{\n");
    for d in decls {
        out.push_str("  ");
        out.push_str(d);
        out.push('\n');
    }
    out.push_str("  sigmaProp(");
    out.push_str(&term.src);
    out.push_str(")\n}\n");
    out
}

// ---- Pass implementations --------------------------------------------------

/// Pass 1 — predef-by-predef, ≥3 call sites each, varied arg shapes.
/// Source-of-truth: LOWERING-SHAPE-AUDIT.md §"Predef builtins".
fn gen_predef_pass(seed: u64) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let calls_per_predef = 5;
    for (idx, (name, builder)) in PREDEF_BUILDERS.iter().enumerate() {
        for k in 0..calls_per_predef {
            let mut rng = Rng::new(mix_seed(seed, (idx as u64) * 31 + k));
            let mut env = Env::fresh();
            if let Some((decls, term)) = builder(&mut env, &mut rng) {
                if term.ty == Ty::Boolean {
                    let src = assemble(&decls, &term);
                    let id = format!("predef_{:03}_{}_{}", idx, sanitize(name), k);
                    out.push((id, src));
                }
            }
        }
    }
    out
}

/// Pass 2 — method-by-method. Source-of-truth: LOWERING-SHAPE-AUDIT.md
/// §"Method dispatch" + the two registry call sites.
fn gen_method_pass(seed: u64) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let calls_per_method = 5;
    for (idx, (name, builder)) in METHOD_BUILDERS.iter().enumerate() {
        for k in 0..calls_per_method {
            let mut rng = Rng::new(mix_seed(seed ^ 0xA5A5_A5A5, (idx as u64) * 31 + k));
            let mut env = Env::fresh();
            if let Some((decls, term)) = builder(&mut env, &mut rng) {
                if term.ty == Ty::Boolean {
                    let src = assemble(&decls, &term);
                    let id = format!("method_{:03}_{}_{}", idx, sanitize(name), k);
                    out.push((id, src));
                }
            }
        }
    }
    out
}

/// Pass 3 — numeric coercion: every type pair × every binop.
/// Source-of-truth: `numeric_upcast_pair` + `narrow_upcast` enumerations
/// flagged in LOWERING-SHAPE-AUDIT.md §"Numeric coercion".
fn gen_numeric_pass(seed: u64) -> Vec<(String, String)> {
    let types: &[Ty] = &[Ty::Byte, Ty::Short, Ty::Int, Ty::Long, Ty::BigInt];
    let arith_ops = ["+", "-", "*"];
    let cmp_ops = ["<", "<=", ">", ">=", "=="];
    let mut out = Vec::new();
    let mut counter = 0;
    for (i, t1) in types.iter().enumerate() {
        for (j, t2) in types.iter().enumerate() {
            for (k, op) in arith_ops.iter().enumerate() {
                let mut rng = Rng::new(mix_seed(
                    seed ^ 0x3333_3333,
                    (i as u64) * 1000 + (j as u64) * 100 + k as u64,
                ));
                let env = Env::fresh();
                let a = match env.pick_or_lit(t1, &mut rng) {
                    Some(t) => t,
                    None => continue,
                };
                let b = match env.pick_or_lit(t2, &mut rng) {
                    Some(t) => t,
                    None => continue,
                };
                // For type pair to be valid, they must match (no implicit
                // widening across signed-numeric in source); we test
                // explicit upcast-then-binop instead.
                let (a_src, b_src, out_ty) = if t1 == t2 {
                    (a.src.clone(), b.src.clone(), t1.clone())
                } else {
                    // Cast smaller → larger via the source-language `.toX`
                    // method (Scala's user-facing surface; `upcast[T](...)`
                    // is only an internal Rust-side predef name).
                    let larger = numeric_rank_max(t1, t2);
                    let to_method = match &larger {
                        Ty::Long => ".toLong",
                        Ty::Int => ".toInt",
                        Ty::Short => ".toShort",
                        Ty::Byte => ".toByte",
                        Ty::BigInt => ".toBigInt",
                        _ => continue,
                    };
                    let a2 = if t1 == &larger {
                        a.src.clone()
                    } else {
                        format!("({}){}", a.src, to_method)
                    };
                    let b2 = if t2 == &larger {
                        b.src.clone()
                    } else {
                        format!("({}){}", b.src, to_method)
                    };
                    (a2, b2, larger)
                };
                let decls = vec![format!(
                    "val r: {} = ({}) {} ({})",
                    out_ty.ergo_str(),
                    a_src,
                    op,
                    b_src
                )];
                let zero: String = match out_ty {
                    Ty::BigInt => "byteArrayToBigInt(fromBase16(\"00\"))".to_string(),
                    Ty::Long => "0L".to_string(),
                    Ty::Int => "0".to_string(),
                    Ty::Short => "0.toShort".to_string(),
                    Ty::Byte => "0.toByte".to_string(),
                    _ => continue,
                };
                let term = Term::new(format!("r >= {}", zero), Ty::Boolean);
                let src = assemble(&decls, &term);
                let id = format!(
                    "numeric_{:03}_{}_{}_{}",
                    counter,
                    short_ty(t1),
                    short_ty(t2),
                    sanitize(op)
                );
                out.push((id, src));
                counter += 1;
            }
            // Comparison ops on like-typed pair.
            if t1 == t2 {
                for (k, op) in cmp_ops.iter().enumerate() {
                    let mut rng =
                        Rng::new(mix_seed(seed ^ 0x4444_4444, (i as u64) * 100 + k as u64));
                    let env = Env::fresh();
                    let a = match env.pick_or_lit(t1, &mut rng) {
                        Some(t) => t,
                        None => continue,
                    };
                    let b = match env.pick_or_lit(t2, &mut rng) {
                        Some(t) => t,
                        None => continue,
                    };
                    let term = Term::new(format!("({}) {} ({})", a.src, op, b.src), Ty::Boolean);
                    let src = assemble(&[], &term);
                    let id = format!(
                        "numeric_cmp_{:03}_{}_{}",
                        counter,
                        short_ty(t1),
                        sanitize(op)
                    );
                    out.push((id, src));
                    counter += 1;
                }
            }
        }
    }
    out
}

fn numeric_rank_max(a: &Ty, b: &Ty) -> Ty {
    let r = |t: &Ty| match t {
        Ty::Byte => 0,
        Ty::Short => 1,
        Ty::Int => 2,
        Ty::Long => 3,
        Ty::BigInt => 4,
        _ => -1,
    };
    if r(a) >= r(b) {
        a.clone()
    } else {
        b.clone()
    }
}

fn short_ty(t: &Ty) -> &'static str {
    match t {
        Ty::Byte => "byte",
        Ty::Short => "short",
        Ty::Int => "int",
        Ty::Long => "long",
        Ty::BigInt => "bigint",
        _ => "x",
    }
}

/// Pass 4 — composition stress (depth ≤ 3, ≤ 50 lines).
/// Random combination of 1–3 predef + method emitters in a single block,
/// chained via val bindings. Every step's locally-bound identifiers are
/// suffixed with `_s<step>` so two builders that both bind e.g. `h` don't
/// collide.
fn gen_composition_pass(seed: u64) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let n_programs = 220;
    for i in 0..n_programs {
        let mut rng = Rng::new(mix_seed(seed ^ 0x7777_7777, i as u64));
        let mut env = Env::fresh();
        let depth = (rng.gen_range(1, 4)) as usize;
        let mut decls: Vec<String> = Vec::new();
        let mut bool_terms: Vec<Term> = Vec::new();
        let mut composed = 0;
        for step in 0..depth {
            // Pick a builder (50/50 predef vs method).
            let pick_predef = rng.flip();
            let res = if pick_predef {
                let (_, b) = PREDEF_BUILDERS[rng.pick_idx(PREDEF_BUILDERS.len())];
                b(&mut env, &mut rng)
            } else {
                let (_, b) = METHOD_BUILDERS[rng.pick_idx(METHOD_BUILDERS.len())];
                b(&mut env, &mut rng)
            };
            if let Some((d, mut t)) = res {
                let suffix = format!("_s{}", step);
                let renamed = rename_locals(&d, &mut t.src, &suffix);
                decls.extend(renamed);
                if t.ty == Ty::Boolean {
                    bool_terms.push(t);
                    composed += 1;
                }
            }
        }
        if composed == 0 || decls.len() + 4 > 50 {
            continue;
        }
        // Combine the bool terms with random && / ||
        let combined = bool_terms
            .iter()
            .map(|t| format!("({})", t.src))
            .collect::<Vec<_>>()
            .join(if rng.flip() { " && " } else { " || " });
        let term = Term::new(combined, Ty::Boolean);
        let src = assemble(&decls, &term);
        // Bound at 50 lines (defensive — should never fire post-decls check)
        if src.lines().count() > 50 {
            continue;
        }
        let id = format!("composition_{:03}", i);
        out.push((id, src));
    }
    out
}

/// Rewrite every `val IDENT = …` definition in `decls` to bind `IDENT<suffix>`
/// instead, and rewrite all whole-word references to those identifiers in
/// the rest of `decls` and in `term_src`. ASCII-only (ErgoScript source is
/// ASCII), so byte iteration is safe.
fn rename_locals(decls: &[String], term_src: &mut String, suffix: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for d in decls {
        let trimmed = d.trim_start();
        if let Some(rest) = trimmed.strip_prefix("val ") {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if end > 0 {
                let n = &rest[..end];
                if !names.iter().any(|x| x == n) {
                    names.push(n.to_string());
                }
            }
        }
    }
    if names.is_empty() {
        return decls.to_vec();
    }
    let rewrite = |s: &str| -> String {
        let bytes = s.as_bytes();
        let mut out = String::with_capacity(s.len() + names.len() * 4);
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            let starts_ident = (c.is_ascii_alphabetic() || c == b'_')
                && (i == 0 || {
                    let p = bytes[i - 1];
                    !(p.is_ascii_alphanumeric() || p == b'_')
                });
            if starts_ident {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let ident = &s[start..i];
                out.push_str(ident);
                if names.iter().any(|n| n == ident) {
                    out.push_str(suffix);
                }
            } else {
                out.push(c as char);
                i += 1;
            }
        }
        out
    };
    let new_decls: Vec<String> = decls.iter().map(|d| rewrite(d)).collect();
    *term_src = rewrite(term_src);
    new_decls
}

/// Pass 5 — walker-gap targeting. Bias toward the variants flagged with
/// walker gaps in IR-PASS-COVERAGE-MATRIX.md so CSE/replace_all paths get
/// exercised. We emit each gap-variant in a context where it appears
/// inside a `val` whose binding is referenced ≥2 times → triggers CSE
/// extraction → walker passes recurse into the variant.
fn gen_walker_gap_pass(seed: u64) -> Vec<(String, String)> {
    // Each entry: (label, decl-emitter that produces a Coll[Byte] / Long /
    // Boolean expression, Boolean predicate referencing the val twice).
    type Builder = fn(&mut Rng) -> (Vec<String>, Term);
    let bs: &[(&str, Builder)] = &[
        ("byteArrayToLong", |_| {
            (
                vec![
                    "val src = SELF.id".into(),
                    "val l = byteArrayToLong(src)".into(),
                ],
                Term::new("l == l", Ty::Boolean),
            )
        }),
        ("byteArrayToBigInt", |_| {
            (
                vec!["val bi = byteArrayToBigInt(SELF.id)".into()],
                Term::new("bi == bi", Ty::Boolean),
            )
        }),
        ("createProveDlog", |_| {
            (
                vec!["val sp = proveDlog(groupGenerator)".into()],
                Term::new(
                    "sp.propBytes.size > 0 && sp.propBytes.size > 0",
                    Ty::Boolean,
                ),
            )
        }),
        ("createProveDhTuple", |_| {
            (
                vec!["val sp = proveDHTuple(groupGenerator, groupGenerator, groupGenerator, groupGenerator)".into()],
                Term::new("sp.propBytes.size > 0 && sp.propBytes.size > 0", Ty::Boolean),
            )
        }),
        ("decodePoint", |_| {
            (
                vec![
                    "val gE = decodePoint(SELF.id)".into(),
                    "val gE2 = gE.multiply(gE)".into(),
                ],
                Term::new(
                    "gE2 != groupGenerator || gE2 == groupGenerator",
                    Ty::Boolean,
                ),
            )
        }),
        ("exponentiate", |rng| {
            let s = lit(&Ty::BigInt, rng).unwrap();
            (
                vec![format!("val ge = groupGenerator.exp({})", s.src)],
                Term::new("ge != groupGenerator || ge == groupGenerator", Ty::Boolean),
            )
        }),
        ("longToByteArray", |_| {
            (
                vec![
                    "val ba = longToByteArray(SELF.value)".into(),
                    "val ba2 = ba.append(ba)".into(),
                ],
                Term::new("ba2.size >= ba.size", Ty::Boolean),
            )
        }),
        ("multiplyGroup", |_| {
            (
                vec![
                    "val g = groupGenerator.multiply(groupGenerator)".into(),
                    "val gg = g.multiply(g)".into(),
                ],
                Term::new("gg != groupGenerator || gg == groupGenerator", Ty::Boolean),
            )
        }),
        ("optionGetOrElse", |_| {
            (
                vec![
                    "val o = SELF.R4[Long]".into(),
                    "val v = o.getOrElse(0L)".into(),
                    "val w = v + v".into(),
                ],
                Term::new("w >= 0L", Ty::Boolean),
            )
        }),
    ];
    let mut out = Vec::new();
    let per_target = 8;
    for (idx, (name, builder)) in bs.iter().enumerate() {
        for k in 0..per_target {
            let mut rng = Rng::new(mix_seed(seed ^ 0xCAFE_BABE, (idx as u64) * 17 + k));
            let (decls, term) = builder(&mut rng);
            let src = assemble(&decls, &term);
            let id = format!("walker_{:03}_{}_{}", idx, sanitize(name), k);
            out.push((id, src));
        }
    }
    out
}

// ---- Driver ----------------------------------------------------------------

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn run_all_passes(seed: u64) -> Vec<(String, String)> {
    // Per-pass sub-seeds are derived inside each pass via mix_seed; passes
    // run in fixed order so the resulting Vec is deterministic.
    let mut out = Vec::new();
    out.extend(gen_predef_pass(seed));
    out.extend(gen_method_pass(seed));
    out.extend(gen_numeric_pass(seed));
    out.extend(gen_composition_pass(seed));
    out.extend(gen_walker_gap_pass(seed));
    out
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("diff_fuzz")
        .join("corpus")
}

fn write_corpus(seed: u64, programs: &[(String, String)]) -> PathBuf {
    let dir = corpus_dir();
    fs::create_dir_all(&dir).expect("mkdir corpus dir");
    // Remove existing gen_*.es files (per-seed scoping is implicit in
    // filename prefix; we wipe all gen_* on every regenerate so that
    // changing the seed doesn't leave stale entries with the prior seed
    // still in the directory).
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("gen_") && s.ends_with(".es"))
                .unwrap_or(false)
            {
                let _ = fs::remove_file(&p);
            }
        }
    }
    let seed_hex = format!("{:016x}", seed);
    for (name, src) in programs {
        let fname = format!("gen_{}_{}.es", seed_hex, name);
        let path = dir.join(&fname);
        fs::write(&path, src).unwrap_or_else(|e| panic!("write {}: {}", path.display(), e));
    }
    dir
}

// ---- Tests -----------------------------------------------------------------

#[test]
#[ignore]
fn gen_corpus() {
    let seed: u64 = std::env::var("DIFF_FUZZ_GEN_SEED")
        .ok()
        .and_then(|s| {
            if let Some(stripped) = s.strip_prefix("0x") {
                u64::from_str_radix(stripped, 16).ok()
            } else {
                s.parse::<u64>().ok()
            }
        })
        .unwrap_or(DEFAULT_SEED);
    eprintln!("=== diff_fuzz_gen ===");
    eprintln!("seed: 0x{:016x}", seed);
    let programs = run_all_passes(seed);
    eprintln!("programs generated: {}", programs.len());
    let mut counts: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    for (name, src) in &programs {
        let bucket = if name.starts_with("predef_") {
            "predef"
        } else if name.starts_with("method_") {
            "method"
        } else if name.starts_with("numeric_") {
            "numeric"
        } else if name.starts_with("composition_") {
            "composition"
        } else if name.starts_with("walker_") {
            "walker"
        } else {
            "other"
        };
        *counts.entry(bucket).or_insert(0) += 1;
        // Sanity: bound on lines.
        let n_lines = src.lines().count();
        if n_lines > 50 {
            panic!("program {} exceeds 50 lines ({})", name, n_lines);
        }
    }
    for (k, v) in &counts {
        eprintln!("  {:12}: {}", k, v);
    }
    let dir = write_corpus(seed, &programs);
    eprintln!("corpus dir: {}", dir.display());
    assert!(
        programs.len() >= 500,
        "generator emitted only {} programs (need >=500)",
        programs.len()
    );
}

#[test]
#[ignore]
fn gen_corpus_determinism() {
    // Run all passes twice with the same seed, compare full vec.
    let a = run_all_passes(DEFAULT_SEED);
    let b = run_all_passes(DEFAULT_SEED);
    assert_eq!(a.len(), b.len(), "program count differs");
    for (i, ((na, sa), (nb, sb))) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(na, nb, "program name differs at index {}", i);
        assert_eq!(sa, sb, "program source differs at index {} ({})", i, na);
    }
}

#[test]
fn no_program_exceeds_50_lines() {
    // Run with default seed; should never emit a 50+ line program.
    let programs = run_all_passes(DEFAULT_SEED);
    for (name, src) in &programs {
        let n = src.lines().count();
        assert!(n <= 50, "program {} has {} lines", name, n);
    }
}

#[test]
fn coverage_pass_counts_nonzero() {
    let programs = run_all_passes(DEFAULT_SEED);
    let buckets = ["predef_", "method_", "numeric_", "composition_", "walker_"];
    for prefix in buckets {
        let n = programs
            .iter()
            .filter(|(n, _)| n.starts_with(prefix))
            .count();
        assert!(n > 0, "pass {} produced 0 programs", prefix);
    }
}

#[test]
fn target_500_programs() {
    let programs = run_all_passes(DEFAULT_SEED);
    assert!(
        programs.len() >= 500,
        "default-seed program count {} below 500",
        programs.len()
    );
}

#[allow(dead_code)]
fn _force_path_used(_p: &Path) {}
