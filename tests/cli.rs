use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_repo_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("temp repo dir should be created");
    path
}

fn run_special(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_special"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("special command should run")
}

fn write_live_and_planned_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        r#"/**
@spec DEMO
Demo root.

@spec DEMO.LIVE
Live child.

@spec DEMO.PLANNED
Planned child.

@planned
*/
"#,
    )
    .expect("spec fixture should be written");

    fs::write(
        root.join("checks.rs"),
        [
            "/",
            "/ @verifies DEMO\n",
            "fn verifies_demo_root() {}\n\n",
            "/",
            "/ @verifies DEMO.LIVE\n",
            "fn verifies_demo_live() {}\n\n",
            "/",
            "/ @verifies DEMO.PLANNED\n",
            "fn verifies_demo_planned() {}\n",
        ]
        .concat(),
    )
    .expect("verify fixture should be written");
}

fn write_unsupported_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        r#"/**
@spec DEMO
Demo root.

@spec DEMO.UNSUPPORTED
Live child without support.
*/
"#,
    )
    .expect("spec fixture should be written");

    fs::write(
        root.join("checks.rs"),
        ["/", "/ @verifies DEMO\n", "fn verifies_demo_root() {}\n"].concat(),
    )
    .expect("verify fixture should be written");
}

fn write_lint_error_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        r#"/**
@spec DEMO
Demo root.
*/
"#,
    )
    .expect("spec fixture should be written");

    fs::write(
        root.join("checks.rs"),
        ["/", "/ @verifies UNKNOWN\n", "fn verifies_unknown() {}\n"].concat(),
    )
    .expect("verify fixture should be written");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND
fn spec_materializes_live_spec_tree() {
    let root = temp_repo_dir("special-cli-spec");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(stdout.contains("DEMO.LIVE"));
    assert!(stdout.contains("verifies: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.LIVE_ONLY
fn spec_hides_planned_items_by_default() {
    let root = temp_repo_dir("special-cli-live-only");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(stdout.contains("DEMO.LIVE"));
    assert!(!stdout.contains("DEMO.PLANNED"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.ALL
fn spec_all_includes_planned_items() {
    let root = temp_repo_dir("special-cli-all");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--all"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO.PLANNED [planned]"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.UNSUPPORTED
fn spec_unsupported_filters_live_items_without_support() {
    let root = temp_repo_dir("special-cli-unsupported");
    write_unsupported_fixture(&root);

    let output = run_special(&root, &["spec", "--unsupported"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO\n"));
    assert!(stdout.contains("DEMO.UNSUPPORTED [unsupported]"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.JSON
fn spec_json_emits_json_output() {
    let root = temp_repo_dir("special-cli-json");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--json"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("\"nodes\""));
    assert!(stdout.contains("\"DEMO\""));
    assert!(!stdout.contains("DEMO.PLANNED"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.HTML
fn spec_html_emits_html_output() {
    let root = temp_repo_dir("special-cli-html");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<!doctype html>"));
    assert!(stdout.contains("DEMO"));
    assert!(!stdout.contains("DEMO.PLANNED"));

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
    assert!(stdout.contains("unknown spec id `UNKNOWN` referenced by @verifies"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.UNSUPPORTED_EXCLUDED
fn lint_does_not_report_unsupported_live_specs() {
    let root = temp_repo_dir("special-cli-lint-clean");
    write_unsupported_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(stdout.trim(), "Lint clean.");

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
