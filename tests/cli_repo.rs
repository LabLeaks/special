/**
@module SPECIAL.TESTS.CLI_REPO
`special repo` output and repo-wide quality tests in `tests/cli_repo.rs`.

@group SPECIAL.REPO_COMMAND_GROUP
special repo can surface repo-wide quality signals that are not tied to a single architecture module.

@spec SPECIAL.REPO_COMMAND
special repo materializes repo-wide quality signals for the current repository.

@spec SPECIAL.REPO_COMMAND.JSON
special repo --json emits structured repo-wide quality signals.

@spec SPECIAL.REPO_COMMAND.HTML
special repo --html emits an HTML repo-wide quality view.

@spec SPECIAL.REPO_COMMAND.VERBOSE
special repo --verbose includes fuller detail for repo-wide quality signals when built-in analyzers provide it.

@spec SPECIAL.REPO_COMMAND.TRACEABILITY
special repo --experimental surfaces repo-wide Rust implementation traceability from owned items through Rust tests to live, planned, or deprecated spec claims when built-in analyzers can identify those paths honestly.

@spec SPECIAL.REPO_COMMAND.TRACEABILITY.DEFAULT_HIDDEN
special repo hides experimental traceability evidence unless `--experimental` is present.

@spec SPECIAL.REPO_COMMAND.JSON.TRACEABILITY
special repo --json --experimental includes structured experimental Rust implementation traceability evidence at the repo boundary.

@spec SPECIAL.REPO_COMMAND.DUPLICATION
special repo surfaces repo-wide duplicate-logic signals from owned implementation when built-in analyzers can identify substantively similar code shapes honestly.

@spec SPECIAL.REPO_COMMAND.UNREACHED_CODE
special repo surfaces repo-wide unowned unreached-code indicators when built-in analyzers can identify them honestly.
*/
// @fileimplements SPECIAL.TESTS.CLI_REPO
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{
    run_special, temp_repo_dir, top_level_help_commands,
    write_duplicate_item_signals_module_analysis_fixture,
    write_many_duplicate_item_signals_module_analysis_fixture,
    write_traceability_cross_file_module_fixture, write_traceability_module_analysis_fixture,
    write_unreached_code_module_analysis_fixture,
};

#[test]
// @verifies SPECIAL.HELP.REPO_COMMAND
fn top_level_help_presents_repo_command() {
    let root = temp_repo_dir("special-cli-repo-help");

    let output = run_special(&root, &["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        top_level_help_commands(&stdout)
            .iter()
            .any(|(name, summary)| name == "repo"
                && summary == "Inspect repo-wide quality signals")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND
fn repo_materializes_repo_wide_quality_signals() {
    let root = temp_repo_dir("special-cli-repo");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["repo"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special repo"));
    assert!(stdout.contains("repo-wide signals"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.DUPLICATION
fn repo_surfaces_repo_wide_duplication_signals() {
    let root = temp_repo_dir("special-cli-repo-duplication");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["repo"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("duplicate items: 2"));
    assert!(stdout.contains("duplicate items meaning:"));
    assert!(stdout.contains("duplicate items exact:"));
    assert!(
        stdout.contains(
            "duplicate item: DEMO:alpha.rs:first_duplicate [function; duplicate peers 1]"
        )
    );
    assert!(
        stdout.contains(
            "duplicate item: DEMO:beta.rs:second_duplicate [function; duplicate peers 1]"
        )
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.UNREACHED_CODE
fn repo_surfaces_unowned_unreached_code() {
    let root = temp_repo_dir("special-cli-repo-unreached");
    write_unreached_code_module_analysis_fixture(&root);

    let output = run_special(&root, &["repo", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("unowned unreached items: 1"));
    assert!(stdout.contains("unowned unreached items meaning:"));
    assert!(stdout.contains("unowned unreached item: hidden.rs:hidden_unreached [function]"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.JSON
fn repo_json_includes_structured_repo_signals() {
    let root = temp_repo_dir("special-cli-repo-json");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["repo", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(
        json["analysis"]["repo_signals"]["duplicate_items"],
        Value::from(2)
    );
    assert_eq!(
        json["analysis"]["repo_signals"]["duplicate_item_details"]
            .as_array()
            .expect("duplicate items should be an array")
            .len(),
        2
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.HTML
fn repo_html_emits_html_output() {
    let root = temp_repo_dir("special-cli-repo-html");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["repo", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<!doctype html>"));
    assert!(stdout.contains("special repo"));
    assert!(stdout.contains("duplicate items"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.VERBOSE
fn repo_verbose_includes_fuller_repo_signal_detail() {
    let duplicate_root = temp_repo_dir("special-cli-repo-verbose-duplicates");
    write_many_duplicate_item_signals_module_analysis_fixture(&duplicate_root);

    let normal_output = run_special(&duplicate_root, &["repo"]);
    assert!(normal_output.status.success());
    let normal_stdout = String::from_utf8(normal_output.stdout).expect("stdout should be utf-8");

    let verbose_output = run_special(&duplicate_root, &["repo", "--verbose"]);
    assert!(verbose_output.status.success());
    let verbose_stdout = String::from_utf8(verbose_output.stdout).expect("stdout should be utf-8");

    assert!(normal_stdout.contains("duplicate items: 6"));
    assert!(
        !normal_stdout
            .contains("duplicate item: DEMO:zeta.rs:zeta_duplicate [function; duplicate peers 5]")
    );
    assert!(
        verbose_stdout
            .contains("duplicate item: DEMO:zeta.rs:zeta_duplicate [function; duplicate peers 5]")
    );

    fs::remove_dir_all(&duplicate_root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.EXPERIMENTAL
fn repo_experimental_opt_in_surfaces_experimental_analysis() {
    let root = temp_repo_dir("special-cli-repo-experimental-opt-in");
    write_traceability_module_analysis_fixture(&root);

    let baseline = run_special(&root, &["repo"]);
    assert!(baseline.status.success());
    let baseline_stdout = String::from_utf8(baseline.stdout).expect("stdout should be utf-8");
    assert!(!baseline_stdout.contains("experimental traceability"));

    let experimental = run_special(&root, &["repo", "--experimental"]);
    assert!(experimental.status.success());
    let experimental_stdout =
        String::from_utf8(experimental.stdout).expect("stdout should be utf-8");
    assert!(experimental_stdout.contains("experimental traceability"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.TRACEABILITY.DEFAULT_HIDDEN
fn repo_hides_traceability_without_experimental() {
    let root = temp_repo_dir("special-cli-repo-default-hides-traceability");
    write_traceability_module_analysis_fixture(&root);

    let output = run_special(&root, &["repo"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!stdout.contains("experimental traceability"));
    assert!(!stdout.contains("live spec item:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.TRACEABILITY
fn repo_surfaces_experimental_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability");
    write_traceability_cross_file_module_fixture(&root);

    let output = run_special(&root, &["repo", "--experimental", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("experimental traceability"));
    assert!(stdout.contains("live spec item: DEMO:render_entry"));
    assert!(stdout.contains("live spec item: DEMO:render_spec_html"));
    assert!(stdout.contains("live spec item: DEMO:helper_impl"));
    assert!(stdout.contains("live spec item: DEMO:live_impl"));
    assert!(stdout.contains("unknown item: DEMO:orphan_impl"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.REPO_COMMAND.JSON.TRACEABILITY
fn repo_json_includes_structured_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-json");
    write_traceability_module_analysis_fixture(&root);

    let output = run_special(&root, &["repo", "--experimental", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(
        json["analysis"]["traceability"]["analyzed_items"],
        Value::from(5)
    );
    assert!(
        json["analysis"]["traceability"]["live_spec_items"]
            .as_array()
            .expect("live traceability should be an array")
            .iter()
            .any(|item| {
                item["module_id"].as_str() == Some("DEMO")
                    && item["name"].as_str() == Some("live_impl")
                    && item["live_specs"]
                        .as_array()
                        .expect("live specs should be an array")
                        .iter()
                        .any(|spec| spec == "APP.LIVE")
            })
    );
    assert!(
        json["analysis"]["traceability"]["unknown_items"]
            .as_array()
            .expect("unknown traceability should be an array")
            .iter()
            .any(|item| {
                item["module_id"].as_str() == Some("DEMO")
                    && item["name"].as_str() == Some("orphan_impl")
            })
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn repo_non_verbose_experimental_traceability_stays_summary_only() {
    let root = temp_repo_dir("special-cli-repo-traceability-summary-only");
    write_traceability_cross_file_module_fixture(&root);

    let text_output = run_special(&root, &["repo", "--experimental"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text_stdout.contains("experimental traceability"));
    assert!(!text_stdout.contains("live spec item:"));
    assert!(!text_stdout.contains("unknown item:"));

    let json_output = run_special(&root, &["repo", "--experimental", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let traceability = json["analysis"]["traceability"]
        .as_object()
        .expect("traceability summary should be an object");
    assert!(!traceability.contains_key("live_spec_items"));
    assert!(!traceability.contains_key("unknown_items"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
