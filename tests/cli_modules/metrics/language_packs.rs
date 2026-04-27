use std::collections::BTreeSet;
/*
@module SPECIAL.TESTS.CLI_MODULES.METRICS.LANGUAGE_PACKS
Language-pack-specific metrics coverage tests for `special arch --metrics`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES.METRICS.LANGUAGE_PACKS

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.QUALITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.ITEM_SIGNALS

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.ITEM_SIGNALS.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.ITEM_SIGNALS.QUALITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.GO.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.GO.QUALITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.GO.ITEM_SIGNALS

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.GO.ITEM_SIGNALS.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.GO.ITEM_SIGNALS.QUALITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.QUALITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.ITEM_SIGNALS

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.ITEM_SIGNALS.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.ITEM_SIGNALS.QUALITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.QUALITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.ITEM_SIGNALS

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.ITEM_SIGNALS.COMPLEXITY

// @fileverifies SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.ITEM_SIGNALS.QUALITY
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

fn item_signal_names(demo: &Value, field: &str) -> BTreeSet<String> {
    demo["analysis"]["item_signals"][field]
        .as_array()
        .expect("item signal group should be an array")
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
    assert!(stdout.contains("complexity functions: 6"));
    assert!(stdout.contains("cyclomatic total: 7"));
    assert!(stdout.contains("cyclomatic max: 2"));
    assert!(stdout.contains("cognitive total: 1"));
    assert!(stdout.contains("cognitive max: 1"));
    assert!(stdout.contains("quality public functions: 2"));
    assert!(stdout.contains("quality parameters: 3"));
    assert!(stdout.contains("quality bool params: 1"));
    assert!(stdout.contains("quality raw string params: 2"));
    assert!(stdout.contains("quality panic sites: 1"));
    assert!(stdout.contains("dependency refs: 2"));
    assert!(stdout.contains("dependency targets: 2"));
    assert!(stdout.contains("dependency target: ./shared (1)"));
    assert!(stdout.contains("dependency target: node:fs (1)"));
    assert!(stdout.contains("unreached items: 3"));
    assert!(stdout.contains("isolated item: isolatedExternal"));
    assert!(stdout.contains("unreached item: unreachedClusterEntry"));
    assert!(stdout.contains("unreached item: unreachedClusterLeaf"));
    assert!(stdout.contains("highest complexity item: entry"));
    assert!(stdout.contains("parameter-heavy item: entry"));
    assert!(stdout.contains("stringly boundary item: entry"));
    assert!(stdout.contains("panic-heavy item: entry"));
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
        demo["analysis"]["complexity"]["function_count"],
        Value::from(6)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["total_cyclomatic"],
        Value::from(7)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cyclomatic"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["total_cognitive"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cognitive"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["public_function_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["quality"]["parameter_count"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["quality"]["bool_parameter_count"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["raw_string_parameter_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["quality"]["panic_site_count"],
        Value::from(1)
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
    assert!(item_signal_names(demo, "highest_complexity_items").contains("entry"));
    assert!(item_signal_names(demo, "parameter_heavy_items").contains("entry"));
    assert!(item_signal_names(demo, "stringly_boundary_items").contains("entry"));
    assert!(item_signal_names(demo, "panic_heavy_items").contains("entry"));

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
    assert!(stdout.contains("complexity functions: 5"));
    assert!(stdout.contains("cyclomatic total: 6"));
    assert!(stdout.contains("cyclomatic max: 2"));
    assert!(stdout.contains("cognitive total: 1"));
    assert!(stdout.contains("cognitive max: 1"));
    assert!(stdout.contains("quality public functions: 1"));
    assert!(stdout.contains("quality parameters: 3"));
    assert!(stdout.contains("quality bool params: 1"));
    assert!(stdout.contains("quality raw string params: 2"));
    assert!(stdout.contains("quality panic sites: 1"));
    assert!(stdout.contains("dependency refs: 2"));
    assert!(stdout.contains("dependency targets: 2"));
    assert!(stdout.contains("dependency target: fmt (1)"));
    assert!(stdout.contains("dependency target: shared (1)"));
    assert!(stdout.contains("unreached items: 3"));
    assert!(stdout.contains("isolated item: isolatedExternal"));
    assert!(stdout.contains("unreached item: unreachedClusterEntry"));
    assert!(stdout.contains("unreached item: unreachedClusterLeaf"));
    assert!(stdout.contains("highest complexity item: Entry"));
    assert!(stdout.contains("parameter-heavy item: Entry"));
    assert!(stdout.contains("stringly boundary item: Entry"));
    assert!(stdout.contains("panic-heavy item: Entry"));
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
        demo["analysis"]["complexity"]["function_count"],
        Value::from(5)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["total_cyclomatic"],
        Value::from(6)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cyclomatic"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["total_cognitive"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cognitive"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["public_function_count"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["parameter_count"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["quality"]["bool_parameter_count"],
        Value::from(1)
    );
    assert_eq!(
        demo["analysis"]["quality"]["raw_string_parameter_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["quality"]["panic_site_count"],
        Value::from(1)
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
    assert!(item_signal_names(demo, "highest_complexity_items").contains("Entry"));
    assert!(item_signal_names(demo, "parameter_heavy_items").contains("Entry"));
    assert!(item_signal_names(demo, "stringly_boundary_items").contains("Entry"));
    assert!(item_signal_names(demo, "panic_heavy_items").contains("Entry"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.TOOLCHAIN_DEGRADED
fn modules_metrics_warns_when_typescript_analyzer_enrichment_is_unavailable() {
    let root = temp_repo_dir("special-cli-modules-metrics-typescript-degraded");
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("src")).expect("source dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module APP`\nApp module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("src/app.ts"),
        "// @fileimplements APP\nexport function run() { return 1; }\n",
    )
    .expect("TypeScript fixture should be written");

    let output = run_special(&root, &["arch", "--metrics"]);
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("TypeScript analyzer enrichment degraded"));
    assert!(stderr.contains("install a resolvable `typescript` package"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.GO.TOOLCHAIN_DEGRADED
fn modules_metrics_warns_when_go_analyzer_enrichment_is_unavailable() {
    let root = temp_repo_dir("special-cli-modules-metrics-go-degraded");
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("app")).expect("source dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(root.join("go.mod"), "module example.com/app\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module APP`\nApp module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements APP\npackage app\n\nfunc Run() int { return 1 }\n",
    )
    .expect("Go fixture should be written");

    let output = run_special(&root, &["arch", "--metrics"]);
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("Go analyzer enrichment degraded"));
    assert!(stderr.contains("provide a working `go` tool"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.COUPLING
fn modules_metrics_uses_typescript_compiler_resolution_for_path_alias_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-typescript-alias-coupling");
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("src")).expect("source dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(root.join(".tool-versions"), "nodejs 24.15.0\n")
        .expect(".tool-versions should be written");
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@app/*":["src/*"]}},"include":["src/**/*.ts"]}"#,
    )
    .expect("tsconfig should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module APP`\nApp module.\n\n### `@module SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("src/app.ts"),
        "// @fileimplements APP\nimport { sharedValue } from \"@app/shared\";\n\nexport function run() { return sharedValue(); }\n",
    )
    .expect("TypeScript app fixture should be written");
    fs::write(
        root.join("src/shared.ts"),
        "// @fileimplements SHARED\nexport function sharedValue() { return 1; }\n",
    )
    .expect("TypeScript shared fixture should be written");

    let output = run_special(&root, &["arch", "APP", "--metrics", "--json"]);
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        !stderr.contains("TypeScript analyzer enrichment degraded"),
        "TypeScript compiler-backed coupling test must execute the enriched analyzer path: {stderr}"
    );

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let app = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "APP")))
        .expect("app module should be present");
    assert_eq!(app["analysis"]["coupling"]["fan_out"], Value::from(1));
    assert_eq!(
        app["analysis"]["coupling"]["external_target_count"],
        Value::from(0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.GO.COUPLING
fn modules_metrics_uses_go_list_resolution_for_module_import_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-go-list-coupling");
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("shared")).expect("shared dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(root.join(".tool-versions"), "go 1.23.12\n")
        .expect(".tool-versions should be written");
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module APP`\nApp module.\n\n### `@module SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements APP\npackage app\n\nimport \"example.com/demo/shared\"\n\nfunc Run() int { return shared.SharedValue() }\n",
    )
    .expect("Go app fixture should be written");
    fs::write(
        root.join("shared/shared.go"),
        "// @fileimplements SHARED\npackage shared\n\nfunc SharedValue() int { return 1 }\n",
    )
    .expect("Go shared fixture should be written");

    let output = run_special(&root, &["arch", "APP", "--metrics", "--json"]);
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        !stderr.contains("Go analyzer enrichment degraded"),
        "Go tool-backed coupling test must execute the enriched analyzer path: {stderr}"
    );

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let app = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "APP")))
        .expect("app module should be present");
    assert_eq!(app["analysis"]["coupling"]["fan_out"], Value::from(1));
    assert_eq!(
        app["analysis"]["coupling"]["external_target_count"],
        Value::from(0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
