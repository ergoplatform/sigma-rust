//! The runner's command line, through the built binary: the two verbs of SANTA's self-contained
//! runner contract, the self-compare, and what a bad call exits with. None of these needs a SANTA
//! checkout; a whole run against one is `santa-run` (see the README).
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn blitzen(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_blitzen"))
        .args(args)
        .output()
        .expect("start the runner")
}

/// A fresh directory of one test's own, removed again when the test ends.
struct Temp(PathBuf);

impl Temp {
    fn new(tag: &str) -> Temp {
        let dir = std::env::temp_dir().join(format!("blitzen-cli-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create the test directory");
        Temp(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn arg(&self) -> &str {
        self.0.to_str().expect("a utf-8 path")
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

/// `one <santa-dir> <relpath> <out-file>` reads `<santa-dir>/vectors/<relpath>` and writes that
/// file's actuals, by entry name, to `<out-file>`.
#[test]
fn one_writes_the_actuals_of_one_file() {
    let dir = Temp::new("one");
    // sbox_minimal of SANTA's `vectors/wire/v5/authored/Box.json`
    let hex =
        "c0843d09020101000000000000000000000000000000000000000000000000000000000000000000000000";
    let rel = "wire/v5/authored/Box.json";
    let vector = json!({"schema": "santa-wire/v1", "entries": [
        {"name": "e#0", "kind": "Box", "bytes_hex": hex}
    ]});
    let file = dir.path().join("vectors").join(rel);
    std::fs::create_dir_all(file.parent().expect("a parent")).expect("create the slice");
    std::fs::write(&file, vector.to_string()).expect("write the vector");
    let out_file = dir.path().join("out").join("Box.json");

    let ran = blitzen(&[
        "one",
        dir.arg(),
        rel,
        out_file.to_str().expect("a utf-8 path"),
    ]);
    assert!(ran.status.success(), "{}", text(&ran.stderr));
    let actuals: Value =
        serde_json::from_str(&std::fs::read_to_string(&out_file).expect("the actuals file"))
            .expect("the actuals are JSON");
    assert_eq!(actuals, json!({"e#0": {"bytes_hex": hex, "error": null}}));
}

/// `run` on a directory that is no SANTA checkout could not run: exit 1 and the harness's
/// reason, never a panic. The reason names the file it looked for in the directory given as the
/// checkout, and the directory given for the results is the one whose earlier results are gone
/// (a run that fails leaves no other run's results behind: runner-integration contract, §4.3).
#[test]
fn run_refuses_a_directory_that_is_no_checkout() {
    let checkout = Temp::new("run-checkout");
    let out = Temp::new("run-out");
    let old_results = out.path().join("results.json");
    let old_actuals = out.path().join("actuals").join("x.json");
    std::fs::create_dir_all(old_actuals.parent().expect("a parent")).expect("create actuals/");
    std::fs::write(&old_results, "{}").expect("write an earlier results.json");
    std::fs::write(&old_actuals, "{}").expect("write an earlier actuals file");

    let runner_json = concat!(env!("CARGO_MANIFEST_DIR"), "/runner.json");
    let ran = blitzen(&["run", runner_json, checkout.arg(), out.arg()]);
    assert_eq!(ran.status.code(), Some(1), "{}", text(&ran.stderr));
    let said = text(&ran.stderr);
    let looked_for = checkout.path().join("vectors").join("grading.json");
    assert!(said.starts_with("blitzen: "), "{said}");
    assert!(said.contains(&looked_for.display().to_string()), "{said}");
    assert!(
        !old_results.exists(),
        "the earlier results.json is still there"
    );
    assert!(!old_actuals.exists(), "the earlier actuals are still there");
}

/// `run` reads its scope from the file it is given: one that is not there could not run.
#[test]
fn run_refuses_a_missing_runner_json() {
    let dir = Temp::new("scope");
    let missing = dir.path().join("no-runner.json");
    let out = dir.path().join("out");
    let ran = blitzen(&[
        "run",
        missing.to_str().expect("a utf-8 path"),
        dir.arg(),
        out.to_str().expect("a utf-8 path"),
    ]);
    assert_eq!(ran.status.code(), Some(1), "{}", text(&ran.stderr));
    assert!(text(&ran.stderr).contains("no-runner.json"));
}

/// A call that names no mode exits 2 with the three forms; so does the call of the convention
/// this runner had before.
#[test]
fn a_bad_call_is_a_usage_error() {
    for args in [&[][..], &["run"][..], &["emit", "vectors", "out"][..]] {
        let ran = blitzen(args);
        assert_eq!(ran.status.code(), Some(2), "{args:?}");
        let said = text(&ran.stderr);
        for form in [
            "blitzen run <runner.json>",
            "blitzen one <santa-dir>",
            "blitzen <vector.json | dir>",
        ] {
            assert!(said.contains(form), "{args:?}: {said}");
        }
    }
}

/// One argument that is no verb is a vector file (or a directory of them) to self-compare: its
/// entries are run and counted, and the call exits 0 whatever is coal.
#[test]
fn a_file_is_self_compared() {
    let ran = blitzen(&[&fixture("nipopow-chain-32.json")]);
    assert_eq!(ran.status.code(), Some(0), "{}", text(&ran.stderr));
    let said = text(&ran.stdout);
    assert!(said.starts_with("blitzen: "), "{said}");
    assert!(said.contains(" 2 entries across 1 files"), "{said}");
}

/// A directory with no vector file in it is a bad call of the self-compare: exit 2.
#[test]
fn an_empty_directory_is_not_self_compared() {
    let dir = Temp::new("empty");
    let ran = blitzen(&[dir.arg()]);
    assert_eq!(ran.status.code(), Some(2), "{}", text(&ran.stdout));
    assert!(text(&ran.stderr).contains("no .json vector files found"));
}
