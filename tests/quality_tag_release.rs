/**
@group SPECIAL.DISTRIBUTION.RELEASE_FLOW
special local release publication flow.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.PREPARE_CHANGELOG
the release script has a prepare phase that writes the exact-version `CHANGELOG.md` section from release-visible bullet lines entered during that phase.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.PREFLIGHT
before publishing, the release script automatically rejects missing or placeholder changelog entries and tracked private or generated project paths.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.VALIDATION_EVIDENCE
the release script has a validate phase that runs deterministic release validation commands and records ignored evidence tied to the release version and revision.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.REJECTS_LEGACY_CHECKLIST_BYPASS
the release script rejects legacy `--skip-checklist` and `--yes` bypass flags.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.DRY_RUN
the release script dry-run prints the planned prepare, validate, and publish pipeline plus publication commands without creating a tag, moving the main bookmark, pushing to origin, or updating Homebrew.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.MATCHES_MANIFEST_VERSION
the release script requires the requested tag version to exactly match the current `Cargo.toml` package version.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.RELEASE_REVISION
the release script publishes the current Jujutsu working-copy revision when it contains changes, or its parent when the working-copy revision is an empty child.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_AND_TAG
the release script publishes only through the explicit publish phase, which pushes the `main` bookmark with Jujutsu and the release Git tag to origin.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.VERIFIES_GITHUB_RELEASE
after pushing the release tag, the release script waits for the GitHub release artifacts to publish and verifies the release asset set.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.UPDATES_HOMEBREW
after the GitHub release is published, the release script updates the Homebrew tap formula for the current version and verifies the published formula against the release assets.

@module SPECIAL.TESTS.QUALITY_TAG_RELEASE
Release publication flow tests in `tests/quality_tag_release.rs`.
*/
// @fileimplements SPECIAL.TESTS.QUALITY_TAG_RELEASE
#[path = "support/quality.rs"]
mod support;

use serde_json::Value;
use std::{fs, path::Path, process::Command};

use support::{
    current_package_version, current_python_executable, default_release_revision,
    default_release_revset, release_tag_command_output, release_tag_dry_run,
    release_tag_live_output, tag_exists, tag_points_at_default_release_revision,
};

fn run_jj(temp_root: &Path, args: &[&str]) {
    let output = Command::new("jj")
        .args(args)
        .current_dir(temp_root)
        .output()
        .expect("jj command should run");
    assert!(
        output.status.success(),
        "jj {:?} failed\nstdout:\n{}\n\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn jj_commit_id(temp_root: &Path, revset: &str) -> String {
    let output = Command::new("jj")
        .args(["log", "-r", revset, "--no-graph", "-T", "commit_id"])
        .current_dir(temp_root)
        .output()
        .expect("jj log should run");
    assert!(
        output.status.success(),
        "jj log failed\nstdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("commit id should be utf-8")
        .trim()
        .to_string()
}

fn copy_release_script_fixture(temp_root: &Path) {
    let scripts = temp_root.join("scripts");
    fs::create_dir_all(&scripts).expect("scripts directory should be created");
    for file in ["tag-release.py", "release_tooling.py"] {
        fs::copy(
            support::repo_root().join("scripts").join(file),
            scripts.join(file),
        )
        .unwrap_or_else(|error| panic!("copy {file} into fixture: {error}"));
    }
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.DRY_RUN
fn release_tag_dry_run_lists_pipeline_and_publication_commands() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &[]);
    let revision = payload["revision"].clone();

    assert_eq!(payload["tag"], Value::String(format!("v{version}")));
    assert_eq!(
        payload["release_revset"],
        Value::String(default_release_revset())
    );
    assert!(
        payload["revision"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "revision should be a non-empty string"
    );
    let pipeline = payload["pipeline"]
        .as_array()
        .expect("pipeline should be an array");
    let phases: Vec<_> = pipeline
        .iter()
        .map(|entry| entry["phase"].as_str().expect("phase should be a string"))
        .collect();
    assert_eq!(phases, vec!["prepare", "validate", "publish"]);
    assert!(
        pipeline[0]["produces"]
            .as_str()
            .is_some_and(|value| value.contains("CHANGELOG.md")),
        "prepare phase should produce changelog content"
    );
    assert!(
        payload["validation_commands"]
            .as_array()
            .is_some_and(|commands| commands.len() >= 3),
        "dry-run should expose deterministic validation commands"
    );
    assert_eq!(
        payload["bookmark_command"]
            .as_array()
            .expect("bookmark_command should be an array"),
        &vec![
            Value::String("jj".to_string()),
            Value::String("bookmark".to_string()),
            Value::String("set".to_string()),
            Value::String("main".to_string()),
            Value::String("-r".to_string()),
            revision.clone(),
        ]
    );
    let tag_command = payload["tag_command"]
        .as_array()
        .expect("tag_command should be an array");
    assert_eq!(tag_command[0], Value::String("jj".to_string()));
    assert_eq!(tag_command[1], Value::String("tag".to_string()));
    assert_eq!(tag_command[2], Value::String("set".to_string()));
    let expected_tail = vec![
        Value::String(format!("v{version}")),
        Value::String("-r".to_string()),
        revision,
    ];
    if tag_command.len() == 7 {
        assert_eq!(tag_command[3], Value::String("--allow-move".to_string()));
        assert_eq!(&tag_command[4..], expected_tail.as_slice());
    } else {
        assert_eq!(tag_command.len(), 6);
        assert_eq!(&tag_command[3..], expected_tail.as_slice());
    }
    assert_eq!(
        payload["push_main_command"]
            .as_array()
            .expect("push_main_command should be an array"),
        &vec![
            Value::String("jj".to_string()),
            Value::String("git".to_string()),
            Value::String("push".to_string()),
            Value::String("--bookmark".to_string()),
            Value::String("main".to_string()),
        ]
    );
    assert_eq!(
        payload["push_tag_command"]
            .as_array()
            .expect("push_tag_command should be an array"),
        &if tag_command.len() == 7 {
            vec![
                Value::String("git".to_string()),
                Value::String("push".to_string()),
                Value::String("--force".to_string()),
                Value::String("origin".to_string()),
                Value::String(format!("refs/tags/v{version}")),
            ]
        } else {
            vec![
                Value::String("git".to_string()),
                Value::String("push".to_string()),
                Value::String("origin".to_string()),
                Value::String(format!("refs/tags/v{version}")),
            ]
        }
    );
    assert_eq!(
        payload["update_homebrew_formula_command"]
            .as_array()
            .expect("update_homebrew_formula_command should be an array"),
        &vec![
            Value::String(current_python_executable()),
            Value::String(
                support::repo_root()
                    .join("scripts/update-homebrew-formula.py")
                    .display()
                    .to_string(),
            ),
        ]
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.RELEASE_REVISION
fn release_tag_dry_run_targets_default_release_revision() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &[]);

    assert_eq!(
        payload["revision"],
        Value::String(default_release_revision())
    );
}

#[test]
fn release_tag_dry_run_uses_current_non_empty_jj_revision() {
    let root = std::env::temp_dir().join(format!(
        "special-tag-release-current-revision-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("existing temp repo should be removable");
    }
    fs::create_dir_all(&root).expect("temp repo should be created");
    copy_release_script_fixture(&root);
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"tag-fixture\"\nversion = \"1.2.3\"\nedition = \"2024\"\n",
    )
    .expect("Cargo.toml fixture should be written");

    run_jj(&root, &["git", "init", "."]);
    run_jj(
        &root,
        &[
            "--config=user.name=Test User",
            "--config=user.email=test@example.com",
            "commit",
            "-m",
            "release revision",
        ],
    );
    run_jj(&root, &["edit", "@-"]);

    let current = jj_commit_id(&root, "@");
    let parent = jj_commit_id(&root, "@-");
    let mut command = support::python3_command();
    let output = command
        .arg("scripts/tag-release.py")
        .arg("1.2.3")
        .arg("--dry-run")
        .current_dir(&root)
        .output()
        .expect("tag release dry-run should run");
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("dry-run output should be valid json");

    assert_ne!(current, parent);
    assert_eq!(payload["revision"], current);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.REJECTS_LEGACY_CHECKLIST_BYPASS
fn release_tag_script_rejects_legacy_checklist_bypass_flags() {
    let version = current_package_version();
    let execution = release_tag_live_output(&version, &["--skip-checklist"]);
    let output = execution.output;

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--skip-checklist"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(execution.mock_log.is_empty());
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.PREPARE_CHANGELOG
fn release_tag_prepare_writes_exact_changelog_section() {
    let root = std::env::temp_dir().join(format!(
        "special-tag-release-prepare-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("existing temp repo should be removable");
    }
    fs::create_dir_all(root.join("scripts")).expect("scripts directory should be created");
    fs::copy(
        support::repo_root().join("scripts/tag-release.py"),
        root.join("scripts/tag-release.py"),
    )
    .expect("tag-release fixture should be copied");
    fs::copy(
        support::repo_root().join("scripts/release_tooling.py"),
        root.join("scripts/release_tooling.py"),
    )
    .expect("release_tooling fixture should be copied");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"tag-fixture\"\nversion = \"1.2.3\"\nedition = \"2024\"\n",
    )
    .expect("Cargo.toml fixture should be written");
    fs::write(root.join("CHANGELOG.md"), "# Changelog\n\n## 1.2.2 - 2026-01-01\n\n- Old.\n")
        .expect("changelog fixture should be written");

    run_jj(&root, &["git", "init", "."]);
    run_jj(
        &root,
        &[
            "--config=user.name=Test User",
            "--config=user.email=test@example.com",
            "commit",
            "-m",
            "fixture",
        ],
    );

    let mut command = support::python3_command();
    let mut child = command
        .arg("scripts/tag-release.py")
        .arg("1.2.3")
        .arg("--prepare")
        .current_dir(&root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("prepare should spawn");
    use std::io::Write as _;
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(b"Added release pipeline.\nFixed docs.\n\n")
        .expect("prepare input should be written");
    let output = child.wait_with_output().expect("prepare should finish");
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let changelog = fs::read_to_string(root.join("CHANGELOG.md"))
        .expect("changelog should be readable");
    assert!(changelog.contains("## 1.2.3 - "));
    assert!(changelog.contains("- Added release pipeline."));
    assert!(changelog.contains("- Fixed docs."));
    assert!(changelog.find("## 1.2.3").unwrap() < changelog.find("## 1.2.2").unwrap());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.PREFLIGHT
fn release_tag_publish_rejects_bad_changelog_and_private_tracked_paths() {
    let root = std::env::temp_dir().join(format!(
        "special-tag-release-preflight-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("existing temp repo should be removable");
    }
    fs::create_dir_all(&root).expect("temp repo should be created");
    copy_release_script_fixture(&root);
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"tag-fixture\"\nversion = \"1.2.3\"\nedition = \"2024\"\n",
    )
    .expect("Cargo.toml fixture should be written");
    fs::write(
        root.join("CHANGELOG.md"),
        "# Changelog\n\n## 1.2.3 - 2026-01-01\n\n- TODO.\n",
    )
    .expect("changelog fixture should be written");

    run_jj(&root, &["git", "init", "."]);
    run_jj(
        &root,
        &[
            "--config=user.name=Test User",
            "--config=user.email=test@example.com",
            "commit",
            "-m",
            "fixture",
        ],
    );

    let mut placeholder_command = support::python3_command();
    let placeholder_output = placeholder_command
        .arg("scripts/tag-release.py")
        .arg("1.2.3")
        .arg("--publish")
        .arg("--allow-existing-tag")
        .arg("--allow-mock-publish")
        .current_dir(&root)
        .output()
        .expect("publish preflight should run");
    assert!(!placeholder_output.status.success());
    assert!(
        String::from_utf8_lossy(&placeholder_output.stderr).contains("real bullet notes"),
        "stderr:\n{}",
        String::from_utf8_lossy(&placeholder_output.stderr)
    );

    fs::write(
        root.join("CHANGELOG.md"),
        "# Changelog\n\n## 1.2.3 - 2026-01-01\n\n- Real release note.\n",
    )
    .expect("changelog fixture should be repaired");
    fs::create_dir_all(root.join(".codex-evals")).expect("private directory should be created");
    fs::write(root.join(".codex-evals/private.md"), "private\n")
        .expect("private tracked fixture should be written");

    let mut private_path_command = support::python3_command();
    let private_path_output = private_path_command
        .arg("scripts/tag-release.py")
        .arg("1.2.3")
        .arg("--publish")
        .arg("--allow-existing-tag")
        .arg("--allow-mock-publish")
        .current_dir(&root)
        .output()
        .expect("publish preflight should run");
    assert!(!private_path_output.status.success());
    assert!(
        String::from_utf8_lossy(&private_path_output.stderr).contains(".codex-evals/private.md"),
        "stderr:\n{}",
        String::from_utf8_lossy(&private_path_output.stderr)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.VALIDATION_EVIDENCE
fn release_tag_validate_records_version_and_revision_evidence() {
    let root = std::env::temp_dir().join(format!(
        "special-tag-release-validate-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("existing temp repo should be removable");
    }
    fs::create_dir_all(&root).expect("temp repo should be created");
    copy_release_script_fixture(&root);
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"tag-fixture\"\nversion = \"1.2.3\"\nedition = \"2024\"\n",
    )
    .expect("Cargo.toml fixture should be written");
    fs::write(
        root.join("CHANGELOG.md"),
        "# Changelog\n\n## 1.2.3 - 2026-01-01\n\n- Release validation pipeline.\n",
    )
    .expect("changelog fixture should be written");

    run_jj(&root, &["git", "init", "."]);
    run_jj(
        &root,
        &[
            "--config=user.name=Test User",
            "--config=user.email=test@example.com",
            "commit",
            "-m",
            "fixture",
        ],
    );
    let revision = jj_commit_id(&root, "@-");

    let mut command = support::python3_command();
    let output = command
        .arg("scripts/tag-release.py")
        .arg("1.2.3")
        .arg("--validate")
        .arg("--allow-existing-tag")
        .arg("--allow-mock-publish")
        .current_dir(&root)
        .output()
        .expect("validate should run");
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let evidence: Value = serde_json::from_str(
        &fs::read_to_string(root.join("_project/release/1.2.3.json"))
            .expect("validation evidence should be written"),
    )
    .expect("validation evidence should be json");

    assert_eq!(evidence["version"], Value::String("1.2.3".to_string()));
    assert_eq!(evidence["revision"], Value::String(revision));
    assert!(
        evidence["commands"]
            .as_array()
            .is_some_and(|commands| commands.len() >= 3),
        "validation evidence should record deterministic validation commands"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_AND_TAG
fn release_tag_publish_runs_publication_steps_after_pipeline_gates() {
    let version = current_package_version();
    let execution = release_tag_live_output(&version, &["--publish"]);
    let output = execution.output;

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(&format!("Published release v{version}"))
    );
    let labels: Vec<_> = execution
        .mock_log
        .iter()
        .map(|entry| entry["label"].as_str().expect("label should be a string"))
        .collect();
    assert_eq!(labels.first().copied(), Some("bookmark_main"));
    assert_eq!(labels.last().copied(), Some("verify_homebrew_formula"));
    assert!(labels.contains(&"push_main"));
    assert!(labels.contains(&"push_tag"));
    assert!(labels.contains(&"verify_github_release"));
    assert!(labels.contains(&"update_homebrew_formula"));
    let tag = format!("v{version}");
    let expected = if tag_points_at_default_release_revision(&tag) {
        vec![
            "bookmark_main",
            "push_main",
            "push_tag",
            "verify_github_release",
            "update_homebrew_formula",
            "verify_homebrew_formula",
        ]
    } else {
        vec![
            "bookmark_main",
            "set_tag",
            "push_main",
            "push_tag",
            "verify_github_release",
            "update_homebrew_formula",
            "verify_homebrew_formula",
        ]
    };
    assert_eq!(labels, expected);
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_AND_TAG
fn release_tag_dry_run_pushes_main_bookmark_and_release_tag() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &[]);

    assert_eq!(
        payload["push_main_command"]
            .as_array()
            .expect("push_main_command should be an array"),
        &vec![
            Value::String("jj".to_string()),
            Value::String("git".to_string()),
            Value::String("push".to_string()),
            Value::String("--bookmark".to_string()),
            Value::String("main".to_string()),
        ]
    );
    assert_eq!(
        payload["push_tag_command"]
            .as_array()
            .expect("push_tag_command should be an array"),
        &if tag_exists(&format!("v{version}")) {
            vec![
                Value::String("git".to_string()),
                Value::String("push".to_string()),
                Value::String("--force".to_string()),
                Value::String("origin".to_string()),
                Value::String(format!("refs/tags/v{version}")),
            ]
        } else {
            vec![
                Value::String("git".to_string()),
                Value::String("push".to_string()),
                Value::String("origin".to_string()),
                Value::String(format!("refs/tags/v{version}")),
            ]
        }
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_AND_TAG
fn release_tag_dry_run_force_pushes_existing_tag_when_requested() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &["--allow-existing-tag"]);

    assert_eq!(
        payload["push_tag_command"]
            .as_array()
            .expect("push_tag_command should be an array"),
        &if tag_exists(&format!("v{version}")) {
            vec![
                Value::String("git".to_string()),
                Value::String("push".to_string()),
                Value::String("--force".to_string()),
                Value::String("origin".to_string()),
                Value::String(format!("refs/tags/v{version}")),
            ]
        } else {
            vec![
                Value::String("git".to_string()),
                Value::String("push".to_string()),
                Value::String("origin".to_string()),
                Value::String(format!("refs/tags/v{version}")),
            ]
        }
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.VERIFIES_GITHUB_RELEASE
fn release_tag_dry_run_includes_github_release_verification_step() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &[]);

    assert_eq!(
        payload["verify_github_release_command"]
            .as_array()
            .expect("verify_github_release_command should be an array"),
        &vec![
            Value::String("bash".to_string()),
            Value::String(
                support::repo_root()
                    .join("scripts/verify-github-release-published.sh")
                    .display()
                    .to_string(),
            ),
        ]
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.UPDATES_HOMEBREW
fn release_tag_dry_run_includes_homebrew_update_and_verification_steps() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &[]);

    assert_eq!(
        payload["update_homebrew_formula_command"]
            .as_array()
            .expect("update_homebrew_formula_command should be an array"),
        &vec![
            Value::String(current_python_executable()),
            Value::String(
                support::repo_root()
                    .join("scripts/update-homebrew-formula.py")
                    .display()
                    .to_string(),
            ),
        ]
    );
    assert_eq!(
        payload["verify_homebrew_formula_command"]
            .as_array()
            .expect("verify_homebrew_formula_command should be an array"),
        &vec![
            Value::String("bash".to_string()),
            Value::String(
                support::repo_root()
                    .join("scripts/verify-homebrew-formula.sh")
                    .display()
                    .to_string(),
            ),
        ]
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.MATCHES_MANIFEST_VERSION
fn release_tag_script_requires_requested_tag_to_match_manifest_version() {
    let output = release_tag_command_output("0.3.1", &[]);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("does not match Cargo.toml version"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
