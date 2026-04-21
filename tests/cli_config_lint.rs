/**
@group SPECIAL.CONFIG.ROOT_DISCOVERY
special discovers a project root from explicit config, VCS state, or the current directory.

@spec SPECIAL.CONFIG.SPECIAL_TOML
special uses `special.toml` as an explicit project anchor when present.

@spec SPECIAL.CONFIG.SPECIAL_TOML.EXPLICIT_ROOT
special uses the optional `root` value in `special.toml` as the explicit project root, resolved relative to the config file.

@spec SPECIAL.CONFIG.SPECIAL_TOML.ANCESTOR_CONFIG
when launched from a nested subdirectory, special finds the nearest ancestor `special.toml` and resolves `root = "."` relative to that config file rather than the launch directory.

@spec SPECIAL.CONFIG.SPECIAL_TOML.OPTIONAL
special does not require `special.toml` for basic operation.

@spec SPECIAL.CONFIG.SPECIAL_TOML.SUPPRESSES_IMPLICIT_ROOT_WARNING
when `special.toml` determines the project root, special does not emit implicit-root discovery warnings.

@spec SPECIAL.CONFIG.SPECIAL_TOML.KEY_VALUE_SYNTAX
special exits with an error when `special.toml` contains a non-empty, non-comment line that does not use `key = "value"` syntax.

@spec SPECIAL.CONFIG.SPECIAL_TOML.QUOTED_STRING_VALUES
special exits with an error when `special.toml` uses an unquoted value.

@spec SPECIAL.CONFIG.SPECIAL_TOML.UNKNOWN_KEYS
special exits with an error when `special.toml` uses an unknown key.

@spec SPECIAL.CONFIG.SPECIAL_TOML.DUPLICATE_KEYS_REJECTED
special exits with an error when `special.toml` declares the same key more than once.

@spec SPECIAL.CONFIG.SPECIAL_TOML.EXISTING_ROOT_REQUIRED
special exits with an error when `special.toml` points `root` at a path that does not exist.

@spec SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_BE_DIRECTORY
special exits with an error when `special.toml` points `root` at a file instead of a directory.

@spec SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_NOT_BE_EMPTY
special exits with an error when `special.toml` sets `root` to an empty string.

@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION
special reads an optional `version` key from `special.toml` to select parser and linter rules.

@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION.DEFAULTS_TO_LEGACY
without a `version` key in `special.toml`, special defaults `@planned` to backward-looking within-block ownership instead of version 1's adjacent ownership.

@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION.UNKNOWN_REJECTED
special exits with an error when `special.toml` uses an unsupported `version` value.

@spec SPECIAL.CONFIG.SPECIAL_TOML.VERSION.MISSING_WARNS_AND_ASSUMES_LEGACY
when `special.toml` omits `version`, special emits a lint warning that it is using compatibility parsing rules and points to `version = "1"` for the current rules.

@spec SPECIAL.CONFIG.ROOT_DISCOVERY.VCS_DEFAULT
without `special.toml`, special prefers the nearest enclosing VCS root.

@spec SPECIAL.CONFIG.ROOT_DISCOVERY.CWD_FALLBACK
without `special.toml` or VCS metadata, special falls back to the current working directory.

@spec SPECIAL.CONFIG.ROOT_DISCOVERY.IMPLICIT_ROOT_WARNING
without `special.toml`, special warns that it is using an implicit root for discovery.

@spec SPECIAL.CONFIG.ROOT_DISCOVERY.NO_CONFIG_VERSION_WARNING
without `special.toml`, special emits a lint warning that it is using compatibility parsing rules and points to `special init` for creating current config.

@module SPECIAL.TESTS.CLI_CONFIG_LINT
Config parsing and lint warning/error integration tests in `tests/cli_config_lint.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_CONFIG_LINT
#[path = "support/cli.rs"]
mod support;

use std::fs;

use support::{
    rendered_spec_node_ids, run_special, temp_repo_dir, write_lint_error_fixture,
    write_missing_version_fixture, write_non_adjacent_planned_v1_fixture,
    write_orphan_verify_fixture, write_special_toml_root_fixture,
    write_supported_fixture_without_config, write_unverified_current_fixture,
};

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.SUPPRESSES_IMPLICIT_ROOT_WARNING
fn special_toml_suppresses_implicit_root_warning() {
    let root = temp_repo_dir("special-cli-special-toml-warning");
    write_special_toml_root_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(!stderr.contains("warning: using inferred"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.VERSION.MISSING_WARNS_AND_ASSUMES_LEGACY
fn missing_special_toml_version_warns_but_keeps_legacy_behavior() {
    let root = temp_repo_dir("special-cli-special-toml-missing-version");
    write_missing_version_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("warning: missing `version` in special.toml"));
    assert!(stdout.contains("using compatibility parsing rules"));
    assert!(!stdout.contains("error:"));

    let output = run_special(&root, &["specs"]);
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("warning: missing `version` in special.toml"));
    assert!(stderr.contains("set `version = \"1\"`"));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(rendered_spec_node_ids(&stdout).contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.WARNINGS_DO_NOT_FAIL
fn lint_succeeds_when_only_warnings_are_present() {
    let root = temp_repo_dir("special-cli-lint-warning-only");
    fs::write(root.join("special.toml"), "root = \".\"\n").expect("special.toml should be written");

    let output = run_special(&root, &["lint"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("warning: missing `version` in special.toml"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.ROOT_DISCOVERY.NO_CONFIG_VERSION_WARNING
fn missing_special_toml_warns_but_keeps_legacy_behavior() {
    let root = temp_repo_dir("special-cli-missing-special-toml");
    write_supported_fixture_without_config(&root);

    let lint_output = run_special(&root, &["lint"]);
    assert!(lint_output.status.success());

    let lint_stdout = String::from_utf8(lint_output.stdout).expect("stdout should be utf-8");
    assert!(lint_stdout.contains("warning: no special.toml found"));
    assert!(lint_stdout.contains("using compatibility parsing rules"));
    assert!(!lint_stdout.contains("error:"));

    let spec_output = run_special(&root, &["specs"]);
    assert!(spec_output.status.success());

    let spec_stderr = String::from_utf8(spec_output.stderr).expect("stderr should be utf-8");
    assert!(spec_stderr.contains("warning: using current directory"));
    assert!(spec_stderr.contains("warning: no special.toml found"));
    assert!(spec_stderr.contains("run `special init`"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.KEY_VALUE_SYNTAX
fn special_toml_requires_key_value_syntax() {
    let root = temp_repo_dir("special-cli-special-toml-key-value");
    fs::write(root.join("special.toml"), "root\n").expect("special.toml should be written");

    let output = run_special(&root, &["specs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.QUOTED_STRING_VALUES
fn special_toml_requires_quoted_string_values() {
    let root = temp_repo_dir("special-cli-special-toml-quoted");
    fs::write(root.join("special.toml"), "root = workspace\n")
        .expect("special.toml should be written");

    let output = run_special(&root, &["specs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.UNKNOWN_KEYS
fn special_toml_rejects_unknown_keys() {
    let root = temp_repo_dir("special-cli-special-toml-unknown-key");
    fs::write(root.join("special.toml"), "nope = \".\"\n").expect("special.toml should be written");

    let output = run_special(&root, &["specs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 1 uses unknown key `nope`"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.DUPLICATE_KEYS_REJECTED
fn special_toml_rejects_duplicate_keys() {
    let root = temp_repo_dir("special-cli-special-toml-duplicate-key");
    fs::write(
        root.join("special.toml"),
        "root = \".\"\nroot = \"workspace\"\n",
    )
    .expect("special.toml should be written");

    let output = run_special(&root, &["specs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 2"));
    assert!(stderr.contains("root"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.EXISTING_ROOT_REQUIRED
fn special_toml_requires_existing_root_path() {
    let root = temp_repo_dir("special-cli-special-toml-missing-root");
    fs::write(root.join("special.toml"), "root = \"missing\"\n")
        .expect("special.toml should be written");

    let output = run_special(&root, &["specs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("points to a root that does not exist"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_BE_DIRECTORY
fn special_toml_requires_directory_root_path() {
    let root = temp_repo_dir("special-cli-special-toml-file-root");
    fs::write(root.join("special.toml"), "root = \"specs.rs\"\n")
        .expect("special.toml should be written");
    fs::write(root.join("specs.rs"), "// not a directory\n").expect("file root should exist");

    let output = run_special(&root, &["specs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("points to a root that is not a directory"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_NOT_BE_EMPTY
fn special_toml_rejects_empty_root_path() {
    let root = temp_repo_dir("special-cli-special-toml-empty-root");
    fs::write(root.join("special.toml"), "root = \"\"\n").expect("special.toml should be written");

    let output = run_special(&root, &["specs"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 1 must not use an empty root path"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND
fn lint_reports_annotation_errors() {
    let root = temp_repo_dir("special-cli-lint");
    write_lint_error_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("checks.rs:1: error:"));
    assert!(stdout.contains("unknown spec id `UNKNOWN` referenced by @verifies"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.PLANNED_SCOPE
fn lint_rejects_non_adjacent_planned_in_version_1() {
    let root = temp_repo_dir("special-cli-planned-v1");
    write_non_adjacent_planned_v1_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("specs.rs:4: error:"));
    assert!(stdout.contains("backward-looking form is not allowed in version 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PARSE.VERIFIES.ONLY_ATTACHED_SUPPORT_COUNTS
fn lint_reports_orphan_verifies() {
    let root = temp_repo_dir("special-cli-orphan-verify");
    write_orphan_verify_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("checks.rs:1: error:"));
    assert!(stdout.contains("@verifies must attach to the next supported item"));

    let output = run_special(&root, &["specs", "--unverified"]);
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO [unverified]"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.UNVERIFIED_EXCLUDED
fn lint_does_not_report_unverified_current_specs() {
    let root = temp_repo_dir("special-cli-lint-clean");
    write_unverified_current_fixture(&root);
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");

    let output = run_special(&root, &["lint"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(stdout.trim(), "Lint clean.");

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
