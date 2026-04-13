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

fn write_orphan_verify_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        r#"/**
@spec DEMO
Demo root.
*/
"#,
    )
    .expect("spec fixture should be written");

    fs::write(root.join("checks.rs"), "// @verifies DEMO\n")
        .expect("orphan verify fixture should be written");
}

fn write_special_toml_root_fixture(root: &Path) {
    let configured_root = root.join("workspace");
    fs::create_dir_all(&configured_root).expect("configured root should be created");
    fs::write(root.join("special.toml"), "root = \"workspace\"\n")
        .expect("special.toml should be written");

    fs::write(
        configured_root.join("specs.rs"),
        r#"/**
@spec DEMO
Configured root spec.
*/
"#,
    )
    .expect("configured root spec fixture should be written");

    fs::write(
        configured_root.join("checks.rs"),
        ["/", "/ @verifies DEMO\n", "fn verifies_demo() {}\n"].concat(),
    )
    .expect("configured root verify fixture should be written");

    fs::write(
        root.join("decoy.rs"),
        r#"/**
@spec DECOY
This spec should stay outside the configured root.
*/
"#,
    )
    .expect("decoy fixture should be written");
}

fn write_special_toml_dot_root_fixture(root: &Path) -> PathBuf {
    let nested = root.join("nested/deeper");
    fs::create_dir_all(&nested).expect("nested dir should be created");
    fs::write(root.join("special.toml"), "root = \".\"\n").expect("special.toml should be written");

    fs::write(
        root.join("specs.rs"),
        r#"/**
@spec DEMO
Config-root spec.
*/
"#,
    )
    .expect("config-root spec fixture should be written");

    fs::write(
        root.join("checks.rs"),
        ["/", "/ @verifies DEMO\n", "fn verifies_demo() {}\n"].concat(),
    )
    .expect("config-root verify fixture should be written");

    nested
}

#[test]
// @verifies SPECIAL.INIT.CREATES_SPECIAL_TOML
fn init_creates_special_toml_in_current_directory() {
    let root = temp_repo_dir("special-cli-init");

    let output = run_special(&root, &["init"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Created"));
    assert_eq!(
        fs::read_to_string(root.join("special.toml")).expect("special.toml should be created"),
        "root = \".\"\n"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.INIT.DOES_NOT_OVERWRITE_SPECIAL_TOML
fn init_fails_when_special_toml_already_exists() {
    let root = temp_repo_dir("special-cli-init-existing");
    fs::write(root.join("special.toml"), "root = \"workspace\"\n")
        .expect("special.toml should be written");

    let output = run_special(&root, &["init"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("special.toml already exists"));
    assert_eq!(
        fs::read_to_string(root.join("special.toml")).expect("special.toml should still exist"),
        "root = \"workspace\"\n"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
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
// @verifies SPECIAL.SPEC_COMMAND.ID_SCOPE
fn spec_scopes_to_matching_id_and_descendants() {
    let root = temp_repo_dir("special-cli-scope");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "DEMO"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO\n"));
    assert!(stdout.contains("DEMO.LIVE"));
    assert!(!stdout.contains("DEMO.PLANNED"));
    assert!(!stdout.contains("No specs found."));

    let output = run_special(&root, &["spec", "DEMO.LIVE"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO.LIVE"));
    assert!(!stdout.contains("DEMO\n"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.UNSUPPORTED
fn spec_unsupported_filters_live_items_without_support() {
    let root = temp_repo_dir("special-cli-unsupported");
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
    assert!(!stdout.contains("\"body\""));

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
    assert!(stdout.contains("<details><summary>@verifies"));
    assert!(stdout.contains("<pre class=\"code-block\"><code class=\"language-rust\">"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.HTML.CODE_HIGHLIGHTING
fn spec_html_renders_best_effort_code_highlighting() {
    let root = temp_repo_dir("special-cli-html-highlight");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<span class=\"tok-keyword\">fn</span>"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE
fn spec_verbose_includes_verify_bodies() {
    let root = temp_repo_dir("special-cli-verbose");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "DEMO.LIVE", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("declared at:"));
    assert!(stdout.contains("@verifies"));
    assert!(stdout.contains("fn verifies_demo_live() {}"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE.JSON
fn spec_verbose_json_includes_support_bodies() {
    let root = temp_repo_dir("special-cli-verbose-json");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "DEMO.LIVE", "--json", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("\"body\""));
    assert!(stdout.contains("fn verifies_demo_live() {}"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.EXPLICIT_ROOT
fn spec_uses_root_declared_in_special_toml() {
    let root = temp_repo_dir("special-cli-special-toml-root");
    write_special_toml_root_fixture(&root);

    let output = run_special(&root, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(!stdout.contains("DECOY"));
    assert!(!stderr.contains("warning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.ANCESTOR_CONFIG
fn spec_uses_ancestor_special_toml_from_nested_directory() {
    let root = temp_repo_dir("special-cli-special-toml-ancestor");
    let nested = write_special_toml_dot_root_fixture(&root);

    let output = run_special(&nested, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(!stderr.contains("warning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

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
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.KEY_VALUE_SYNTAX
fn special_toml_requires_key_value_syntax() {
    let root = temp_repo_dir("special-cli-special-toml-key-value");
    fs::write(root.join("special.toml"), "root\n").expect("special.toml should be written");

    let output = run_special(&root, &["spec"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 1 must use `key = \"value\"` syntax"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.QUOTED_STRING_VALUES
fn special_toml_requires_quoted_string_values() {
    let root = temp_repo_dir("special-cli-special-toml-quoted");
    fs::write(root.join("special.toml"), "root = workspace\n")
        .expect("special.toml should be written");

    let output = run_special(&root, &["spec"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 1 must use a quoted string value"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.UNKNOWN_KEYS
fn special_toml_rejects_unknown_keys() {
    let root = temp_repo_dir("special-cli-special-toml-unknown-key");
    fs::write(root.join("special.toml"), "nope = \".\"\n").expect("special.toml should be written");

    let output = run_special(&root, &["spec"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(stderr.contains("line 1 uses unknown key `nope`"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.EXISTING_ROOT_REQUIRED
fn special_toml_requires_existing_root_path() {
    let root = temp_repo_dir("special-cli-special-toml-missing-root");
    fs::write(root.join("special.toml"), "root = \"missing\"\n")
        .expect("special.toml should be written");

    let output = run_special(&root, &["spec"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("points to a root that does not exist"));

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
// @verifies SPECIAL.PARSE.VERIFIES.ONLY_ATTACHED_SUPPORT_COUNTS
fn lint_reports_orphan_verifies() {
    let root = temp_repo_dir("special-cli-orphan-verify");
    write_orphan_verify_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@verifies must attach to the next supported item"));

    let output = run_special(&root, &["spec", "--unsupported"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO [unsupported]"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.UNSUPPORTED_EXCLUDED
fn lint_does_not_report_unsupported_live_specs() {
    let root = temp_repo_dir("special-cli-lint-clean");
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

    let output = run_special(&root, &["lint"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(stdout.trim(), "Lint clean.");

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
