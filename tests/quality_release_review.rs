/**
@group SPECIAL.QUALITY.RUST
Rust quality contract surface for clippy and release-review tooling.

@group SPECIAL.QUALITY.RUST.RELEASE_REVIEW
Rust release-review contract surface.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SPEC_OWNED
the release-review wrapper script carries the proving surface for the release-review contract.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.DEFAULT_MODEL
the default release-review mode uses `gpt-5.3-codex`.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.FAST_MODEL
the fast release-review mode uses `gpt-5.3-codex-spark`.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SMART_MODEL
the smart release-review mode uses `gpt-5.4`.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.STRUCTURED_OUTPUT
the release-review wrapper validates structured warning output against the review contract.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.CODE_ONLY_SURFACE
release review operates only on the repo’s code/tooling surface, not general product/spec/architecture prose.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.READ_ONLY_SANDBOX
release review invokes Codex in a read-only sandbox.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.NO_WEB
release review disables web access.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.PROJECT_ROOT_READ_SCOPE
release review grants read access only to the project root.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.DIFF_SCOPED_BY_DEFAULT
without `--full`, release review is diff-scoped against the baseline tag.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.JJ_LATEST_TAG_BASELINE
release review uses the latest reachable semver tag as the default baseline.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SYNTAX_AWARE_CHANGED_CONTEXT
release review extracts syntax-aware changed context for supported languages.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.INPUT_BUDGET
release review budgets prompt input before invoking Codex.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.CHUNKED_CONTEXT
release review splits review context into chunks when needed to fit the input budget.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SKIPPED_CHUNK_WARNINGS
release review emits runner warnings when it must skip or degrade chunk context.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.FULL_SCAN_MODE
`--full` makes release review operate on the full supported review surface instead of the diff-scoped default.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.WARN_ONLY
release review reports findings as warnings rather than failing the wrapper on model findings alone.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.LOCAL_ONLY
release review runs locally and does not publish findings to external services.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.NO_BYTECODE_ARTIFACTS
release review does not leave Python bytecode artifacts in the repo.

@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.MANUAL_ONLY
release review runs only when invoked manually.

@module SPECIAL.TESTS.QUALITY_RELEASE_REVIEW
Release-review wrapper tests in `tests/quality_release_review.rs`.
*/
// @fileimplements SPECIAL.TESTS.QUALITY_RELEASE_REVIEW
#[path = "support/quality.rs"]
mod support;

use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use support::{
    latest_reachable_semver_tag, latest_reachable_semver_tag_for_repo,
    python_entrypoint_runtime_flag, python3_command, release_review_changed_line_ranges,
    release_review_chunk_helper, release_review_dry_run, release_review_extract_context_ranges,
    release_review_extract_full_scan_context_ranges, release_review_merge_responses,
    release_review_passes_for, release_review_schema, release_review_validate_response_shape_err,
    release_review_validate_response_shape_ok, workflow_files,
};

static TEMP_REVIEW_REPO_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_review_temp_repo(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = TEMP_REVIEW_REPO_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .join("special-release-review")
        .join(format!("{prefix}-{}-{nanos}-{counter}", std::process::id()))
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.DEFAULT_MODEL
fn release_review_defaults_to_regular_53_model() {
    let payload = release_review_dry_run(&[]);
    assert_eq!(payload["model"], "gpt-5.3-codex");
    assert_eq!(payload["review_mode"], "default");
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.FAST_MODEL
fn release_review_uses_fast_model_when_requested() {
    let payload = release_review_dry_run(&["--fast"]);
    assert_eq!(payload["model"], "gpt-5.3-codex-spark");
    assert_eq!(payload["review_mode"], "fast");
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SMART_MODEL
fn release_review_uses_smart_model_when_requested() {
    let payload = release_review_dry_run(&["--smart"]);
    assert_eq!(payload["model"], "gpt-5.4");
    assert_eq!(payload["review_mode"], "smart");
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.STRUCTURED_OUTPUT
fn release_review_validator_accepts_valid_payload_shape() {
    let validated = release_review_validate_response_shape_ok(
        r#"{
          "baseline": "v0.2.0",
          "full_scan": false,
          "summary": "clean",
          "warnings": []
        }"#,
    );
    assert_eq!(validated["summary"], "clean");

    let schema = release_review_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["warnings"].is_object());
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.STRUCTURED_OUTPUT
fn release_review_validator_rejects_schema_drift() {
    let stderr = release_review_validate_response_shape_err(
        r#"{
          "baseline": null,
          "full_scan": true,
          "summary": "clean",
          "warnings": [],
          "unexpected": true
        }"#,
    );

    assert!(stderr.contains("unexpected keys"), "stderr:\n{}", stderr);
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.STRUCTURED_OUTPUT
fn release_review_validator_rejects_missing_required_fields() {
    let stderr = release_review_validate_response_shape_err(
        r#"{
          "baseline": null,
          "full_scan": true,
          "summary": "clean"
        }"#,
    );

    assert!(
        stderr.contains("missing required keys"),
        "stderr:\n{}",
        stderr
    );
}

#[test]
fn release_review_validator_rejects_empty_warning_evidence() {
    let stderr = release_review_validate_response_shape_err(
        r#"{
          "baseline": null,
          "full_scan": true,
          "summary": "warn",
          "warnings": [
            {
              "id": "warn-1",
              "category": "test-quality",
              "severity": "warn",
              "title": "Example warning",
              "why_it_matters": "Warnings should stay actionable.",
              "evidence": [],
              "recommendation": "Include anchored evidence."
            }
          ]
        }"#,
    );

    assert!(stderr.contains("must not be empty"), "stderr:\n{}", stderr);
}

#[test]
fn release_review_validator_rejects_non_string_warning_category_cleanly() {
    let stderr = release_review_validate_response_shape_err(
        r#"{
          "baseline": null,
          "full_scan": true,
          "summary": "warn",
          "warnings": [
            {
              "id": "warn-1",
              "category": {"bad": true},
              "severity": "warn",
              "title": "Example warning",
              "why_it_matters": "Warnings should stay actionable.",
              "evidence": [{"path":"src/lib.rs","line":1,"detail":"anchor"}],
              "recommendation": "Use a valid category."
            }
          ]
        }"#,
    );

    assert!(
        stderr.contains("warning `category` must be a string"),
        "stderr:\n{}",
        stderr
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.READ_ONLY_SANDBOX
fn release_review_script_uses_read_only_sandbox() {
    let payload = release_review_dry_run(&[]);
    assert_eq!(payload["codex_invocation"]["sandbox_mode"], "read-only");
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.NO_WEB
fn release_review_script_disables_web_search() {
    let payload = release_review_dry_run(&[]);
    assert_eq!(payload["codex_invocation"]["web_search"], "disabled");
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.PROJECT_ROOT_READ_SCOPE
fn release_review_script_uses_explicit_project_root_read_scope() {
    let payload = release_review_dry_run(&[]);
    assert_eq!(
        payload["codex_invocation"],
        json!({
            "model": "gpt-5.3-codex",
            "sandbox_mode": "read-only",
            "web_search": "disabled",
            "default_permissions": "release_review",
            "filesystem_permissions": {
                ":project_roots": {
                    ".": "read"
                }
            }
        })
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.CODE_ONLY_SURFACE
fn release_review_full_scan_stays_on_code_surface() {
    let payload = release_review_dry_run(&["--full"]);
    let changed_files = payload["changed_files"]
        .as_array()
        .expect("changed_files should be an array");

    assert!(
        changed_files
            .iter()
            .any(|value| { value.as_str() == Some(".github/workflows/release.yml") })
    );
    for value in changed_files {
        let path = value.as_str().expect("changed file should be a string");
        assert!(
            path == "Cargo.toml"
                || path.starts_with("src/")
                || path.starts_with("tests/")
                || path.starts_with("scripts/")
                || path.starts_with(".github/workflows/"),
            "unexpected review-surface path: {path}"
        );
        assert!(
            path == "Cargo.toml"
                || [".rs", ".py", ".sh", ".json", ".yml", ".yaml"]
                    .iter()
                    .any(|suffix| path.ends_with(suffix)),
            "unexpected review-surface file type: {path}"
        );
    }
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.DIFF_SCOPED_BY_DEFAULT
fn release_review_defaults_to_diff_scope() {
    let payload = release_review_dry_run(&[]);
    assert_eq!(payload["full_scan"], false);
    assert!(payload["baseline"].is_string());
    let all_prompts_include_diff = payload["review_passes"]
        .as_array()
        .expect("review_passes should be an array")
        .iter()
        .flat_map(|pass| {
            pass["chunks"]
                .as_array()
                .expect("chunks should be an array")
                .iter()
        })
        .all(|chunk| {
            chunk["prompt"]
                .as_str()
                .expect("prompt should be present")
                .contains("<diff>")
        });
    assert!(all_prompts_include_diff);
}

#[test]
fn release_review_prompt_stays_on_implementation_quality() {
    let payload = release_review_dry_run(&[]);
    let first_prompt = payload["review_passes"]
        .as_array()
        .expect("review_passes should be an array")
        .iter()
        .flat_map(|pass| {
            pass["chunks"]
                .as_array()
                .expect("chunks should be an array")
                .iter()
        })
        .map(|chunk| chunk["prompt"].as_str().expect("prompt should be present"))
        .next()
        .expect("at least one prompt should be present");

    assert!(first_prompt.contains("implementation-quality review of code changes"));
    assert!(first_prompt.contains("Do not perform a spec review or an architecture review."));
    assert!(
        first_prompt.contains(
            "Assume intended product behavior and intended architecture are correct inputs"
        )
    );
    assert!(first_prompt.contains("Do not recommend different product semantics"));
    assert!(first_prompt.contains("You may flag low-level design issues inside the changed code"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.JJ_LATEST_TAG_BASELINE
fn release_review_defaults_to_latest_semver_tag_in_jj_repo() {
    let payload = release_review_dry_run(&[]);
    assert_eq!(payload["backend"], "jj");
    assert_eq!(payload["baseline"], latest_reachable_semver_tag());
}

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

#[test]
fn release_review_uses_latest_reachable_semver_tag_not_global_max_in_jj_repo() {
    let root = unique_review_temp_repo("jj-baseline");
    if root.exists() {
        fs::remove_dir_all(&root).expect("existing temp repo should be removable");
    }
    fs::create_dir_all(&root).expect("temp repo should be created");

    run_jj(&root, &["git", "init", "."]);

    fs::write(root.join("a.txt"), "one\n").expect("first fixture should be written");
    run_jj(
        &root,
        &[
            "--config=user.name=Test User",
            "--config=user.email=test@example.com",
            "commit",
            "-m",
            "first",
        ],
    );
    run_jj(&root, &["tag", "set", "-r", "@-", "v0.1.0"]);

    fs::write(root.join("b.txt"), "two\n").expect("second fixture should be written");
    run_jj(
        &root,
        &[
            "--config=user.name=Test User",
            "--config=user.email=test@example.com",
            "commit",
            "-m",
            "second",
        ],
    );
    run_jj(&root, &["tag", "set", "-r", "@-", "v0.4.1"]);

    run_jj(
        &root,
        &["new", "v0.1.0", "--no-edit", "-m", "unrelated_branch"],
    );
    run_jj(
        &root,
        &[
            "tag",
            "set",
            "-r",
            "subject(exact:unrelated_branch)",
            "v9.0.0",
        ],
    );

    assert_eq!(
        latest_reachable_semver_tag_for_repo(&root, "jj", "@"),
        "v0.4.1".to_string()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn release_review_uses_latest_reachable_semver_tag_not_global_max_in_git_repo() {
    let root = unique_review_temp_repo("git-baseline");
    if root.exists() {
        fs::remove_dir_all(&root).expect("existing temp repo should be removable");
    }
    fs::create_dir_all(&root).expect("temp repo should be created");

    let run_git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .expect("git should run");
        assert!(
            output.status.success(),
            "git {:?} failed\nstdout:\n{}\n\nstderr:\n{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };

    run_git(&["init", "."]);
    run_git(&["config", "user.name", "Test User"]);
    run_git(&["config", "user.email", "test@example.com"]);

    fs::write(root.join("a.txt"), "one\n").expect("first fixture should be written");
    run_git(&["add", "a.txt"]);
    run_git(&["commit", "-m", "first"]);
    run_git(&["tag", "v0.1.0"]);

    fs::write(root.join("b.txt"), "two\n").expect("second fixture should be written");
    run_git(&["add", "b.txt"]);
    run_git(&["commit", "-m", "second"]);
    run_git(&["tag", "v0.4.1"]);

    run_git(&["checkout", "-b", "unrelated", "v0.1.0"]);
    fs::write(root.join("c.txt"), "three\n").expect("third fixture should be written");
    run_git(&["add", "c.txt"]);
    run_git(&["commit", "-m", "unrelated"]);
    run_git(&["tag", "v9.0.0"]);
    run_git(&["checkout", "-"]);

    assert_eq!(
        latest_reachable_semver_tag_for_repo(&root, "git", "HEAD"),
        "v0.4.1".to_string()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SYNTAX_AWARE_CHANGED_CONTEXT
fn release_review_extracts_local_rust_item_context() {
    let content = r#"fn untouched() {
    println!("untouched");
}

fn changed_target() {
    let value = 1;
    println!("{value}");
}

fn trailing() {
    println!("trailing");
}
"#;
    let ranges = release_review_extract_context_ranges("src/example.rs", content, 6, 6);
    assert_eq!(
        ranges,
        json!([[5, 8]]),
        "changed rust context should resolve to the enclosing item range, not the whole file"
    );
}

#[test]
fn release_review_ignores_string_and_comment_braces_when_extracting_rust_context() {
    let content = r#"fn changed_target() {
    let template = "{ not structural }";
    // comment with }
}

fn trailing() {
    println!("trailing");
}
"#;
    let ranges = release_review_extract_context_ranges("src/example.rs", content, 2, 2);
    assert_eq!(ranges, json!([[1, 4]]));
}

#[test]
fn release_review_full_scan_covers_unmatched_rust_preamble() {
    let content = r#"use crate::demo::Demo;

fn changed_target() {
    println!("hi");
}
"#;
    let ranges = release_review_extract_full_scan_context_ranges("src/example.rs", content);
    assert_eq!(ranges, json!([[1, 5]]));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.INPUT_BUDGET
fn release_review_chunks_stay_within_budget() {
    let payload = release_review_dry_run(&["--full"]);
    let review_passes = payload["review_passes"]
        .as_array()
        .expect("review_passes should be an array");

    for pass in review_passes {
        for chunk in pass["chunks"]
            .as_array()
            .expect("chunks should be an array")
        {
            let estimated_chars = chunk["estimated_chars"]
                .as_u64()
                .expect("estimated chars should be numeric");
            let prompt = chunk["prompt"].as_str().expect("prompt should be present");
            assert!(
                estimated_chars <= 128_000,
                "chunk exceeded estimated input budget: {}",
                estimated_chars
            );
            assert!(
                prompt.len() <= 128_000,
                "chunk exceeded actual input budget: {}",
                prompt.len()
            );
        }
    }
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.CHUNKED_CONTEXT
fn release_review_splits_oversized_context_into_multiple_chunks() {
    let payload = release_review_chunk_helper(80_000);
    let chunks = payload["chunks"]
        .as_array()
        .expect("chunks should be an array");
    assert!(
        chunks.len() >= 2,
        "oversized review contexts should be chunked"
    );
    assert!(
        payload["runner_warnings"]
            .as_array()
            .expect("runner_warnings should be an array")
            .is_empty()
    );
}

#[test]
fn release_review_adds_default_pass_for_unmatched_changed_files() {
    let passes = release_review_passes_for(&["src/parser.rs", "tests/support/quality.rs"]);
    let passes = passes.as_array().expect("passes should be an array");

    assert!(passes.iter().any(|pass| pass["name"] == "core_modeling"));
    assert!(passes.iter().any(|pass| {
        pass["name"] == "default"
            && pass["files"]
                .as_array()
                .expect("files should be an array")
                .iter()
                .any(|value| value.as_str() == Some("tests/support/quality.rs"))
    }));
}

#[test]
fn release_review_merge_output_is_stable_across_response_order() {
    let responses = json!([
        [
            "quality_tooling",
            2,
            {
                "warnings": [{
                    "id": "warn-a",
                    "category": "maintainability",
                    "severity": "warn",
                    "title": "Chunked warning",
                    "why_it_matters": "Ordering should not change identity.",
                    "evidence": [{
                        "path": "scripts/review-rust-release-style.py",
                        "line": 100,
                        "detail": "First anchor"
                    }],
                    "recommendation": "Keep ids stable."
                }]
            }
        ],
        [
            "core_modeling",
            1,
            {
                "warnings": [{
                    "id": "warn-b",
                    "category": "type-design",
                    "severity": "warn",
                    "title": "Model warning",
                    "why_it_matters": "Sorting should use stable anchors.",
                    "evidence": [{
                        "path": "src/model.rs",
                        "line": 10,
                        "detail": "Model anchor"
                    }],
                    "recommendation": "Keep output deterministic."
                }]
            }
        ]
    ]);
    let first = release_review_merge_responses(&responses);
    let reversed = json!([responses[1].clone(), responses[0].clone()]);
    let second = release_review_merge_responses(&reversed);

    assert_eq!(first["warnings"], second["warnings"]);
}

#[test]
fn release_review_merge_uses_stable_ids_instead_of_chunk_ordinals() {
    let payload = release_review_merge_responses(&json!([[
        "quality_tooling",
        7,
        {
            "warnings": [{
                "id": "warn-a",
                "category": "maintainability",
                "severity": "warn",
                "title": "Chunked warning",
                "why_it_matters": "Chunk indices should not leak into stable ids.",
                "evidence": [{
                    "path": "scripts/review-rust-release-style.py",
                    "line": 100,
                    "detail": "First anchor"
                }],
                "recommendation": "Derive ids from stable evidence."
            }]
        }
    ]]));

    let id = payload["warnings"][0]["id"].as_str().unwrap_or_default();
    assert!(id.starts_with("quality_tooling:maintainability:"));
    assert!(!id.contains("chunk7"));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SKIPPED_CHUNK_WARNINGS
fn release_review_reports_runner_warning_for_unsendable_chunk() {
    let payload = release_review_chunk_helper(140_000);
    assert!(
        payload["runner_warnings"]
            .as_array()
            .expect("runner_warnings should be an array")
            .iter()
            .any(|warning| warning
                .as_str()
                .unwrap()
                .contains("exceeds 128000 char budget"))
    );
    assert!(
        payload["chunks"]
            .as_array()
            .expect("chunks should be an array")
            .iter()
            .any(|chunk| {
                chunk["files"]
                    .as_array()
                    .expect("files should be an array")
                    .iter()
                    .any(|value| value.as_str() == Some("src/example.rs"))
                    && chunk["file_contexts"]
                        .as_array()
                        .expect("file_contexts should be an array")
                        .is_empty()
            }),
        "oversized file snippets should still fall back to a diff-only review chunk"
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.FULL_SCAN_MODE
fn release_review_supports_full_scan_mode() {
    let payload = release_review_dry_run(&["--full"]);
    assert_eq!(payload["full_scan"], true);
    assert_eq!(payload["baseline"], Value::Null);
    assert!(
        payload["changed_files"]
            .as_array()
            .expect("changed_files should be an array")
            .len()
            > 1
    );
}

#[test]
fn release_review_tracks_actual_changed_lines_not_hunk_header_ranges() {
    let diff = r#"diff --git a/src/example.rs b/src/example.rs
index 1111111..2222222 100644
--- a/src/example.rs
+++ b/src/example.rs
@@ -10,3 +10,4 @@ fn demo() {
     let keep = 1;
-    removed();
+    added();
+    also_added();
     trailing();
 }
"#;

    let ranges = release_review_changed_line_ranges(diff);
    assert_eq!(ranges, json!({"src/example.rs": [[11, 12]]}));
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.WARN_ONLY
fn release_review_exits_successfully_when_codex_returns_warnings() {
    let mut command = python3_command();
    command
        .arg("scripts/review-rust-release-style.py")
        .arg("--allow-mock")
        .arg("--full")
        .current_dir(support::repo_root())
        .env("SPECIAL_RUST_RELEASE_REVIEW_ALLOW_MOCK", "1")
        .env(
            "SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT",
            r#"{
              "baseline": null,
              "full_scan": true,
              "summary": "One warn-level issue found.",
              "warnings": [
                {
                  "id": "warn-1",
                  "category": "test-quality",
                  "severity": "warn",
                  "title": "Example warning",
                  "why_it_matters": "Warn-only findings should not fail the script.",
                  "evidence": [
                    {
                      "path": "src/cli.rs",
                      "line": 1,
                      "detail": "Example evidence"
                    }
                  ],
                  "recommendation": "Tighten the test boundary."
                }
              ]
            }"#,
        );
    let output = command
        .output()
        .expect("release review script should run with mocked output");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let response: Value =
        serde_json::from_slice(&output.stdout).expect("script should print mocked json");
    assert_eq!(response["warnings"].as_array().unwrap().len(), 1);
    assert!(
        response.get("runner_warnings").is_none(),
        "runner warnings should stay local wrapper metadata"
    );
}

#[test]
fn release_review_rejects_mock_env_without_explicit_test_flag() {
    let output = python3_command()
        .arg("scripts/review-rust-release-style.py")
        .arg("--full")
        .env("SPECIAL_RUST_RELEASE_REVIEW_ALLOW_MOCK", "1")
        .env(
            "SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT",
            r#"{"baseline":null,"full_scan":true,"summary":"clean","warnings":[]}"#,
        )
        .output()
        .expect("release review script should run");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("mock controls are test-only"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.LOCAL_ONLY
fn release_review_refuses_live_codex_invocation_in_ci() {
    let output = python3_command()
        .arg("scripts/review-rust-release-style.py")
        .arg("--full")
        .env("CI", "true")
        .output()
        .expect("release review script should run");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("rust release review is local-only and must not invoke Codex from CI")
    );
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.NO_BYTECODE_ARTIFACTS
fn release_python_entrypoints_disable_bytecode_writes() {
    let review_flag = python_entrypoint_runtime_flag("review-rust-release-style.py");
    let tag_flag = python_entrypoint_runtime_flag("tag-release.py");

    assert_eq!(review_flag["dont_write_bytecode"], true);
    assert_eq!(tag_flag["dont_write_bytecode"], true);
}

#[test]
// @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.MANUAL_ONLY
fn release_review_is_not_wired_into_ci_workflows_or_release_publication() {
    for workflow in workflow_files() {
        let contents =
            fs::read_to_string(&workflow).expect("workflow file should be readable as utf-8");
        assert!(
            !contents.contains("review-rust-release-style.py"),
            "workflow {} should not invoke the local codex release review directly",
            workflow.display()
        );
    }

    let tag_release = fs::read_to_string(support::repo_root().join("scripts/tag-release.py"))
        .expect("tag-release.py should be readable");
    assert!(
        !tag_release.contains("review-rust-release-style.py"),
        "release publication should not invoke the local codex review script"
    );
}
