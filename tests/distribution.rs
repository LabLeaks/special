/**
@group SPECIAL.DISTRIBUTION.CRATES_IO
special crates.io package identity.

@spec SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME
special publishes the package as `special-cli`.

@spec SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME
special installs the `special` binary from the `special-cli` package.

@group SPECIAL.DISTRIBUTION.GITHUB_RELEASES
special GitHub release distribution.

@group SPECIAL.DISTRIBUTION.HOMEBREW
special Homebrew distribution.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.REPOSITORY_URL
special release automation declares the `https://github.com/LabLeaks/special` repository URL.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW
special keeps a GitHub Actions release workflow in `.github/workflows/release.yml`.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.PUBLISHED
special publishes GitHub Releases for versioned distribution.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.ARCHIVES
special GitHub release automation publishes versioned release archives for supported target platforms.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.CHECKSUMS
special GitHub release automation publishes checksums for its release archives.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA
special ships a Homebrew formula in LabLeaks/homebrew-tap.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PATH
special keeps its Homebrew formula at `Formula/special.rb` in LabLeaks/homebrew-tap.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PLATFORM_SELECTION
special selects its platform-specific Homebrew archive URL and checksum with Homebrew's standard `on_system_conditional` and `on_arch_conditional` helpers.

@spec SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL
special installs the `special` binary from LabLeaks/homebrew-tap.

@attests SPECIAL.DISTRIBUTION.HOMEBREW.INSTALLS_SPECIAL
artifact: brew install LabLeaks/homebrew-tap/special (confirmed local install for /opt/homebrew/bin/special at v0.4.0)
owner: gk
last_reviewed: 2026-04-16

@module SPECIAL.TESTS.DISTRIBUTION
Distribution/release asset integration tests in `tests/distribution.rs`.
*/
// @fileimplements SPECIAL.TESTS.DISTRIBUTION
use std::path::PathBuf;
use std::process::Command;
use std::{fs, path::Path};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(path: impl AsRef<Path>) -> String {
    fs::read_to_string(repo_root().join(path)).expect("repo file should be readable")
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

fn release_assets() -> Value {
    serde_json::from_str(include_str!("../scripts/release-assets.json"))
        .expect("release assets json should be valid")
}

fn package_metadata() -> Value {
    cargo_metadata()["packages"]
        .as_array()
        .expect("packages should be an array")
        .iter()
        .find(|package| package["name"].as_str() == Some("special-cli"))
        .cloned()
        .expect("cargo metadata should include the special-cli package")
}

fn dist_command() -> Command {
    let mut command = Command::new("mise");
    command.args(["exec", "--", "dist"]);
    command.current_dir(repo_root());
    command
}

fn dist_manifest() -> Value {
    let package = package_metadata();
    let version = package["version"]
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
    let package = package_metadata();
    let package_name = package["name"]
        .as_str()
        .expect("package name should be a string");

    assert_eq!(package_name, "special-cli");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME
fn cargo_package_installs_special_binary() {
    let package = package_metadata();
    let targets = package["targets"]
        .as_array()
        .expect("targets should be an array");

    let has_special_bin = targets.iter().any(|target| {
        target["name"].as_str() == Some("special")
            && target["kind"]
                .as_array()
                .map(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("bin")))
                .unwrap_or(false)
    });

    assert!(
        has_special_bin,
        "cargo metadata should expose a `special` binary target"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.REPOSITORY_URL
fn github_release_repository_url_is_declared() {
    let package = package_metadata();
    let repository = package["repository"]
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
    let workflow = read_repo_file(".github/workflows/release.yml");
    assert!(workflow.contains("tags:\n      - '**[0-9]+.[0-9]+.[0-9]+*'"));
    assert!(!workflow.contains("Validate release tag shape"));
    assert!(workflow.contains("rm -f artifacts/*-dist-manifest.json"));

    let package = package_metadata();
    let version = package["version"]
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
// @verifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PATH
fn homebrew_formula_uses_the_standard_formula_path() {
    let updater = read_repo_file("scripts/update-homebrew-formula.py");
    assert!(
        updater.contains("FORMULA_PATH = \"Formula/special.rb\""),
        "Homebrew updater should target the standard Formula path"
    );

    let verifier = read_repo_file("scripts/verify-homebrew-formula.sh");
    assert!(
        verifier.contains("contents/Formula/special.rb"),
        "Homebrew verification should read the standard Formula path"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA.PLATFORM_SELECTION
fn homebrew_formula_uses_standard_platform_selection_helpers() {
    let updater = read_repo_file("scripts/update-homebrew-formula.py");
    let verifier = read_repo_file("scripts/verify-homebrew-formula.sh");

    assert!(
        updater.contains("archive = on_system_conditional("),
        "Homebrew updater should select the archive with on_system_conditional"
    );
    assert!(
        updater.contains("sha256 on_system_conditional("),
        "Homebrew updater should select sha256 with on_system_conditional"
    );
    assert!(
        updater.contains("on_arch_conditional("),
        "Homebrew updater should use on_arch_conditional for architecture-specific values"
    );
    assert!(
        updater.contains(
            "url \"https://github.com/LabLeaks/special/releases/download/v{version}/#{{archive}}\""
        ),
        "Homebrew updater should emit a single active url from the selected archive"
    );
    assert!(
        verifier.contains("formula is missing templated release asset url"),
        "Homebrew verification should accept the templated archive URL shape"
    );
    assert!(
        verifier.contains("formula is missing archive selector entry"),
        "Homebrew verification should validate archive selector entries instead of expanded asset URLs"
    );
}

#[test]
fn homebrew_formula_verifier_requires_release_asset_digests() {
    let verifier = read_repo_file("scripts/verify-homebrew-formula.sh");

    assert!(
        verifier.contains("release asset is missing digest"),
        "Homebrew verification should fail cleanly when required release assets omit digest metadata"
    );
}

#[test]
fn homebrew_formula_verifier_checks_selector_checksum_pairing() {
    let verifier = read_repo_file("scripts/verify-homebrew-formula.sh");

    assert!(
        verifier.contains("formula checksum selector entry does not contain expected checksum"),
        "Homebrew verification should validate selector-level checksum pairing, not just loose checksum presence"
    );
}

#[test]
fn homebrew_formula_verifier_rejects_unmapped_required_assets_cleanly() {
    let verifier = read_repo_file("scripts/verify-homebrew-formula.sh");

    assert!(
        verifier.contains("required release asset has no Homebrew selector mapping"),
        "Homebrew verification should fail cleanly when a required archive lacks selector mapping"
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

    let mut archive_names: Vec<_> = release_archives
        .iter()
        .map(|archive| {
            archive["name"]
                .as_str()
                .expect("archive name should be a string")
        })
        .collect();
    archive_names.sort_unstable();

    let release_assets = release_assets();
    let mut expected_archive_names: Vec<_> = release_assets["archives"]
        .as_array()
        .expect("archives should be an array")
        .iter()
        .map(|archive| archive.as_str().expect("archive should be a string"))
        .collect();
    expected_archive_names.sort_unstable();

    assert_eq!(archive_names, expected_archive_names);

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
            assets
                .iter()
                .any(|asset| asset["kind"].as_str() == Some("executable")),
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
    let release_assets = release_assets();

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

    for archive_name in release_assets["archives"]
        .as_array()
        .expect("archives should be an array")
        .iter()
        .map(|archive| archive.as_str().expect("archive should be a string"))
    {
        let checksum_name = format!("{archive_name}.sha256");
        assert!(
            artifacts
                .get(&checksum_name)
                .and_then(|artifact| artifact["kind"].as_str())
                == Some("checksum"),
            "dist manifest should define checksum artifact {checksum_name}"
        );
    }
}
