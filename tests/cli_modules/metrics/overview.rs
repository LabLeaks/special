/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS.OVERVIEW
General metrics view-shape, ownership, scoping, and structured-summary tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.OVERVIEW
use std::fs;

use serde_json::Value;

use crate::support::{
    find_node_by_id, run_special, temp_repo_dir, write_binary_entrypoint_root_fixture,
    write_coupling_module_analysis_fixture, write_dependency_module_analysis_fixture,
    write_item_scoped_module_analysis_fixture, write_module_analysis_fixture,
    write_restricted_visibility_root_fixture, write_source_local_module_analysis_fixture,
};
use crate::typescript_test_fixtures::write_typescript_module_analysis_fixture;

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS
fn modules_metrics_surface_module_ownership_granularity() {
    let root = temp_repo_dir("special-cli-modules-coverage");
    write_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special arch metrics"));
    assert!(stdout.contains("total modules: 1"));
    assert!(stdout.contains("total areas: 0"));
    assert!(stdout.contains("file-scoped implements: 1"));
    assert!(stdout.contains("item-scoped implements: 0"));
    assert!(stdout.contains("owned lines:"));
    assert!(stdout.contains("public items: 1"));
    assert!(stdout.contains("internal items: 1"));
    assert!(stdout.contains("owned lines by module"));
    assert!(stdout.contains("DEMO:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.SURFACE
fn modules_metrics_surface_owned_lines_and_rust_item_counts() {
    let root = temp_repo_dir("special-cli-modules-metrics");
    write_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("owned lines:"));
    assert!(stdout.contains("public items: 1"));
    assert!(stdout.contains("internal items: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_default_unscoped_view_omits_per_module_analysis_blocks() {
    let root = temp_repo_dir("special-cli-modules-metrics-unscoped-summary");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special arch metrics"));
    assert!(stdout.contains("external dependency targets by module"));
    assert!(!stdout.contains("dependency refs:"));
    assert!(!stdout.contains("dependency targets:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_default_unscoped_view_surfaces_repo_level_coupling_rankings() {
    let root = temp_repo_dir("special-cli-modules-metrics-unscoped-coupling-summary");
    write_coupling_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("fan out by module"));
    assert!(stdout.contains("DEMO.API: 1"));
    assert!(stdout.contains("fan in by module"));
    assert!(stdout.contains("DEMO.SHARED: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_scoped_view_keeps_per_module_analysis_blocks_without_verbose() {
    let root = temp_repo_dir("special-cli-modules-metrics-scoped-summary");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("dependency refs: 2"));
    assert!(stdout.contains("dependency targets: 2"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON
fn modules_metrics_json_includes_structured_analysis() {
    let root = temp_repo_dir("special-cli-modules-metrics-json");
    write_item_scoped_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert!(json["metrics"].is_object());
    assert_eq!(json["metrics"]["total_modules"], Value::from(1));
    assert_eq!(json["metrics"]["total_areas"], Value::from(0));
    assert_eq!(json["metrics"]["item_scoped_implements"], Value::from(1));
    assert!(
        json["metrics"]["owned_lines_by_module"]
            .as_array()
            .expect("owned lines by module should be an array")
            .iter()
            .any(|entry| entry["value"].as_str() == Some("DEMO"))
    );

    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["coverage"]["file_scoped_implements"],
        Value::from(0)
    );
    assert_eq!(
        demo["analysis"]["coverage"]["item_scoped_implements"],
        Value::from(1)
    );
    assert_eq!(demo["analysis"]["metrics"]["public_items"], Value::from(1));
    assert_eq!(
        demo["analysis"]["metrics"]["internal_items"],
        Value::from(0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_json_default_unscoped_view_omits_node_analysis() {
    let root = temp_repo_dir("special-cli-modules-metrics-json-unscoped-summary");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert!(demo["analysis"].is_null());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_json_scoped_view_keeps_node_analysis_without_verbose() {
    let root = temp_repo_dir("special-cli-modules-metrics-json-scoped-summary");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["dependencies"]["reference_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["dependencies"]["distinct_targets"],
        Value::from(2)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_json_default_unscoped_view_includes_repo_level_coupling_rankings() {
    let root = temp_repo_dir("special-cli-modules-metrics-json-unscoped-coupling-summary");
    write_coupling_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert!(
        json["metrics"]["fan_out_by_module"]
            .as_array()
            .expect("fan_out_by_module should be an array")
            .iter()
            .any(|entry| {
                entry["value"].as_str() == Some("DEMO.API") && entry["count"].as_u64() == Some(1)
            })
    );
    assert!(
        json["metrics"]["fan_in_by_module"]
            .as_array()
            .expect("fan_in_by_module should be an array")
            .iter()
            .any(|entry| {
                entry["value"].as_str() == Some("DEMO.SHARED") && entry["count"].as_u64() == Some(1)
            })
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_json_omits_detailed_lists_from_default_view() {
    let root = temp_repo_dir("special-cli-modules-metrics-json-default-summary");
    write_typescript_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["dependencies"]["reference_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["dependencies"]["distinct_targets"],
        Value::from(2)
    );
    assert!(
        demo["analysis"]["dependencies"]["targets"].is_null(),
        "default metrics json should omit dependency target detail"
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["analyzed_items"],
        Value::from(6)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["unreached_item_count"],
        Value::from(3)
    );
    assert!(
        demo["analysis"]["item_signals"]["unreached_items"].is_null(),
        "default metrics json should omit per-item signal detail"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_treat_restricted_visibility_items_as_reachability_roots() {
    let root = temp_repo_dir("special-cli-modules-restricted-visibility-root");
    write_restricted_visibility_root_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["unreached_item_count"],
        Value::from(0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_treat_binary_main_as_a_reachability_root() {
    let root = temp_repo_dir("special-cli-modules-binary-main-root");
    write_binary_entrypoint_root_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["unreached_item_count"],
        Value::from(0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_treat_header_implements_after_module_docs_as_file_scoped() {
    let root = temp_repo_dir("special-cli-modules-metrics-source-local-header");
    write_source_local_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");

    assert_eq!(
        demo["analysis"]["coverage"]["file_scoped_implements"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["coverage"]["item_scoped_implements"],
        Value::from(0)
    );
    assert_eq!(demo["analysis"]["metrics"]["public_items"], Value::from(1));
    assert_eq!(
        demo["analysis"]["metrics"]["internal_items"],
        Value::from(1)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
