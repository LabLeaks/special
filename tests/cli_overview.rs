/**
@module SPECIAL.TESTS.CLI_OVERVIEW
Bare `special` overview command tests in `tests/cli_overview.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_OVERVIEW
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{
    run_special, temp_repo_dir, write_lint_error_fixture,
    write_traceability_module_analysis_fixture, write_typescript_traceability_fixture,
};

#[test]
// @verifies SPECIAL.OVERVIEW.COMMAND
fn bare_special_prints_compact_repo_overview() {
    let root = temp_repo_dir("special-cli-overview");
    write_traceability_module_analysis_fixture(&root);

    let output = run_special(&root, &[]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special\n"));
    assert!(stdout.contains("  lint\n"));
    assert!(stdout.contains("  specs\n"));
    assert!(stdout.contains("  arch\n"));
    assert!(stdout.contains("  health\n"));
    assert!(stdout.contains("  look next\n"));
    assert!(stdout.contains("special specs --metrics"));
    assert!(stdout.contains("special arch --metrics"));
    assert!(stdout.contains("special health --metrics"));
    assert!(!stdout.contains("traceability\n"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.OVERVIEW.COMMAND.JSON
fn bare_special_json_emits_compact_overview() {
    let root = temp_repo_dir("special-cli-overview-json");
    write_typescript_traceability_fixture(&root);

    let output = run_special(&root, &["--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["lint"]["errors"], 0);
    assert_eq!(json["specs"]["total_specs"], 1);
    assert_eq!(json["arch"]["total_modules"], 2);
    assert!(json["health"]["duplicate_items"].is_number());
    assert!(json["health"]["unowned_items"].is_number());
    assert!(json.get("traceability").is_none());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.OVERVIEW.COMMAND.FAILS_ON_LINT_ERRORS
fn bare_special_fails_when_combined_lint_has_errors() {
    let root = temp_repo_dir("special-cli-overview-lint-error");
    write_lint_error_fixture(&root);

    let output = run_special(&root, &[]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special\n"));
    assert!(stdout.contains("errors: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn subcommand_help_explains_parallel_metrics_and_verbose_flags() {
    let root = temp_repo_dir("special-cli-help-parallel-flags");

    let specs_help = run_special(&root, &["specs", "--help"]);
    assert!(specs_help.status.success());
    let specs_stdout = String::from_utf8(specs_help.stdout).expect("stdout should be utf-8");
    assert!(specs_stdout.contains("Show deeper contract analysis for the declared spec view"));
    assert!(specs_stdout.contains("Show more attached support detail within the current view"));

    let arch_help = run_special(&root, &["arch", "--help"]);
    assert!(arch_help.status.success());
    let arch_stdout = String::from_utf8(arch_help.stdout).expect("stdout should be utf-8");
    assert!(
        arch_stdout
            .contains("Show deeper implementation analysis for the current architecture view")
    );
    assert!(arch_stdout.contains("Show more implementation detail within the current view"));

    let health_help = run_special(&root, &["health", "--help"]);
    assert!(health_help.status.success());
    let health_stdout = String::from_utf8(health_help.stdout).expect("stdout should be utf-8");
    assert!(health_stdout.contains("Show deeper analysis for the current health view"));
    assert!(health_stdout.contains("Show more item-level detail within the current health view"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
