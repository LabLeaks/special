/**
@module SPECIAL.TESTS.QUALITY_TAG_RELEASE
Release-tagging wrapper tests in `tests/quality_tag_release.rs`.
*/
// @implements SPECIAL.TESTS.QUALITY_TAG_RELEASE
#[path = "support/quality.rs"]
mod support;

use std::string::String;

use serde_json::Value;

use support::{
    current_package_version, current_python_executable, current_revision, latest_semver_tag,
    release_tag_command_output, release_tag_dry_run, release_tag_live_output,
    release_tag_live_output_with_input, release_tag_validate_preview_shape_err, review_script_path,
    split_stdout_json_prefix,
};

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.PREVIOUS_TAG_DIFF
fn release_tag_script_runs_default_previous_tag_review_before_tagging() {
    let version = current_package_version();
    let payload = release_tag_dry_run(
        &version,
        &[],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[]}"#,
    );

    assert_eq!(
        payload["review_command"]
            .as_array()
            .expect("review_command should be an array"),
        &vec![
            Value::String(current_python_executable()),
            Value::String(review_script_path()),
            Value::String("--head".to_string()),
            Value::String(current_revision()),
            Value::String("--dry-run".to_string()),
            Value::String("--allow-mock".to_string())
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
            Value::String(current_revision())
        ]
    );
    assert_eq!(payload["review_preview"]["baseline"], latest_semver_tag());
    assert_eq!(payload["review_preview"]["full_scan"], Value::Bool(false));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.DRY_RUN
fn release_tag_dry_run_uses_review_dry_run_preview() {
    let version = current_package_version();
    let payload = release_tag_dry_run(
        &version,
        &[],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[]}"#,
    );

    assert!(payload.get("review_preview").is_some());
    assert!(payload.get("review_warnings").is_none());
    assert!(payload.get("would_prompt").is_none());
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.SKIP_REVIEW
fn release_tag_dry_run_can_skip_review_entirely() {
    let version = current_package_version();
    let payload = release_tag_dry_run(
        &version,
        &["--skip-review"],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[]}"#,
    );

    assert_eq!(payload["skip_review"], Value::Bool(true));
    assert!(payload.get("review_command").is_none());
    assert!(payload.get("review_preview").is_none());
}

#[test]
fn release_tag_dry_run_ignores_mocked_live_review_payloads() {
    let version = current_package_version();
    let output = release_tag_command_output(
        &version,
        &[],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[],"extra":true}"#,
        None,
    );

    assert!(output.status.success());
    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("dry-run output should be valid json");
    assert!(payload["review_preview"]["review_passes"].is_array());
    assert!(payload["review_preview"].get("warnings").is_none());
}

#[test]
fn release_tag_preview_validator_rejects_malformed_chunk_shape() {
    let stderr = release_tag_validate_preview_shape_err(
        r#"{
          "model":"gpt-5.3-codex",
          "review_mode":"default",
          "codex_invocation":{},
          "schema_path":"scripts/rust-release-review.schema.json",
          "backend":"jj",
          "baseline":"v0.2.0",
          "head":"@",
          "full_scan":false,
          "changed_files":["src/parser.rs"],
          "runner_warnings":[],
          "review_passes":[{
            "name":"core_modeling",
            "focus":["types"],
            "files":["src/parser.rs"],
            "chunks":[{"chunk_index":1,"files":["src/parser.rs"],"prompt":"x"}]
          }]
        }"#,
    );

    assert!(
        stderr.contains("review chunk missing required keys"),
        "stderr:\n{}",
        stderr
    );
}

#[test]
fn release_tag_preview_validator_rejects_malformed_file_context_entries() {
    let stderr = release_tag_validate_preview_shape_err(
        r#"{
          "model":"gpt-5.3-codex",
          "review_mode":"default",
          "codex_invocation":{},
          "schema_path":"scripts/rust-release-review.schema.json",
          "backend":"jj",
          "baseline":"v0.2.0",
          "head":"@",
          "full_scan":false,
          "changed_files":["src/parser.rs"],
          "runner_warnings":[],
          "review_passes":[{
            "name":"core_modeling",
            "focus":["types"],
            "files":["src/parser.rs"],
            "chunks":[{
              "chunk_index":1,
              "chunk_count":1,
              "files":["src/parser.rs"],
              "estimated_chars":100,
              "file_contexts":[{"path":"src/parser.rs","start_line":"1","end_line":2,"content":"x"}],
              "prompt":"x"
            }]
          }]
        }"#,
    );

    assert!(
        stderr.contains("file_context `start_line` must be an integer"),
        "stderr:\n{}",
        stderr
    );
}

#[test]
fn release_tag_preview_validator_rejects_inverted_file_context_ranges() {
    let stderr = release_tag_validate_preview_shape_err(
        r#"{
          "model":"gpt-5.3-codex",
          "review_mode":"default",
          "codex_invocation":{},
          "schema_path":"scripts/rust-release-review.schema.json",
          "backend":"jj",
          "baseline":"v0.2.0",
          "head":"@",
          "full_scan":false,
          "changed_files":["src/parser.rs"],
          "runner_warnings":[],
          "review_passes":[{
            "name":"core_modeling",
            "focus":["types"],
            "files":["src/parser.rs"],
            "chunks":[{
              "chunk_index":1,
              "chunk_count":1,
              "files":["src/parser.rs"],
              "estimated_chars":100,
              "file_contexts":[{"path":"src/parser.rs","start_line":5,"end_line":2,"content":"x"}],
              "prompt":"x"
            }]
          }]
        }"#,
    );

    assert!(
        stderr.contains("`end_line` must be at least `start_line`"),
        "stderr:\n{}",
        stderr
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.MODE_FLAGS
fn release_tag_script_passes_through_fast_and_smart_flags() {
    let version = current_package_version();
    let fast_payload = release_tag_dry_run(
        &version,
        &["--fast"],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[]}"#,
    );
    assert_eq!(
        fast_payload["review_command"]
            .as_array()
            .expect("review_command should be an array"),
        &vec![
            Value::String(current_python_executable()),
            Value::String(review_script_path()),
            Value::String("--fast".to_string()),
            Value::String("--head".to_string()),
            Value::String(current_revision()),
            Value::String("--dry-run".to_string()),
            Value::String("--allow-mock".to_string())
        ]
    );

    let smart_payload = release_tag_dry_run(
        &version,
        &["--smart"],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[]}"#,
    );
    assert_eq!(
        smart_payload["review_command"]
            .as_array()
            .expect("review_command should be an array"),
        &vec![
            Value::String(current_python_executable()),
            Value::String(review_script_path()),
            Value::String("--smart".to_string()),
            Value::String("--head".to_string()),
            Value::String(current_revision()),
            Value::String("--dry-run".to_string()),
            Value::String("--allow-mock".to_string())
        ]
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.SCOPE_OVERRIDES
fn release_tag_script_passes_through_review_scope_overrides() {
    let version = current_package_version();
    let full_payload = release_tag_dry_run(
        &version,
        &["--full"],
        r#"{"baseline":null,"full_scan":true,"summary":"clean","warnings":[]}"#,
    );
    assert_eq!(
        full_payload["review_command"]
            .as_array()
            .expect("review_command should be an array"),
        &vec![
            Value::String(current_python_executable()),
            Value::String(review_script_path()),
            Value::String("--full".to_string()),
            Value::String("--head".to_string()),
            Value::String(current_revision()),
            Value::String("--dry-run".to_string()),
            Value::String("--allow-mock".to_string())
        ]
    );

    let base_payload = release_tag_dry_run(
        &version,
        &["--base", "v0.1.0"],
        r#"{"baseline":"v0.1.0","full_scan":false,"summary":"clean","warnings":[]}"#,
    );
    assert_eq!(
        base_payload["review_command"]
            .as_array()
            .expect("review_command should be an array"),
        &vec![
            Value::String(current_python_executable()),
            Value::String(review_script_path()),
            Value::String("--base".to_string()),
            Value::String("v0.1.0".to_string()),
            Value::String("--head".to_string()),
            Value::String(current_revision()),
            Value::String("--dry-run".to_string()),
            Value::String("--allow-mock".to_string())
        ]
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.SURFACES_REVIEW_FAILURES
fn release_tag_script_prints_structured_review_payload_on_failure() {
    let version = current_package_version();
    let output = release_tag_live_output(
        &version,
        &[],
        r#"{
          "baseline":"v0.2.0",
          "full_scan":false,
          "summary":"One warn-level issue found.",
          "warnings":[
            {
              "id":"warn-1",
              "category":"maintainability",
              "severity":"warn",
              "title":"Example warning",
              "why_it_matters":"Review output should be surfaced on failure.",
              "evidence":[{"path":"src/cli.rs","line":1,"detail":"Example evidence"}],
              "recommendation":"Show the review payload before aborting."
            }
          ]
        }"#,
        Some("1"),
    );

    assert!(!output.status.success());
    let (response, remainder) = split_stdout_json_prefix(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(response["warnings"].as_array().unwrap().len(), 1);
    assert_eq!(response["warnings"][0]["title"], "Example warning");
    assert!(remainder.trim().is_empty());
    assert!(stderr.contains("release review did not complete cleanly; tag not created"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.PROMPTS_ON_WARNINGS
fn release_tag_script_reports_that_warnings_would_prompt() {
    let version = current_package_version();
    let output = release_tag_live_output_with_input(
        &version,
        &[],
        r#"{
          "baseline":"v0.2.0",
          "full_scan":false,
          "summary":"One warn-level issue found.",
          "warnings":[
            {
              "id":"warn-1",
              "category":"release-risk",
              "severity":"warn",
              "title":"Example warning",
              "why_it_matters":"Warn-only findings should prompt before tagging.",
              "evidence":[{"path":"src/cli.rs","line":1,"detail":"Example evidence"}],
              "recommendation":"Tighten the release flow."
            }
          ]
        }"#,
        None,
        "n\n",
    );

    assert!(!output.status.success());
    let (response, remainder) = split_stdout_json_prefix(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(response["warnings"].as_array().unwrap().len(), 1);
    assert!(remainder.contains("Rust release review returned 1 warning(s). Create"));
    assert!(stderr.contains("aborted release tagging"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.NON_INTERACTIVE_WARNINGS_ABORT
fn release_tag_script_aborts_cleanly_when_warning_prompt_has_no_input() {
    let version = current_package_version();
    let output = release_tag_live_output(
        &version,
        &[],
        r#"{
          "baseline":"v0.2.0",
          "full_scan":false,
          "summary":"One warn-level issue found.",
          "warnings":[
            {
              "id":"warn-1",
              "category":"release-risk",
              "severity":"warn",
              "title":"Example warning",
              "why_it_matters":"Warn-only findings should not crash without stdin.",
              "evidence":[{"path":"src/cli.rs","line":1,"detail":"Example evidence"}],
              "recommendation":"Abort cleanly."
            }
          ]
        }"#,
        None,
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("interactive confirmation is unavailable"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.MATCHES_MANIFEST_VERSION
fn release_tag_script_requires_requested_tag_to_match_manifest_version() {
    let output = release_tag_command_output(
        "0.3.1",
        &[],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[]}"#,
        None,
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("does not match Cargo.toml version"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.VALIDATES_REVIEW_PAYLOAD
fn release_tag_script_rejects_malformed_review_payloads() {
    let output = release_tag_live_output(
        "0.3.0",
        &[],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean"}"#,
        None,
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing required keys"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = release_tag_live_output(
        "0.3.0",
        &[],
        r#"{"baseline":"v0.2.0","full_scan":false,"summary":"clean","warnings":[],"extra":true}"#,
        None,
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unexpected keys"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = release_tag_live_output(
        "0.3.0",
        &[],
        r#"{
          "baseline":"v0.2.0",
          "full_scan":false,
          "summary":"clean",
          "warnings":[{"id":"warn-1","category":"release-risk"}]
        }"#,
        None,
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("warning missing required keys"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
