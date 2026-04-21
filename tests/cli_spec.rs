/**
@module SPECIAL.TESTS.CLI_SPEC
`special specs` output and filtering tests in `tests/cli_spec.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_SPEC
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{
    find_node_by_id, html_node_has_badge, rendered_spec_node_ids, rendered_spec_node_line,
    run_special, run_special_raw, temp_repo_dir, write_current_and_planned_fixture,
    write_deprecated_release_fixture, write_file_attest_fixture, write_file_verify_fixture,
    write_planned_release_fixture, write_special_toml_dot_root_fixture,
    write_special_toml_root_fixture, write_unverified_current_fixture,
};

#[test]
// @verifies SPECIAL.SPEC_COMMAND
fn spec_materializes_declared_spec_tree() {
    let root = temp_repo_dir("special-cli-spec");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(node_ids.contains(&"DEMO.PLANNED".to_string()));
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

    let output = run_special(&root, &["specs"]);
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
// @verifies SPECIAL.HELP.SPECS_COMMAND_PLURAL_PRIMARY
fn singular_spec_alias_is_rejected() {
    let root = temp_repo_dir("special-cli-spec-singular-rejected");

    let output = run_special_raw(&root, &["spec"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("unrecognized subcommand"));
    assert!(stderr.contains("spec"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.DEFAULT_ALL
fn spec_includes_planned_items_by_default() {
    let root = temp_repo_dir("special-cli-spec-default-all");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(rendered_spec_node_ids(&stdout).contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.CURRENT_ONLY
fn spec_current_excludes_planned_items() {
    let root = temp_repo_dir("special-cli-spec-all");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--current"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(!node_ids.contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.PLANNED_ONLY
fn spec_planned_shows_only_planned_items() {
    let root = temp_repo_dir("special-cli-spec-planned");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--planned"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.PLANNED".to_string()));
    assert!(!node_ids.contains(&"DEMO.LIVE".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.PLANNED_RELEASE_METADATA
fn spec_surfaces_planned_release_metadata_across_output_modes() {
    let root = temp_repo_dir("special-cli-planned-release-metadata");
    write_planned_release_fixture(&root);

    let text_output = run_special(&root, &["specs", "--planned"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    let planned_line =
        rendered_spec_node_line(&text_stdout, "DEMO.PLANNED").expect("planned node should render");
    assert!(planned_line.contains("[planned: 0.3.0]"));

    let json_output = run_special(&root, &["specs", "--planned", "--json"]);
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

    let html_output = run_special(&root, &["specs", "--planned", "--html"]);
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

    let text_output = run_special(&root, &["specs"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    let deprecated_line = rendered_spec_node_line(&text_stdout, "DEMO.DEPRECATED")
        .expect("deprecated node should render");
    assert!(deprecated_line.contains("[deprecated: 0.6.0]"));

    let json_output = run_special(&root, &["specs", "--json"]);
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

    let html_output = run_special(&root, &["specs", "--html"]);
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
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "DEMO"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(node_ids.contains(&"DEMO.PLANNED".to_string()));
    assert!(!stdout.contains("No specs found."));

    let output = run_special(&root, &["specs", "DEMO.LIVE"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(!node_ids.contains(&"DEMO".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.UNVERIFIED
fn spec_unverified_filters_current_items_without_support() {
    let root = temp_repo_dir("special-cli-unsupported");
    write_unverified_current_fixture(&root);

    let output = run_special(&root, &["specs", "--unverified"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.UNVERIFIED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn spec_short_u_filters_unverified_current_items() {
    let root = temp_repo_dir("special-cli-unverified-short");
    write_unverified_current_fixture(&root);

    let output = run_special(&root, &["specs", "-u"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.UNVERIFIED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn spec_rejects_planned_and_unverified_filter_combo() {
    let root = temp_repo_dir("special-cli-unverified-planned-conflict");
    write_unverified_current_fixture(&root);

    let output = run_special(&root, &["specs", "--planned", "--unverified"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--planned"));
    assert!(stderr.contains("--unverified"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.METRICS
fn spec_metrics_text_surfaces_contract_health_counts() {
    let root = temp_repo_dir("special-cli-spec-metrics");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "-m"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special specs metrics"));
    assert!(stdout.contains("total specs:"));
    assert!(stdout.contains("unverified specs:"));
    assert!(stdout.contains("verified specs:"));
    assert!(stdout.contains("verifies:"));
    assert!(stdout.contains("attests:"));
    assert!(stdout.contains("specs by file"));
    assert!(stdout.contains("current specs by top-level id"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn spec_metrics_json_includes_structured_metrics() {
    let root = temp_repo_dir("special-cli-spec-metrics-json");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "-m", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert!(json["metrics"].is_object());
    assert_eq!(json["metrics"]["total_specs"], Value::from(3));
    assert_eq!(json["metrics"]["verifies"], Value::from(3));
    assert_eq!(json["metrics"]["attests"], Value::from(0));
    let specs_by_file = json["metrics"]["specs_by_file"]
        .as_array()
        .expect("specs_by_file should be an array");
    assert_eq!(specs_by_file.len(), 1);
    assert_eq!(specs_by_file[0]["count"], Value::from(3));
    assert!(
        specs_by_file[0]["value"]
            .as_str()
            .expect("specs_by_file value should be a string")
            .ends_with("specs.rs")
    );
    assert!(
        !specs_by_file[0]["value"]
            .as_str()
            .expect("specs_by_file value should be a string")
            .starts_with(root.to_string_lossy().as_ref())
    );
    assert_eq!(
        json["metrics"]["current_specs_by_top_level_id"],
        Value::Array(vec![serde_json::json!({
            "value": "DEMO",
            "count": 2
        })])
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.JSON
fn spec_json_emits_json_output() {
    let root = temp_repo_dir("special-cli-json");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--json"]);
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
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<!doctype html>"));
    assert!(stdout.contains("<title>special specs</title>"));
    assert!(stdout.contains("<span class=\"node-id\">DEMO</span>"));
    assert!(stdout.contains("<span class=\"node-id\">DEMO.LIVE</span>"));
    assert!(stdout.contains("<span class=\"node-id\">DEMO.PLANNED</span>"));
    assert!(!stdout.contains("<summary>@verifies"));
    assert!(!stdout.contains("verifies_demo_root"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.VERBOSE.HTML.CODE_HIGHLIGHTING
fn spec_verbose_html_renders_best_effort_code_highlighting() {
    let root = temp_repo_dir("special-cli-html-highlight");
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--html", "--verbose"]);
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
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--html", "--verbose"]);
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
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--verbose"]);
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

    let output = run_special(&root, &["specs", "--verbose"]);
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

    let output = run_special(&root, &["specs", "--verbose"]);
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
    write_current_and_planned_fixture(&root);

    let output = run_special(&root, &["specs", "--json", "--verbose"]);
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

    let output = run_special(&root, &["specs", "--json", "--verbose"]);
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

    let output = run_special(&root, &["specs"]);
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

    let output = run_special(&nested, &["specs"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(rendered_spec_node_ids(&stdout), vec!["DEMO".to_string()]);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
