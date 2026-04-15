/**
@module SPECIAL.TESTS.CLI_MODULES
`special modules` command tests in `tests/cli_modules.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES
/**
@spec SPECIAL.MODULE_COMMAND
special modules materializes the current live architecture module view.

@spec SPECIAL.MODULE_COMMAND.AREA_NODES
special modules materializes both concrete `@module` nodes and structural `@area` nodes in the architecture tree.

@spec SPECIAL.MODULE_COMMAND.KIND_LABELS
special modules identifies architecture node kinds in output so `@area` nodes remain distinguishable from `@module` nodes.

@spec SPECIAL.MODULE_COMMAND.SINGULAR_ALIAS
special module remains accepted as an alias for special modules.

@spec SPECIAL.MODULE_COMMAND.LIVE_ONLY
special modules excludes planned modules by default.

@spec SPECIAL.MODULE_COMMAND.ALL
special modules --all includes planned modules.

@spec SPECIAL.MODULE_COMMAND.ID_SCOPE
special modules MODULE.ID scopes the materialized view to the matching module and its descendants.

@spec SPECIAL.MODULE_COMMAND.UNSUPPORTED
special modules --unsupported shows live `@module` nodes with zero direct `@implements` attachments.

@spec SPECIAL.MODULE_COMMAND.FAILS_ON_ERRORS
special modules exits with an error status when architecture diagnostics include errors, even if it still prints diagnostics and best-effort rendered output.

@spec SPECIAL.MODULE_COMMAND.JSON
special modules --json emits the materialized module view as JSON.

@spec SPECIAL.MODULE_COMMAND.HTML
special modules --html emits the materialized module view as HTML.

@spec SPECIAL.MODULE_COMMAND.VERBOSE
special modules --verbose shows attached `@implements` locations and bodies for review.

@spec SPECIAL.MODULE_COMMAND.VERBOSE.JSON
special modules --json --verbose includes attached `@implements` bodies in JSON output.

@spec SPECIAL.MODULE_COMMAND.VERBOSE.HTML
special modules --html --verbose includes attached `@implements` bodies in collapsed detail blocks.

@spec SPECIAL.MODULE_COMMAND.PLANNED_RELEASE_METADATA
when a planned module declares release metadata, special modules surfaces that release string in text, json, and html output.

@group SPECIAL.MODULE_COMMAND.METRICS_GROUP
special modules can materialize slower implementation analysis evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS
special modules --metrics surfaces architecture analysis evidence, including ownership coverage and per-module implementation summaries from built-in language analyzers.

@group SPECIAL.MODULE_COMMAND.METRICS.COMPLEXITY
special modules can explain and summarize complexity evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.QUALITY
special modules --metrics surfaces language-agnostic quality evidence categories when a built-in analyzer can extract them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.COMPLEXITY.EXPLANATIONS
special modules --metrics explains complexity evidence in plain language and in precise structural terms from a shared analysis registry.

@spec SPECIAL.MODULE_COMMAND.METRICS.QUALITY.EXPLANATIONS
special modules --metrics explains quality evidence in plain language and in precise structural terms from a shared analysis registry.

@group SPECIAL.MODULE_COMMAND.METRICS.ITEM_SIGNALS
special modules can surface item-level evidence within owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.ITEM_SIGNALS.EXPLANATIONS
special modules --metrics explains item-level evidence categories in plain language and in precise structural terms from a shared analysis registry.

@spec SPECIAL.MODULE_COMMAND.METRICS.COUPLING
special modules --metrics surfaces module-to-module coupling evidence when built-in analyzers can resolve owned dependency targets to concrete modules.

@spec SPECIAL.MODULE_COMMAND.METRICS.COUPLING.EXPLANATIONS
special modules explains coupling metrics in plain language and in precise structural terms from a shared analysis registry.

@group SPECIAL.MODULE_COMMAND.METRICS.RUST
special modules can surface Rust-specific implementation evidence for owned Rust code through the built-in Rust analyzer.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.SURFACE
special modules --metrics surfaces Rust public and internal item counts for owned Rust implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY
special modules --metrics surfaces Rust function complexity summaries for owned implementation, including analyzed function count plus total and maximum cyclomatic complexity.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY.COGNITIVE
special modules --metrics surfaces Rust cognitive complexity summaries for owned implementation, including total and maximum cognitive complexity.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.QUALITY
special modules --metrics surfaces Rust quality evidence, including public API parameter shape, stringly typed boundaries, and recoverability signals.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.DEPENDENCIES
special modules --metrics surfaces Rust `use`-path dependency evidence from owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.COUPLING
special modules --metrics derives generic module coupling evidence from Rust `use` targets when those targets resolve to uniquely owned files.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS
special modules --metrics surfaces per-item Rust evidence for owned implementation, including internally connected, outbound-heavy, and isolated items.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.COMPLEXITY
special modules --metrics surfaces highest-complexity Rust items within owned implementation so unusual local hotspots are visible inside a claimed module boundary.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.QUALITY
special modules --metrics surfaces parameter-heavy, stringly boundary, and panic-heavy Rust items within owned implementation so unusual local craftsmanship signals are visible inside a claimed module boundary.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON
special modules --json --metrics includes structured architecture analysis summaries.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.QUALITY
special modules --json --metrics includes structured quality evidence summaries when available.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.COUPLING
special modules --json --metrics includes structured module coupling summaries when available.

@group SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST
special modules --json --metrics can include Rust-specific structured analysis evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY
special modules --json --metrics includes structured Rust function complexity summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY.COGNITIVE
special modules --json --metrics includes structured Rust cognitive complexity summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.QUALITY
special modules --json --metrics includes structured Rust quality evidence summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.DEPENDENCIES
special modules --json --metrics includes structured Rust dependency targets for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS
special modules --json --metrics includes structured per-item Rust evidence for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.COMPLEXITY
special modules --json --metrics includes structured highest-complexity Rust item evidence for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.QUALITY
special modules --json --metrics includes structured parameter-heavy, stringly boundary, and panic-heavy Rust item evidence for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.VERBOSE
special modules --metrics --verbose includes uncovered or weakly covered file paths plus per-module implementation coverage details.

@group SPECIAL.MODULE_PARSE
special parses architecture module declarations and implementation attachments.

@spec SPECIAL.MODULE_PARSE.MARKDOWN_DECLARATIONS
special parses `@area` and `@module` declarations from markdown heading annotations under the project root.

@spec SPECIAL.MODULE_PARSE.MODULE_DECLARATIONS
special parses `@module MODULE.ID` declarations from supported source comments.

@spec SPECIAL.MODULE_PARSE.AREA_DECLARATIONS
special parses `@area AREA.ID` declarations as structural architecture nodes from supported declaration surfaces.

@group SPECIAL.MODULE_PARSE.PLANNED
special records `@planned` on declared modules from supported module declaration surfaces.

@spec SPECIAL.MODULE_PARSE.PLANNED.MODULE_ONLY
`@planned` may only apply to `@module`, not `@area`.

@spec SPECIAL.MODULE_PARSE.PLANNED.EXACT_STANDALONE_MARKER
special only accepts an exact standalone `@planned` marker on the next line after a module declaration.

@group SPECIAL.MODULE_PARSE.IMPLEMENTS
special parses explicit item-scoped and file-scoped module attachments from supported source files.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.MODULE_ONLY
`@implements` and `@fileimplements` may only reference ids declared as `@module`, not `@area`.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE
`@fileimplements` records a file-scoped module attachment without requiring an owned item body.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE
`@implements` attaches the next supported owned item body to the module attachment.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.EXACT_DIRECTIVE_SHAPE
`@implements` and `@fileimplements` accept exactly one module id and reject trailing content.

@spec SPECIAL.MODULE_PARSE.FOREIGN_TAG_BOUNDARIES
special treats foreign line-start `@...` and `\\...` tags as boundaries for attached module text without materializing them as part of the module description.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_FILE_SCOPE
when a file declares more than one `@fileimplements`, special lint reports an error.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_ITEM_SCOPE
when more than one `@implements` attaches to the same owned code item, special lint reports an error.

@spec SPECIAL.MODULE_PARSE.LIVE_MODULES_REQUIRE_IMPLEMENTATION
live `@module` nodes require a direct `@implements` or `@fileimplements` attachment unless they are planned.

@spec SPECIAL.MODULE_PARSE.AREAS_ARE_STRUCTURAL_ONLY
`@area` nodes are structural architecture nodes and do not require direct `@implements` attachments.
*/
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{
    find_node_by_id, rendered_spec_node_ids, run_special, temp_repo_dir, top_level_help_commands,
    write_area_implements_fixture, write_area_modules_fixture,
    write_cognitive_complexity_module_analysis_fixture, write_complexity_module_analysis_fixture,
    write_coupling_module_analysis_fixture, write_dependency_module_analysis_fixture,
    write_duplicate_file_scoped_implements_fixture, write_duplicate_item_scoped_implements_fixture,
    write_implements_with_trailing_content_fixture,
    write_item_scoped_item_signals_module_analysis_fixture,
    write_item_scoped_module_analysis_fixture, write_item_signals_module_analysis_fixture,
    write_missing_intermediate_modules_fixture, write_mixed_purpose_source_local_module_fixture,
    write_module_analysis_fixture, write_modules_fixture, write_planned_area_fixture,
    write_planned_area_invalid_suffix_fixture, write_quality_module_analysis_fixture,
    write_source_local_module_analysis_fixture, write_source_local_modules_fixture,
    write_unimplemented_module_fixture, write_unknown_implements_fixture,
    write_unsupported_module_fixture,
};

#[test]
// @verifies SPECIAL.HELP.MODULES_COMMAND_PLURAL_PRIMARY
fn top_level_help_presents_modules_as_the_primary_command_name() {
    let root = temp_repo_dir("special-cli-modules-help-primary");

    let output = run_special(&root, &["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        top_level_help_commands(&stdout)
            .iter()
            .any(|(name, summary)| name == "modules"
                && summary == "Materialize and inspect architecture modules")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND
fn modules_materializes_live_module_tree() {
    let root = temp_repo_dir("special-cli-modules");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules"]);
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
// @verifies SPECIAL.MODULE_COMMAND.LIVE_ONLY
fn modules_default_view_excludes_planned_nodes() {
    let root = temp_repo_dir("special-cli-modules-live-only");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(!node_ids.contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.SINGULAR_ALIAS
fn singular_module_alias_still_materializes_the_same_view() {
    let root = temp_repo_dir("special-cli-module-singular-alias");
    write_modules_fixture(&root);

    let plural_output = run_special(&root, &["modules"]);
    assert!(plural_output.status.success());
    let singular_output = run_special(&root, &["module"]);
    assert!(singular_output.status.success());

    assert_eq!(plural_output.stdout, singular_output.stdout);
    assert_eq!(plural_output.stderr, singular_output.stderr);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.ALL
fn modules_all_includes_planned_modules() {
    let root = temp_repo_dir("special-cli-modules-all");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--all"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(rendered_spec_node_ids(&stdout).contains(&"DEMO.PLANNED".to_string()));
    assert!(stdout.contains("[planned: 0.4.0]"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.PLANNED_RELEASE_METADATA
fn modules_surface_planned_release_metadata_across_output_modes() {
    let root = temp_repo_dir("special-cli-modules-planned-release");
    write_modules_fixture(&root);

    let text_output = run_special(&root, &["modules", "--all"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text_stdout.contains("[planned: 0.4.0]"));

    let json_output = run_special(&root, &["modules", "--all", "--json"]);
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
        .expect("planned module should be present");
    assert_eq!(
        planned["planned_release"],
        Value::String("0.4.0".to_string())
    );

    let html_output = run_special(&root, &["modules", "--all", "--html"]);
    assert!(html_output.status.success());
    let html_stdout = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html_stdout.contains("<span class=\"badge badge-planned\">planned: 0.4.0</span>"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS
fn modules_coverage_surfaces_architecture_wide_coverage_summary() {
    let root = temp_repo_dir("special-cli-modules-coverage");
    write_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("coverage"));
    assert!(stdout.contains("analyzed files: 2"));
    assert!(stdout.contains("covered files: 1"));
    assert!(stdout.contains("uncovered files: 1"));
    assert!(stdout.contains("weak files: 1"));
    assert!(stdout.contains("covered files: 1"));
    assert!(stdout.contains("file-scoped implements: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.VERBOSE
fn modules_coverage_verbose_includes_uncovered_and_weak_paths() {
    let root = temp_repo_dir("special-cli-modules-coverage-verbose");
    write_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("uncovered path: hidden.rs"));
    assert!(stdout.contains("weak path: main.rs"));
    assert!(stdout.contains("covered file: main.rs"));
    assert!(stdout.contains("weak file: main.rs"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.SURFACE
fn modules_metrics_surface_owned_lines_and_rust_item_counts() {
    let root = temp_repo_dir("special-cli-modules-metrics");
    write_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("owned lines:"));
    assert!(stdout.contains("public items: 1"));
    assert!(stdout.contains("internal items: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON
fn modules_metrics_json_includes_structured_analysis() {
    let root = temp_repo_dir("special-cli-modules-metrics-json");
    write_item_scoped_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(
        json["analysis"]["coverage"]["analyzed_files"],
        Value::from(1)
    );
    assert_eq!(json["analysis"]["coverage"]["weak_files"], Value::from(1));

    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["coverage"]["covered_files"],
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
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.DEPENDENCIES
fn modules_metrics_surface_rust_use_path_dependency_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-dependencies");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("dependency refs: 2"));
    assert!(stdout.contains("dependency targets: 2"));
    assert!(stdout.contains("dependency target: crate::shared::util::helper (1)"));
    assert!(stdout.contains("dependency target: serde_json::Value (1)"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY
fn modules_metrics_surface_rust_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-complexity");
    write_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("complexity functions: 2"));
    assert!(stdout.contains("cyclomatic total: 8"));
    assert!(stdout.contains("cyclomatic max: 7"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY
fn modules_metrics_json_includes_structured_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-complexity-json");
    write_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["complexity"]["function_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["total_cyclomatic"],
        Value::from(8)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cyclomatic"],
        Value::from(7)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.DEPENDENCIES
fn modules_metrics_json_includes_structured_dependency_targets() {
    let root = temp_repo_dir("special-cli-modules-metrics-dependencies-json");
    write_dependency_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
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
    let targets = demo["analysis"]["dependencies"]["targets"]
        .as_array()
        .expect("dependency targets should be an array");
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("crate::shared::util::helper".to_string())
            && target["count"] == 1
    }));
    assert!(targets.iter().any(|target| {
        target["path"] == Value::String("serde_json::Value".to_string()) && target["count"] == 1
    }));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_module_coupling_output(stdout: &str) {
    assert!(stdout.contains("fan out: 1"));
    assert!(stdout.contains("efferent coupling: 1"));
    assert!(stdout.contains("instability: 1.00"));
    assert!(stdout.contains("fan in: 1"));
    assert!(stdout.contains("afferent coupling: 1"));
    assert!(stdout.contains("fan out meaning: this module reaches into other owned modules."));
    assert!(stdout.contains(
        "fan out exact: distinct outbound concrete-module dependencies resolved from owned code."
    ));
    assert!(stdout.contains(
        "instability exact: efferent coupling / (afferent coupling + efferent coupling)."
    ));
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.COUPLING
fn modules_metrics_surface_module_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-coupling");
    write_coupling_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_module_coupling_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.COUPLING.EXPLANATIONS
fn modules_metrics_surface_module_coupling_explanations() {
    let root = temp_repo_dir("special-cli-modules-metrics-coupling-explanations");
    write_coupling_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_module_coupling_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.COUPLING
fn modules_metrics_surface_rust_derived_module_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-coupling");
    write_coupling_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_module_coupling_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.COUPLING
fn modules_metrics_json_includes_structured_module_coupling() {
    let root = temp_repo_dir("special-cli-modules-metrics-coupling-json");
    write_coupling_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let api = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.API"))
        })
        .expect("api module should be present");
    assert_eq!(api["analysis"]["coupling"]["fan_out"], Value::from(1));
    assert_eq!(
        api["analysis"]["coupling"]["efferent_coupling"],
        Value::from(1)
    );
    assert_eq!(api["analysis"]["coupling"]["instability"], Value::from(1.0));

    let shared = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.SHARED"))
        })
        .expect("shared module should be present");
    assert_eq!(shared["analysis"]["coupling"]["fan_in"], Value::from(1));
    assert_eq!(
        shared["analysis"]["coupling"]["afferent_coupling"],
        Value::from(1)
    );
    assert_eq!(
        shared["analysis"]["coupling"]["instability"],
        Value::from(0.0)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY.COGNITIVE
fn modules_metrics_surface_rust_cognitive_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-cognitive");
    write_cognitive_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("cognitive total: 3"));
    assert!(stdout.contains("cognitive max: 3"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY.COGNITIVE
fn modules_metrics_json_includes_structured_cognitive_complexity_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-cognitive-json");
    write_cognitive_complexity_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["complexity"]["total_cognitive"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["complexity"]["max_cognitive"],
        Value::from(3)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_quality_output(stdout: &str) {
    assert!(stdout.contains("quality public functions: 1"));
    assert!(stdout.contains("quality parameters: 3"));
    assert!(stdout.contains("quality bool params: 1"));
    assert!(stdout.contains("quality raw string params: 2"));
    assert!(stdout.contains("quality panic sites: 1"));
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.QUALITY
fn modules_metrics_surface_quality_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-quality");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_quality_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.QUALITY
fn modules_metrics_surface_rust_quality_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-quality");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_quality_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.QUALITY
fn modules_metrics_json_includes_structured_quality_summary() {
    let root = temp_repo_dir("special-cli-modules-metrics-quality-json");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
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

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.QUALITY
fn modules_metrics_json_includes_structured_rust_quality_evidence() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-quality-json");
    write_quality_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
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

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.COMPLEXITY.EXPLANATIONS
fn modules_metrics_explains_complexity_evidence_in_text_and_html() {
    let root = temp_repo_dir("special-cli-modules-metrics-complexity-explanations");
    write_item_signals_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["modules", "--metrics"]);
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text.contains("cyclomatic total meaning:"));
    assert!(text.contains("cyclomatic total exact:"));
    assert!(text.contains("cognitive max meaning:"));

    let html_output = run_special(&root, &["modules", "--metrics", "--html"]);
    assert!(html_output.status.success());
    let html = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html.contains("cyclomatic total meaning:"));
    assert!(html.contains("cyclomatic total exact:"));
    assert!(html.contains("cognitive max meaning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.QUALITY.EXPLANATIONS
fn modules_metrics_explains_quality_evidence_in_text_and_html() {
    let root = temp_repo_dir("special-cli-modules-metrics-quality-explanations");
    write_item_signals_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["modules", "--metrics"]);
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text.contains("quality parameters meaning:"));
    assert!(text.contains("quality raw string params exact:"));
    assert!(text.contains("quality panic sites meaning:"));

    let html_output = run_special(&root, &["modules", "--metrics", "--html"]);
    assert!(html_output.status.success());
    let html = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html.contains("quality parameters meaning:"));
    assert!(html.contains("quality raw string params exact:"));
    assert!(html.contains("quality panic sites meaning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.ITEM_SIGNALS.EXPLANATIONS
fn modules_metrics_explains_item_signal_evidence_in_text_and_html() {
    let root = temp_repo_dir("special-cli-modules-metrics-item-signal-explanations");
    write_item_signals_module_analysis_fixture(&root);

    let text_output = run_special(&root, &["modules", "--metrics"]);
    assert!(text_output.status.success());
    let text = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text.contains("connected item meaning:"));
    assert!(text.contains("highest complexity item exact:"));
    assert!(text.contains("stringly boundary item meaning:"));

    let html_output = run_special(&root, &["modules", "--metrics", "--html"]);
    assert!(html_output.status.success());
    let html = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html.contains("connected item meaning:"));
    assert!(html.contains("highest complexity item exact:"));
    assert!(html.contains("stringly boundary item meaning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_item_signals_output(stdout: &str) {
    assert!(stdout.contains("item signals analyzed: 6"));
    assert!(stdout.contains("connected item: core_helper"));
    assert!(stdout.contains("outbound-heavy item: outbound_heavy"));
    assert!(stdout.contains("isolated item: isolated_external"));
    assert!(stdout.contains("highest complexity item: complex_hotspot"));
    assert!(stdout.contains("parameter-heavy item: outbound_heavy"));
    assert!(stdout.contains("stringly boundary item: outbound_heavy"));
    assert!(stdout.contains("panic-heavy item: outbound_heavy"));
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS
fn modules_metrics_surface_rust_item_signals() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_item_signals_output(&stdout);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.COMPLEXITY
fn modules_metrics_surface_rust_item_complexity_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-complexity");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("highest complexity item: complex_hotspot"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.QUALITY
fn modules_metrics_surface_rust_item_quality_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-quality");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("parameter-heavy item: outbound_heavy"));
    assert!(stdout.contains("stringly boundary item: outbound_heavy"));
    assert!(stdout.contains("panic-heavy item: outbound_heavy"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS
fn modules_metrics_surface_rust_item_signals_for_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals-item-scoped");
    write_item_scoped_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("item signals analyzed: 3"));
    assert!(stdout.contains("connected item: connected"));
    assert!(stdout.contains("isolated item: isolated_external"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS
fn modules_metrics_json_includes_structured_rust_item_signals() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals-json");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["analyzed_items"],
        Value::from(6)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["isolated_items"][0]["name"],
        Value::from("isolated_external")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["outbound_heavy_items"][0]["name"],
        Value::from("outbound_heavy")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["connected_items"][0]["name"],
        Value::from("helper_leaf")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.COMPLEXITY
fn modules_metrics_json_includes_structured_rust_item_complexity_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-complexity-json");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["highest_complexity_items"][0]["name"],
        Value::from("complex_hotspot")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["highest_complexity_items"][0]["cognitive"],
        demo["analysis"]["complexity"]["max_cognitive"]
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.QUALITY
fn modules_metrics_json_includes_structured_rust_item_quality_drilldown() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-quality-json");
    write_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["parameter_heavy_items"][0]["name"],
        Value::from("outbound_heavy")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["parameter_heavy_items"][0]["parameter_count"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["stringly_boundary_items"][0]["raw_string_parameter_count"],
        Value::from(2)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["panic_heavy_items"][0]["panic_site_count"],
        Value::from(1)
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS
fn modules_metrics_json_includes_structured_rust_item_signals_for_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-metrics-rust-item-signals-item-scoped-json");
    write_item_scoped_item_signals_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("demo module should be present");
    assert_eq!(
        demo["analysis"]["item_signals"]["analyzed_items"],
        Value::from(3)
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["connected_items"][0]["name"],
        Value::from("connected")
    );
    assert_eq!(
        demo["analysis"]["item_signals"]["isolated_items"][0]["name"],
        Value::from("isolated_external")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_metrics_treat_header_implements_after_module_docs_as_file_scoped() {
    let root = temp_repo_dir("special-cli-modules-metrics-source-local-header");
    write_source_local_module_analysis_fixture(&root);

    let output = run_special(&root, &["modules", "--metrics", "--json"]);
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

#[test]
// @verifies SPECIAL.MODULE_COMMAND.ID_SCOPE
fn modules_scope_to_matching_id_and_descendants() {
    let root = temp_repo_dir("special-cli-modules-scope");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "DEMO"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(!node_ids.contains(&"DEMO.PLANNED".to_string()));

    let output = run_special(&root, &["modules", "DEMO.LIVE"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert_eq!(node_ids, vec!["DEMO.LIVE".to_string()]);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.UNSUPPORTED
fn modules_unsupported_filters_live_modules_without_implements() {
    let root = temp_repo_dir("special-cli-modules-unsupported");
    write_unsupported_module_fixture(&root);

    let output = run_special(&root, &["modules", "--unsupported"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.UNSUPPORTED".to_string()));
    assert!(stderr.contains(
        "live module `DEMO.UNSUPPORTED` has no direct @implements or @fileimplements attachment"
    ));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.FAILS_ON_ERRORS
fn modules_fails_when_module_diagnostics_are_present() {
    let root = temp_repo_dir("special-cli-modules-fails-on-errors");
    write_unimplemented_module_fixture(&root);

    let output = run_special(&root, &["modules"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(rendered_spec_node_ids(&stdout).contains(&"DEMO".to_string()));
    assert!(
        stderr
            .contains("live module `DEMO` has no direct @implements or @fileimplements attachment")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.JSON
fn modules_json_emits_json_output() {
    let root = temp_repo_dir("special-cli-modules-json");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO module should be present");
    assert_eq!(demo["implements"].as_array().map(Vec::len), Some(1));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.HTML
fn modules_html_emits_html_output() {
    let root = temp_repo_dir("special-cli-modules-html");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<title>special modules</title>"));
    assert!(stdout.contains("<span class=\"node-id\">DEMO</span>"));
    assert!(stdout.contains("implements: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.VERBOSE
fn modules_verbose_includes_implementation_bodies() {
    let root = temp_repo_dir("special-cli-modules-verbose");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@implements"));
    assert!(stdout.contains("@fileimplements"));
    assert!(stdout.contains("fn implements_demo_live() {}"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.VERBOSE.JSON
fn modules_verbose_json_includes_implementation_bodies() {
    let root = temp_repo_dir("special-cli-modules-json-verbose");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo_live = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.LIVE"))
        })
        .expect("DEMO.LIVE module should be present");
    let implementation = demo_live["implements"]
        .as_array()
        .and_then(|items| items.first())
        .expect("implementation should be present");
    assert_eq!(
        implementation["body"],
        Value::String("fn implements_demo_live() {}".to_string())
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.VERBOSE.HTML
fn modules_verbose_html_includes_implementation_bodies() {
    let root = temp_repo_dir("special-cli-modules-html-verbose");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--html", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<details><summary>@implements "));
    assert!(stdout.contains("<details><summary>@fileimplements "));
    assert!(stdout.contains("implements_demo_live"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.MARKDOWN_DECLARATIONS
fn modules_read_markdown_architecture_declarations_when_present() {
    let root = temp_repo_dir("special-cli-modules-architecture-doc");
    write_area_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["id"], Value::String("DEMO".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.MODULE_DECLARATIONS
fn modules_parse_source_local_module_declarations() {
    let root = temp_repo_dir("special-cli-modules-source-local-decls");
    write_source_local_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO module should be present");
    let local = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.LOCAL"))
        })
        .expect("DEMO.LOCAL module should be present");
    assert_eq!(demo["kind"], Value::String("module".to_string()));
    assert_eq!(local["kind"], Value::String("module".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.FOREIGN_TAG_BOUNDARIES
fn modules_stop_text_before_foreign_comment_tags() {
    let root = temp_repo_dir("special-cli-modules-foreign-tag-boundary");
    write_mixed_purpose_source_local_module_fixture(&root);

    let output = run_special(&root, &["modules", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let module = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO module should be present");
    assert_eq!(
        module["text"],
        Value::String("Renders the demo export surface.".to_string())
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.AREA_DECLARATIONS
fn modules_parse_area_declarations_from_architecture_doc() {
    let root = temp_repo_dir("special-cli-modules-area-declarations");
    write_area_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["kind"], Value::String("area".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE
fn modules_record_top_of_file_implements_without_owned_item_body() {
    let root = temp_repo_dir("special-cli-modules-file-scope");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO module should be present");
    let implementation = demo["implements"]
        .as_array()
        .and_then(|items| items.first())
        .expect("file-scoped implementation should be present");
    assert!(implementation["body"].is_null());
    assert!(implementation["body_location"].is_null());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE
fn modules_attach_owned_item_bodies_for_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-item-scope");
    write_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--json", "--verbose"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo_live = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.LIVE"))
        })
        .expect("DEMO.LIVE module should be present");
    let implementation = demo_live["implements"]
        .as_array()
        .and_then(|items| items.first())
        .expect("item-scoped implementation should be present");
    assert_eq!(
        implementation["body"],
        Value::String("fn implements_demo_live() {}".to_string())
    );
    assert!(implementation["body_location"].is_object());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.UNKNOWN_IMPLEMENTS_REFS
fn lint_reports_unknown_implements_references() {
    let root = temp_repo_dir("special-cli-modules-lint-unknown");
    write_unknown_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains(
            "unknown module id `DEMO.MISSING` referenced by @implements or @fileimplements"
        )
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.EXACT_DIRECTIVE_SHAPE
fn lint_rejects_trailing_content_after_implements_module_id() {
    let root = temp_repo_dir("special-cli-modules-lint-implements-trailing");
    write_implements_with_trailing_content_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("unexpected trailing content after @implements module id"));
    assert!(stdout.contains("unexpected trailing content after @fileimplements module id"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.LINT_COMMAND.INTERMEDIATE_MODULES
fn lint_reports_missing_intermediate_module_ids() {
    let root = temp_repo_dir("special-cli-modules-lint-intermediate");
    write_missing_intermediate_modules_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("missing intermediate module `DEMO`"));
    assert!(stdout.contains("missing intermediate module `DEMO.CHILD`"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_FILE_SCOPE
fn lint_reports_duplicate_file_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-lint-duplicate-file-scope");
    write_duplicate_file_scoped_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("duplicate @fileimplements"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_ITEM_SCOPE
fn lint_reports_duplicate_item_scoped_implements() {
    let root = temp_repo_dir("special-cli-modules-lint-duplicate-item-scope");
    write_duplicate_item_scoped_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("duplicate @implements for attached item"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.AREA_NODES
fn modules_materialize_area_nodes() {
    let root = temp_repo_dir("special-cli-modules-area-nodes");
    write_area_modules_fixture(&root);

    let text_output = run_special(&root, &["modules"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&text_stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));

    let json_output = run_special(&root, &["modules", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["kind"], Value::String("area".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.KIND_LABELS
fn modules_label_area_nodes_by_kind_in_output() {
    let root = temp_repo_dir("special-cli-modules-kind-labels");
    write_area_modules_fixture(&root);

    let text_output = run_special(&root, &["modules"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    assert!(text_stdout.contains("DEMO [area]"));

    let json_output = run_special(&root, &["modules", "--json"]);
    assert!(json_output.status.success());
    let json: Value =
        serde_json::from_slice(&json_output.stdout).expect("json output should be valid json");
    let area = json["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find_map(|node| find_node_by_id(node, "DEMO")))
        .expect("DEMO area should be present");
    assert_eq!(area["kind"], Value::String("area".to_string()));

    let html_output = run_special(&root, &["modules", "--html"]);
    assert!(html_output.status.success());
    let html_stdout = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html_stdout.contains(">area</span>"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.IMPLEMENTS.MODULE_ONLY
fn lint_rejects_implements_on_area_ids() {
    let root = temp_repo_dir("special-cli-modules-area-implements");
    write_area_implements_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@implements and @fileimplements may only reference @module ids"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.PLANNED.MODULE_ONLY
fn lint_rejects_planned_areas() {
    let root = temp_repo_dir("special-cli-modules-planned-area");
    write_planned_area_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@planned may only apply to @module, not @area"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.PLANNED.EXACT_STANDALONE_MARKER
fn lint_rejects_standalone_planned_suffixes_in_module_declarations() {
    let root = temp_repo_dir("special-cli-modules-planned-invalid-suffix");
    write_planned_area_invalid_suffix_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("use an exact standalone `@planned` marker with no trailing suffix"));
    assert!(!stdout.contains("planned areas"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.LIVE_MODULES_REQUIRE_IMPLEMENTATION
fn lint_rejects_live_modules_without_direct_implements() {
    let root = temp_repo_dir("special-cli-modules-unimplemented");
    write_unimplemented_module_fixture(&root);

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout
            .contains("live module `DEMO` has no direct @implements or @fileimplements attachment")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.AREAS_ARE_STRUCTURAL_ONLY
fn modules_unsupported_filter_does_not_treat_areas_as_missing_implementation() {
    let root = temp_repo_dir("special-cli-modules-areas-structural-only");
    write_area_modules_fixture(&root);

    let output = run_special(&root, &["modules", "--unsupported"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(!node_ids.contains(&"DEMO".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
