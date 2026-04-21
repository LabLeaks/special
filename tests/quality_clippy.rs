/**
@group SPECIAL.QUALITY.RUST.CLIPPY
Pinned clippy contract for the Rust quality tooling surface.

@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS
the pinned clippy verification surface succeeds only when the repo uses the canonical clippy command shape.

@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.MISE_EXEC
the pinned clippy verification surface runs clippy through `mise exec --`.

@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.CARGO_CLIPPY
the pinned clippy verification surface invokes `cargo clippy`.

@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_TARGETS
the pinned clippy verification surface passes `--all-targets`.

@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_FEATURES
the pinned clippy verification surface passes `--all-features`.

@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.DENY_WARNINGS
the pinned clippy verification surface denies warnings.

@spec SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED
the clippy verification script carries the proving surface for the pinned clippy contract.

@module SPECIAL.TESTS.QUALITY_CLIPPY
Pinned clippy-contract tests in `tests/quality_clippy.rs`.
*/
// @fileimplements SPECIAL.TESTS.QUALITY_CLIPPY
#[path = "support/quality.rs"]
mod support;

use std::process::Command;

use support::{clippy_script, repo_root};

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
