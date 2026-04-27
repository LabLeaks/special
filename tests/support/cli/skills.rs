/**
@module SPECIAL.TESTS.SUPPORT.CLI.SKILLS
Bundled skills fixture and output helpers for CLI integration tests.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI.SKILLS
use std::fs;
use std::path::{Path, PathBuf};

use super::run_special_with_input;

pub fn write_skills_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
}

pub fn write_invalid_skills_root_fixture(root: &Path) {
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \"missing\"\n",
    )
    .expect("special.toml should be written");
}

pub fn install_skills(root: &Path) -> std::process::Output {
    write_skills_fixture(root);
    run_special_with_input(root, &["skills", "install"], "project\n")
}

pub fn bundled_skill_ids() -> Vec<&'static str> {
    vec![
        "define-product-specs",
        "evolve-module-architecture",
        "find-planned-work",
        "inspect-current-spec-state",
        "ship-product-change",
        "use-project-patterns",
        "validate-architecture-implementation",
        "validate-product-contract",
    ]
}

pub fn bundled_skill_markdown(skill_id: &str) -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir
        .join("templates/skills")
        .join(skill_id)
        .join("SKILL.md");
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("skill template {path:?} should exist: {err}"))
}

pub fn skills_command_shape_lines(output: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_section = false;

    for line in output.lines() {
        if line.trim() == "Command shapes:" {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.trim().is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if line.starts_with("  special ") {
            lines.push(line.trim().to_string());
        }
    }

    lines
}

pub fn skills_install_destination_lines(output: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_section = false;

    for line in output.lines() {
        if line.trim() == "Install destinations:" {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.trim().is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if line.starts_with("  ") {
            lines.push(line.trim().to_string());
        }
    }

    lines
}

pub fn skills_install_destinations(output: &str) -> Vec<(String, String)> {
    skills_install_destination_lines(output)
        .into_iter()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let mut parts = trimmed.split_whitespace();
            let name = parts.next()?;
            let summary = trimmed[name.len()..].trim_start();
            Some((name.to_string(), summary.to_string()))
        })
        .collect()
}
