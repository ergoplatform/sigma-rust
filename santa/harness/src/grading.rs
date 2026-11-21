// A copy of SANTA's tools/santa-check/src/grading.rs (the interpreter of vectors/grading.json, docs/contract/grading.md);
// the only change is the path of structural_equal. Keep it in step with that file; the oracle self-check catches drift.
//! The grading language (`santa-grading/v1`): every way SANTA grades an entry, as data in `vectors/grading.json`.
//! A type is a list of branches (the first whose `when` holds applies); a branch lists its dimensions, each nice
//! when its comparisons hold. The parser is strict: an unknown key, operand form, path root or schema version is
//! refused, never guessed at.

use crate::equal::structural_equal;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fmt;

/// The schema version this module reads.
pub const SCHEMA: &str = "santa-grading/v1";

/// A parsed `vectors/grading.json`: each tier's default label, and the grading types by label.
#[derive(Debug, Clone, PartialEq)]
pub struct Grading {
    pub tiers: BTreeMap<String, String>,
    pub types: BTreeMap<String, GradingType>,
}

/// One grading type: its branches in order, and the dimensions graded red when no branch matches.
#[derive(Debug, Clone, PartialEq)]
pub struct GradingType {
    pub branches: Vec<Branch>,
    pub unmatched: Vec<String>,
}

/// A branch applies when every `when` pair holds (the value at the path equals the literal).
#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    pub when: Vec<(Path, Value)>,
    pub dims: Vec<Dim>,
}

/// A dimension: nice when every comparison holds (and, if `clean`, the actual's error is null); `requires` names
/// earlier dimensions that must be nice; `claimed` grades it only for a runner that claims cost.
#[derive(Debug, Clone, PartialEq)]
pub struct Dim {
    pub name: String,
    pub compare: Vec<Comparison>,
    pub requires: Vec<String>,
    pub claimed: bool,
    pub clean: bool,
}

/// Two operands that must be equal; `case_insensitive` compares two strings ignoring ASCII case, `nonempty` makes a
/// null or empty operand (`""`, `[]`, `{}`) fail.
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    pub a: Operand,
    pub b: Operand,
    pub case_insensitive: bool,
    pub nonempty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Path(Path),
    Lit(Value),
    First(Vec<Path>),
    Pluck { list: Path, field: String },
}

/// A path into the actual, the entry's expected (`expected.…` = `entry.expected.…`), the entry, or the vector file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub root: Root,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Root {
    Actual,
    Expected,
    Entry,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GradingError {
    UnknownSchema(String),
    UnknownKey { at: String, key: String },
    Malformed { at: String, why: String },
    UnknownLabel(String),
    NoTierDefault(String),
    NoBranch { label: String },
}

impl fmt::Display for GradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GradingError::UnknownSchema(s) => write!(f, "grading: unknown schema {s:?} (this reads {SCHEMA})"),
            GradingError::UnknownKey { at, key } => write!(f, "grading: unknown key {key:?} at {at}"),
            GradingError::Malformed { at, why } => write!(f, "grading: {at}: {why}"),
            GradingError::UnknownLabel(l) => write!(f, "grading: no grading type {l:?}"),
            GradingError::NoTierDefault(t) => write!(f, "grading: the tier {t:?} has no default label"),
            GradingError::NoBranch { label } => write!(f, "grading: no branch of {label:?} matches the entry"),
        }
    }
}

impl std::error::Error for GradingError {}

fn malformed(at: &str, why: impl Into<String>) -> GradingError {
    GradingError::Malformed { at: at.to_string(), why: why.into() }
}

/// An object whose keys are all in `allowed`.
fn object<'a>(v: &'a Value, at: &str, allowed: &[&str]) -> Result<&'a Map<String, Value>, GradingError> {
    let m = v.as_object().ok_or_else(|| malformed(at, "not an object"))?;
    if let Some(k) = m.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(GradingError::UnknownKey { at: at.to_string(), key: k.clone() });
    }
    Ok(m)
}

fn string_list(v: Option<&Value>, at: &str) -> Result<Vec<String>, GradingError> {
    match v {
        None => Ok(Vec::new()),
        Some(Value::Array(a)) => a
            .iter()
            .map(|s| s.as_str().map(str::to_string).ok_or_else(|| malformed(at, "not a list of strings")))
            .collect(),
        Some(_) => Err(malformed(at, "not a list of strings")),
    }
}

fn flag(m: &Map<String, Value>, key: &str, default: bool, at: &str) -> Result<bool, GradingError> {
    match m.get(key) {
        None => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(malformed(&format!("{at}.{key}"), "not a boolean")),
    }
}

impl Path {
    fn parse(s: &str, at: &str) -> Result<Path, GradingError> {
        let mut parts = s.split('.');
        let root = match parts.next() {
            Some("actual") => Root::Actual,
            Some("expected") => Root::Expected,
            Some("entry") => Root::Entry,
            Some("file") => Root::File,
            _ => return Err(malformed(at, format!("path {s:?} does not start with actual, expected, entry or file"))),
        };
        let keys: Vec<String> = parts.map(str::to_string).collect();
        if keys.iter().any(String::is_empty) {
            return Err(malformed(at, format!("path {s:?} has an empty segment")));
        }
        Ok(Path { root, keys })
    }
}

impl Operand {
    fn parse(v: &Value, at: &str) -> Result<Operand, GradingError> {
        let m = object(v, at, &["path", "lit", "first", "pluck", "field"])?;
        let forms = ["path", "lit", "first", "pluck"].iter().filter(|k| m.contains_key(**k)).count();
        if forms != 1 {
            return Err(malformed(at, "an operand has exactly one of path, lit, first, pluck"));
        }
        if m.contains_key("field") != m.contains_key("pluck") {
            return Err(malformed(at, "field goes with pluck, and only with it"));
        }
        let path = |v: &Value, at: &str| -> Result<Path, GradingError> {
            Path::parse(v.as_str().ok_or_else(|| malformed(at, "a path is a string"))?, at)
        };
        if let Some(p) = m.get("path") {
            Ok(Operand::Path(path(p, at)?))
        } else if let Some(l) = m.get("lit") {
            Ok(Operand::Lit(l.clone()))
        } else if let Some(f) = m.get("first") {
            let list = f.as_array().filter(|a| !a.is_empty()).ok_or_else(|| malformed(at, "first is a list of paths"))?;
            Ok(Operand::First(list.iter().map(|p| path(p, at)).collect::<Result<_, _>>()?))
        } else {
            let field = m["field"].as_str().ok_or_else(|| malformed(at, "field is a string"))?.to_string();
            Ok(Operand::Pluck { list: path(&m["pluck"], at)?, field })
        }
    }
}

impl Comparison {
    fn parse(v: &Value, at: &str) -> Result<Comparison, GradingError> {
        let m = object(v, at, &["a", "b", "case_insensitive", "nonempty"])?;
        let operand = |k: &str| -> Result<Operand, GradingError> {
            Operand::parse(m.get(k).ok_or_else(|| malformed(at, format!("missing {k}")))?, &format!("{at}.{k}"))
        };
        Ok(Comparison {
            a: operand("a")?,
            b: operand("b")?,
            case_insensitive: flag(m, "case_insensitive", false, at)?,
            nonempty: flag(m, "nonempty", false, at)?,
        })
    }
}

impl Dim {
    fn parse(v: &Value, at: &str, earlier: &[Dim]) -> Result<Dim, GradingError> {
        let m = object(v, at, &["name", "compare", "requires", "claimed", "clean"])?;
        let name = m.get("name").and_then(Value::as_str).ok_or_else(|| malformed(at, "a dimension needs a name"))?;
        if earlier.iter().any(|d| d.name == name) {
            return Err(malformed(at, format!("the dimension {name:?} is listed twice")));
        }
        let compare = m
            .get("compare")
            .and_then(Value::as_array)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| malformed(at, "compare is a non-empty list"))?
            .iter()
            .enumerate()
            .map(|(i, c)| Comparison::parse(c, &format!("{at}.compare[{i}]")))
            .collect::<Result<Vec<_>, _>>()?;
        let requires = string_list(m.get("requires"), &format!("{at}.requires"))?;
        if let Some(r) = requires.iter().find(|r| !earlier.iter().any(|d| &d.name == *r)) {
            return Err(malformed(at, format!("requires {r:?}, which is not an earlier dimension of the branch")));
        }
        Ok(Dim {
            name: name.to_string(),
            compare,
            requires,
            claimed: flag(m, "claimed", false, at)?,
            clean: flag(m, "clean", true, at)?,
        })
    }
}

impl Branch {
    fn parse(v: &Value, at: &str) -> Result<Branch, GradingError> {
        let m = object(v, at, &["when", "dims"])?;
        let when = match m.get("when") {
            None => Vec::new(),
            Some(w) => {
                let at = format!("{at}.when");
                let pairs = w.as_object().ok_or_else(|| malformed(&at, "when is an object of path: literal"))?;
                pairs.iter().map(|(k, lit)| Ok((Path::parse(k, &at)?, lit.clone()))).collect::<Result<_, _>>()?
            }
        };
        let raw = m.get("dims").and_then(Value::as_array).filter(|a| !a.is_empty());
        let raw = raw.ok_or_else(|| malformed(at, "dims is a non-empty list"))?;
        let mut dims: Vec<Dim> = Vec::new();
        for (i, d) in raw.iter().enumerate() {
            let dim = Dim::parse(d, &format!("{at}.dims[{i}]"), &dims)?;
            dims.push(dim);
        }
        Ok(Branch { when, dims })
    }
}

impl GradingType {
    fn parse(v: &Value, at: &str) -> Result<GradingType, GradingError> {
        let m = object(v, at, &["branches", "unmatched"])?;
        let branches = m
            .get("branches")
            .and_then(Value::as_array)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| malformed(at, "branches is a non-empty list"))?
            .iter()
            .enumerate()
            .map(|(i, b)| Branch::parse(b, &format!("{at}.branches[{i}]")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(GradingType { branches, unmatched: string_list(m.get("unmatched"), &format!("{at}.unmatched"))? })
    }
}

impl Grading {
    /// Parses a `vectors/grading.json` document, refusing anything this version does not know.
    pub fn parse(doc: &Value) -> Result<Grading, GradingError> {
        let m = object(doc, "grading", &["schema", "tiers", "types"])?;
        match m.get("schema").and_then(Value::as_str) {
            Some(SCHEMA) => {}
            Some(other) => return Err(GradingError::UnknownSchema(other.to_string())),
            None => return Err(malformed("grading", "missing schema")),
        }
        let types_obj = m.get("types").and_then(Value::as_object).ok_or_else(|| malformed("grading", "types is an object"))?;
        let mut types = BTreeMap::new();
        for (label, t) in types_obj {
            types.insert(label.clone(), GradingType::parse(t, &format!("types.{label}"))?);
        }
        let tiers_obj = m.get("tiers").and_then(Value::as_object).ok_or_else(|| malformed("grading", "tiers is an object"))?;
        let mut tiers = BTreeMap::new();
        for (tier, label) in tiers_obj {
            let label = label.as_str().ok_or_else(|| malformed(&format!("tiers.{tier}"), "a label is a string"))?;
            if !types.contains_key(label) {
                return Err(malformed(&format!("tiers.{tier}"), format!("names no grading type {label:?}")));
            }
            tiers.insert(tier.clone(), label.to_string());
        }
        Ok(Grading { tiers, types })
    }

    /// The label an entry of `tier` in the vector file `file` is graded by: the file's own `grading`, else the tier's
    /// default.
    pub fn label_for(&self, tier: &str, file: &Value) -> Result<&str, GradingError> {
        match file.get("grading") {
            Some(Value::String(l)) => self
                .types
                .get_key_value(l.as_str())
                .map(|(k, _)| k.as_str())
                .ok_or_else(|| GradingError::UnknownLabel(l.clone())),
            Some(_) => Err(malformed("grading", "a vector file's grading is a string")),
            None => self.tiers.get(tier).map(String::as_str).ok_or_else(|| GradingError::NoTierDefault(tier.to_string())),
        }
    }
}

// ── the interpreter ─────────────────────────────────────────────────────────────────────────────────────────────

/// One dimension's grade: `NotApplicable` is not graded (not counted).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Nice,
    Red,
    NotApplicable,
}

impl Grade {
    pub fn as_str(self) -> &'static str {
        match self {
            Grade::Nice => "nice",
            Grade::Red => "red",
            Grade::NotApplicable => "n/a",
        }
    }
}

/// An entry's verdict: one of the global outcomes, or its dimensions in the branch's order.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Exceeded { limit: Value },
    Panicked,
    NotImplemented,
    Graded(Vec<(String, Grade)>),
}

impl Verdict {
    /// The verdict as the oracle writes it. Only graded dimensions appear: an `n/a` one is not graded, so an oracle
    /// verdict does not depend on which ungraded dimensions a branch happens to list.
    pub fn to_json(&self) -> Value {
        match self {
            Verdict::Exceeded { limit } => serde_json::json!({ "outcome": "exceeded", "limit": limit }),
            Verdict::Panicked => serde_json::json!({ "outcome": "panicked" }),
            Verdict::NotImplemented => serde_json::json!({ "outcome": "not-implemented" }),
            Verdict::Graded(dims) => {
                let dims: Map<String, Value> = dims
                    .iter()
                    .filter(|(_, g)| *g != Grade::NotApplicable)
                    .map(|(n, g)| (n.clone(), Value::from(g.as_str())))
                    .collect();
                serde_json::json!({ "outcome": "graded", "dims": dims })
            }
        }
    }
}

/// Where an entry's grading goes: a global outcome (exceeded / panicked / not-implemented), a branch by its index,
/// or the type's `unmatched` dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Outcome,
    Branch(usize),
    Unmatched,
}

/// What one entry is graded from: the vector entry (with its `expected`), the runner's actual (absent = `{}`), the
/// vector file's envelope, and whether the runner claims the cost dimension.
pub struct Input<'a> {
    pub entry: &'a Value,
    pub actual: Option<&'a Value>,
    pub file: &'a Value,
    pub claims_cost: bool,
}

struct Ctx<'v> {
    actual: &'v Value,
    entry: &'v Value,
    file: &'v Value,
}

impl Path {
    fn resolve<'v>(&self, ctx: &Ctx<'v>) -> Option<&'v Value> {
        let start = match self.root {
            Root::Actual => ctx.actual,
            Root::Expected => ctx.entry.get("expected")?,
            Root::Entry => ctx.entry,
            Root::File => ctx.file,
        };
        self.keys.iter().try_fold(start, |v, k| v.get(k.as_str()))
    }
}

impl Operand {
    fn eval(&self, ctx: &Ctx) -> Value {
        match self {
            Operand::Path(p) => p.resolve(ctx).cloned().unwrap_or(Value::Null),
            Operand::Lit(v) => v.clone(),
            Operand::First(paths) => paths.iter().find_map(|p| p.resolve(ctx)).cloned().unwrap_or(Value::Null),
            Operand::Pluck { list, field } => match list.resolve(ctx) {
                Some(Value::Array(items)) => {
                    Value::Array(items.iter().map(|i| i.get(field.as_str()).cloned().unwrap_or(Value::Null)).collect())
                }
                _ => Value::Null,
            },
        }
    }
}

impl Comparison {
    fn holds(&self, ctx: &Ctx) -> bool {
        let (a, b) = (self.a.eval(ctx), self.b.eval(ctx));
        let empty = |v: &Value| match v {
            Value::Null => true,
            Value::String(s) => s.is_empty(),
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            _ => false,
        };
        if self.nonempty && (empty(&a) || empty(&b)) {
            return false;
        }
        if self.case_insensitive {
            matches!((&a, &b), (Value::String(x), Value::String(y)) if x.eq_ignore_ascii_case(y))
        } else {
            structural_equal(&a, &b)
        }
    }
}

impl Grading {
    /// Grades one entry by the type `label`: the global outcomes first, then the first branch whose `when` holds.
    /// Where an entry's grading goes under the type `label`: a global outcome, the first branch whose `when` holds, or
    /// the type's `unmatched` dimensions.
    pub fn route(&self, label: &str, input: &Input) -> Result<Route, GradingError> {
        let empty = Value::Object(Map::new());
        let actual = input.actual.unwrap_or(&empty);
        self.route_in(label, &Ctx { actual, entry: input.entry, file: input.file })
    }

    fn route_in(&self, label: &str, ctx: &Ctx) -> Result<Route, GradingError> {
        let t = self.types.get(label).ok_or_else(|| GradingError::UnknownLabel(label.to_string()))?;
        if matches!(ctx.actual.get("error").and_then(Value::as_str), Some("exceeded" | "panicked" | "not-implemented")) {
            return Ok(Route::Outcome);
        }
        let holds = |b: &Branch| b.when.iter().all(|(p, lit)| structural_equal(p.resolve(ctx).unwrap_or(&Value::Null), lit));
        match t.branches.iter().position(holds) {
            Some(i) => Ok(Route::Branch(i)),
            None if t.unmatched.is_empty() => Err(GradingError::NoBranch { label: label.to_string() }),
            None => Ok(Route::Unmatched),
        }
    }

    /// Grades one entry by the type `label`: the global outcomes first, then the first branch whose `when` holds.
    pub fn grade(&self, label: &str, input: &Input) -> Result<Verdict, GradingError> {
        let empty = Value::Object(Map::new());
        let actual = input.actual.unwrap_or(&empty);
        let ctx = Ctx { actual, entry: input.entry, file: input.file };
        let t = &self.types[label];
        let branch = match self.route_in(label, &ctx)? {
            Route::Outcome => {
                return Ok(match actual["error"].as_str() {
                    Some("exceeded") => Verdict::Exceeded { limit: actual.get("limit").cloned().unwrap_or(Value::Null) },
                    Some("panicked") => Verdict::Panicked,
                    _ => Verdict::NotImplemented,
                });
            }
            Route::Unmatched => {
                return Ok(Verdict::Graded(t.unmatched.iter().map(|d| (d.clone(), Grade::Red)).collect()));
            }
            Route::Branch(i) => &t.branches[i],
        };
        let clean = actual.get("error").is_none_or(Value::is_null);
        let mut dims: Vec<(String, Grade)> = Vec::with_capacity(branch.dims.len());
        for d in &branch.dims {
            let required = d.requires.iter().all(|r| dims.iter().any(|(n, g)| n == r && *g == Grade::Nice));
            let claimed = !d.claimed || (input.claims_cost && !d.compare[0].b.eval(&ctx).is_null());
            let grade = if !required || !claimed {
                Grade::NotApplicable
            } else if (!d.clean || clean) && d.compare.iter().all(|c| c.holds(&ctx)) {
                Grade::Nice
            } else {
                Grade::Red
            };
            dims.push((d.name.clone(), grade));
        }
        Ok(Verdict::Graded(dims))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// One tier, one type: a reject branch (when) and a catch-all with a dependent dimension.
    fn minimal() -> serde_json::Value {
        json!({
            "schema": "santa-grading/v1",
            "tiers": { "eval": "eval" },
            "types": { "eval": { "branches": [
                { "when": { "expected.error": "errored" },
                  "dims": [ { "name": "reject", "clean": false,
                              "compare": [ { "a": { "path": "actual.error" }, "b": { "lit": "errored" } } ] } ] },
                { "dims": [
                    { "name": "value", "compare": [ { "a": { "path": "actual.value" }, "b": { "path": "expected.value" } } ] },
                    { "name": "cost", "requires": ["value"], "claimed": true,
                      "compare": [ { "a": { "path": "actual.cost" }, "b": { "path": "expected.cost" } } ] } ] }
            ] } }
        })
    }

    /// The minimal file with the value at `pointer` set (its parent must exist; the last key may be new).
    fn with(mut doc: serde_json::Value, pointer: &str, v: serde_json::Value) -> serde_json::Value {
        let (parent, key) = pointer.rsplit_once('/').expect("a JSON pointer");
        let parent = doc.pointer_mut(parent).expect("pointer into the minimal file");
        match parent {
            serde_json::Value::Object(m) => {
                m.insert(key.to_string(), v);
            }
            serde_json::Value::Array(a) => a[key.parse::<usize>().expect("an index")] = v,
            _ => panic!("{pointer}: parent is neither an object nor an array"),
        }
        doc
    }

    #[test]
    fn parses_a_minimal_file() {
        let g = Grading::parse(&minimal()).expect("parses");
        assert_eq!(g.tiers.get("eval").map(String::as_str), Some("eval"));
        let t = &g.types["eval"];
        assert_eq!(t.branches.len(), 2);
        assert!(t.unmatched.is_empty());
        assert_eq!(t.branches[0].when.len(), 1);
        assert_eq!(t.branches[0].when[0].0, Path { root: Root::Expected, keys: vec!["error".into()] });
        assert_eq!(t.branches[0].when[0].1, json!("errored"));
        let reject = &t.branches[0].dims[0];
        assert_eq!((reject.name.as_str(), reject.clean, reject.claimed), ("reject", false, false));
        assert_eq!(reject.compare[0].b, Operand::Lit(json!("errored")));
        assert!(t.branches[1].when.is_empty());
        let names: Vec<&str> = t.branches[1].dims.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["value", "cost"]);
        let cost = &t.branches[1].dims[1];
        assert_eq!((cost.requires.clone(), cost.claimed, cost.clean), (vec!["value".to_string()], true, true));
        assert!(!cost.compare[0].case_insensitive);
    }

    #[test]
    fn refuses_an_unknown_schema() {
        let e = Grading::parse(&with(minimal(), "/schema", json!("santa-grading/v2"))).unwrap_err();
        assert_eq!(e, GradingError::UnknownSchema("santa-grading/v2".into()));
    }

    #[test]
    fn refuses_an_unknown_key() {
        let doc = with(minimal(), "/types/eval/branches/1/dims/1/requries", json!(["value"]));
        match Grading::parse(&doc).unwrap_err() {
            GradingError::UnknownKey { key, .. } => assert_eq!(key, "requries"),
            e => panic!("{e}"),
        }
    }

    #[test]
    fn refuses_an_operand_with_two_forms() {
        let doc = with(minimal(), "/types/eval/branches/1/dims/0/compare/0/a", json!({ "path": "actual.x", "lit": 1 }));
        assert!(matches!(Grading::parse(&doc).unwrap_err(), GradingError::Malformed { .. }));
    }

    #[test]
    fn refuses_an_unknown_path_root() {
        let doc = with(minimal(), "/types/eval/branches/1/dims/0/compare/0/a", json!({ "path": "result.x" }));
        assert!(matches!(Grading::parse(&doc).unwrap_err(), GradingError::Malformed { .. }));
    }

    #[test]
    fn refuses_requires_of_a_later_dim() {
        let doc = with(minimal(), "/types/eval/branches/1/dims/0/requires", json!(["cost"]));
        assert!(matches!(Grading::parse(&doc).unwrap_err(), GradingError::Malformed { .. }));
    }

    #[test]
    fn label_for_prefers_the_file_envelope() {
        let mut doc = minimal();
        doc["types"]["x"] = doc["types"]["eval"].clone();
        let g = Grading::parse(&doc).unwrap();
        assert_eq!(g.label_for("eval", &json!({ "grading": "x" })).unwrap(), "x");
        assert_eq!(g.label_for("eval", &json!({})).unwrap(), "eval");
    }

    #[test]
    fn label_for_refuses_an_unknown_label() {
        let g = Grading::parse(&minimal()).unwrap();
        assert_eq!(g.label_for("eval", &json!({ "grading": "nope" })).unwrap_err(), GradingError::UnknownLabel("nope".into()));
    }

    // ── the interpreter ─────────────────────────────────────────────────────────────────────────────────────────

    /// A grading file with one type, `t`, the default of the tier `t`.
    fn one(t: serde_json::Value) -> Grading {
        Grading::parse(&json!({ "schema": "santa-grading/v1", "tiers": { "t": "t" }, "types": { "t": t } })).expect("parses")
    }

    fn grade_with(g: &Grading, entry: serde_json::Value, actual: Option<serde_json::Value>, file: serde_json::Value, claims_cost: bool) -> Result<Verdict, GradingError> {
        g.grade("t", &Input { entry: &entry, actual: actual.as_ref(), file: &file, claims_cost })
    }

    fn graded(dims: &[(&str, Grade)]) -> Verdict {
        Verdict::Graded(dims.iter().map(|(n, g)| (n.to_string(), *g)).collect())
    }

    fn cmp(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
        json!({ "a": a, "b": b })
    }

    fn eval_type() -> Grading {
        Grading::parse(&minimal()).map(|mut g| {
            g.tiers.insert("t".into(), "eval".into());
            g.types.insert("t".into(), g.types["eval"].clone());
            g
        }).unwrap()
    }

    #[test]
    fn precedence_beats_every_branch() {
        let g = eval_type();
        let errored = json!({ "expected": { "error": "errored" } });
        let v = grade_with(&g, errored.clone(), Some(json!({ "error": "exceeded", "limit": "memory" })), json!({}), true).unwrap();
        assert_eq!(v, Verdict::Exceeded { limit: json!("memory") });
        assert_eq!(grade_with(&g, errored.clone(), Some(json!({ "error": "panicked", "note": "x" })), json!({}), true).unwrap(), Verdict::Panicked);
        assert_eq!(grade_with(&g, errored, Some(json!({ "error": "not-implemented" })), json!({}), true).unwrap(), Verdict::NotImplemented);
    }

    #[test]
    fn the_first_matching_branch_applies() {
        let g = eval_type();
        let v = grade_with(&g, json!({ "expected": { "error": "errored" } }), Some(json!({ "error": "errored" })), json!({}), true).unwrap();
        assert_eq!(v, graded(&[("reject", Grade::Nice)]));
        let entry = json!({ "expected": { "value": 1, "cost": 5 } });
        let v = grade_with(&g, entry, Some(json!({ "value": 1, "cost": 5, "error": null })), json!({}), true).unwrap();
        assert_eq!(v, graded(&[("value", Grade::Nice), ("cost", Grade::Nice)]));
    }

    #[test]
    fn a_missing_actual_is_the_empty_object() {
        let v = grade_with(&eval_type(), json!({ "expected": { "value": 1, "cost": 5 } }), None, json!({}), true).unwrap();
        assert_eq!(v, graded(&[("value", Grade::Red), ("cost", Grade::NotApplicable)]));
    }

    #[test]
    fn clean_requires_a_null_error() {
        let entry = json!({ "expected": { "value": 1, "cost": 5 } });
        let v = grade_with(&eval_type(), entry, Some(json!({ "value": 1, "cost": 5, "error": "errored" })), json!({}), true).unwrap();
        assert_eq!(v, graded(&[("value", Grade::Red), ("cost", Grade::NotApplicable)]));
    }

    #[test]
    fn a_reject_dim_is_not_clean() {
        let entry = json!({ "expected": { "error": "errored" } });
        let v = grade_with(&eval_type(), entry.clone(), Some(json!({ "error": "errored" })), json!({}), true).unwrap();
        assert_eq!(v, graded(&[("reject", Grade::Nice)]));
        let v = grade_with(&eval_type(), entry, Some(json!({ "value": 1, "error": null })), json!({}), true).unwrap();
        assert_eq!(v, graded(&[("reject", Grade::Red)]));
    }

    #[test]
    fn numbers_compare_numerically() {
        let entry = json!({ "expected": { "value": 5, "cost": 5 } });
        let v = grade_with(&eval_type(), entry, Some(json!({ "value": 5.0, "cost": 5, "error": null })), json!({}), false).unwrap();
        assert_eq!(v, graded(&[("value", Grade::Nice), ("cost", Grade::NotApplicable)]));
    }

    #[test]
    fn case_insensitive_needs_two_strings() {
        let mut c = cmp(json!({ "path": "actual.h" }), json!({ "path": "expected.h" }));
        c["case_insensitive"] = json!(true);
        let g = one(json!({ "branches": [ { "dims": [ { "name": "h", "compare": [c] } ] } ] }));
        let nice = grade_with(&g, json!({ "expected": { "h": "ab" } }), Some(json!({ "h": "AB" })), json!({}), false).unwrap();
        assert_eq!(nice, graded(&[("h", Grade::Nice)]));
        let red = grade_with(&g, json!({ "expected": {} }), Some(json!({})), json!({}), false).unwrap();
        assert_eq!(red, graded(&[("h", Grade::Red)]));
    }

    #[test]
    fn first_takes_the_first_present_path() {
        let b = json!({ "first": ["entry.expected_bytes_hex", "entry.bytes_hex"] });
        let g = one(json!({ "branches": [ { "dims": [ { "name": "roundtrip", "compare": [cmp(json!({ "path": "actual.bytes_hex" }), b)] } ] } ] }));
        let v = grade_with(&g, json!({ "bytes_hex": "01" }), Some(json!({ "bytes_hex": "01" })), json!({}), false).unwrap();
        assert_eq!(v, graded(&[("roundtrip", Grade::Nice)]));
        let v = grade_with(&g, json!({ "bytes_hex": "01", "expected_bytes_hex": "02" }), Some(json!({ "bytes_hex": "01" })), json!({}), false).unwrap();
        assert_eq!(v, graded(&[("roundtrip", Grade::Red)]));
    }

    #[test]
    fn pluck_maps_a_list() {
        let a = json!({ "pluck": "file.chain", "field": "interlinks" });
        let g = one(json!({ "branches": [ { "dims": [ { "name": "interlinks", "compare": [cmp(a, json!({ "path": "actual.interlinks" }))] } ] } ] }));
        let file = json!({ "chain": [ { "interlinks": [1] }, { "interlinks": [2] } ] });
        let v = grade_with(&g, json!({}), Some(json!({ "interlinks": [[1], [2]] })), file.clone(), false).unwrap();
        assert_eq!(v, graded(&[("interlinks", Grade::Nice)]));
        let v = grade_with(&g, json!({}), Some(json!({ "interlinks": [[1], [3]] })), file, false).unwrap();
        assert_eq!(v, graded(&[("interlinks", Grade::Red)]));
    }

    #[test]
    fn requires_makes_a_dim_not_applicable() {
        let g = one(json!({ "branches": [ { "dims": [
            { "name": "valid", "compare": [cmp(json!({ "path": "actual.valid" }), json!({ "lit": true }))] },
            { "name": "post_digest", "requires": ["valid"],
              "compare": [cmp(json!({ "path": "actual.post_digest" }), json!({ "path": "expected.post_digest" }))] } ] } ] }));
        let v = grade_with(&g, json!({ "expected": { "post_digest": "aa" } }), Some(json!({ "valid": false, "post_digest": "aa" })), json!({}), false).unwrap();
        assert_eq!(v, graded(&[("valid", Grade::Red), ("post_digest", Grade::NotApplicable)]));
    }

    #[test]
    fn claimed_needs_the_claim_and_an_expected() {
        let g = one(json!({ "branches": [ { "dims": [
            { "name": "cost", "claimed": true, "compare": [cmp(json!({ "path": "actual.cost" }), json!({ "path": "expected.cost" }))] } ] } ] }));
        let five = json!({ "expected": { "cost": 5 } });
        assert_eq!(grade_with(&g, five.clone(), Some(json!({ "cost": 5 })), json!({}), false).unwrap(), graded(&[("cost", Grade::NotApplicable)]));
        assert_eq!(grade_with(&g, json!({ "expected": { "cost": null } }), Some(json!({ "cost": 5 })), json!({}), true).unwrap(), graded(&[("cost", Grade::NotApplicable)]));
        assert_eq!(grade_with(&g, five, Some(json!({ "cost": null })), json!({}), true).unwrap(), graded(&[("cost", Grade::Red)]));
    }

    #[test]
    fn unmatched_dims_are_red() {
        let g = one(json!({ "branches": [ { "when": { "entry.kind": "a" },
            "dims": [ { "name": "value", "compare": [cmp(json!({ "path": "actual.v" }), json!({ "path": "expected.v" }))] } ] } ],
            "unmatched": ["value"] }));
        let v = grade_with(&g, json!({ "kind": "b", "expected": { "v": 1 } }), Some(json!({ "v": 1 })), json!({}), false).unwrap();
        assert_eq!(v, graded(&[("value", Grade::Red)]));
    }

    #[test]
    fn no_branch_and_no_unmatched_is_an_error() {
        let g = one(json!({ "branches": [ { "when": { "entry.kind": "a" },
            "dims": [ { "name": "value", "compare": [cmp(json!({ "path": "actual.v" }), json!({ "path": "expected.v" }))] } ] } ] }));
        let e = grade_with(&g, json!({ "kind": "b" }), Some(json!({})), json!({}), false).unwrap_err();
        assert_eq!(e, GradingError::NoBranch { label: "t".into() });
    }

    #[test]
    fn nonempty_fails_null_and_empty() {
        let mut c = cmp(json!({ "path": "actual.d" }), json!({ "path": "expected.d" }));
        c["nonempty"] = json!(true);
        let g = one(json!({ "branches": [ { "dims": [ { "name": "d", "compare": [c] } ] } ] }));
        for (e, a) in [(json!(null), json!(null)), (json!(""), json!("")), (json!([]), json!([])), (json!({}), json!({}))] {
            let v = grade_with(&g, json!({ "expected": { "d": e } }), Some(json!({ "d": a })), json!({}), false).unwrap();
            assert_eq!(v, graded(&[("d", Grade::Red)]));
        }
        let v = grade_with(&g, json!({ "expected": { "d": "ab" } }), Some(json!({ "d": "ab" })), json!({}), false).unwrap();
        assert_eq!(v, graded(&[("d", Grade::Nice)]));
    }

    #[test]
    fn first_takes_the_first_present_path_even_when_null() {
        let b = json!({ "first": ["entry.expected_bytes_hex", "entry.bytes_hex"] });
        let mut c = cmp(json!({ "path": "actual.bytes_hex" }), b);
        c["case_insensitive"] = json!(true);
        let g = one(json!({ "branches": [ { "dims": [ { "name": "roundtrip", "compare": [c] } ] } ] }));
        let v = grade_with(&g, json!({ "bytes_hex": "01", "expected_bytes_hex": null }), Some(json!({ "bytes_hex": "01" })), json!({}), false).unwrap();
        assert_eq!(v, graded(&[("roundtrip", Grade::Red)]));
    }

    #[test]
    fn refuses_a_tier_default_naming_no_type_and_says_which_tier() {
        match Grading::parse(&with(minimal(), "/tiers/eval", json!("nope"))).unwrap_err() {
            GradingError::Malformed { at, why } => {
                assert_eq!(at, "tiers.eval");
                assert!(why.contains("nope"), "{why}");
            }
            e => panic!("{e}"),
        }
    }
}

