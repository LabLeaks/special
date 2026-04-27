/**
@group SPECIAL.SKILLS.COMMAND
Structure for the `special skills` command contract surface.

@spec SPECIAL.SKILLS.COMMAND.HELP
`special skills` prints the bundled skills overview, command shapes, and install guidance without mutating the repo.

@spec SPECIAL.SKILLS.COMMAND.HELP.NO_ROOT_WARNING
`special skills` does not emit project-root/config warnings when only printing the overview.

@spec SPECIAL.SKILLS.COMMAND.HELP.INSTALL_DESTINATION_GUIDANCE
`special skills` describes project, global, and custom install destinations without probing or requiring a valid destination up front.

@spec SPECIAL.SKILLS.COMMAND.EMITS_SKILL_TO_STDOUT
`special skills SKILL_ID` writes the bundled skill markdown to stdout without installing it.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND
`special skills install` is the install entrypoint for bundled skills.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ALL_SKILLS_DEFAULT
without a skill id, `special skills install` installs every bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ONE_SKILL
with a skill id, `special skills install SKILL_ID` installs only that bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.VALIDATES_SKILL_ID_BEFORE_PROMPT
`special skills install SKILL_ID` validates the skill id before prompting for a destination.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROMPTS_FOR_DESTINATION
without an explicit destination flag, `special skills install` prompts for the destination.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.REJECTS_UNKNOWN_PROMPT_CHOICES
the interactive destination prompt rejects unknown choices and reprompts.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.NON_INTERACTIVE_DESTINATION
`special skills install` accepts `--destination` so installs can run non-interactively.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.FORCE_OVERWRITE
`special skills install --force` overwrites conflicting installed skills without interactive confirmation.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROJECT_DESTINATION
the `project` destination installs into the current repository’s `.agents/skills/` directory.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.GLOBAL_DESTINATION
the `global` destination installs into `$CODEX_HOME/skills`, or `~/.codex/skills` when `CODEX_HOME` is unset.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.CUSTOM_DESTINATION
the `custom` destination installs into a user-provided path.

@spec SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.OVERWRITE_PROMPT
without `--force`, `special skills install` prompts before overwriting an installed skill.

@spec SPECIAL.SKILLS.COMMAND.WRITES_PROJECT_SKILLS_DIRECTORY
project installs create the repo-local `.agents/skills/` directory when needed.

@spec SPECIAL.SKILLS.COMMAND.USES_AGENT_SKILLS_LAYOUT
installed bundled skills use the `.agents/skills/SKILL_ID/SKILL.md` layout.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_SHIP_CHANGE_SKILL
`special skills install` includes the `ship-product-change` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_DEFINE_PRODUCT_SPECS_SKILL
`special skills install` includes the `define-product-specs` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_VALIDATE_PRODUCT_CONTRACT_SKILL
`special skills install` includes the `validate-product-contract` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_VALIDATE_ARCHITECTURE_IMPLEMENTATION_SKILL
`special skills install` includes the `validate-architecture-implementation` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_USE_PROJECT_PATTERNS_SKILL
`special skills install` includes the `use-project-patterns` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_EVOLVE_MODULE_ARCHITECTURE_SKILL
`special skills install` includes the `evolve-module-architecture` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_INSPECT_CURRENT_SPEC_STATE_SKILL
`special skills install` includes the `inspect-current-spec-state` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.INSTALLS_FIND_PLANNED_WORK_SKILL
`special skills install` includes the `find-planned-work` bundled skill.

@spec SPECIAL.SKILLS.COMMAND.BUNDLES_REFERENCES_FOR_PROGRESSIVE_DISCLOSURE
installed bundled skills keep their reference files for progressive disclosure.

@spec SPECIAL.SKILLS.COMMAND.INCLUDES_TRIGGER_EVAL_FIXTURES
installed bundled skills include trigger-eval fixtures where the bundled skill ships them.

@module SPECIAL.TESTS.CLI_SKILLS
`special skills` command tests in `tests/cli_skills.rs`.
*/
// @fileimplements SPECIAL.TESTS.CLI_SKILLS
#[path = "support/cli.rs"]
mod support;

use std::fs;

use support::{
    bundled_skill_ids, bundled_skill_markdown, install_skills, installed_skill_ids,
    listed_skill_ids, run_special, run_special_with_env_removed, run_special_with_input,
    run_special_with_input_and_env, skills_command_shape_lines, skills_install_destinations,
    temp_repo_dir, write_invalid_skills_root_fixture, write_skills_fixture,
};

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.HELP
fn skills_prints_help_and_explanatory_text() {
    let root = temp_repo_dir("special-cli-skills-dir");
    write_skills_fixture(&root);

    let output = run_special(&root, &["skills"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert_eq!(
        skills_command_shape_lines(&stdout),
        vec![
            "special skills".to_string(),
            "special skills SKILL_ID".to_string(),
            "special skills install [SKILL_ID]".to_string(),
            "special skills install [SKILL_ID] --destination DESTINATION".to_string(),
            "special skills install [SKILL_ID] --destination DESTINATION --force".to_string(),
        ]
    );
    assert_eq!(
        listed_skill_ids(&stdout),
        bundled_skill_ids()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    );
    assert!(!root.join(".agents/skills").exists());
    assert!(!stderr.contains("warning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.HELP.NO_ROOT_WARNING
fn skills_overview_does_not_emit_root_warning_without_config() {
    let root = temp_repo_dir("special-cli-skills-no-warning");

    let output = run_special(&root, &["skills"]);
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(!stderr.contains("warning:"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.HELP.INSTALL_DESTINATION_GUIDANCE
fn skills_overview_describes_install_destinations_without_probing_environment() {
    let root = temp_repo_dir("special-cli-skills-invalid-project");
    write_invalid_skills_root_fixture(&root);

    let output = run_special(&root, &["skills"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let destinations = skills_install_destinations(&stdout);
    assert_eq!(
        destinations
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["project", "global", "custom"]
    );
    assert!(destinations.iter().all(|(_, summary)| !summary.is_empty()));
    assert!(!stderr.contains("warning:"));

    let install_output = run_special(
        &root,
        &[
            "skills",
            "install",
            "ship-product-change",
            "--destination",
            "project",
        ],
    );
    assert!(!install_output.status.success());
    let install_stderr = String::from_utf8(install_output.stderr).expect("stderr should be utf-8");
    assert!(install_stderr.contains("project install destination unavailable"));
    assert!(install_stderr.contains("points to a root that does not exist"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.HELP.SKILLS_COMMAND_SHAPES
fn skills_help_flag_describes_skills_command_shapes() {
    let root = temp_repo_dir("special-cli-skills-help");
    let output = run_special(&root, &["skills", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        skills_command_shape_lines(&stdout),
        vec![
            "special skills".to_string(),
            "special skills SKILL_ID".to_string(),
            "special skills install [SKILL_ID]".to_string(),
            "special skills install [SKILL_ID] --destination DESTINATION".to_string(),
            "special skills install [SKILL_ID] --destination DESTINATION --force".to_string(),
        ]
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.EMITS_SKILL_TO_STDOUT
fn skills_prints_one_skill_to_stdout() {
    let root = temp_repo_dir("special-cli-skills-print");
    write_skills_fixture(&root);

    let output = run_special(&root, &["skills", "ship-product-change"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim_end_matches('\n'),
        bundled_skill_markdown("ship-product-change").trim_end_matches('\n')
    );
    assert!(!root.join(".agents/skills").exists());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ALL_SKILLS_DEFAULT
fn skills_install_without_skill_id_installs_all_bundled_skills() {
    let root = temp_repo_dir("special-cli-skills-install-all");

    let output = install_skills(&root);
    assert!(output.status.success());

    let expected = bundled_skill_ids();
    assert_eq!(
        installed_skill_ids(&root.join(".agents/skills")),
        expected.into_iter().map(str::to_string).collect::<Vec<_>>()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND
fn skills_install_is_the_install_entrypoint() {
    let root = temp_repo_dir("special-cli-skills-install-entry");
    write_skills_fixture(&root);

    let output = run_special_with_input(&root, &["skills", "install"], "project\n");
    assert!(output.status.success());

    assert_eq!(
        installed_skill_ids(&root.join(".agents/skills")),
        bundled_skill_ids()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROMPTS_FOR_DESTINATION
fn skills_install_prompts_for_available_destinations() {
    let root = temp_repo_dir("special-cli-skills-prompt-destination");
    write_skills_fixture(&root);

    let output = run_special_with_input(
        &root,
        &["skills", "install", "ship-product-change"],
        "project\n",
    );
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Select install destination:"));
    assert!(stdout.contains("Destination [project/global/custom]:"));
    assert!(stdout.contains("project"));
    assert!(stdout.contains("global"));
    assert!(stdout.contains("custom"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.REJECTS_UNKNOWN_PROMPT_CHOICES
fn skills_install_reprompts_on_unknown_destination_choice() {
    let root = temp_repo_dir("special-cli-skills-invalid-destination-choice");
    write_skills_fixture(&root);

    let output = run_special_with_input(
        &root,
        &["skills", "install", "ship-product-change"],
        "globl\nproject\n",
    );
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Enter `project`, `global`, or `custom`."));
    assert!(
        root.join(".agents/skills/ship-product-change/SKILL.md")
            .is_file()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.NON_INTERACTIVE_DESTINATION
fn skills_install_supports_non_interactive_destination_flag() {
    let root = temp_repo_dir("special-cli-skills-non-interactive-destination");
    let codex_home = root.join("codex-home");
    let custom = root.join("custom-skills");
    write_skills_fixture(&root);

    let project_output = run_special(
        &root,
        &[
            "skills",
            "install",
            "ship-product-change",
            "--destination",
            "project",
        ],
    );
    assert!(project_output.status.success());
    let project_stdout = String::from_utf8(project_output.stdout).expect("stdout should be utf-8");
    assert!(!project_stdout.contains("Select install destination:"));
    assert!(
        root.join(".agents/skills/ship-product-change/SKILL.md")
            .is_file()
    );

    let global_output = run_special_with_input_and_env(
        &root,
        &[
            "skills",
            "install",
            "define-product-specs",
            "--destination",
            "global",
        ],
        "",
        &[("CODEX_HOME", &codex_home)],
    );
    assert!(global_output.status.success());
    assert!(
        codex_home
            .join("skills/define-product-specs/SKILL.md")
            .is_file()
    );

    let custom_output = run_special(
        &root,
        &[
            "skills",
            "install",
            "validate-product-contract",
            "--destination",
            custom.to_str().expect("custom path should be utf-8"),
        ],
    );
    assert!(custom_output.status.success());
    assert!(custom.join("validate-product-contract/SKILL.md").is_file());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn global_destination_requires_global_env() {
    let root = temp_repo_dir("special-cli-skills-global-env");
    write_skills_fixture(&root);

    let overview = run_special_with_env_removed(&root, &["skills"], &["CODEX_HOME", "HOME"]);
    assert!(overview.status.success());
    let overview_stderr = String::from_utf8(overview.stderr).expect("stderr should be utf-8");
    assert!(!overview_stderr.contains("global install destination unavailable"));

    let install = run_special_with_env_removed(
        &root,
        &[
            "skills",
            "install",
            "ship-product-change",
            "--destination",
            "global",
        ],
        &["CODEX_HOME", "HOME"],
    );
    assert!(!install.status.success());
    let install_stderr = String::from_utf8(install.stderr).expect("stderr should be utf-8");
    assert!(install_stderr.contains("global install destination unavailable"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.FORCE_OVERWRITE
fn skills_install_supports_force_overwrite_without_prompt() {
    let root = temp_repo_dir("special-cli-skills-force-overwrite");
    write_skills_fixture(&root);
    let existing = root.join(".agents/skills/ship-product-change");
    fs::create_dir_all(&existing).expect("existing skill dir should be created");
    fs::write(existing.join("SKILL.md"), "existing")
        .expect("existing skill file should be written");

    let output = run_special(
        &root,
        &[
            "skills",
            "install",
            "ship-product-change",
            "--destination",
            "project",
            "--force",
        ],
    );
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(!stdout.contains("Overwrite?"));
    assert_ne!(
        fs::read_to_string(existing.join("SKILL.md")).expect("installed skill should exist"),
        "existing"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.VALIDATES_SKILL_ID_BEFORE_PROMPT
fn skills_install_rejects_unknown_skill_before_prompting() {
    let root = temp_repo_dir("special-cli-skills-invalid-id");
    write_skills_fixture(&root);

    let output = run_special(&root, &["skills", "install", "nope"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("unknown skill id `nope`"));
    assert!(!stderr.contains("interactive input required"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.PROJECT_DESTINATION
fn skills_install_project_destination_writes_into_repo_skills_dir() {
    let root = temp_repo_dir("special-cli-skills-project-destination");
    write_skills_fixture(&root);

    let output = run_special_with_input(
        &root,
        &["skills", "install", "ship-product-change"],
        "project\n",
    );
    assert!(output.status.success());

    assert!(
        root.join(".agents/skills/ship-product-change/SKILL.md")
            .is_file()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.WRITES_PROJECT_SKILLS_DIRECTORY
fn skills_writes_project_local_skills_directory() {
    let root = temp_repo_dir("special-cli-skills-project");

    let output = install_skills(&root);
    assert!(output.status.success());

    assert_eq!(
        installed_skill_ids(&root.join(".agents/skills")),
        bundled_skill_ids()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.ONE_SKILL
fn skills_install_one_skill_only_installs_the_selected_skill() {
    let root = temp_repo_dir("special-cli-skills-install-one");
    write_skills_fixture(&root);

    let output = run_special_with_input(
        &root,
        &["skills", "install", "define-product-specs"],
        "project\n",
    );
    assert!(output.status.success());

    assert!(
        root.join(".agents/skills/define-product-specs/SKILL.md")
            .is_file()
    );
    assert!(
        !root
            .join(".agents/skills/ship-product-change/SKILL.md")
            .exists()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.GLOBAL_DESTINATION
fn skills_install_supports_global_destination() {
    let root = temp_repo_dir("special-cli-skills-global");
    let codex_home = root.join("codex-home");
    write_skills_fixture(&root);

    let output = run_special_with_input_and_env(
        &root,
        &["skills", "install", "ship-product-change"],
        "global\n",
        &[("CODEX_HOME", &codex_home)],
    );
    assert!(output.status.success());

    assert!(
        codex_home
            .join("skills/ship-product-change/SKILL.md")
            .is_file()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.CUSTOM_DESTINATION
fn skills_install_supports_custom_destination() {
    let root = temp_repo_dir("special-cli-skills-custom");
    let custom = root.join("custom-skills");
    write_skills_fixture(&root);

    let input = format!("custom\n{}\n", custom.display());
    let output =
        run_special_with_input(&root, &["skills", "install", "ship-product-change"], &input);
    assert!(output.status.success());

    assert!(custom.join("ship-product-change/SKILL.md").is_file());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALL_SUBCOMMAND.OVERWRITE_PROMPT
fn skills_install_prompts_before_overwriting_existing_skill() {
    let root = temp_repo_dir("special-cli-skills-overwrite");
    write_skills_fixture(&root);
    let existing = root.join(".agents/skills/ship-product-change");
    fs::create_dir_all(&existing).expect("existing skill dir should be created");
    fs::write(existing.join("SKILL.md"), "existing")
        .expect("existing skill file should be written");

    let output = run_special_with_input(
        &root,
        &["skills", "install", "ship-product-change"],
        "project\nn\n",
    );
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("aborted skill install"));
    assert_eq!(
        fs::read_to_string(existing.join("SKILL.md")).expect("existing skill should remain"),
        "existing"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.USES_AGENT_SKILLS_LAYOUT
fn skills_use_standard_skill_directories_with_support_files() {
    let root = temp_repo_dir("special-cli-skills-layout");

    let output = install_skills(&root);
    assert!(output.status.success());

    let skills_root = root.join(".agents/skills");
    assert!(skills_root.join("ship-product-change/SKILL.md").is_file());
    assert!(
        skills_root
            .join("ship-product-change/references/change-workflow.md")
            .is_file()
    );
    assert!(
        skills_root
            .join("define-product-specs/references/spec-writing.md")
            .is_file()
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_SHIP_CHANGE_SKILL
fn skills_install_ship_change_skill() {
    let root = temp_repo_dir("special-cli-skills-ship");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(root.join(".agents/skills/ship-product-change/SKILL.md"))
        .expect("ship skill should exist");
    assert_eq!(skill, bundled_skill_markdown("ship-product-change"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_DEFINE_PRODUCT_SPECS_SKILL
fn skills_install_define_product_specs_skill() {
    let root = temp_repo_dir("special-cli-skills-define");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(root.join(".agents/skills/define-product-specs/SKILL.md"))
        .expect("define skill should exist");
    assert_eq!(skill, bundled_skill_markdown("define-product-specs"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_VALIDATE_PRODUCT_CONTRACT_SKILL
fn skills_install_validate_product_contract_skill() {
    let root = temp_repo_dir("special-cli-skills-validate");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(root.join(".agents/skills/validate-product-contract/SKILL.md"))
        .expect("validate skill should exist");
    assert_eq!(skill, bundled_skill_markdown("validate-product-contract"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_VALIDATE_ARCHITECTURE_IMPLEMENTATION_SKILL
fn skills_install_validate_architecture_implementation_skill() {
    let root = temp_repo_dir("special-cli-skills-validate-architecture");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(
        root.join(".agents/skills/validate-architecture-implementation/SKILL.md"),
    )
    .expect("validate architecture skill should exist");
    assert_eq!(
        skill,
        bundled_skill_markdown("validate-architecture-implementation")
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_USE_PROJECT_PATTERNS_SKILL
fn skills_install_use_project_patterns_skill() {
    let root = temp_repo_dir("special-cli-skills-use-patterns");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(root.join(".agents/skills/use-project-patterns/SKILL.md"))
        .expect("use-project-patterns skill should exist");
    assert_eq!(skill, bundled_skill_markdown("use-project-patterns"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_EVOLVE_MODULE_ARCHITECTURE_SKILL
fn skills_install_evolve_module_architecture_skill() {
    let root = temp_repo_dir("special-cli-skills-evolve-architecture");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(root.join(".agents/skills/evolve-module-architecture/SKILL.md"))
        .expect("evolve architecture skill should exist");
    assert_eq!(skill, bundled_skill_markdown("evolve-module-architecture"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_INSPECT_CURRENT_SPEC_STATE_SKILL
fn skills_install_inspect_current_spec_state_skill() {
    let root = temp_repo_dir("special-cli-skills-current");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(root.join(".agents/skills/inspect-current-spec-state/SKILL.md"))
        .expect("inspect-current-spec-state skill should exist");
    assert_eq!(skill, bundled_skill_markdown("inspect-current-spec-state"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INSTALLS_FIND_PLANNED_WORK_SKILL
fn skills_install_find_planned_work_skill() {
    let root = temp_repo_dir("special-cli-skills-planned");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skill = fs::read_to_string(root.join(".agents/skills/find-planned-work/SKILL.md"))
        .expect("find-planned-work skill should exist");
    assert_eq!(skill, bundled_skill_markdown("find-planned-work"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.BUNDLES_REFERENCES_FOR_PROGRESSIVE_DISCLOSURE
fn skills_bundle_reference_docs_for_progressive_disclosure() {
    let root = temp_repo_dir("special-cli-skills-references");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skills_root = root.join(".agents/skills");
    let expected_reference_files = [
        "ship-product-change/references/change-workflow.md",
        "ship-product-change/references/trigger-evals.md",
        "define-product-specs/references/spec-writing.md",
        "define-product-specs/references/trigger-evals.md",
        "evolve-module-architecture/references/trigger-evals.md",
        "use-project-patterns/references/pattern-workflow.md",
        "use-project-patterns/references/trigger-evals.md",
        "validate-architecture-implementation/references/validation-checklist.md",
        "validate-architecture-implementation/references/trigger-evals.md",
        "validate-product-contract/references/validation-checklist.md",
        "validate-product-contract/references/trigger-evals.md",
        "inspect-current-spec-state/references/state-walkthrough.md",
        "inspect-current-spec-state/references/trigger-evals.md",
        "find-planned-work/references/planned-workflow.md",
        "find-planned-work/references/trigger-evals.md",
    ];

    for relative_path in expected_reference_files {
        assert!(
            skills_root.join(relative_path).is_file(),
            "expected bundled reference file {relative_path}"
        );
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.SKILLS.COMMAND.INCLUDES_TRIGGER_EVAL_FIXTURES
fn skills_include_trigger_eval_fixtures() {
    let root = temp_repo_dir("special-cli-skills-trigger-evals");
    let output = install_skills(&root);
    assert!(output.status.success());

    let skills_root = root.join(".agents/skills");
    let trigger_files = [
        "ship-product-change/references/trigger-evals.md",
        "define-product-specs/references/trigger-evals.md",
        "evolve-module-architecture/references/trigger-evals.md",
        "use-project-patterns/references/trigger-evals.md",
        "validate-architecture-implementation/references/trigger-evals.md",
        "validate-product-contract/references/trigger-evals.md",
        "inspect-current-spec-state/references/trigger-evals.md",
        "find-planned-work/references/trigger-evals.md",
    ];

    for relative_path in trigger_files {
        let contents =
            fs::read_to_string(skills_root.join(relative_path)).expect("trigger eval should exist");
        assert!(contents.contains("## Should Trigger"));
        assert!(contents.contains("## Should Not Trigger"));
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}
