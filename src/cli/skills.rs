/**
@module SPECIAL.CLI.SKILLS
User-facing `special skills` command boundary in `src/cli/skills.rs`. This module owns overview rendering, destination prompting, and install entrypoint behavior without owning the bundled skill catalog itself.
*/
// @fileimplements SPECIAL.CLI.SKILLS
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail};
use askama::Template;
use clap::{Args, Subcommand};

use crate::config::resolve_project_root;
use crate::skills::{
    BundledSkill, bundled_skill, bundled_skills, conflicting_skill_paths, install_bundled_skills,
    primary_skill_contents, resolve_global_skills_root,
};

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub(super) struct SkillsArgs {
    skill_id: Option<String>,

    #[command(subcommand)]
    command: Option<SkillsCommand>,
}

#[derive(Debug, Subcommand)]
enum SkillsCommand {
    #[command(about = "Install one or all bundled skills after prompting for destination")]
    Install(SkillsInstallArgs),
}

#[derive(Debug, Args)]
struct SkillsInstallArgs {
    skill_id: Option<String>,

    #[arg(
        long = "destination",
        value_name = "DESTINATION",
        help = "Install destination: `project`, `global`, or a custom path"
    )]
    destination: Option<String>,

    #[arg(
        long = "force",
        help = "Overwrite conflicting installed skill directories without interactive confirmation"
    )]
    force: bool,
}

#[derive(Debug, Clone)]
struct ProjectSkillsDestination {
    path: Option<PathBuf>,
    warning: Option<String>,
    unavailable_reason: Option<String>,
}

impl ProjectSkillsDestination {
    fn display(&self) -> String {
        display_destination_path(
            self.path.as_deref(),
            self.unavailable_reason.as_deref(),
            "failed to resolve project root",
        )
    }

    fn require_path(&self) -> Result<PathBuf> {
        match &self.path {
            Some(path) => {
                if let Some(warning) = &self.warning {
                    eprintln!("{warning}");
                }
                Ok(path.clone())
            }
            None => bail!(
                "project install destination unavailable: {}",
                self.unavailable_reason
                    .as_deref()
                    .unwrap_or("failed to resolve project root")
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct GlobalSkillsDestination {
    path: Option<PathBuf>,
    unavailable_reason: Option<String>,
}

impl GlobalSkillsDestination {
    fn display(&self) -> String {
        display_destination_path(
            self.path.as_deref(),
            self.unavailable_reason.as_deref(),
            "failed to resolve global skills root",
        )
    }

    fn require_path(&self) -> Result<PathBuf> {
        match &self.path {
            Some(path) => Ok(path.clone()),
            None => bail!(
                "global install destination unavailable: {}",
                self.unavailable_reason
                    .as_deref()
                    .unwrap_or("failed to resolve global skills root")
            ),
        }
    }
}

enum InstallDestination {
    Project,
    Global,
    Custom(PathBuf),
}

enum PromptDestinationChoice {
    Project,
    Global,
    Custom,
}

const SKILLS_OVERVIEW_INTRO: [&str; 2] = [
    "special skills",
    "Print bundled skill help or install bundled skills.",
];

const SKILLS_COMMAND_SHAPES: [(&str, &str); 5] = [
    (
        "special skills",
        "Show this overview and the available bundled skill ids.",
    ),
    (
        "special skills SKILL_ID",
        "Print one bundled skill's primary SKILL.md to stdout.",
    ),
    (
        "special skills install [SKILL_ID]",
        "Install one or all bundled skills after prompting for destination.",
    ),
    (
        "special skills install [SKILL_ID] --destination DESTINATION",
        "Install one or all bundled skills without interactive destination prompts.",
    ),
    (
        "special skills install [SKILL_ID] --destination DESTINATION --force",
        "Overwrite conflicting installed skill directories without interactive confirmation.",
    ),
];

const SKILLS_INSTALL_DESTINATIONS: [(&str, &str); 3] = [
    (
        "project",
        "install into the current repository's .agents/skills/ directory",
    ),
    (
        "global",
        "install into $CODEX_HOME/skills, or ~/.codex/skills when CODEX_HOME is unset",
    ),
    (
        "custom",
        "pass --destination PATH or prompt for a destination path",
    ),
];

#[derive(Template)]
#[template(path = "cli/skills_overview.txt", escape = "none")]
struct SkillsOverviewTemplate<'a> {
    intro: &'a [&'a str; 2],
    command_shapes: &'a [(&'a str, &'a str); 5],
    destinations: &'a [(&'a str, &'a str); 3],
    skills: &'a [BundledSkill],
}

impl InstallDestination {
    fn parse_cli_value(value: &str) -> Result<Self> {
        match value.trim() {
            "project" => Ok(Self::Project),
            "global" => Ok(Self::Global),
            "custom" => bail!("`--destination custom` is ambiguous; pass an explicit path instead"),
            path if !path.is_empty() => Ok(Self::Custom(PathBuf::from(path))),
            _ => bail!("`--destination` requires `project`, `global`, or a custom path"),
        }
    }

    fn resolve(
        &self,
        project_destination: &ProjectSkillsDestination,
        global_destination: &GlobalSkillsDestination,
    ) -> Result<PathBuf> {
        match self {
            Self::Project => project_destination.require_path(),
            Self::Global => global_destination.require_path(),
            Self::Custom(path) => Ok(path.clone()),
        }
    }
}

impl PromptDestinationChoice {
    fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "1" | "project" => Ok(Self::Project),
            "2" | "global" => Ok(Self::Global),
            "3" | "custom" => Ok(Self::Custom),
            _ => bail!("Enter `project`, `global`, or `custom`."),
        }
    }
}

pub(super) fn execute_skills(args: SkillsArgs, current_dir: &Path) -> Result<ExitCode> {
    match (args.skill_id.as_deref(), args.command) {
        (Some(skill_id), None) => {
            println!("{}", primary_skill_contents(skill_id)?);
            Ok(ExitCode::SUCCESS)
        }
        (None, None) => {
            println!("{}", render_skills_overview());
            Ok(ExitCode::SUCCESS)
        }
        (None, Some(SkillsCommand::Install(install_args))) => {
            if let Some(skill_id) = install_args.skill_id.as_deref()
                && bundled_skill(skill_id).is_none()
            {
                bail!("unknown skill id `{skill_id}`");
            }

            let project_destination = resolve_project_skills_destination(current_dir);
            let global_destination = resolve_global_skills_destination();
            let destination_root = match install_args.destination.as_deref() {
                Some(destination) => InstallDestination::parse_cli_value(destination)?
                    .resolve(&project_destination, &global_destination)?,
                None => prompt_install_destination(&project_destination, &global_destination)?,
            };
            handle_skill_overwrite_prompts(
                &destination_root,
                install_args.skill_id.as_deref(),
                install_args.force,
            )?;
            let installed =
                install_bundled_skills(&destination_root, install_args.skill_id.as_deref(), true)?;
            println!(
                "Installed {installed} skill{} into {}",
                if installed == 1 { "" } else { "s" },
                destination_root.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        (Some(_), Some(_)) => bail!(
            "choose either `special skills SKILL_ID` or `special skills install [SKILL_ID]`, not both"
        ),
    }
}

fn resolve_project_skills_destination(current_dir: &Path) -> ProjectSkillsDestination {
    match resolve_project_root(current_dir) {
        Ok(resolution) => ProjectSkillsDestination {
            path: Some(resolution.root.join(".agents/skills")),
            warning: resolution.warning(),
            unavailable_reason: None,
        },
        Err(err) => ProjectSkillsDestination {
            path: None,
            warning: None,
            unavailable_reason: Some(err.to_string()),
        },
    }
}

fn resolve_global_skills_destination() -> GlobalSkillsDestination {
    match resolve_global_skills_root() {
        Ok(path) => GlobalSkillsDestination {
            path: Some(path),
            unavailable_reason: None,
        },
        Err(err) => GlobalSkillsDestination {
            path: None,
            unavailable_reason: Some(err.to_string()),
        },
    }
}

fn render_skills_overview() -> String {
    SkillsOverviewTemplate {
        intro: &SKILLS_OVERVIEW_INTRO,
        command_shapes: &SKILLS_COMMAND_SHAPES,
        destinations: &SKILLS_INSTALL_DESTINATIONS,
        skills: bundled_skills(),
    }
    .render()
    .expect("skills overview template should render")
}

fn display_destination_path(
    path: Option<&Path>,
    unavailable_reason: Option<&str>,
    fallback_reason: &str,
) -> String {
    match path {
        Some(path) => path.display().to_string(),
        None => format!(
            "unavailable ({})",
            unavailable_reason.unwrap_or(fallback_reason)
        ),
    }
}

fn prompt_install_destination(
    project_destination: &ProjectSkillsDestination,
    global_destination: &GlobalSkillsDestination,
) -> Result<PathBuf> {
    println!("Select install destination:");
    println!("  project  {}", project_destination.display());
    println!("  global   {}", global_destination.display());
    println!("  custom   prompt for a destination path");

    loop {
        let choice = prompt_line("Destination [project/global/custom]: ")?;
        match PromptDestinationChoice::parse(&choice) {
            Ok(PromptDestinationChoice::Project) => return project_destination.require_path(),
            Ok(PromptDestinationChoice::Global) => return global_destination.require_path(),
            Ok(PromptDestinationChoice::Custom) => {
                let custom = prompt_line("Custom destination path: ")?;
                let trimmed = custom.trim();
                if trimmed.is_empty() {
                    println!("Enter a non-empty destination path.");
                    continue;
                }
                return Ok(PathBuf::from(trimmed));
            }
            Err(_) => println!("Enter `project`, `global`, or `custom`."),
        }
    }
}

fn handle_skill_overwrite_prompts(
    destination_root: &Path,
    skill_id: Option<&str>,
    force: bool,
) -> Result<()> {
    if force {
        return Ok(());
    }
    for (skill, path) in conflicting_skill_paths(destination_root, skill_id)? {
        let answer = prompt_line(&format!(
            "Skill `{}` already exists at `{}`. Overwrite? [y/N]: ",
            skill.id,
            path.display()
        ))?;
        let trimmed = answer.trim().to_ascii_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            bail!("aborted skill install");
        }
    }
    Ok(())
}

fn prompt_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input)?;
    if bytes_read == 0 {
        bail!("interactive input required");
    }
    Ok(input)
}
