/**
@module SPECIAL.TESTS.CLI_SPEC
`special spec` output and filtering tests in `tests/cli_spec.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_SPEC
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{
    find_node_by_id, html_node_has_badge, rendered_spec_node_ids, rendered_spec_node_line,
    run_special, temp_repo_dir, write_deprecated_release_fixture, write_file_attest_fixture,
    write_file_verify_fixture, write_live_and_planned_fixture, write_planned_release_fixture,
    write_special_toml_dot_root_fixture, write_special_toml_root_fixture,
    write_unsupported_live_fixture,
};

#[test]
// @verifies SPECIAL.SPEC_COMMAND
fn spec_materializes_live_spec_tree() {
    let root = temp_repo_dir("special-cli-spec");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(!node_ids.contains(&"DEMO.PLANNED".to_string()));
    assert!(!stderr.contains("warning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.FAILS_ON_ERRORS
fn spec_fails_when_annotation_errors_are_present() {
    let root = temp_repo_dir("special-cli-spec-errors");
    fs::write(
        root.join("demo.rs"),
        "/// @spec DEMO\n/// Root claim.\n/// @spec DEMO\n/// Duplicate claim.\n",
    )
    .expect("fixture should be written");

    let output = run_special(&root, &["spec"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(stderr.contains("duplicate"));
    assert!(stderr.contains("DEMO"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HELP.SPECS_COMMAND_PLURAL_PRIMARY
fn top_level_help_presents_specs_as_the_primary_command_name() {
    let root = temp_repo_dir("special-cli-specs-help-primary");

    let output = run_special(&root, &["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special specs"));
    assert!(!stdout.contains("Examples:\n  special spec\n"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.SINGULAR_ALIAS
fn singular_spec_alias_still_materializes_the_same_view() {
    let root = temp_repo_dir("special-cli-spec-singular-alias");
    write_live_and_planned_fixture(&root);

    let plural_output = run_special(&root, &["specs"]);
    assert!(plural_output.status.success());
    let singular_output = run_special(&root, &["spec"]);
    assert!(singular_output.status.success());

    assert_eq!(plural_output.stdout, singular_output.stdout);
    assert_eq!(plural_output.stderr, singular_output.stderr);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.LIVE_ONLY
fn spec_hides_planned_items_by_default() {
    let root = temp_repo_dir("special-cli-spec-live-only");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!rendered_spec_node_ids(&stdout).contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.ALL
fn spec_all_includes_planned_items() {
    let root = temp_repo_dir("special-cli-spec-all");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--all"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.PLANNED_RELEASE_METADATA
fn spec_surfaces_planned_release_metadata_across_output_modes() {
    let root = temp_repo_dir("special-cli-planned-release-metadata");
    write_planned_release_fixture(&root);

    let text_output = run_special(&root, &["spec", "--all"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    let planned_line =
        rendered_spec_node_line(&text_stdout, "DEMO.PLANNED").expect("planned node should render");
    assert!(planned_line.contains("[planned: 0.3.0]"));

    let json_output = run_special(&root, &["spec", "--all", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let planned = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.PLANNED"))
        })
        .expect("planned node should be present");
    assert_eq!(
        planned["planned_release"],
        Value::String("0.3.0".to_string())
    );
    assert_eq!(planned["planned"], Value::Bool(true));
    assert_eq!(planned["deprecated"], Value::Bool(false));

    let html_output = run_special(&root, &["spec", "--all", "--html"]);
    assert!(html_output.status.success());
    let html_stdout = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html_node_has_badge(
        &html_stdout,
        "DEMO.PLANNED",
        "badge-planned",
        "planned: 0.3.0"
    ));
    assert!(!html_node_has_badge(
        &html_stdout,
        "DEMO.PLANNED",
        "badge-deprecated",
        "deprecated: 0.3.0"
    ));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.DEPRECATED_METADATA
fn spec_surfaces_deprecated_release_metadata_across_output_modes() {
    let root = temp_repo_dir("special-cli-deprecated-release-metadata");
    write_deprecated_release_fixture(&root);

    let text_output = run_special(&root, &["spec"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    let deprecated_line = rendered_spec_node_line(&text_stdout, "DEMO.DEPRECATED")
        .expect("deprecated node should render");
    assert!(deprecated_line.contains("[deprecated: 0.6.0]"));

    let json_output = run_special(&root, &["spec", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let deprecated = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.DEPRECATED"))
        })
        .expect("deprecated node should be present");
    assert_eq!(
        deprecated["deprecated_release"],
        Value::String("0.6.0".to_string())
    );
    assert_eq!(deprecated["deprecated"], Value::Bool(true));
    assert_eq!(deprecated["planned"], Value::Bool(false));

    let html_output = run_special(&root, &["spec", "--html"]);
    assert!(html_output.status.success());
    let html_stdout = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html_node_has_badge(
        &html_stdout,
        "DEMO.DEPRECATED",
        "badge-deprecated",
        "deprecated: 0.6.0"
    ));
    assert!(!html_node_has_badge(
        &html_stdout,
        "DEMO.DEPRECATED",
        "badge-planned",
        "planned: 0.6.0"
    ));

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
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(!node_ids.contains(&"DEMO.PLANNED".to_string()));
    assert!(!stdout.contains("No specs found."));

    let output = run_special(&root, &["spec", "DEMO.LIVE"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(!node_ids.contains(&"DEMO".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.UNSUPPORTED
fn spec_unsupported_filters_live_items_without_support() {
    let root = temp_repo_dir("special-cli-unsupported");
    write_unsupported_live_fixture(&root);

    let output = run_special(&root, &["spec", "--unsupported"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.UNSUPPORTED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.JSON
fn spec_json_emits_json_output() {
    let root = temp_repo_dir("special-cli-json");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert!(json["nodes"].is_array());
    let ids = json["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|node| node["id"].as_str())
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert!(ids.contains(&"DEMO".to_string()));
    assert!(!ids.contains(&"DEMO.PLANNED".to_string()));
    assert!(json["body"].is_null());

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
    assert!(stdout.contains("<title>special specs</title>"));
    assert!(stdout.contains("<span class=\"node-id\">DEMO</span>"));
    assert!(stdout.contains("<span class=\"node-id\">DEMO.LIVE</span>"));
    assert!(!stdout.contains("<span class=\"node-id\">DEMO.PLANNED</span>"));
    assert!(!stdout.contains("<summary>@verifies"));
    assert!(!stdout.contains("verifies_demo_root"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE.HTML.CODE_HIGHLIGHTING
fn spec_verbose_html_renders_best_effort_code_highlighting() {
    let root = temp_repo_dir("special-cli-html-highlight");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--html", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<code class=\"language-rust\">"));
    assert!(stdout.contains("style=\"color:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE.HTML
fn spec_verbose_html_includes_support_bodies() {
    let root = temp_repo_dir("special-cli-html-verbose");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--html", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("verifies: 1"));
    assert!(stdout.contains("verifies_demo_root"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE
fn spec_verbose_includes_verify_bodies() {
    let root = temp_repo_dir("special-cli-verbose");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("body at:"));
    assert!(stdout.contains("fn verifies_demo_root() {}"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE
fn spec_verbose_includes_file_verify_bodies() {
    let root = temp_repo_dir("special-cli-file-verbose");
    write_file_verify_fixture(&root);

    let output = run_special(&root, &["spec", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@fileverifies"));
    assert!(stdout.contains("fn verifies_demo_root() {}"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE
fn spec_verbose_includes_file_attest_bodies() {
    let root = temp_repo_dir("special-cli-file-attest-verbose");
    write_file_attest_fixture(&root);

    let output = run_special(&root, &["spec", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@fileattests"));
    assert!(stdout.contains("# Review Notes"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE.JSON
fn spec_verbose_json_includes_support_bodies() {
    let root = temp_repo_dir("special-cli-json-verbose");
    write_live_and_planned_fixture(&root);

    let output = run_special(&root, &["spec", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO node should be present");
    let verify = demo["verifies"]
        .as_array()
        .and_then(|verifies| verifies.first())
        .expect("verify should be present");
    assert_eq!(
        verify["body"],
        Value::String("fn verifies_demo_root() {}".to_string())
    );
    assert_eq!(
        verify["body_location"]["line"],
        Value::Number(serde_json::Number::from(2))
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE.JSON
fn spec_verbose_json_includes_file_attest_scope() {
    let root = temp_repo_dir("special-cli-file-attest-json");
    write_file_attest_fixture(&root);

    let output = run_special(&root, &["spec", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO node should be present");
    let attest = demo["attests"]
        .as_array()
        .and_then(|attests| attests.first())
        .expect("attest should be present");
    assert_eq!(attest["scope"], Value::String("file".to_string()));
    assert!(
        attest["body"]
            .as_str()
            .expect("attest body should be present")
            .contains("# Review Notes")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.EXPLICIT_ROOT
fn spec_uses_root_declared_in_special_toml() {
    let root = temp_repo_dir("special-cli-root");
    write_special_toml_root_fixture(&root);

    let output = run_special(&root, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert_eq!(node_ids, vec!["DEMO".to_string()]);
    assert!(!stderr.contains("warning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.ANCESTOR_CONFIG
fn spec_uses_ancestor_special_toml_from_nested_directory() {
    let root = temp_repo_dir("special-cli-dot-root");
    let nested = write_special_toml_dot_root_fixture(&root);

    let output = run_special(&nested, &["spec"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(rendered_spec_node_ids(&stdout), vec!["DEMO".to_string()]);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
