/**
@group SPECIAL.DISTRIBUTION.RELEASE_FLOW
special local release publication flow.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.CHECKLIST
before publishing, the release script interactively confirms easy-to-forget release tasks such as updating public docs, updating `CHANGELOG.md`, bumping the release version, and running core validation.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.SKIP_CHECKLIST_BYPASSES_CHECKLIST
the release script accepts `--skip-checklist` to bypass the interactive prerelease checklist.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.DRY_RUN
the release script dry-run prints the planned checklist and publication commands without creating a tag, moving the main bookmark, pushing to origin, or updating Homebrew.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.MATCHES_MANIFEST_VERSION
the release script requires the requested tag version to exactly match the current `Cargo.toml` package version.

@spec SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_AND_TAG
the release script publishes the release revision by pushing both the `main` bookmark and the release tag to origin.

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

use support::{
    current_package_version, current_python_executable, release_tag_command_output,
    release_tag_dry_run, release_tag_live_output, release_tag_live_output_with_input,
};

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.DRY_RUN
fn release_tag_dry_run_lists_checklist_and_publication_commands() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &[]);
    let revision = payload["revision"].clone();

    assert_eq!(payload["tag"], Value::String(format!("v{version}")));
    assert!(
        payload["revision"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "revision should be a non-empty string"
    );
    assert_eq!(
        payload["checklist"]
            .as_array()
            .expect("checklist should be an array")
            .len(),
        4
    );
    let checklist = payload["checklist"]
        .as_array()
        .expect("checklist should be an array");
    let checklist_ids: Vec<_> = checklist
        .iter()
        .map(|entry| {
            entry["id"]
                .as_str()
                .expect("checklist id should be a string")
        })
        .collect();
    assert_eq!(
        checklist_ids,
        vec!["readme", "changelog", "version", "validation"]
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
    assert_eq!(
        payload["tag_command"]
            .as_array()
            .expect("tag_command should be an array"),
        &vec![
            Value::String("jj".to_string()),
            Value::String("tag".to_string()),
            Value::String("set".to_string()),
            Value::String(format!("v{version}")),
            Value::String("-r".to_string()),
            revision,
        ]
    );
    assert_eq!(
        payload["push_command"]
            .as_array()
            .expect("push_command should be an array"),
        &vec![
            Value::String("jj".to_string()),
            Value::String("git".to_string()),
            Value::String("push".to_string()),
            Value::String("--bookmark".to_string()),
            Value::String("main".to_string()),
            Value::String("--tag".to_string()),
            Value::String(format!("v{version}")),
        ]
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
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.CHECKLIST
fn release_tag_script_aborts_when_checklist_answer_is_no() {
    let version = current_package_version();
    let execution = release_tag_live_output_with_input(&version, &[], "n\n");
    let output = execution.output;

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("aborted release publishing"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(execution.mock_log.is_empty());
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.CHECKLIST
fn release_tag_script_aborts_cleanly_when_checklist_has_no_input() {
    let version = current_package_version();
    let execution = release_tag_live_output(&version, &[]);
    let output = execution.output;

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("interactive release checklist is unavailable"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--skip-checklist"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(execution.mock_log.is_empty());
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.SKIP_CHECKLIST_BYPASSES_CHECKLIST
fn release_tag_script_skip_checklist_bypasses_checklist_and_runs_publication_steps() {
    let version = current_package_version();
    let execution = release_tag_live_output(&version, &["--skip-checklist"]);
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
    assert_eq!(
        labels,
        vec![
            "bookmark_main",
            "set_tag",
            "push_release",
            "verify_github_release",
            "update_homebrew_formula",
            "verify_homebrew_formula",
        ]
    );
}

#[test]
// @verifies SPECIAL.DISTRIBUTION.RELEASE_FLOW.PUSHES_MAIN_AND_TAG
fn release_tag_dry_run_pushes_main_bookmark_and_release_tag() {
    let version = current_package_version();
    let payload = release_tag_dry_run(&version, &[]);

    assert_eq!(
        payload["push_command"]
            .as_array()
            .expect("push_command should be an array"),
        &vec![
            Value::String("jj".to_string()),
            Value::String("git".to_string()),
            Value::String("push".to_string()),
            Value::String("--bookmark".to_string()),
            Value::String("main".to_string()),
            Value::String("--tag".to_string()),
            Value::String(format!("v{version}")),
        ]
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
