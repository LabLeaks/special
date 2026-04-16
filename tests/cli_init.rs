/**
@module SPECIAL.TESTS.CLI_INIT
`special init` command tests in `tests/cli_init.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_INIT
#[path = "support/cli.rs"]
mod support;

use std::fs;

use support::{
    run_special, temp_repo_dir, top_level_help_command_names, top_level_help_command_summaries,
};

/**
@spec SPECIAL.INIT.SURFACES_DISCOVERY_ERRORS
special init exits with an error when ancestor root discovery fails instead of ignoring the failure and writing nested config.

@spec SPECIAL.INIT.CREATES_SPECIAL_TOML
special init creates `special.toml` in the current directory with `version = "1"` and `root = "."`.

@spec SPECIAL.INIT.DOES_NOT_OVERWRITE_SPECIAL_TOML
special init fails instead of overwriting an existing `special.toml` in the current directory.

@spec SPECIAL.INIT.REJECTS_NESTED_ACTIVE_CONFIG
when the current directory is already governed by an ancestor `special.toml`, special init fails instead of creating a nested config by accident.
*/

#[test]
// @verifies SPECIAL.INIT.CREATES_SPECIAL_TOML
fn init_creates_special_toml_in_current_directory() {
    let root = temp_repo_dir("special-cli-init");

    let output = run_special(&root, &["init"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.starts_with("Created "));
    assert!(stdout.contains("special.toml"));
    assert_eq!(
        fs::read_to_string(root.join("special.toml")).expect("special.toml should be created"),
        "version = \"1\"\nroot = \".\"\n"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.INIT.DOES_NOT_OVERWRITE_SPECIAL_TOML
fn init_fails_when_special_toml_already_exists() {
    let root = temp_repo_dir("special-cli-init-existing");
    fs::write(root.join("special.toml"), "root = \"workspace\"\n")
        .expect("special.toml should be written");

    let output = run_special(&root, &["init"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("special.toml already exists"));
    assert_eq!(
        fs::read_to_string(root.join("special.toml")).expect("special.toml should still exist"),
        "root = \"workspace\"\n"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.INIT.REJECTS_NESTED_ACTIVE_CONFIG
fn init_rejects_nested_directory_already_governed_by_ancestor_config() {
    let root = temp_repo_dir("special-cli-init-nested");
    let nested = root.join("nested/deeper");
    fs::create_dir_all(&nested).expect("nested dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("ancestor special.toml should be written");

    let output = run_special(&nested, &["init"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("already governs"));
    assert!(!nested.join("special.toml").exists());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.INIT.SURFACES_DISCOVERY_ERRORS
fn init_surfaces_ancestor_discovery_errors() {
    let root = temp_repo_dir("special-cli-init-discovery-error");
    let nested = root.join("nested/deeper");
    fs::create_dir_all(&nested).expect("nested dir should be created");
    fs::write(root.join("special.toml"), "root = unquoted\n")
        .expect("ancestor special.toml should be written");

    let output = run_special(&nested, &["init"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to parse special.toml"));
    assert!(!nested.join("special.toml").exists());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HELP.TOP_LEVEL_COMMANDS
fn top_level_help_lists_command_summaries() {
    let root = temp_repo_dir("special-cli-help");

    let output = run_special(&root, &["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let command_names = top_level_help_command_names(&stdout);
    assert_eq!(
        command_names,
        vec!["specs", "modules", "lint", "init", "skills"]
    );
    let summaries = top_level_help_command_summaries(&stdout);
    assert_eq!(summaries.len(), command_names.len());
    assert!(summaries.iter().all(|summary| !summary.is_empty()));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HELP.SUBCOMMAND
fn help_subcommand_matches_top_level_help_surface() {
    let root = temp_repo_dir("special-cli-help-subcommand");

    let help_output = run_special(&root, &["help"]);
    assert!(help_output.status.success());

    let flag_output = run_special(&root, &["--help"]);
    assert!(flag_output.status.success());

    let help_stdout = String::from_utf8(help_output.stdout).expect("stdout should be utf-8");
    let flag_stdout = String::from_utf8(flag_output.stdout).expect("stdout should be utf-8");
    assert_eq!(help_stdout.trim(), flag_stdout.trim());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.VERSION.FLAGS
fn version_flags_print_current_cli_version() {
    let root = temp_repo_dir("special-cli-version");

    for args in [["-v"], ["--version"]] {
        let output = run_special(&root, &args);
        assert!(output.status.success(), "{args:?} should succeed");

        let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
        let trimmed = stdout.trim();
        assert!(trimmed.starts_with("special "));
        assert!(trimmed.ends_with(env!("CARGO_PKG_VERSION")));

        let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
        assert!(stderr.trim().is_empty(), "{args:?} should not write stderr");
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
