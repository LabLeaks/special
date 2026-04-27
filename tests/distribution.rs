/**
@group SPECIAL.DISTRIBUTION.CRATES_IO
special crates.io package identity.

@spec SPECIAL.DISTRIBUTION.CRATES_IO.PACKAGE_NAME
special publishes the package as `special-cli`.

@spec SPECIAL.DISTRIBUTION.CRATES_IO.BINARY_NAME
special installs the `special` binary from the `special-cli` package.

@group SPECIAL.DISTRIBUTION.GITHUB_RELEASES
special GitHub release distribution.

@group SPECIAL.DISTRIBUTION.BUILD_SCRIPT
special Cargo build-script distribution behavior.

@group SPECIAL.DISTRIBUTION.HOMEBREW
special Homebrew distribution.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.REPOSITORY_URL
special release automation declares the `https://github.com/LabLeaks/special` repository URL.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.WORKFLOW
special keeps a GitHub Actions release workflow in `.github/workflows/release.yml`.

@spec SPECIAL.DISTRIBUTION.GITHUB_RELEASES.LEAN_KERNEL
special GitHub release automation embeds the standalone Lean traceability kernel in host-native local artifact builds so those released `special` binaries can use the Lean kernel without requiring Lean at user runtime; cross-target artifact builds do not embed a host-built Lean executable.

@spec SPECIAL.DISTRIBUTION.BUILD_SCRIPT.LANGUAGE_PACK_REGISTRY
special's Cargo build script generates a compile-time built-in language-pack registry from shipped language-pack source files.

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
artifact: brew install LabLeaks/homebrew-tap/special (confirmed local install for /opt/homebrew/bin/special at v0.8.0)
owner: gk
last_reviewed: 2026-04-27

@module SPECIAL.TESTS.DISTRIBUTION
Distribution/release asset integration tests in `tests/distribution.rs`.
*/
// @fileimplements SPECIAL.TESTS.DISTRIBUTION
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::Path};

use serde_json::{Value, json};

static HOMEBREW_VERIFIER_COUNTER: AtomicU64 = AtomicU64::new(0);

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

fn current_package_version() -> String {
    package_metadata()["version"]
        .as_str()
        .expect("package version should be a string")
        .to_string()
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

fn base64_encode(input: &str) -> String {
    let mut child = Command::new("mise")
        .args([
            "exec",
            "--",
            "python3",
            "-c",
            "import base64,sys; print(base64.b64encode(sys.stdin.buffer.read()).decode())",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("python3 should run");
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child
        .wait_with_output()
        .expect("python output should be captured");
    assert!(
        output.status.success(),
        "python3 base64 helper should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("base64 output should be utf-8")
        .trim()
        .to_string()
}

fn homebrew_selector_arms() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("special-cli-aarch64-apple-darwin.tar.xz", "macos", "arm"),
        ("special-cli-x86_64-apple-darwin.tar.xz", "macos", "intel"),
        (
            "special-cli-aarch64-unknown-linux-gnu.tar.xz",
            "linux",
            "arm",
        ),
        (
            "special-cli-x86_64-unknown-linux-gnu.tar.xz",
            "linux",
            "intel",
        ),
    ]
}

fn expected_release_sha(index: usize) -> String {
    format!("{:064x}", index + 1)
}

fn expected_release_sha_for_archive(name: &str) -> String {
    release_assets()["homebrew_formula_archives"]
        .as_array()
        .expect("homebrew formula archives should be an array")
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            (value.as_str() == Some(name)).then(|| expected_release_sha(index))
        })
        .unwrap_or_else(|| panic!("archive {name} should exist in release assets"))
}

fn valid_release_assets_json(mut mutate: impl FnMut(&mut Vec<Value>)) -> String {
    let release_assets = release_assets();
    let required = release_assets["homebrew_formula_archives"]
        .as_array()
        .expect("homebrew formula archives should be an array");
    let mut assets = required
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let name = name.as_str().expect("archive name should be a string");
            json!({
                "name": name,
                "digest": format!("sha256:{}", expected_release_sha(index)),
            })
        })
        .collect::<Vec<_>>();
    mutate(&mut assets);
    json!({ "assets": assets }).to_string()
}

fn valid_formula_for_release(version: &str, sha_override: Option<(&str, String)>) -> String {
    let mut macos_arm = String::new();
    let mut macos_intel = String::new();
    let mut linux_arm = String::new();
    let mut linux_intel = String::new();
    let mut sha_macos_arm = String::new();
    let mut sha_macos_intel = String::new();
    let mut sha_linux_arm = String::new();
    let mut sha_linux_intel = String::new();

    for (archive, os_name, arch) in homebrew_selector_arms() {
        let sha = if let Some((target_archive, override_sha)) = &sha_override {
            if *target_archive == archive {
                override_sha.clone()
            } else {
                expected_release_sha_for_archive(archive)
            }
        } else {
            expected_release_sha_for_archive(archive)
        };
        match (os_name, arch) {
            ("macos", "arm") => {
                macos_arm = archive.to_string();
                sha_macos_arm = sha;
            }
            ("macos", "intel") => {
                macos_intel = archive.to_string();
                sha_macos_intel = sha;
            }
            ("linux", "arm") => {
                linux_arm = archive.to_string();
                sha_linux_arm = sha;
            }
            ("linux", "intel") => {
                linux_intel = archive.to_string();
                sha_linux_intel = sha;
            }
            _ => unreachable!("unexpected selector arm"),
        }
    }

    format!(
        r#"class Special < Formula
  version "{version}"
  archive = on_system_conditional(
    macos: on_arch_conditional(
      arm: "{macos_arm}",
      intel: "{macos_intel}"
    ),
    linux: on_arch_conditional(
      arm: "{linux_arm}",
      intel: "{linux_intel}"
    )
  )
  sha256 on_system_conditional(
    macos: on_arch_conditional(
      arm: "{sha_macos_arm}",
      intel: "{sha_macos_intel}"
    ),
    linux: on_arch_conditional(
      arm: "{sha_linux_arm}",
      intel: "{sha_linux_intel}"
    )
  )
  url "https://github.com/LabLeaks/special/releases/download/v{version}/#{{archive}}"

  def install
    bin.install "special"
  end
end
"#
    )
}

fn run_homebrew_formula_verifier(release_json: &str, formula: &str) -> std::process::Output {
    let counter = HOMEBREW_VERIFIER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "special-homebrew-verifier-{}-{nanos}-{counter}",
        std::process::id()
    ));
    let bin_dir = temp.join("bin");
    fs::create_dir_all(&bin_dir).expect("temp bin dir should be created");

    let gh_script = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${{1:-}}" == "release" && "${{2:-}}" == "view" ]]; then
cat <<'JSON'
{release_json}
JSON
elif [[ "${{1:-}}" == "api" ]]; then
cat <<'B64'
{formula_b64}
B64
else
echo "unexpected gh invocation: $*" >&2
exit 1
fi
"#,
        release_json = release_json,
        formula_b64 = base64_encode(formula),
    );
    let gh_path = bin_dir.join("gh");
    fs::write(&gh_path, gh_script).expect("fake gh script should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&gh_path)
            .expect("fake gh metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&gh_path, permissions).expect("fake gh should be executable");
    }

    let path = std::env::var("PATH").expect("PATH should be set");
    let output = Command::new("bash")
        .arg("scripts/verify-homebrew-formula.sh")
        .current_dir(repo_root())
        .env("PATH", format!("{}:{path}", bin_dir.display()))
        .output()
        .expect("homebrew verifier should run");
    fs::remove_dir_all(&temp).expect("homebrew verifier temp dir should be removed");
    output
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
// @verifies SPECIAL.DISTRIBUTION.BUILD_SCRIPT.LANGUAGE_PACK_REGISTRY
fn build_script_generates_the_builtin_language_pack_registry() {
    let build_script = read_repo_file("build.rs");

    assert!(
        build_script.contains("src/language_packs"),
        "build script should discover shipped language pack sources"
    );
    assert!(
        build_script.contains("language_pack_registry.rs"),
        "build script should emit the generated language pack registry"
    );
    assert!(
        build_script.contains("REGISTERED_LANGUAGE_PACKS"),
        "generated registry should expose the built-in language pack descriptor list"
    );
    assert!(
        build_script.contains("stem != \"mod\"") && build_script.contains("stem != \"python\""),
        "registry generation should skip helper modules that are not built-in language packs"
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.LEAN_KERNEL
fn github_release_workflow_embeds_lean_traceability_kernel_for_host_native_builds() {
    let workflow = read_repo_file(".github/workflows/release.yml");
    let dist_config = read_repo_file("dist-workspace.toml");
    let build_script = read_repo_file("build.rs");
    let verifier = read_repo_file("scripts/verify-lean-traceability-kernel.py");
    assert!(
        workflow.contains("uses: jdx/mise-action@v2"),
        "release builds should install the project tool manager before building the Lean kernel"
    );
    assert!(
        workflow.contains("SPECIAL_BUILD_LEAN_KERNEL: \"1\""),
        "release builds should request the standalone Lean traceability kernel"
    );
    assert!(
        dist_config.contains("allow-dirty = [\"ci\"]"),
        "cargo-dist should allow the intentional release workflow customization"
    );
    assert!(
        build_script.contains("lean_kernel_target_matches_host")
            && build_script.contains("skipping embedded Lean traceability kernel for cross target"),
        "cross builds should not embed a Lean executable built for the host architecture"
    );
    assert!(
        !build_script.contains("will use the Rust reference kernel"),
        "cross builds should not claim a production fallback to the debug/test Rust reference kernel"
    );
    assert!(
        verifier.contains("\"-Krelease\", \"build\", \"special_traceability_kernel\"")
            && verifier.contains("SPECIAL_TRACEABILITY_KERNEL_EXE")
            && verifier.contains("SPECIAL_REQUIRE_LEAN_KERNEL_TESTS"),
        "Lean verification should exercise the release-built kernel through required Rust-vs-Lean equivalence tests"
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

    let version = current_package_version();
    let release_json = valid_release_assets_json(|_| {});
    let formula = valid_formula_for_release(&version, None);
    let success = run_homebrew_formula_verifier(&release_json, &formula);
    assert!(
        success.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&success.stdout),
        String::from_utf8_lossy(&success.stderr)
    );

    let missing_url = run_homebrew_formula_verifier(
        &release_json,
        &formula.replace(
            &format!(
                "url \"https://github.com/LabLeaks/special/releases/download/v{version}/#{{archive}}\""
            ),
            "",
        ),
    );
    assert!(!missing_url.status.success());
    assert!(
        String::from_utf8_lossy(&missing_url.stderr)
            .contains("formula is missing templated release asset url"),
        "stderr:\n{}",
        String::from_utf8_lossy(&missing_url.stderr)
    );

    let missing_selector = run_homebrew_formula_verifier(
        &release_json,
        &formula.replace(
            "arm: \"special-cli-aarch64-apple-darwin.tar.xz\"",
            "arm: \"wrong-archive.tar.xz\"",
        ),
    );
    assert!(!missing_selector.status.success());
    assert!(
        String::from_utf8_lossy(&missing_selector.stderr)
            .contains("formula is missing archive selector entry"),
        "stderr:\n{}",
        String::from_utf8_lossy(&missing_selector.stderr)
    );
}

#[test]
fn homebrew_formula_verifier_requires_release_asset_digests() {
    let version = current_package_version();
    let release_json = valid_release_assets_json(|assets| {
        assets[0]
            .as_object_mut()
            .expect("asset should be an object")
            .remove("digest");
    });
    let formula = valid_formula_for_release(&version, None);
    let output = run_homebrew_formula_verifier(&release_json, &formula);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release asset is missing digest"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn homebrew_formula_verifier_checks_selector_checksum_pairing() {
    let version = current_package_version();
    let release_json = valid_release_assets_json(|_| {});
    let formula = valid_formula_for_release(
        &version,
        Some((
            "special-cli-x86_64-unknown-linux-gnu.tar.xz",
            "f".repeat(64),
        )),
    );
    let output = run_homebrew_formula_verifier(&release_json, &formula);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("formula checksum selector entry does not contain expected checksum"),
        "stderr:\n{}",
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
