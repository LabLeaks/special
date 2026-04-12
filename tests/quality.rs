use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn clippy_script() -> String {
    fs::read_to_string(repo_root().join("scripts/verify-rust-clippy.sh"))
        .expect("clippy verification script should be readable")
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS
fn pinned_clippy_contract_passes() {
    let root = repo_root();
    let output = Command::new("bash")
        .arg("scripts/verify-rust-clippy.sh")
        .current_dir(&root)
        .output()
        .expect("clippy verification script should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.MISE_EXEC
fn clippy_script_uses_mise_exec() {
    let script = clippy_script();
    assert!(script.contains("mise exec --"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.CARGO_CLIPPY
fn clippy_script_invokes_cargo_clippy() {
    let script = clippy_script();
    assert!(script.contains("cargo clippy"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_TARGETS
fn clippy_script_passes_all_targets() {
    let script = clippy_script();
    assert!(script.contains("--all-targets"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_FEATURES
fn clippy_script_passes_all_features() {
    let script = clippy_script();
    assert!(script.contains("--all-features"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.DENY_WARNINGS
fn clippy_script_denies_warnings() {
    let script = clippy_script();
    assert!(script.contains("-- -D warnings"));
}
