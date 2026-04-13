use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cargo_metadata() -> Value {
    let output = Command::new("mise")
        .args([
            "exec",
            "--",
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(repo_root())
        .output()
        .expect("cargo metadata should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("cargo metadata output should be valid json")
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME
fn crates_io_package_name_is_special_cli() {
    let metadata = cargo_metadata();
    let package_name = metadata["packages"][0]["name"]
        .as_str()
        .expect("package name should be a string");

    assert_eq!(package_name, "special-cli");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME
fn cargo_package_installs_special_binary() {
    let metadata = cargo_metadata();
    let targets = metadata["packages"][0]["targets"]
        .as_array()
        .expect("targets should be an array");

    let has_special_bin = targets.iter().any(|target| {
        target["name"].as_str() == Some("special")
            && target["kind"]
                .as_array()
                .map(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("bin")))
                .unwrap_or(false)
    });

    assert!(has_special_bin, "cargo metadata should expose a `special` binary target");
}
