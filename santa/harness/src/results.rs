//! What a runner writes to `<out-dir>/results.json` (SANTA's `santa-runner-results/v1`, runner-integration §4.3): per
//! slice the files graded, the dimension counts, the outcome counts and one record per red; the totals.
use crate::grading::{Grade, Verdict};
use crate::selection::VectorFile;
use crate::Scope;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Default)]
struct Slice {
    files: Vec<String>,
    /// per dimension: (graded, nice)
    dims: BTreeMap<String, (u64, u64)>,
    not_implemented: u64,
    panicked: u64,
    exceeded: u64,
    red: Vec<Value>,
}

/// The results accumulated over a run, slice by slice.
#[derive(Default)]
pub struct Tally {
    slices: BTreeMap<String, Slice>,
}

/// A short rendering for people (never compared): the compact JSON, cut at 200 characters.
fn brief(v: &Value) -> String {
    let s = v.to_string();
    match s.char_indices().nth(200) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s,
    }
}

/// What the entry expects, for the record: its `expected`, or a wire entry's bytes.
fn expected_of(entry: &Value) -> String {
    let v = entry.get("expected").or_else(|| entry.get("expected_bytes_hex")).or_else(|| entry.get("bytes_hex"));
    brief(v.unwrap_or(&Value::Null))
}

impl Tally {
    fn slice(&mut self, f: &VectorFile) -> &mut Slice {
        self.slices.entry(format!("{}/{}/{}", f.tier, f.version, f.provenance)).or_default()
    }

    /// A file graded (listed in its slice's `files`).
    pub fn add_file(&mut self, f: &VectorFile) {
        self.slice(f).files.push(f.rel.clone());
    }

    /// One entry's verdict: its counts, and a red record for each red dimension or outcome.
    pub fn add_verdict(&mut self, f: &VectorFile, entry: &Value, actual: Option<&Value>, v: &Verdict) {
        let name = entry["name"].as_str().unwrap_or("").to_string();
        let got = actual.map_or_else(|| "missing".to_string(), brief);
        let record = |dim: &str, note: Option<&Value>| {
            let mut r = json!({"file": f.rel, "entry": name, "dim": dim, "expected": expected_of(entry), "got": got});
            if let Some(n) = note.and_then(Value::as_str) {
                r["note"] = json!(n);
            }
            r
        };
        let s = self.slice(f);
        match v {
            Verdict::Exceeded { .. } => {
                s.exceeded += 1;
                s.red.push(record("exceeded", None));
            }
            Verdict::Panicked => {
                s.panicked += 1;
                s.red.push(record("panicked", actual.and_then(|a| a.get("note"))));
            }
            Verdict::NotImplemented => {
                s.not_implemented += 1;
                s.red.push(record("not-implemented", None));
            }
            Verdict::Graded(dims) => {
                for (dim, g) in dims {
                    if *g == Grade::NotApplicable {
                        continue;
                    }
                    let c = s.dims.entry(dim.clone()).or_default();
                    c.0 += 1;
                    if *g == Grade::Nice {
                        c.1 += 1;
                    } else {
                        s.red.push(record(dim, None));
                    }
                }
            }
        }
    }

    /// One line per slice, for people: `<slice>: <dim> nice/graded … · <n> red`.
    pub fn summary(&self) -> Vec<String> {
        self.slices
            .iter()
            .map(|(k, s)| {
                let dims: Vec<String> = s.dims.iter().map(|(d, (g, n))| format!("{d} {n}/{g}")).collect();
                format!("{k}: {} · {} red", dims.join(" · "), s.red.len())
            })
            .collect()
    }

    /// `santa-runner-results/v1`.
    pub fn results_json(&self, santa: Option<String>, scope: &Scope) -> Value {
        let mut slices = Map::new();
        let (mut red, mut not_implemented) = (0usize, 0u64);
        for (k, s) in &self.slices {
            red += s.red.len();
            not_implemented += s.not_implemented;
            let dims: Map<String, Value> =
                s.dims.iter().map(|(d, (g, n))| (d.clone(), json!({"graded": g, "nice": n}))).collect();
            slices.insert(k.clone(), json!({
                "files": s.files, "dims": dims, "not_implemented": s.not_implemented, "panicked": s.panicked,
                "exceeded": s.exceeded, "red": s.red,
            }));
        }
        let mut sc = json!({"version": scope.version, "tiers": scope.tiers, "cost": scope.cost});
        if let Some(k) = &scope.kinds {
            sc["kinds"] = json!(k);
        }
        if !scope.activation {
            sc["activation"] = json!(false);
        }
        json!({
            "schema": "santa-runner-results/v1", "santa": santa, "scope": sc, "slices": slices,
            "totals": {"red": red, "not_implemented": not_implemented},
        })
    }
}
