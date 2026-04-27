#[allow(dead_code)]
#[path = "../src/language_packs/go/test_fixtures.rs"]
mod go_test_fixtures;
#[path = "cli_modules/metrics.rs"]
mod metrics;
#[path = "cli_modules/parse.rs"]
mod parse;
/**
@module SPECIAL.TESTS.CLI_MODULES
`special arch` command tests in `tests/cli_modules.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_MODULES
/**
@spec SPECIAL.MODULE_COMMAND
special arch materializes the declared architecture view.

@spec SPECIAL.MODULE_COMMAND.AREA_NODES
special arch materializes both concrete `@module` nodes and structural `@area` nodes in the architecture tree.

@spec SPECIAL.MODULE_COMMAND.KIND_LABELS
special arch identifies architecture node kinds in output so `@area` nodes remain distinguishable from `@module` nodes.

@spec SPECIAL.MODULE_COMMAND.DEFAULT_ALL
special arch includes both current and planned modules by default.

@spec SPECIAL.MODULE_COMMAND.CURRENT_ONLY
special arch --current excludes planned modules.

@spec SPECIAL.MODULE_COMMAND.PLANNED_ONLY
special arch --planned shows only planned modules.

@spec SPECIAL.MODULE_COMMAND.ID_SCOPE
special arch MODULE.ID scopes the materialized view to the matching module and its descendants.

@spec SPECIAL.MODULE_COMMAND.UNIMPLEMENTED
special arch --unimplemented shows current `@module` nodes with zero direct `@implements` attachments.

@spec SPECIAL.MODULE_COMMAND.FAILS_ON_ERRORS
special arch exits with an error status when architecture diagnostics include errors, even if it still prints diagnostics and best-effort rendered output.

@spec SPECIAL.MODULE_COMMAND.JSON
special arch --json emits the materialized architecture view as JSON.

@spec SPECIAL.MODULE_COMMAND.HTML
special arch --html emits the materialized architecture view as HTML.

@spec SPECIAL.MODULE_COMMAND.VERBOSE
special arch --verbose shows attached `@implements` locations and bodies for review.

@spec SPECIAL.MODULE_COMMAND.VERBOSE.JSON
special arch --json --verbose includes attached `@implements` bodies in JSON output.

@spec SPECIAL.MODULE_COMMAND.VERBOSE.HTML
special arch --html --verbose includes attached `@implements` bodies in collapsed detail blocks.

@spec SPECIAL.MODULE_COMMAND.PLANNED_RELEASE_METADATA
when a planned module declares release metadata, special arch surfaces that release string in text, json, and html output.

@group SPECIAL.MODULE_COMMAND.METRICS_GROUP
special arch can materialize slower implementation analysis evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS
special arch --metrics surfaces module ownership granularity and per-module implementation summaries from built-in language analyzers.

@group SPECIAL.MODULE_COMMAND.METRICS.COMPLEXITY
special arch can explain and summarize complexity evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.QUALITY
special arch --metrics surfaces language-agnostic quality evidence categories when a built-in analyzer can extract them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.UNREACHED_CODE
special arch --metrics surfaces conservative unreached-code indicators within owned implementation when built-in analyzers can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.COMPLEXITY.EXPLANATIONS
special arch --metrics --verbose explains complexity evidence in plain language and in precise structural terms from a shared analysis registry.

@spec SPECIAL.MODULE_COMMAND.METRICS.QUALITY.EXPLANATIONS
special arch --metrics --verbose explains quality evidence in plain language and in precise structural terms from a shared analysis registry.

@group SPECIAL.MODULE_COMMAND.METRICS.ITEM_SIGNALS
special arch can surface item-level evidence within owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.ITEM_SIGNALS.EXPLANATIONS
special arch --metrics --verbose explains item-level evidence categories in plain language and in precise structural terms from a shared analysis registry.

@spec SPECIAL.MODULE_COMMAND.METRICS.COUPLING
special arch --metrics surfaces module-to-module coupling evidence when built-in analyzers can resolve owned dependency targets to concrete modules.

@spec SPECIAL.MODULE_COMMAND.METRICS.COUPLING.EXPLANATIONS
special arch --metrics --verbose explains coupling metrics in plain language and in precise structural terms from a shared analysis registry.

@group SPECIAL.MODULE_COMMAND.METRICS.RUST
special arch can surface Rust-specific implementation evidence for owned Rust code through the built-in Rust analyzer.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.SURFACE
special arch --metrics surfaces Rust public and internal item counts for owned Rust implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY
special arch --metrics surfaces Rust function complexity summaries for owned implementation, including analyzed function count plus total and maximum cyclomatic complexity.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.COMPLEXITY.COGNITIVE
special arch --metrics surfaces Rust cognitive complexity summaries for owned implementation, including total and maximum cognitive complexity.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.QUALITY
special arch --metrics surfaces Rust quality evidence, including public API parameter shape, stringly typed boundaries, and recoverability signals.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.DEPENDENCIES
special arch --metrics --verbose surfaces Rust `use`-path dependency evidence from owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.COUPLING
special arch --metrics derives generic module coupling evidence from Rust `use` targets when those targets resolve to uniquely owned files.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS
special arch --metrics --verbose surfaces per-item Rust evidence for owned implementation, including internally connected, outbound-heavy, isolated, and unreached items when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.COMPLEXITY
special arch --metrics --verbose surfaces highest-complexity Rust items within owned implementation so unusual local hotspots are visible inside a claimed module boundary.

@spec SPECIAL.MODULE_COMMAND.METRICS.RUST.ITEM_SIGNALS.QUALITY
special arch --metrics --verbose surfaces parameter-heavy, stringly boundary, and panic-heavy Rust items within owned implementation so unusual local craftsmanship signals are visible inside a claimed module boundary.

@group SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT_GROUP
special arch can surface TypeScript-specific implementation evidence for owned TypeScript code through the built-in TypeScript analyzer.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT
special arch --metrics --verbose surfaces built-in TypeScript implementation evidence for owned TypeScript code, including public and internal item counts, complexity summaries, quality evidence, import-path dependency evidence, coupling derived from owned relative imports, and per-item connected, outbound-heavy, isolated, and unreached signals when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.COUPLING
special arch --metrics uses TypeScript compiler module resolution when available so path aliases can contribute owned dependency targets to module coupling.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.TOOLCHAIN_DEGRADED
special arch --metrics reports when TypeScript compiler-backed analyzer enrichment is unavailable and metrics fall back to weaker parser-derived evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.COMPLEXITY
special arch --metrics surfaces TypeScript function complexity summaries for owned implementation, including analyzed function count plus total and maximum cyclomatic and cognitive complexity.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.QUALITY
special arch --metrics surfaces TypeScript quality evidence, including public API parameter shape, stringly typed boundaries, and throw-site recoverability signals.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.ITEM_SIGNALS
special arch --metrics --verbose surfaces per-item TypeScript evidence for owned implementation, including internally connected, outbound-heavy, isolated, and unreached items when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.ITEM_SIGNALS.COMPLEXITY
special arch --metrics --verbose surfaces highest-complexity TypeScript items within owned implementation so unusual local hotspots are visible inside a claimed module boundary.

@spec SPECIAL.MODULE_COMMAND.METRICS.TYPESCRIPT.ITEM_SIGNALS.QUALITY
special arch --metrics --verbose surfaces parameter-heavy, stringly boundary, and panic-heavy TypeScript items within owned implementation so unusual local craftsmanship signals are visible inside a claimed module boundary.

@group SPECIAL.MODULE_COMMAND.METRICS.GO_GROUP
special arch can surface Go-specific implementation evidence for owned Go code through the built-in Go analyzer.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO
special arch --metrics --verbose surfaces built-in Go implementation evidence for owned Go code, including public and internal item counts, complexity summaries, quality evidence, import-path dependency evidence, coupling derived from owned local imports, and per-item connected, outbound-heavy, isolated, and unreached signals when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO.COUPLING
special arch --metrics uses go list package resolution when available so module import paths can contribute owned dependency targets to module coupling.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO.TOOLCHAIN_DEGRADED
special arch --metrics reports when go-list-backed analyzer enrichment is unavailable and metrics fall back to weaker parser-derived evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO.COMPLEXITY
special arch --metrics surfaces Go function complexity summaries for owned implementation, including analyzed function count plus total and maximum cyclomatic and cognitive complexity.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO.QUALITY
special arch --metrics surfaces Go quality evidence, including public API parameter shape, stringly typed boundaries, and panic-site recoverability signals.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO.ITEM_SIGNALS
special arch --metrics --verbose surfaces per-item Go evidence for owned implementation, including internally connected, outbound-heavy, isolated, and unreached items when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO.ITEM_SIGNALS.COMPLEXITY
special arch --metrics --verbose surfaces highest-complexity Go items within owned implementation so unusual local hotspots are visible inside a claimed module boundary.

@spec SPECIAL.MODULE_COMMAND.METRICS.GO.ITEM_SIGNALS.QUALITY
special arch --metrics --verbose surfaces parameter-heavy, stringly boundary, and panic-heavy Go items within owned implementation so unusual local craftsmanship signals are visible inside a claimed module boundary.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON
special arch --json --metrics includes structured architecture analysis summaries.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.QUALITY
special arch --json --metrics includes structured quality evidence summaries when available.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.COUPLING
special arch --json --metrics includes structured module coupling summaries when available.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.UNREACHED_CODE
special arch --json --metrics --verbose includes structured unreached-code indicators when built-in analyzers can identify them honestly.

@group SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST
special arch --json --metrics can include Rust-specific structured analysis evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY
special arch --json --metrics includes structured Rust function complexity summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.COMPLEXITY.COGNITIVE
special arch --json --metrics includes structured Rust cognitive complexity summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.QUALITY
special arch --json --metrics includes structured Rust quality evidence summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.DEPENDENCIES
special arch --json --metrics --verbose includes structured Rust dependency targets for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS
special arch --json --metrics --verbose includes structured per-item Rust evidence for owned implementation, including unreached Rust items when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.COMPLEXITY
special arch --json --metrics --verbose includes structured highest-complexity Rust item evidence for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.RUST.ITEM_SIGNALS.QUALITY
special arch --json --metrics --verbose includes structured parameter-heavy, stringly boundary, and panic-heavy Rust item evidence for owned implementation.

@group SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT_GROUP
special arch --json --metrics can include TypeScript-specific structured analysis evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT
special arch --json --metrics --verbose includes structured TypeScript implementation evidence for owned TypeScript code, including public and internal item counts, complexity summaries, quality summaries, dependency targets, and per-item signals.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.COMPLEXITY
special arch --json --metrics includes structured TypeScript function complexity summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.QUALITY
special arch --json --metrics includes structured TypeScript quality evidence summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.ITEM_SIGNALS
special arch --json --metrics --verbose includes structured per-item TypeScript evidence for owned implementation, including unreached TypeScript items when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.ITEM_SIGNALS.COMPLEXITY
special arch --json --metrics --verbose includes structured highest-complexity TypeScript item evidence for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.TYPESCRIPT.ITEM_SIGNALS.QUALITY
special arch --json --metrics --verbose includes structured parameter-heavy, stringly boundary, and panic-heavy TypeScript item evidence for owned implementation.

@group SPECIAL.MODULE_COMMAND.METRICS.JSON.GO_GROUP
special arch --json --metrics can include Go-specific structured analysis evidence.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.GO
special arch --json --metrics --verbose includes structured Go implementation evidence for owned Go code, including public and internal item counts, complexity summaries, quality summaries, dependency targets, and per-item signals.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.COMPLEXITY
special arch --json --metrics includes structured Go function complexity summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.QUALITY
special arch --json --metrics includes structured Go quality evidence summaries for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.ITEM_SIGNALS
special arch --json --metrics --verbose includes structured per-item Go evidence for owned implementation, including unreached Go items when the built-in analyzer can identify them honestly.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.ITEM_SIGNALS.COMPLEXITY
special arch --json --metrics --verbose includes structured highest-complexity Go item evidence for owned implementation.

@spec SPECIAL.MODULE_COMMAND.METRICS.JSON.GO.ITEM_SIGNALS.QUALITY
special arch --json --metrics --verbose includes structured parameter-heavy, stringly boundary, and panic-heavy Go item evidence for owned implementation.

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

@spec SPECIAL.MODULE_PARSE.CURRENT_MODULES_REQUIRE_IMPLEMENTATION
current `@module` nodes require a direct `@implements` or `@fileimplements` attachment unless they are planned.

@spec SPECIAL.MODULE_PARSE.AREAS_ARE_STRUCTURAL_ONLY
`@area` nodes are structural architecture nodes and do not require direct `@implements` attachments.
*/
#[path = "support/cli.rs"]
mod support;
#[allow(dead_code)]
#[path = "../src/language_packs/typescript/test_fixtures.rs"]
mod typescript_test_fixtures;

use std::fs;

use serde_json::Value;

use support::{
    find_node_by_id, html_node_has_badge, rendered_spec_node_ids, rendered_spec_node_line,
    run_special, run_special_raw, temp_repo_dir, top_level_help_commands, write_modules_fixture,
    write_unimplemented_child_module_fixture, write_unimplemented_module_fixture,
};

#[test]
// @verifies SPECIAL.HELP.ARCH_COMMAND_PRIMARY
fn top_level_help_presents_arch_as_the_primary_command_name() {
    let root = temp_repo_dir("special-cli-modules-help-primary");

    let output = run_special(&root, &["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        top_level_help_commands(&stdout)
            .iter()
            .any(|(name, summary)| name == "arch"
                && summary == "Materialize and inspect architecture")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND
fn modules_materializes_current_module_tree() {
    let root = temp_repo_dir("special-cli-modules");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch"]);
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
// @verifies SPECIAL.MODULE_COMMAND.DEFAULT_ALL
fn modules_default_view_includes_planned_nodes() {
    let root = temp_repo_dir("special-cli-modules-default-all");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HELP.ARCH_COMMAND_PRIMARY
fn modules_and_module_aliases_are_rejected() {
    let root = temp_repo_dir("special-cli-module-aliases-rejected");

    let modules_output = run_special_raw(&root, &["modules"]);
    assert!(!modules_output.status.success());
    let modules_stderr = String::from_utf8(modules_output.stderr).expect("stderr should be utf-8");
    assert!(modules_stderr.contains("unrecognized subcommand"));
    assert!(modules_stderr.contains("modules"));

    let module_output = run_special_raw(&root, &["module"]);
    assert!(!module_output.status.success());
    let module_stderr = String::from_utf8(module_output.stderr).expect("stderr should be utf-8");
    assert!(module_stderr.contains("unrecognized subcommand"));
    assert!(module_stderr.contains("module"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.CURRENT_ONLY
fn modules_current_excludes_planned_modules() {
    let root = temp_repo_dir("special-cli-modules-all");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--current"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!rendered_spec_node_ids(&stdout).contains(&"DEMO.PLANNED".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.PLANNED_ONLY
fn modules_planned_shows_only_planned_modules() {
    let root = temp_repo_dir("special-cli-modules-planned");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--planned"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let planned_line =
        rendered_spec_node_line(&stdout, "DEMO.PLANNED").expect("planned module should render");
    assert!(planned_line.contains("[planned: 0.4.0]"));
    assert!(!rendered_spec_node_ids(&stdout).contains(&"DEMO.API".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.PLANNED_RELEASE_METADATA
fn modules_surface_planned_release_metadata_across_output_modes() {
    let root = temp_repo_dir("special-cli-modules-planned-release");
    write_modules_fixture(&root);

    let text_output = run_special(&root, &["arch", "--planned"]);
    assert!(text_output.status.success());
    let text_stdout = String::from_utf8(text_output.stdout).expect("stdout should be utf-8");
    let planned_line = rendered_spec_node_line(&text_stdout, "DEMO.PLANNED")
        .expect("planned module should render");
    assert!(planned_line.contains("[planned: 0.4.0]"));

    let json_output = run_special(&root, &["arch", "--planned", "--json"]);
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

    let html_output = run_special(&root, &["arch", "--planned", "--html"]);
    assert!(html_output.status.success());
    let html_stdout = String::from_utf8(html_output.stdout).expect("stdout should be utf-8");
    assert!(html_node_has_badge(
        &html_stdout,
        "DEMO.PLANNED",
        "badge-planned",
        "planned: 0.4.0"
    ));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.ID_SCOPE
fn modules_scope_to_matching_id_and_descendants() {
    let root = temp_repo_dir("special-cli-modules-scope");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch", "DEMO"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO".to_string()));
    assert!(node_ids.contains(&"DEMO.LIVE".to_string()));
    assert!(node_ids.contains(&"DEMO.PLANNED".to_string()));

    let output = run_special(&root, &["arch", "DEMO.LIVE"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert_eq!(node_ids, vec!["DEMO.LIVE".to_string()]);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.UNIMPLEMENTED
fn modules_unimplemented_filters_current_modules_without_implements() {
    let root = temp_repo_dir("special-cli-modules-unimplemented");
    write_unimplemented_child_module_fixture(&root);

    let output = run_special(&root, &["arch", "--unimplemented"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.UNIMPLEMENTED".to_string()));
    assert!(stderr.contains(
        "current module `DEMO.UNIMPLEMENTED` has no direct @implements or @fileimplements attachment"
    ));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_short_u_still_filters_unimplemented_current_modules() {
    let root = temp_repo_dir("special-cli-modules-unimplemented-short");
    write_unimplemented_child_module_fixture(&root);

    let output = run_special(&root, &["arch", "-u"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let node_ids = rendered_spec_node_ids(&stdout);
    assert!(node_ids.contains(&"DEMO.UNIMPLEMENTED".to_string()));
    assert!(stderr.contains(
        "current module `DEMO.UNIMPLEMENTED` has no direct @implements or @fileimplements attachment"
    ));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn modules_rejects_planned_and_unimplemented_filter_combo() {
    let root = temp_repo_dir("special-cli-modules-unimplemented-planned-conflict");
    write_unimplemented_child_module_fixture(&root);

    let output = run_special(&root, &["arch", "--planned", "--unimplemented"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("--planned"));
    assert!(stderr.contains("--unimplemented"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.FAILS_ON_ERRORS
fn modules_fails_when_module_diagnostics_are_present() {
    let root = temp_repo_dir("special-cli-modules-fails-on-errors");
    write_unimplemented_module_fixture(&root);

    let output = run_special(&root, &["arch"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(rendered_spec_node_ids(&stdout).contains(&"DEMO".to_string()));
    assert!(
        stderr.contains(
            "current module `DEMO` has no direct @implements or @fileimplements attachment"
        )
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.JSON
fn modules_json_emits_json_output() {
    let root = temp_repo_dir("special-cli-modules-json");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--json"]);
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

    let output = run_special(&root, &["arch", "--html"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<title>special arch</title>"));
    assert!(stdout.contains("<span class=\"node-id\">DEMO</span>"));
    assert!(stdout.contains("implements: 1"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.VERBOSE
fn modules_verbose_includes_implementation_bodies() {
    let root = temp_repo_dir("special-cli-modules-verbose");
    write_modules_fixture(&root);

    let output = run_special(&root, &["arch", "--verbose"]);
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

    let output = run_special(&root, &["arch", "--json", "--verbose"]);
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
        .and_then(|items: &Vec<Value>| items.first())
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

    let output = run_special(&root, &["arch", "--html", "--verbose"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("<details><summary>@implements "));
    assert!(stdout.contains("<details><summary>@fileimplements "));
    assert!(stdout.contains("implements_demo_live"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
