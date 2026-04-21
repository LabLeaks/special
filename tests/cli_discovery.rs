/**
@module SPECIAL.TESTS.CLI_DISCOVERY
Shared discovery and markdown declaration tests in `tests/cli_discovery.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_DISCOVERY
/**
@group SPECIAL.DISCOVERY
special discovers spec and module annotation files from the active project root.

@group SPECIAL.CONFIG.SPECIAL_TOML.IGNORE
special.toml can exclude paths from shared annotation discovery.

@spec SPECIAL.CONFIG.SPECIAL_TOML.IGNORE.SHARED_DISCOVERY
special.toml `ignore` patterns exclude matching paths from both spec and module discovery, including module metrics coverage.

@spec SPECIAL.DISCOVERY.DEFAULT_VCS_IGNORES
special respects `.gitignore` and `.jjignore` by default when discovering spec and module annotations.

@spec SPECIAL.SPEC_COMMAND.MARKDOWN_DECLARATIONS
special specs materializes `@group` and `@spec` declarations from markdown heading annotations under the project root.

@spec SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS
special arch materializes `@area` and `@module` declarations from markdown heading annotations under the project root.

@spec SPECIAL.SPEC_COMMAND.NO_SPECIAL_SPECS_DIRECTORY
special specs does not require declarations to live under a special `specs/` directory.

@spec SPECIAL.MODULE_COMMAND.NO_SPECIAL_ARCHITECTURE_FILE
special arch does not require declarations to live in a privileged `_project/ARCHITECTURE.md` file.
*/
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;

use support::{find_node_by_id, run_special, temp_repo_dir, write_markdown_declarations_fixture};

#[test]
// @verifies SPECIAL.SPEC_COMMAND.MARKDOWN_DECLARATIONS
fn specs_materialize_markdown_declarations() {
    let root = temp_repo_dir("special-cli-discovery-markdown-specs");
    write_markdown_declarations_fixture(&root);

    let output = run_special(&root, &["specs"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO"));
    assert!(stdout.contains("DEMO.MARKDOWN"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.MARKDOWN_DECLARATIONS
fn modules_materialize_markdown_declarations() {
    let root = temp_repo_dir("special-cli-discovery-markdown-modules");
    write_markdown_declarations_fixture(&root);

    let output = run_special(&root, &["arch"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("DEMO.AREA"));
    assert!(stdout.contains("DEMO.MODULE"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SPEC_COMMAND.NO_SPECIAL_SPECS_DIRECTORY
fn specs_do_not_require_special_specs_directory() {
    let root = temp_repo_dir("special-cli-discovery-no-specs-dir");
    write_markdown_declarations_fixture(&root);

    let output = run_special(&root, &["specs", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.MARKDOWN"))
        })
        .expect("demo markdown spec should be present");
    assert_eq!(demo["text"], Value::String("Demo root claim.".to_string()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.MODULE_COMMAND.NO_SPECIAL_ARCHITECTURE_FILE
fn modules_do_not_require_special_architecture_doc() {
    let root = temp_repo_dir("special-cli-discovery-no-architecture-doc");
    write_markdown_declarations_fixture(&root);

    let output = run_special(&root, &["arch", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let demo = json["nodes"]
        .as_array()
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|node| find_node_by_id(node, "DEMO.AREA"))
        })
        .expect("demo area should be present");
    assert_eq!(
        demo["text"],
        Value::String("Demo architecture area.".to_string())
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.CONFIG.SPECIAL_TOML.IGNORE.SHARED_DISCOVERY
fn special_toml_ignore_patterns_hide_matching_paths_from_specs_modules_and_metrics() {
    let root = temp_repo_dir("special-cli-discovery-ignore");
    write_markdown_declarations_fixture(&root);
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\nignore = [\"ignored/**\"]\n",
    )
    .expect("special.toml should be written");
    fs::create_dir_all(root.join("ignored")).expect("ignored dir should be created");
    fs::write(
        root.join("ignored/specs.md"),
        "### `@spec IGNORED.SPEC`\nIgnored markdown spec.\n\n### `@module IGNORED.MODULE`\nIgnored markdown module.\n",
    )
    .expect("ignored markdown fixture should be written");
    fs::write(
        root.join("ignored/hidden.rs"),
        "// @fileimplements IGNORED.MODULE\nfn ignored_hidden() {}\n",
    )
    .expect("ignored implementation fixture should be written");

    let specs_output = run_special(&root, &["specs"]);
    assert!(specs_output.status.success());
    let specs_stdout = String::from_utf8(specs_output.stdout).expect("stdout should be utf-8");
    assert!(specs_stdout.contains("DEMO"));
    assert!(!specs_stdout.contains("IGNORED.SPEC"));

    let modules_output = run_special(&root, &["arch", "--metrics", "--verbose"]);
    assert!(modules_output.status.success());
    let modules_stdout = String::from_utf8(modules_output.stdout).expect("stdout should be utf-8");
    assert!(modules_stdout.contains("DEMO.AREA"));
    assert!(!modules_stdout.contains("IGNORED.MODULE"));
    assert!(!modules_stdout.contains("ignored/hidden.rs"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.DISCOVERY.DEFAULT_VCS_IGNORES
fn discovery_respects_gitignore_and_jjignore_by_default() {
    let root = temp_repo_dir("special-cli-discovery-vcs-ignore");
    write_markdown_declarations_fixture(&root);
    fs::write(root.join(".gitignore"), "ignored-git/**\n").expect(".gitignore should be written");
    fs::write(root.join(".jjignore"), "ignored-jj/**\n").expect(".jjignore should be written");

    fs::create_dir_all(root.join("ignored-git")).expect("gitignored dir should be created");
    fs::write(
        root.join("ignored-git/specs.md"),
        "### `@spec IGNORED.GIT`\nIgnored git claim.\n",
    )
    .expect("gitignored markdown should be written");

    fs::create_dir_all(root.join("ignored-jj")).expect("jjignored dir should be created");
    fs::write(
        root.join("ignored-jj/arch.md"),
        "### `@module IGNORED.JJ`\nIgnored jj module.\n",
    )
    .expect("jjignored markdown should be written");

    let specs_output = run_special(&root, &["specs"]);
    assert!(specs_output.status.success());
    let specs_stdout = String::from_utf8(specs_output.stdout).expect("stdout should be utf-8");
    assert!(!specs_stdout.contains("IGNORED.GIT"));

    let modules_output = run_special(&root, &["arch"]);
    assert!(modules_output.status.success());
    let modules_stdout = String::from_utf8(modules_output.stdout).expect("stdout should be utf-8");
    assert!(!modules_stdout.contains("IGNORED.JJ"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
