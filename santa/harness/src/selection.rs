//! Selection (runner-integration §4.1): which vector files of a SANTA checkout a scope covers.
use crate::{Scope, VERSIONS};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// A vector file, by its path relative to `vectors/` and the slice it sits in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorFile {
    pub rel: String,
    pub tier: String,
    pub version: String,
    pub provenance: String,
}

/// Every `*.json` under `vectors/` but the grading rules at its root, sorted by path; a file that is not at
/// `<tier>/<version>/<provenance>/<op>.json` is refused.
pub fn discover(vectors: &Path) -> Result<Vec<VectorFile>, String> {
    fn walk(d: &Path, base: &Path, out: &mut Vec<String>) {
        let Ok(rd) = fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, base, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("json") {
                out.push(p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut rels = Vec::new();
    walk(vectors, vectors, &mut rels);
    rels.retain(|r| r != "grading.json");
    rels.sort();
    rels.into_iter()
        .map(|rel| {
            let p: Vec<&str> = rel.split('/').collect();
            if p.len() != 4 {
                return Err(format!("vectors/{rel}: not at <tier>/<version>/<provenance>/<op>.json"));
            }
            Ok(VectorFile { tier: p[0].into(), version: p[1].into(), provenance: p[2].into(), rel })
        })
        .collect()
}

fn version_index(v: &str) -> Option<usize> {
    VERSIONS.iter().position(|x| *x == v)
}

/// The files `scope` selects: its tiers; the `any` slot, or a slot at or below its version; and for a wire file, its
/// one kind among `kinds` (when declared) and, when `activation` is false, not an `activation_rule: true` envelope.
pub fn select(files: &[VectorFile], vectors: &Path, scope: &Scope) -> Result<Vec<VectorFile>, String> {
    let vmax = version_index(&scope.version).ok_or(format!("unknown version {:?}", scope.version))?;
    let mut out = Vec::new();
    for f in files {
        let slot = f.version == "any" || version_index(&f.version).is_some_and(|i| i <= vmax);
        if !scope.tiers.contains(&f.tier) || !slot {
            continue;
        }
        if f.tier == "wire" && (scope.kinds.is_some() || !scope.activation) {
            let text = fs::read_to_string(vectors.join(&f.rel)).map_err(|e| format!("vectors/{}: {e}", f.rel))?;
            let doc: Value = serde_json::from_str(&text).map_err(|e| format!("vectors/{}: {e}", f.rel))?;
            let mut kinds: Vec<&str> =
                doc["entries"].as_array().into_iter().flatten().map(|e| e["kind"].as_str().unwrap_or("")).collect();
            kinds.sort_unstable();
            kinds.dedup();
            let kind = match kinds.as_slice() {
                [k] if !k.is_empty() => *k,
                ks => return Err(format!("vectors/{}: a wire file holds one kind, this one {ks:?}", f.rel)),
            };
            if scope.kinds.as_ref().is_some_and(|ks| !ks.iter().any(|k| k == kind)) {
                continue;
            }
            if !scope.activation && doc["activation_rule"].as_bool().unwrap_or(false) {
                continue;
            }
        }
        out.push(f.clone());
    }
    Ok(out)
}
