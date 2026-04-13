/**
@module SPECIAL.TESTS.CLI_MODULES
`special modules` command tests in `tests/cli_modules.rs`.
*/
// @implements SPECIAL.TESTS.CLI_MODULES
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

@group SPECIAL.MODULE_PARSE
special parses architecture module declarations and implementation attachments.

@spec SPECIAL.MODULE_PARSE.ARCHITECTURE_DOC
special reads `_project/ARCHITECTURE.md` as an optional central architecture module declaration file.

@spec SPECIAL.MODULE_PARSE.MODULE_DECLARATIONS
special parses `@module MODULE.ID` declarations from supported source comments.

@spec SPECIAL.MODULE_PARSE.AREA_DECLARATIONS
special parses `@area AREA.ID` declarations as structural architecture nodes from `_project/ARCHITECTURE.md`.

@group SPECIAL.MODULE_PARSE.PLANNED
special records `@planned` on declared modules from supported module declaration surfaces.

@spec SPECIAL.MODULE_PARSE.PLANNED.MODULE_ONLY
`@planned` may only apply to `@module`, not `@area`.

@spec SPECIAL.MODULE_PARSE.PLANNED.EXACT_STANDALONE_MARKER
special only accepts an exact standalone `@planned` marker on the next line after a module declaration.

@group SPECIAL.MODULE_PARSE.IMPLEMENTS
special parses `@implements MODULE.ID` markers from supported source files as module attachments.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.MODULE_ONLY
`@implements` may only reference ids declared as `@module`, not `@area`.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.FILE_SCOPE
when `@implements` appears at the top of a file, special records a file-scoped module attachment without requiring an owned item body.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.ITEM_SCOPE
when `@implements` appears immediately above a supported owned item, special attaches the owned item body to the module attachment.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.EXACT_DIRECTIVE_SHAPE
`@implements` accepts exactly one module id and rejects trailing content.

@spec SPECIAL.MODULE_PARSE.FOREIGN_TAG_BOUNDARIES
special treats foreign line-start `@...` and `\\...` tags as boundaries for attached module text without materializing them as part of the module description.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_FILE_SCOPE
when a file declares more than one file-scoped `@implements`, special lint reports an error.

@spec SPECIAL.MODULE_PARSE.IMPLEMENTS.DUPLICATE_ITEM_SCOPE
when more than one `@implements` attaches to the same owned code item, special lint reports an error.

@spec SPECIAL.MODULE_PARSE.LIVE_MODULES_REQUIRE_IMPLEMENTATION
live `@module` nodes require a direct `@implements` attachment unless they are planned.

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
    write_duplicate_file_scoped_implements_fixture, write_duplicate_item_scoped_implements_fixture,
    write_implements_with_trailing_content_fixture, write_missing_intermediate_modules_fixture,
    write_mixed_purpose_source_local_module_fixture, write_modules_fixture,
    write_planned_area_fixture, write_planned_area_invalid_suffix_fixture,
    write_source_local_modules_fixture, write_unimplemented_module_fixture,
    write_unknown_implements_fixture, write_unsupported_module_fixture,
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
    assert!(stderr.contains("live module `DEMO.UNSUPPORTED` has no direct @implements attachment"));

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
    assert!(stderr.contains("live module `DEMO` has no direct @implements attachment"));

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
    assert!(stdout.contains("implements_demo_live"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_PARSE.ARCHITECTURE_DOC
fn modules_read_central_architecture_doc_when_present() {
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
    assert!(stdout.contains("unknown module id `DEMO.MISSING` referenced by @implements"));

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
    assert!(stdout.contains("duplicate file-scoped @implements"));

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
    assert!(stdout.contains("@implements may only reference @module ids"));

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
    assert!(stdout.contains("live module `DEMO` has no direct @implements attachment"));

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
