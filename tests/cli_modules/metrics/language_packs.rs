use std::collections::BTreeSet;
/**
@module SPECIAL.TESTS.CLI_MODULES.METRICS.LANGUAGE_PACKS
Language-pack-specific metrics coverage tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.LANGUAGE_PACKS
use std::fs;

use serde_json::Value;

use crate::go_test_fixtures::write_go_module_analysis_fixture;
use crate::support::{find_node_by_id, run_special, temp_repo_dir};
use crate::typescript_test_fixtures::write_typescript_module_analysis_fixture;

fn unreached_item_names(demo: &Value) -> BTreeSet<String> {
    demo["analysis"]["item_signals"]["unreached_items"]
        .as_array()
        .expect("unreached items should be an array")
        .iter()
        .filter_map(|item| item["name"].as_str().map(ToString::to_string))
        .collect()
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT
fn modules_metrics_surface_typescript_analysis() {
    let root = temp_repo_dir("special-cli-modules-metrics-typescript");
    write_typescript_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(stdout.contains("public items: 2"));
    assert!(stdout.contains("internal items: 4"));
    assert!(stdout.contains("dependency refs: 2"));
    assert!(stdout.contains("dependency targets: 2"));
    assert!(stdout.contains("dependency target: ./shared (1)"));
    assert!(stdout.contains("dependency target: node:fs (1)"));
    assert!(stdout.contains("unreached items: 3"));
    assert!(stdout.contains("isolated item: isolatedExternal"));
    assert!(stdout.contains("unreached item: unreachedClusterEntry"));
    assert!(stdout.contains("unreached item: unreachedClusterLeaf"));
    assert!(stdout.contains("fan out: 1"));
    assert!(stdout.contains("external dependency targets: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT
fn modules_metrics_json_includes_structured_typescript_analysis() {
    let root = temp_repo_dir("special-cli-modules-metrics-typescript-json");
    write_typescript_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(demo["analysis"]["metrics"]["public_items"], Value::from(2));
    assert_eq!(
        demo["analysis"]["metrics"]["internal_items"],
        Value::from(4)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["unreached_item_count"],
        Value::from(3)
    );
    let targets = demo["analysis"]["dependencies"]["targets"]
        .as_array()
        .expect("dependency targets should be an array");
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("./shared".to_string()) && target["count"] == 1
    }));
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("node:fs".to_string()) && target["count"] == 1
    }));
    assert_eq!(
        unreached_item_names(demo),
        BTreeSet::from([
            "isolatedExternal".to_string(),
            "unreachedClusterEntry".to_string(),
            "unreachedClusterLeaf".to_string(),
        ])
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.GO
fn modules_metrics_surface_go_analysis() {
    let root = temp_repo_dir("special-cli-modules-metrics-go");
    write_go_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(stdout.contains("public items: 1"));
    assert!(stdout.contains("internal items: 4"));
    assert!(stdout.contains("dependency refs: 2"));
    assert!(stdout.contains("dependency targets: 2"));
    assert!(stdout.contains("dependency target: fmt (1)"));
    assert!(stdout.contains("dependency target: shared (1)"));
    assert!(stdout.contains("unreached items: 3"));
    assert!(stdout.contains("isolated item: isolatedExternal"));
    assert!(stdout.contains("unreached item: unreachedClusterEntry"));
    assert!(stdout.contains("unreached item: unreachedClusterLeaf"));
    assert!(stdout.contains("fan out: 1"));
    assert!(stdout.contains("external dependency targets: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.GO
fn modules_metrics_json_includes_structured_go_analysis() {
    let root = temp_repo_dir("special-cli-modules-metrics-go-json");
    write_go_module_analysis_fixture(&root);

    let output = run_special(&root, &["arch", "--metrics", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(demo["analysis"]["metrics"]["public_items"], Value::from(1));
    assert_eq!(
        demo["analysis"]["metrics"]["internal_items"],
        Value::from(4)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["unreached_item_count"],
        Value::from(3)
    );
    let targets = demo["analysis"]["dependencies"]["targets"]
        .as_array()
        .expect("dependency targets should be an array");
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("fmt".to_string()) && target["count"] == 1
    }));
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("shared".to_string()) && target["count"] == 1
    }));
    assert_eq!(
        unreached_item_names(demo),
        BTreeSet::from([
            "isolatedExternal".to_string(),
            "unreachedClusterEntry".to_string(),
            "unreachedClusterLeaf".to_string(),
        ])
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
