//! The bounded-file watchdog (runner-integration §3a): a child in its own process group, killed at its memory or wall
//! budget, its crash described. Children run as `bash <script>`: executing a just-written script races with other test
//! threads' forks ("text file busy").
use santa_harness::bounds::{budget, group_memory, run_child, Budget, Outcome};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use std::time::{Duration, Instant};

/// A script in a scratch directory of its own, the directory removed when the test ends.
struct Script(PathBuf);
impl std::ops::Deref for Script {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.0
    }
}
impl Drop for Script {
    fn drop(&mut self) {
        if let Some(d) = self.0.parent() {
            let _ = fs::remove_dir_all(d);
        }
    }
}

fn script(tag: &str, body: &str) -> Script {
    let d = std::env::temp_dir().join(format!("santa-harness-bounds-{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    let s = d.join("run.sh");
    fs::write(&s, format!("#!/usr/bin/env bash\n{body}\n")).unwrap();
    Script(s)
}

fn run(s: &Path, memory_mb: u64, wall_s: u64) -> Outcome {
    let mut cmd = Command::new("bash");
    cmd.arg(s);
    run_child(cmd, &Budget { memory_mb, wall_s }, &s.with_file_name("stderr"), None, None)
}

fn alive(pid: &str) -> bool {
    fs::read_to_string(format!("/proc/{}/stat", pid.trim()))
        .map(|st| !st.rsplit(')').next().unwrap_or("").trim_start().starts_with('Z'))
        .unwrap_or(false)
}

/// An interrupted runner (SIGINT while a bounded child runs) kills the child's group and dies of the same signal: the
/// child sits in a group of its own, so a terminal's Ctrl-C never reaches it. The fixture runs the child under
/// run_child and is sent the signal.
#[test]
fn an_interrupted_runner_kills_its_childs_group_and_dies_of_the_signal() {
    let s = script("interrupted", "sleep 30 &\necho $! > \"$(dirname \"$0\")/child.pid\"\nsleep 30");
    let pid_file = s.with_file_name("child.pid");
    let mut fixture = Command::new(env!("CARGO_BIN_EXE_santa-harness-fixture")).arg("hold").arg(&*s).spawn().unwrap();
    let t0 = Instant::now();
    while !pid_file.exists() {
        assert!(t0.elapsed() < Duration::from_secs(10), "the child never started");
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(50));
    // SAFETY: a signal to the fixture we spawned.
    unsafe {
        libc::kill(fixture.id() as i32, libc::SIGINT);
    }
    assert_eq!(fixture.wait().unwrap().signal(), Some(libc::SIGINT), "the runner dies of the signal it was sent");
    std::thread::sleep(Duration::from_millis(200));
    assert!(!alive(&fs::read_to_string(&pid_file).unwrap()), "the grandchild outlived its runner");
}

/// An interrupted runner also removes its scratch directory (main_run's .children/) before it dies: the signal it
/// re-raises ends it before main_run's own cleanup could run.
#[test]
fn an_interrupted_runner_removes_its_scratch_directory() {
    let s = script("interrupted-scratch", "sleep 30 &\necho $! > \"$(dirname \"$0\")/child.pid\"\nsleep 30");
    let scratch = s.with_file_name("children");
    fs::create_dir_all(&scratch).unwrap();
    fs::write(scratch.join("0.stderr"), "").unwrap();
    let pid_file = s.with_file_name("child.pid");
    let mut fixture =
        Command::new(env!("CARGO_BIN_EXE_santa-harness-fixture")).arg("hold").arg(&*s).arg(&scratch).spawn().unwrap();
    let t0 = Instant::now();
    while !pid_file.exists() {
        assert!(t0.elapsed() < Duration::from_secs(10), "the child never started");
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(50));
    // SAFETY: a signal to the fixture we spawned.
    unsafe {
        libc::kill(fixture.id() as i32, libc::SIGINT);
    }
    assert_eq!(fixture.wait().unwrap().signal(), Some(libc::SIGINT));
    assert!(!scratch.exists(), "the scratch directory outlived the interrupted runner");
}

#[test]
fn a_child_past_its_wall_time_is_exceeded_time() {
    let t0 = Instant::now();
    assert_eq!(run(&script("wall", "sleep 30"), 1024, 1), Outcome::Exceeded("time"));
    assert!(t0.elapsed() < Duration::from_secs(3));
}

#[test]
fn a_child_past_its_memory_is_exceeded_memory() {
    let s = script("memory", "exec python3 -I -c 'a = []\nwhile True: a.append(bytearray(10**7))'");
    assert_eq!(run(&s, 64, 30), Outcome::Exceeded("memory"));
}

#[test]
fn a_childs_grandchild_dies_with_its_group() {
    let s = script("group", "sleep 30 &\necho $! > \"$(dirname \"$0\")/child.pid\"\nsleep 30");
    assert_eq!(run(&s, 1024, 1), Outcome::Exceeded("time"));
    std::thread::sleep(Duration::from_millis(200));
    let pid = fs::read_to_string(s.with_file_name("child.pid")).unwrap();
    assert!(!alive(&pid), "the grandchild {} outlived its group", pid.trim());
}

#[test]
fn a_crash_is_its_exit_status() {
    assert_eq!(run(&script("exit", "exit 101"), 1024, 30), Outcome::Crashed("process exit status 101".into()));
    assert_eq!(run(&script("segv", "kill -SEGV $$"), 1024, 30), Outcome::Crashed("process killed by signal 11".into()));
}

#[test]
fn a_refused_allocation_abort_is_memory() {
    let s = script("abort", "echo 'memory allocation of 1073741824 bytes failed' >&2\nkill -ABRT $$");
    assert_eq!(run(&s, 1024, 30), Outcome::Exceeded("memory"));
}

#[test]
fn a_clean_child_finishes() {
    assert_eq!(run(&script("ok", "true"), 1024, 30), Outcome::Finished);
}

#[test]
fn a_budget_is_the_envelopes_limits() {
    assert_eq!(budget(&json!({"limits": {"memory_mb": 1280, "wall_s": 10}})), Ok(Some(Budget { memory_mb: 1280, wall_s: 10 })));
    assert_eq!(budget(&json!({"entries": []})), Ok(None));
    assert!(budget(&json!({"limits": {"memory_mb": "lots", "wall_s": 10}})).is_err(), "a malformed budget runs unbounded");
}

/// A process's peak resident size counts, not only its current one: a spike that is over before the next poll is
/// still seen (the kernel keeps the high-water mark, VmHWM).
#[test]
fn the_peak_is_seen_after_the_memory_is_freed() {
    let s = script("peak", "exec python3 -I -c 'import os, sys, time\nos.setpgid(0, 0)\nx = b\"x\" * (100 << 20)\ndel x\nopen(sys.argv[1], \"w\").write(str(os.getpid()))\ntime.sleep(10)' \"$(dirname \"$0\")/freed\"");
    let mut child = Command::new("bash").arg(&*s).spawn().unwrap();
    let flag = s.with_file_name("freed");
    let t0 = Instant::now();
    while !flag.is_file() && t0.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(50));
    let pid: i32 = fs::read_to_string(&flag).unwrap().trim().parse().unwrap();
    let m = group_memory(pid);
    let (now, peak) = (m.now_kib, m.peak_kib);
    let _ = child.kill();
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
    let _ = child.wait();
    assert!(peak >= 100 * 1024, "peak {peak} KiB");
    assert!(now < 64 * 1024, "now {now} KiB");
}

/// A child that reports its own peak (as the runner's `one` verb does) and passed the budget is exceeded memory, even
/// when the spike fell between two polls.
#[test]
fn a_reported_peak_past_the_budget_is_exceeded_memory() {
    let s = script("reported", "exec python3 -I -c 'import sys\nx = b\"x\" * (100 << 20)\ndel x\nhwm = [l.split()[1] for l in open(\"/proc/self/status\") if l.startswith(\"VmHWM:\")][0]\nopen(sys.argv[1], \"w\").write(hwm)' \"$(dirname \"$0\")/peak\"");
    let mut cmd = Command::new("bash");
    cmd.arg(&*s);
    let out = run_child(cmd, &Budget { memory_mb: 64, wall_s: 30 }, &s.with_file_name("stderr"), Some(&s.with_file_name("peak")), None);
    assert_eq!(out, Outcome::Exceeded("memory"));
}

/// The parent's own memory never counts against a child: a forked child's rusage carries the parent's resident size
/// at the fork, so the watchdog must not use it.
#[test]
fn a_parent_holding_memory_does_not_count_against_its_child() {
    let held: Vec<u8> = vec![1; 128 << 20];
    assert_eq!(run(&script("parent", "true"), 64, 30), Outcome::Finished);
    assert_eq!(held.iter().map(|b| *b as u64).sum::<u64>(), 128 << 20);
}

/// The allocation-failure phrase is the out-of-memory signature only on an abort (SIGABRT): a child that printed it
/// and exited otherwise crashed.
#[test]
fn the_phrase_alone_is_not_out_of_memory() {
    let s = script("phrase", "echo 'memory allocation of 1024 bytes failed' >&2\nexit 101");
    assert_eq!(run(&s, 1024, 30), Outcome::Crashed("process exit status 101".into()));
}
