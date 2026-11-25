//! The self-checks a runner makes before it grades anything (runner-integration §4.1, §4.2): every verdict case and
//! every selection case of the SANTA checkout's `oracle/` reproduced; an oracle schema the harness does not know is
//! refused.
use crate::grading::{Grading, Input};
use crate::{parse_scope, selection, structural_equal};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TREE_SEQ: AtomicU64 = AtomicU64::new(0);

/// Ok when every case of every `<santa>/oracle/*.json` is reproduced; otherwise the first failure, naming the file and
/// the case.
pub fn check(santa: &Path, grading: &Grading) -> Result<(), String> {
    let dir = santa.join("oracle");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("{}: no oracle files", dir.display()));
    }
    for p in files {
        let name = format!("oracle/{}", p.file_name().unwrap().to_string_lossy());
        let doc: Value = serde_json::from_str(&fs::read_to_string(&p).map_err(|e| format!("{name}: {e}"))?)
            .map_err(|e| format!("{name}: {e}"))?;
        match doc["schema"].as_str().unwrap_or("") {
            "santa-oracle-grading/v1" => verdicts(&doc, grading).map_err(|e| format!("{name}: {e}"))?,
            "santa-oracle-selection/v1" => selections(&doc).map_err(|e| format!("{name}: {e}"))?,
            s => return Err(format!("{name}: unknown oracle schema {s:?}")),
        }
    }
    Ok(())
}

/// One verdict case: its label, entry, file envelope (default `{}`), actual (`null` = missing) and cost claim
/// (default false), through the interpreter, equal to its verdict.
fn verdicts(doc: &Value, g: &Grading) -> Result<(), String> {
    let empty = json!({});
    for c in doc["cases"].as_array().ok_or("no cases")? {
        let input = Input {
            entry: &c["entry"],
            actual: Some(&c["actual"]).filter(|a| !a.is_null()),
            file: c.get("file").unwrap_or(&empty),
            claims_cost: c["claims_cost"].as_bool().unwrap_or(false),
        };
        let label = c["label"].as_str().ok_or(format!("case {}: no label", c["name"]))?;
        let got = g.grade(label, &input).map_or_else(|e| json!({ "error": e.to_string() }), |v| v.to_json());
        if !structural_equal(&got, &c["verdict"]) {
            return Err(format!("case {}: got {got}, want {}", c["name"], c["verdict"]));
        }
    }
    Ok(())
}

/// The selection cases over a temporary tree built from `files` (runner-integration §4.1): a wire file
/// `{"entries": [{"kind": k}]}` with its `activation_rule` when given, another `.json` `{"entries": []}`,
/// `grading.json` `{}`, a non-JSON file some text.
fn selections(doc: &Value) -> Result<(), String> {
    let n = TREE_SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let root = std::env::temp_dir().join(format!("santa-harness-selection-{}-{n}-{nanos}", std::process::id()));
    // a fresh directory of our own: refuse one that already exists rather than write into it
    fs::create_dir(&root).map_err(|e| format!("{}: {e}", root.display()))?;
    let vectors = root.join("vectors");
    let result = (|| {
        for f in doc["files"].as_array().ok_or("no files")? {
            let rel = f["path"].as_str().ok_or("a file without a path")?;
            // a path the tree cannot hold (absolute, or climbing out with `..`) is refused before anything is written
            if rel.is_empty() || !Path::new(rel).components().all(|c| matches!(c, std::path::Component::Normal(_))) {
                return Err(format!("a file path that leaves the tree: {rel:?}"));
            }
            let path = vectors.join(rel);
            fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            let body = if let Some(kind) = f.get("kind") {
                let mut w = json!({"entries": [{"kind": kind}]});
                if let Some(a) = f.get("activation_rule") {
                    w["activation_rule"] = a.clone();
                }
                w.to_string()
            } else if path.extension().and_then(|x| x.to_str()) != Some("json") {
                "not a vector".to_string()
            } else if rel == "grading.json" {
                "{}".to_string()
            } else {
                json!({"entries": []}).to_string()
            };
            fs::write(&path, body).map_err(|e| e.to_string())?;
        }
        let all = selection::discover(&vectors)?;
        for c in doc["cases"].as_array().ok_or("no cases")? {
            let scope = parse_scope(&c["scope"]).map_err(|e| format!("case {}: {e}", c["name"]))?;
            let got: Vec<String> = selection::select(&all, &vectors, &scope)?.into_iter().map(|f| f.rel).collect();
            let want: Vec<String> =
                c["selects"].as_array().into_iter().flatten().filter_map(|p| p.as_str().map(String::from)).collect();
            if got != want {
                return Err(format!("case {}: selects {got:?}, want {want:?}", c["name"]));
            }
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(&root);
    result
}
