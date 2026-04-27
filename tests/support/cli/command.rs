/**
@module SPECIAL.TESTS.SUPPORT.CLI.COMMAND
Command execution and temp workspace helpers for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.COMMAND
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_REPO_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn temp_repo_dir(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let counter = TEMP_REPO_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{timestamp}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp repo dir should be created");
    path
}

pub fn run_special_raw(root: &Path, args: &[&str]) -> std::process::Output {
    let mut command = special_test_command(root);
    command
        .args(args)
        .output()
        .expect("special command should run")
}

pub fn run_special(root: &Path, args: &[&str]) -> std::process::Output {
    run_special_raw(root, args)
}

pub fn run_special_with_env(
    root: &Path,
    args: &[&str],
    envs: &[(&str, &str)],
) -> std::process::Output {
    let mut command = special_test_command(root);
    command.args(args).current_dir(root);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("special command should run")
}

pub fn spawn_special(root: &Path, args: &[&str]) -> Child {
    let mut command = special_test_command(root);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("special command should run")
}

pub fn run_special_with_input(root: &Path, args: &[&str], input: &str) -> std::process::Output {
    let mut command = special_test_command(root);
    let mut child = command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("special command should run");

    write_child_input(&mut child, input);
    child.wait_with_output().expect("output should be captured")
}

pub fn run_special_with_input_and_env(
    root: &Path,
    args: &[&str],
    input: &str,
    envs: &[(&str, &Path)],
) -> std::process::Output {
    let mut command = special_test_command(root);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }

    let mut child = command.spawn().expect("special command should run");
    write_child_input(&mut child, input);
    child.wait_with_output().expect("output should be captured")
}

pub fn run_special_with_env_removed(
    root: &Path,
    args: &[&str],
    removed_envs: &[&str],
) -> std::process::Output {
    let mut command = special_test_command(root);
    command.args(args);
    for key in removed_envs {
        command.env_remove(key);
    }
    command.output().expect("special command should run")
}

fn special_test_command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_special"));
    command
        .current_dir(root)
        .env("SPECIAL_TRACEABILITY_KERNEL", "rust-reference");
    command
}

pub fn rust_analyzer_available() -> bool {
    Command::new("mise")
        .args(["exec", "--", "rust-analyzer", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn go_toolchain_available() -> bool {
    Command::new("mise")
        .args(["exec", "--", "go", "version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn typescript_traceability_available() -> bool {
    Command::new("mise")
        .args(["exec", "--", "node", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn write_child_input(child: &mut Child, input: &str) {
    let write_result = child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes());
    let _ = child.stdin.take();
    if let Err(error) = write_result {
        let _ = child.kill();
        let _ = child.wait();
        panic!("input should be written: {error}");
    }
}
