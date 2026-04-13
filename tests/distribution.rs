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

fn dist_bin() -> PathBuf {
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .expect("CARGO_HOME or HOME should be set");

    cargo_home.join("bin").join("dist")
}

fn dist_command() -> Command {
    let dist = dist_bin();
    assert!(
        dist.is_file(),
        "cargo-dist should be installed at {}",
        dist.display()
    );

    let mut command = Command::new(dist);
    command.current_dir(repo_root());
    command
}

fn dist_manifest() -> Value {
    let metadata = cargo_metadata();
    let version = metadata["packages"][0]["version"]
        .as_str()
        .expect("package version should be a string");

    let output = dist_command()
        .args([
            "manifest",
            "--artifacts=all",
            "--output-format=json",
            "--no-local-paths",
            "--tag",
            &format!("v{version}"),
            "--allow-dirty",
        ])
        .output()
        .expect("dist manifest should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("dist manifest output should be valid json")
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

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.REPOSITORY_URL
fn github_release_repository_url_is_declared() {
    let metadata = cargo_metadata();
    let repository = metadata["packages"][0]["repository"]
        .as_str()
        .expect("repository should be a string");

    assert_eq!(repository, "https://github.com/LabLeaks/special");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW
fn github_release_workflow_is_committed_and_in_sync() {
    assert!(
        repo_root().join(".github/workflows/release.yml").is_file(),
        "release workflow should be committed"
    );

    let metadata = cargo_metadata();
    let version = metadata["packages"][0]["version"]
        .as_str()
        .expect("package version should be a string");

    let output = dist_command()
        .args([
            "host",
            "--steps=create",
            "--tag",
            &format!("v{version}"),
            "--output-format=json",
            "--allow-dirty",
        ])
        .output()
        .expect("dist host --steps=create should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.ARCHIVES
fn github_release_plan_contains_versioned_archives() {
    let manifest = dist_manifest();
    let artifacts = manifest["artifacts"]
        .as_object()
        .expect("artifacts should be an object");

    let release_archives: Vec<_> = artifacts
        .values()
        .filter(|artifact| artifact["kind"].as_str() == Some("executable-zip"))
        .collect();

    assert!(
        !release_archives.is_empty(),
        "dist manifest should include executable release archives"
    );

    for archive in release_archives {
        let name = archive["name"]
            .as_str()
            .expect("archive name should be a string");
        let assets = archive["assets"]
            .as_array()
            .expect("archive assets should be an array");

        assert!(
            name.starts_with("special-cli-"),
            "archive name should be versioned under the package identity: {name}"
        );
        assert!(
            assets.iter().any(|asset| asset["kind"].as_str() == Some("executable")),
            "archive should include the special executable: {name}"
        );
    }
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.CHECKSUMS
fn github_release_plan_contains_checksums_for_archives() {
    let manifest = dist_manifest();
    let artifacts = manifest["artifacts"]
        .as_object()
        .expect("artifacts should be an object");

    assert!(
        artifacts
            .get("sha256.sum")
            .and_then(|artifact| artifact["kind"].as_str())
            == Some("unified-checksum"),
        "dist manifest should define a unified checksum artifact"
    );

    for artifact in artifacts.values() {
        if artifact["kind"].as_str() == Some("executable-zip") {
            let checksum_name = artifact["checksum"]
                .as_str()
                .expect("archive should declare a checksum artifact");

            assert!(
                artifacts
                    .get(checksum_name)
                    .and_then(|checksum| checksum["kind"].as_str())
                    == Some("checksum"),
                "archive checksum artifact should exist for {checksum_name}"
            );
        }
    }
}
