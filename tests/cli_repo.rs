#[allow(dead_code)]
#[path = "../src/language_packs/go/test_fixtures.rs"]
mod go_test_fixtures;
#[allow(dead_code)]
#[path = "../src/language_packs/rust/test_fixtures.rs"]
mod rust_test_fixtures;
/**
@module SPECIAL.TESTS.CLI_REPO
`special health` output and repo-wide quality tests in `tests/cli_repo.rs`.

@group SPECIAL.HEALTH_COMMAND_GROUP
special health can surface repo-wide quality signals that are not tied to a single architecture module.

@spec SPECIAL.HEALTH_COMMAND
special health materializes repo-wide quality signals for the current repository.

@spec SPECIAL.HEALTH_COMMAND.JSON
special health --json emits structured repo-wide quality signals.

@spec SPECIAL.HEALTH_COMMAND.HTML
special health --html emits an HTML repo-wide quality view.

@spec SPECIAL.HEALTH_COMMAND.VERBOSE
special health --verbose includes fuller detail for repo-wide quality signals when built-in analyzers provide it.

@spec SPECIAL.HEALTH_COMMAND.SCOPE
special health PATH scopes repo-wide quality and traceability reporting to analyzable items in matching files or directories without changing the underlying repo analysis model.

@spec SPECIAL.HEALTH_COMMAND.SYMBOL
special health PATH --symbol NAME narrows the health view to items in that scoped file whose symbol name matches NAME.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY
special health surfaces repo-wide Rust backward trace only when `rust-analyzer` is available; otherwise it says that Rust backward trace is unavailable.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT
special health surfaces built-in TypeScript implementation traceability for analyzable TypeScript source items.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.TOOL_EDGES
special health combines parser and TypeScript compiler edges so import-alias calls can trace to the correct owned implementation item.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.REFERENCE_EDGES
special health combines parser and TypeScript compiler reference edges so callback-style TypeScript support can trace to the owned implementation item that is passed through an intermediary helper.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.JSX_REFERENCE_EDGES
special health combines parser and TypeScript compiler reference edges so TSX component references in JSX can trace through the rendered component stack to the owned implementation items they reference.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.CLIENT_COMPONENT_EDGES
special health combines parser and TypeScript compiler reference edges so Next-style page components can trace through `"use client"` component stacks to the owned implementation items they render.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.EVENT_CALLBACK_EDGES
special health combines parser and TypeScript compiler reference edges so TSX callback props can trace from a rendered component to the shared action helper stack passed through that prop.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.FORWARDED_CALLBACK_EDGES
special health combines parser and TypeScript compiler reference edges so TSX callback props can trace through forwarded component boundaries to the shared action helper stack passed through that prop.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.HOOK_CALLBACK_EDGES
special health combines parser and TypeScript compiler reference edges so hook-returned callbacks can trace from a rendered component to the shared action helper stack passed through that callback.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.EFFECT_CALLBACK_EDGES
special health combines parser and TypeScript compiler reference edges so functions called inside effect callbacks can trace from a rendered component to the shared helper stack invoked inside the callback body.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.CONTEXT_CALLBACK_EDGES
special health combines parser and TypeScript compiler reference edges so shared context-provided callbacks can trace from rendered consumers to the shared action helper stack carried by the context value.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO
special health surfaces built-in Go implementation traceability for analyzable Go source items.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO.TOOL_EDGES
special health combines parser and Go tool-backed package edges so selector calls can trace to the correct owned package item even when exported names collide.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO.REFERENCE_EDGES
special health combines parser and Go tool-backed reference edges so callback-style Go support can trace to the owned implementation item that is passed through an intermediary helper.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.ALL_SUPPORTING_ROOTS
special health includes every verifying test and verified spec claim that supports one traced implementation item when backward trace is available.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNAVAILABLE
special health reports when Rust backward trace is unavailable instead of guessing from weaker analysis.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNEXPLAINED
special health keeps currently unexplained implementation separate from traced and statically mediated implementation.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.DEFAULT_VISIBLE
special health surfaces traceability in the default health view.

@spec SPECIAL.HEALTH_COMMAND.JSON.TRACEABILITY
special health --json includes structured Rust backward-trace output at the repo boundary when available, or a structured unavailable reason when it is not.

@spec SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNEXPLAINED.EVIDENCE
special health surfaces raw unexplained-item evidence, including visibility, test-file placement, declared-module ownership, whether the owning module already has current-spec-traced code, and whether the item is structurally connected inside that module to traced code, across text, HTML, and JSON output when backward trace is available.

@spec SPECIAL.HEALTH_COMMAND.DUPLICATION
special health surfaces repo-wide duplicate-logic signals from owned implementation when built-in analyzers can identify substantively similar code shapes honestly.

@spec SPECIAL.HEALTH_COMMAND.UNOWNED_ITEMS
special health surfaces repo-wide unowned item indicators so code outside declared modules stays visible even when traceability is available.
*/
// @fileimplements SPECIAL.TESTS.CLI_REPO
#[path = "support/cli.rs"]
mod support;
#[allow(dead_code)]
#[path = "../src/language_packs/typescript/test_fixtures.rs"]
mod typescript_test_fixtures;

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Duration;

use serde_json::Value;

use go_test_fixtures::{
    write_go_reference_traceability_fixture, write_go_tool_traceability_fixture,
    write_go_traceability_fixture,
};
use rust_test_fixtures::{
    write_traceability_instance_method_fixture, write_traceability_module_analysis_fixture,
    write_traceability_module_context_fixture, write_traceability_multiple_supports_fixture,
    write_traceability_review_surface_fixture,
};
use support::{
    run_special, rust_analyzer_available, spawn_special, temp_repo_dir, top_level_help_commands,
    write_duplicate_item_signals_module_analysis_fixture,
    write_many_duplicate_item_signals_module_analysis_fixture,
    write_unreached_code_module_analysis_fixture,
};
use typescript_test_fixtures::{
    write_typescript_context_traceability_fixture, write_typescript_effect_traceability_fixture,
    write_typescript_event_traceability_fixture,
    write_typescript_forwarded_callback_traceability_fixture,
    write_typescript_hook_callback_traceability_fixture,
    write_typescript_next_traceability_fixture, write_typescript_react_traceability_fixture,
    write_typescript_reference_traceability_fixture, write_typescript_tool_traceability_fixture,
    write_typescript_traceability_fixture,
};

fn assert_typescript_traceability_unavailable(json: &Value) {
    assert_eq!(json["analysis"]["traceability"], Value::Null);
    assert!(
        json["analysis"]["traceability_unavailable_reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("TypeScript backward trace"))
    );
}

fn assert_go_traceability_unavailable(json: &Value) {
    assert_eq!(json["analysis"]["traceability"], Value::Null);
    assert!(
        json["analysis"]["traceability_unavailable_reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("Go backward trace"))
    );
}

#[test]
// @verifies SPECIAL.HELP.HEALTH_COMMAND
fn top_level_help_presents_repo_command() {
    let root = temp_repo_dir("special-cli-repo-help");

    let output = run_special(&root, &["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(top_level_help_commands(&stdout).iter().any(
        |(name, summary)| name == "health" && summary == "Inspect code health and traceability"
    ));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND
fn repo_materializes_repo_wide_quality_signals() {
    let root = temp_repo_dir("special-cli-repo");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special health"));
    assert!(stdout.contains("repo-wide signals"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.DUPLICATION
fn repo_surfaces_repo_wide_duplication_signals() {
    let root = temp_repo_dir("special-cli-repo-duplication");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("duplicate items: 2"));
    assert!(stdout.contains("duplicate items meaning:"));
    assert!(stdout.contains("duplicate items exact:"));
    assert!(
        !stdout.contains(
            "duplicate item: DEMO:alpha.rs:first_duplicate [function; duplicate peers 1]"
        )
    );
    assert!(
        !stdout.contains(
            "duplicate item: DEMO:beta.rs:second_duplicate [function; duplicate peers 1]"
        )
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.UNOWNED_ITEMS
fn repo_surfaces_unowned_items() {
    let root = temp_repo_dir("special-cli-repo-unreached");
    write_unreached_code_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("unowned items: 1"));
    assert!(stdout.contains("unowned items meaning:"));
    assert!(stdout.contains("unowned item: hidden.rs:hidden_unreached [function]"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.JSON
fn repo_json_includes_structured_repo_signals() {
    let root = temp_repo_dir("special-cli-repo-json");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(
        json["analysis"]["repo_signals"]["duplicate_items"],
        Value::from(2)
    );
    assert!(
        json["analysis"]["repo_signals"]["duplicate_item_details"].is_null(),
        "non-verbose repo signals should omit duplicate detail vectors"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.METRICS
fn repo_metrics_text_surfaces_repo_health_counts() {
    let root = temp_repo_dir("special-cli-repo-metrics");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "-m"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("special health metrics"));
    assert!(stdout.contains("duplicate items: 2"));
    assert!(stdout.contains("unowned items: 0"));
    assert!(stdout.contains("duplicate items by file"));
    assert!(stdout.contains("alpha.rs: 1"));
    assert!(stdout.contains("beta.rs: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn repo_metrics_json_includes_structured_metrics() {
    let root = temp_repo_dir("special-cli-repo-metrics-json");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "-m", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["metrics"]["duplicate_items"], Value::from(2));
    assert_eq!(json["metrics"]["unowned_items"], Value::from(0));
    assert_eq!(
        json["metrics"]["duplicate_items_by_file"],
        Value::Array(vec![
            serde_json::json!({"value": "alpha.rs", "count": 1}),
            serde_json::json!({"value": "beta.rs", "count": 1}),
        ])
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn concurrent_health_runs_wait_cleanly_for_a_shared_cache_fill() {
    let root = temp_repo_dir("special-cli-health-cache-contention");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let cache_lock = {
        let mut hasher = DefaultHasher::new();
        root.hash(&mut hasher);
        let root_hash = hasher.finish();
        std::env::temp_dir()
            .join("special-cache")
            .join(format!("{root_hash:016x}"))
            .join("repo-analysis-v2.json.lock")
    };
    if let Some(parent) = cache_lock.parent() {
        fs::create_dir_all(parent).expect("cache dir should exist");
    }
    fs::write(&cache_lock, b"locked").expect("lock file should be written");

    let first = spawn_special(&root, &["health", "--json"]);
    let second = spawn_special(&root, &["health", "--json"]);

    thread::sleep(Duration::from_millis(150));
    fs::remove_file(&cache_lock).expect("lock file should be removed");

    let first_output = first
        .wait_with_output()
        .expect("first concurrent run should finish");
    let second_output = second
        .wait_with_output()
        .expect("second concurrent run should finish");

    assert!(
        first_output.status.success(),
        "first run failed: {}",
        String::from_utf8_lossy(&first_output.stderr)
    );
    assert!(
        second_output.status.success(),
        "second run failed: {}",
        String::from_utf8_lossy(&second_output.stderr)
    );

    let first_json: Value =
        serde_json::from_slice(&first_output.stdout).expect("first stdout should be valid json");
    let second_json: Value =
        serde_json::from_slice(&second_output.stdout).expect("second stdout should be valid json");
    assert_eq!(first_json, second_json);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn repo_health_emits_progress_to_stderr_in_noninteractive_runs() {
    let root = temp_repo_dir("special-cli-health-progress-stderr");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "--json"]);
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("special health: resolving project root"));
    assert!(stderr.contains("special health: building health view"));
    assert!(stderr.contains("special health: computing repo ownership signals"));
    assert!(stderr.contains("special health: building language analysis contexts"));
    assert!(stderr.contains("special health: cache activity:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn repo_scope_progress_limits_traceability_contexts_to_scoped_languages() {
    let root = temp_repo_dir("special-cli-health-scoped-language-progress");
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("specs dir should be created");
    fs::create_dir_all(root.join("src")).expect("src dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO`\nDemo root.\n\n### `@module DEMO.TS`\nTypeScript module.\n\n### `@module DEMO.RUST`\nRust module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.TS`\nTypeScript flow.\n\n### `@spec APP.RUST`\nRust flow.\n",
    )
    .expect("specs fixture should be written");
    fs::write(
        root.join("src/app.ts"),
        "// @fileimplements DEMO.TS\nexport function liveImpl() {\n    return helper();\n}\n\nfunction helper() {\n    return 1;\n}\n",
    )
    .expect("typescript fixture should be written");
    fs::write(
        root.join("src/app.test.ts"),
        "import { liveImpl } from \"./app\";\n\n// @verifies APP.TS\nexport function verifies_live_impl() {\n    return liveImpl();\n}\n",
    )
    .expect("typescript test fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO.RUST\npub fn live_impl() {}\n",
    )
    .expect("rust fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.RUST\n#[test]\nfn verifies_live_impl() {\n    crate::live_impl();\n}\n",
    )
    .expect("rust test fixture should be written");

    let output = run_special(&root, &["health", "src/app.ts", "--json"]);
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("building typescript analysis context"));
    assert!(!stderr.contains("building rust analysis context"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn repo_metrics_json_includes_traceability_metrics_when_requested() {
    let root = temp_repo_dir("special-cli-repo-metrics-traceability-json");
    write_typescript_traceability_fixture(&root);

    let output = run_special(&root, &["health", "-m", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        assert!(json["metrics"]["traceability"].is_object());
        assert_eq!(
            json["metrics"]["traceability"]["analyzed_items"],
            Value::from(4)
        );
        assert_eq!(
            json["metrics"]["traceability"]["current_spec_items"],
            Value::from(3)
        );
        assert_eq!(
            json["metrics"]["traceability"]["unexplained_items_by_file"],
            Value::Array(vec![serde_json::json!({
                "value": "src/app.ts",
                "count": 1
            })])
        );
        assert_eq!(
            json["metrics"]["traceability"]["unexplained_review_surface_items_by_file"],
            Value::Array(vec![serde_json::json!({
                "value": "src/app.ts",
                "count": 1
            })])
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.HTML
fn repo_html_emits_html_output() {
    let root = temp_repo_dir("special-cli-repo-html");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<!doctype html>"));
    assert!(stdout.contains("special health"));
    assert!(stdout.contains("duplicate items"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.VERBOSE
fn repo_verbose_includes_fuller_repo_signal_detail() {
    let duplicate_root = temp_repo_dir("special-cli-repo-verbose-duplicates");
    write_many_duplicate_item_signals_module_analysis_fixture(&duplicate_root);

    let normal_output = run_special(&duplicate_root, &["health"]);
    assert!(normal_output.status.success());
    let normal_stdout = String::from_utf8(normal_output.stdout).expect("stdout should be utf-8");

    let verbose_output = run_special(&duplicate_root, &["health", "--verbose"]);
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
// @verifies SPECIAL.HEALTH_COMMAND.SCOPE
fn repo_scope_limits_repo_signals_to_matching_files() {
    let root = temp_repo_dir("special-cli-repo-scope-signals");
    write_duplicate_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "alpha.rs", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let duplicate_items = json["analysis"]["repo_signals"]["duplicate_item_details"]
        .as_array()
        .expect("duplicate items should be an array");
    assert_eq!(
        json["analysis"]["repo_signals"]["duplicate_items"],
        Value::from(1)
    );
    assert_eq!(duplicate_items.len(), 1);
    let duplicate = duplicate_items
        .iter()
        .find(|item| item["path"] == "alpha.rs")
        .expect("scoped duplicate details should include alpha.rs");
    assert_eq!(duplicate["path"], Value::from("alpha.rs"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.SCOPE
fn repo_scope_limits_traceability_to_matching_files() {
    let root = temp_repo_dir("special-cli-repo-scope-traceability");
    write_typescript_traceability_fixture(&root);

    let output = run_special(&root, &["health", "src/app.ts", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let current_items = json["analysis"]["traceability"]["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        let current_names = current_items
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect::<Vec<_>>();
        assert!(current_names.contains(&"liveImpl"));
        assert!(current_names.contains(&"helper"));
        assert!(!current_names.contains(&"sharedValue"));
        assert!(
            json["analysis"]["traceability"]["unexplained_items"]
                .as_array()
                .expect("unexplained items should be an array")
                .iter()
                .any(|item| item["name"] == "orphanImpl")
        );
        assert_eq!(
            json["analysis"]["traceability_unavailable_reason"],
            Value::Null
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.SYMBOL
fn repo_symbol_scope_narrows_health_view_to_one_symbol() {
    let root = temp_repo_dir("special-cli-health-symbol");
    write_typescript_traceability_fixture(&root);

    let output = run_special(
        &root,
        &[
            "health",
            "src/app.ts",
            "--symbol",
            "liveImpl",
            "--json",
            "--verbose",
        ],
    );
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let analysis = &json["analysis"]["traceability"];
        let current_items = analysis["current_spec_items"]
            .as_array()
            .expect("current items should be an array");
        assert_eq!(current_items.len(), 1);
        assert_eq!(
            current_items
                .iter()
                .find(|item| item["name"] == "liveImpl")
                .expect("current items should include liveImpl")["name"],
            Value::from("liveImpl")
        );
        assert_eq!(analysis["analyzed_items"], 1);
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.SYMBOL
fn repo_symbol_scope_requires_exactly_one_path() {
    let root = temp_repo_dir("special-cli-health-symbol-requires-path");
    write_typescript_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--symbol", "liveImpl"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--symbol requires exactly one PATH"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY
fn repo_surfaces_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability");
    write_traceability_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("traceability"));
    assert!(stdout.contains(
        "unavailable: Rust backward trace is unavailable because `rust-analyzer` is not installed"
    ));
    assert!(!stdout.contains("current spec item:"));
    assert!(!stdout.contains("unexplained item:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.JSON.TRACEABILITY
fn repo_json_includes_structured_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-json");
    write_traceability_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["analysis"]["traceability"], Value::Null);
    assert_eq!(
        json["analysis"]["traceability_unavailable_reason"],
        Value::from("Rust backward trace is unavailable because `rust-analyzer` is not installed")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.DEFAULT_VISIBLE
fn repo_non_verbose_traceability_stays_summary_only() {
    let root = temp_repo_dir("special-cli-repo-traceability-summary-only");
    write_traceability_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["health"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text_stdout.contains("traceability"));
    assert!(text_stdout.contains("unavailable:"));
    assert!(!text_stdout.contains("current spec item:"));
    assert!(!text_stdout.contains("unexplained item:"));

    let json_output = run_special(&root, &["health", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    assert_eq!(json["analysis"]["traceability"], Value::Null);
    assert!(
        json["analysis"]["traceability_unavailable_reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("rust-analyzer"))
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNAVAILABLE
fn repo_traceability_reports_unavailable_in_html_too() {
    let root = temp_repo_dir("special-cli-repo-traceability-html-unavailable");
    write_traceability_module_analysis_fixture(&root);

    let output = run_special(&root, &["health", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("traceability"));
    assert!(stdout.contains("rust-analyzer"));
    assert!(stdout.contains("unavailable"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.ALL_SUPPORTING_ROOTS
fn repo_traceability_keeps_all_supporting_roots_for_one_item() {
    let root = temp_repo_dir("special-cli-repo-traceability-multiple-supports");
    write_traceability_multiple_supports_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if rust_analyzer_available() {
        let current_items = json["analysis"]["traceability"]["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        let shared_helper = current_items
            .iter()
            .find(|item| item["name"] == "shared_helper")
            .expect("shared helper should be current");

        assert_eq!(
            shared_helper["current_specs"],
            serde_json::json!(["APP.ALPHA", "APP.BETA"])
        );
        assert_eq!(
            shared_helper["verifying_tests"],
            serde_json::json!(["verifies_alpha_path", "verifies_beta_path"])
        );
        assert_eq!(
            json["analysis"]["traceability_unavailable_reason"],
            Value::Null
        );
    } else {
        assert_eq!(json["analysis"]["traceability"], Value::Null);
        assert!(
            json["analysis"]["traceability_unavailable_reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("rust-analyzer"))
        );
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNEXPLAINED
fn repo_traceability_resolves_instance_method_dispatch_when_rust_analyzer_is_available() {
    let root = temp_repo_dir("special-cli-repo-traceability-instance-method");
    write_traceability_instance_method_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if rust_analyzer_available() {
        let current_items = json["analysis"]["traceability"]["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        let current_names = current_items
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect::<Vec<_>>();
        assert!(current_names.contains(&"exercise"));
        assert!(current_names.contains(&"run"));
        assert!(current_names.contains(&"helper"));
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_review_surface_items"],
            Value::from(0)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_public_items"],
            Value::from(0)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_internal_items"],
            Value::from(1)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_module_owned_items"],
            Value::from(1)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_module_backed_items"],
            Value::from(1)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_module_connected_items"],
            Value::from(0)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_module_isolated_items"],
            Value::from(1)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_unowned_items"],
            Value::from(0)
        );
        let unexplained = json["analysis"]["traceability"]["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert_eq!(unexplained.len(), 1);
        let orphan = unexplained
            .iter()
            .find(|item| item["name"] == "orphan_impl")
            .expect("unexplained items should include orphan_impl");
        assert_eq!(orphan["public"], Value::from(false));
        assert_eq!(orphan["test_file"], Value::from(false));
        assert_eq!(orphan["module_backed_by_current_specs"], Value::from(true));
        assert_eq!(
            orphan["module_connected_to_current_specs"],
            Value::from(false)
        );
        assert_eq!(orphan["module_ids"], serde_json::json!(["DEMO"]));
        assert_eq!(
            json["analysis"]["traceability_unavailable_reason"],
            Value::Null
        );
    } else {
        assert_eq!(json["analysis"]["traceability"], Value::Null);
        assert!(
            json["analysis"]["traceability_unavailable_reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("rust-analyzer"))
        );
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNEXPLAINED.EVIDENCE
fn repo_traceability_surfaces_unexplained_evidence_in_text_and_html() {
    let root = temp_repo_dir("special-cli-repo-traceability-unexplained-evidence");
    write_traceability_module_context_fixture(&root);

    let text_output = run_special(&root, &["health", "--verbose"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");

    let html_output = run_special(&root, &["health", "--html", "--verbose"]);
    assert!(html_output.status.success());
    let html_stdout = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");

    if rust_analyzer_available() {
        assert!(text_stdout.contains("unexplained public items: 0"));
        assert!(text_stdout.contains("unexplained review-surface items: 0"));
        assert!(text_stdout.contains("unexplained internal items: 2"));
        assert!(text_stdout.contains("unexplained module-owned items: 2"));
        assert!(text_stdout.contains("unexplained module-backed items: 2"));
        assert!(text_stdout.contains("unexplained module-connected items: 1"));
        assert!(text_stdout.contains("unexplained module-isolated items: 1"));
        assert!(text_stdout.contains("unexplained unowned items: 0"));
        assert!(text_stdout.contains(
            "unexplained review-surface items meaning: these unexplained items are the main review pile: public API or root-visible entrypoints that behave like product surface."
        ));
        assert!(text_stdout.contains(
            "unexplained public items meaning: these unexplained items are public entrypoints or exported API surface."
        ));
        assert!(text_stdout.contains(
            "unexplained internal items exact: count of unexplained implementation items not marked public by the active language pack."
        ));
        assert!(text_stdout.contains(
            "unexplained module-owned items meaning: these unexplained items still belong to at least one declared module."
        ));
        assert!(text_stdout.contains(
            "unexplained module-backed items meaning: these unexplained items sit in modules that already have current-spec-traced code somewhere else."
        ));
        assert!(text_stdout.contains(
            "unexplained module-connected items exact: count of unexplained implementation items in current-spec-backed modules that share a same-module call or reference component with current-spec-traced implementation."
        ));
        assert!(text_stdout.contains(
            "unexplained module-isolated items meaning: these unexplained items are in current-spec-backed modules but still sit outside the connected traced cluster in those modules."
        ));
        assert!(text_stdout.contains(
            "unexplained item: src/lib.rs:connected_helper [internal; module-backed; connected inside module; modules DEMO]"
        ));
        assert!(text_stdout.contains(
            "unexplained item: src/lib.rs:isolated_helper [internal; module-backed; isolated inside module; modules DEMO]"
        ));

        assert!(html_stdout.contains("unexplained review-surface items"));
        assert!(html_stdout.contains("unexplained internal items"));
        assert!(html_stdout.contains("unexplained module-backed items"));
        assert!(html_stdout.contains("unexplained module-connected items"));
        assert!(html_stdout.contains("unexplained module-isolated items"));
        assert!(html_stdout.contains("these unexplained items are the main review pile: public API or root-visible entrypoints that behave like product surface."));
        assert!(
            html_stdout.contains(
                "these unexplained items are public entrypoints or exported API surface."
            )
        );
        assert!(html_stdout.contains("src/lib.rs:connected_helper [internal; module-backed; connected inside module; modules DEMO]"));
        assert!(html_stdout.contains("src/lib.rs:isolated_helper [internal; module-backed; isolated inside module; modules DEMO]"));
    } else {
        assert!(text_stdout.contains("unavailable:"));
        assert!(html_stdout.contains("unavailable"));
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT
fn repo_surfaces_typescript_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript");
    write_typescript_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        let current_names = current_items
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect::<Vec<_>>();
        assert!(current_names.contains(&"liveImpl"));
        assert!(current_names.contains(&"helper"));
        assert!(current_names.contains(&"sharedValue"));

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(unexplained.iter().any(|item| {
            item["name"] == "orphanImpl" && item["review_surface"] == true && item["public"] == true
        }));
        assert_eq!(
            traceability["unexplained_review_surface_items"],
            Value::from(1)
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.TOOL_EDGES
fn repo_surfaces_typescript_tool_backed_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-tool");
    write_typescript_tool_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/left.ts" && item["name"] == "sharedValue" })
        );
        assert!(current_items.iter().any(|item| item["name"] == "liveImpl"));
        assert!(current_items.iter().any(|item| item["name"] == "helper"));

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/right.ts" && item["name"] == "sharedValue" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.REFERENCE_EDGES
fn repo_surfaces_typescript_reference_backed_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-reference");
    write_typescript_reference_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| item["path"] == "src/live.ts" && item["name"] == "liveCallback")
        );
        assert!(current_items.iter().any(|item| item["name"] == "runLive"));
        assert!(current_items.iter().any(|item| item["name"] == "invoke"));

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/dead.ts" && item["name"] == "deadCallback" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/app.ts" && item["name"] == "orphanImpl" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.JSX_REFERENCE_EDGES
fn repo_surfaces_typescript_react_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-react");
    write_typescript_react_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/page.tsx" && item["name"] == "HomePage" })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/shared.tsx" && item["name"] == "Shell" })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/shared.tsx" && item["name"] == "PrimaryButton"
            })
        );

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/page.tsx" && item["name"] == "OrphanPage" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/shared.tsx" && item["name"] == "OrphanWidget" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.CLIENT_COMPONENT_EDGES
fn repo_surfaces_typescript_next_client_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-next");
    write_typescript_next_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "app/page.tsx" && item["name"] == "Page" })
        );
        assert!(current_items.iter().any(|item| {
            item["path"] == "components/counter-panel.tsx" && item["name"] == "CounterPanel"
        }));
        assert!(current_items.iter().any(|item| {
            item["path"] == "components/counter-panel.tsx" && item["name"] == "CounterButton"
        }));

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "app/page.tsx" && item["name"] == "OrphanPage" })
        );
        assert!(unexplained.iter().any(|item| {
            item["path"] == "components/counter-panel.tsx" && item["name"] == "OrphanWidget"
        }));
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.EVENT_CALLBACK_EDGES
fn repo_surfaces_typescript_event_callback_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-event");
    write_typescript_event_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "App" })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/Button.tsx" && item["name"] == "CounterButton"
            })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/actions.ts" && item["name"] == "handleIncrement"
            })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "updateCount" })
        );

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "orphanAction" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "OrphanPage" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/Button.tsx" && item["name"] == "OrphanWidget" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.FORWARDED_CALLBACK_EDGES
fn repo_surfaces_typescript_forwarded_callback_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-forwarded-callback");
    write_typescript_forwarded_callback_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "App" })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/Button.tsx" && item["name"] == "CounterButton"
            })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/Button.tsx" && item["name"] == "Toolbar" })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/actions.ts" && item["name"] == "handleIncrement"
            })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "updateCount" })
        );

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "orphanAction" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "OrphanPage" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/Button.tsx" && item["name"] == "OrphanWidget" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.HOOK_CALLBACK_EDGES
fn repo_surfaces_typescript_hook_callback_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-hook-callback");
    write_typescript_hook_callback_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "App" })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/Button.tsx" && item["name"] == "CounterButton"
            })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/hooks.ts" && item["name"] == "useCounterAction"
            })
        );
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/actions.ts" && item["name"] == "handleIncrement"
            })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "updateCount" })
        );

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "orphanAction" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "OrphanPage" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/Button.tsx" && item["name"] == "OrphanWidget" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/hooks.ts" && item["name"] == "orphanHook" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.EFFECT_CALLBACK_EDGES
fn repo_surfaces_typescript_effect_callback_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-effect-callback");
    write_typescript_effect_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "App" })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/effects.ts" && item["name"] == "syncCount" })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/effects.ts" && item["name"] == "flushCount" })
        );

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "OrphanPage" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/effects.ts" && item["name"] == "orphanEffect" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.TYPESCRIPT.CONTEXT_CALLBACK_EDGES
fn repo_surfaces_typescript_context_callback_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-typescript-context-callback");
    write_typescript_context_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "App" })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "CounterButton" })
        );
        assert!(current_items.iter().any(|item| {
            item["path"] == "src/context.tsx" && item["name"] == "CounterProvider"
        }));
        assert!(current_items.iter().any(|item| {
            item["path"] == "src/context.tsx" && item["name"] == "useCounterContext"
        }));
        assert!(
            current_items.iter().any(|item| {
                item["path"] == "src/actions.ts" && item["name"] == "handleIncrement"
            })
        );
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "updateCount" })
        );

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/App.tsx" && item["name"] == "OrphanPage" })
        );
        assert!(
            unexplained.iter().any(|item| {
                item["path"] == "src/context.tsx" && item["name"] == "orphanContext"
            })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "src/actions.ts" && item["name"] == "orphanAction" })
        );
    } else {
        assert_typescript_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO
fn repo_surfaces_go_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-go");
    write_go_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        let current_names = current_items
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect::<Vec<_>>();
        assert!(current_names.contains(&"LiveImpl"));
        assert!(current_names.contains(&"helper"));
        assert!(current_names.contains(&"SharedValue"));

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(unexplained.iter().any(|item| {
            item["name"] == "OrphanImpl" && item["review_surface"] == true && item["public"] == true
        }));
        assert_eq!(
            traceability["unexplained_review_surface_items"],
            Value::from(1)
        );
    } else {
        assert_go_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO.TOOL_EDGES
fn repo_surfaces_go_tool_backed_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-go-tool");
    write_go_tool_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| { item["path"] == "left/shared.go" && item["name"] == "SharedValue" })
        );
        assert!(current_items.iter().any(|item| item["name"] == "LiveImpl"));
        assert!(current_items.iter().any(|item| item["name"] == "helper"));

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "right/shared.go" && item["name"] == "SharedValue" })
        );
    } else {
        assert_go_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.GO.REFERENCE_EDGES
fn repo_surfaces_go_reference_backed_traceability() {
    let root = temp_repo_dir("special-cli-repo-traceability-go-reference");
    write_go_reference_traceability_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if json["analysis"]["traceability_unavailable_reason"].is_null() {
        let traceability = &json["analysis"]["traceability"];
        let current_items = traceability["current_spec_items"]
            .as_array()
            .expect("current spec items should be an array");
        assert!(
            current_items
                .iter()
                .any(|item| item["path"] == "live/live.go" && item["name"] == "LiveCallback")
        );
        assert!(current_items.iter().any(|item| item["name"] == "LiveImpl"));
        assert!(current_items.iter().any(|item| item["name"] == "invoke"));

        let unexplained = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "dead/dead.go" && item["name"] == "DeadCallback" })
        );
        assert!(
            unexplained
                .iter()
                .any(|item| { item["path"] == "app/main.go" && item["name"] == "OrphanImpl" })
        );
    } else {
        assert_go_traceability_unavailable(&json);
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.UNEXPLAINED.EVIDENCE
fn repo_review_surface_excludes_public_test_helpers() {
    let root = temp_repo_dir("special-cli-repo-traceability-review-surface");
    write_traceability_review_surface_fixture(&root);

    let output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");

    if rust_analyzer_available() {
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_public_items"],
            Value::from(2)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_test_file_items"],
            Value::from(1)
        );
        assert_eq!(
            json["analysis"]["traceability"]["unexplained_review_surface_items"],
            Value::from(1)
        );

        let unexplained = json["analysis"]["traceability"]["unexplained_items"]
            .as_array()
            .expect("unexplained items should be an array");
        let public_names = unexplained
            .iter()
            .filter(|item| item["public"] == true)
            .map(|item| {
                (
                    item["name"].as_str().expect("name should be present"),
                    item["review_surface"] == true,
                    item["test_file"] == true,
                )
            })
            .collect::<Vec<_>>();
        assert!(public_names.contains(&("public_orphan", true, false)));
        assert!(public_names.contains(&("test_public_orphan", false, true)));
        let internal_names = unexplained
            .iter()
            .filter(|item| item["public"] == false)
            .map(|item| {
                (
                    item["name"].as_str().expect("name should be present"),
                    item["review_surface"] == true,
                )
            })
            .collect::<Vec<_>>();
        assert!(internal_names.contains(&("internal_orphan", false)));
    } else {
        assert_eq!(json["analysis"]["traceability"], Value::Null);
        assert_eq!(
            json["analysis"]["traceability_unavailable_reason"],
            Value::from(
                "Rust backward trace is unavailable because `rust-analyzer` is not installed",
            )
        );
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
