/**
@module SPECIAL.CLI
Thin command-line boundary for the Rust application.

@group SPECIAL.HELP
special help surface and top-level command descriptions.

@group SPECIAL.VERSION
special top-level version output surface.

@spec SPECIAL.HELP.TOP_LEVEL_COMMANDS
special `--help` lists the top-level commands with purpose-oriented summaries.

@spec SPECIAL.HELP.SUBCOMMAND
special `help` prints the same top-level help surface as `special --help`.

@spec SPECIAL.HELP.SPECS_COMMAND_PLURAL_PRIMARY
special help text presents the semantic spec command as `special specs`.

@spec SPECIAL.HELP.MODULES_COMMAND_PLURAL_PRIMARY
special help text presents the architecture module command as `special modules`.

@spec SPECIAL.HELP.REPO_COMMAND
special help text presents the repo-wide quality command as `special repo`.

@spec SPECIAL.HELP.SKILLS_COMMAND_SHAPES
special help text explains the `skills`, `skills SKILL_ID`, and `skills install [SKILL_ID]` command shapes.

@spec SPECIAL.VERSION.FLAGS
special `-v` and `--version` print the current CLI version and exit successfully.
*/
// @fileimplements SPECIAL.CLI
use std::env;
use std::ffi::OsStr;
use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

mod init;
mod modules;
mod repo;
mod skills;
mod spec;

use self::init::execute_init;
use self::modules::{ModulesArgs, execute_modules};
use self::repo::{RepoArgs, execute_repo};
use self::skills::{SkillsArgs, execute_skills};
use self::spec::{SpecArgs, execute_lint, execute_spec};

#[derive(Debug, Parser)]
#[command(
    name = "special",
    bin_name = "special",
    about = "Repo-native semantic spec and skill tool",
    after_help = "Examples:\n  special specs\n  special specs SPECIAL.CONFIG --verbose\n  special modules\n  special modules SPECIAL.PARSER --verbose\n  special repo\n  special repo --verbose\n  special lint\n  special init\n  special skills\n  special skills ship-product-change\n  special skills install\n  special skills install define-product-specs",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(
        name = "specs",
        alias = "spec",
        about = "Materialize and inspect semantic specs"
    )]
    Specs(SpecArgs),
    #[command(
        name = "modules",
        alias = "module",
        about = "Materialize and inspect architecture modules"
    )]
    Modules(ModulesArgs),
    #[command(name = "repo", about = "Inspect repo-wide quality signals")]
    Repo(RepoArgs),
    #[command(about = "Check annotations and references for structural problems")]
    Lint,
    #[command(about = "Create a starter special.toml in the current directory")]
    Init,
    #[command(
        about = "List bundled skills, print one skill, or install skills",
        long_about = "Use `special skills` to see available bundled skills and command shapes.\n\nCommand shapes:\n  special skills\n  special skills SKILL_ID\n  special skills install [SKILL_ID]\n  special skills install [SKILL_ID] --destination DESTINATION\n  special skills install [SKILL_ID] --destination DESTINATION --force"
    )]
    Skills(SkillsArgs),
}

pub fn run_from_env() -> ExitCode {
    let args: Vec<_> = env::args_os().collect();

    if let Some(code) = handle_top_level_shortcuts(&args) {
        return code;
    }

    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(err) => {
            let code = err.exit_code();
            let _ = err.print();
            return ExitCode::from(code.clamp(0, u8::MAX.into()) as u8);
        }
    };

    match execute(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::from(1)
        }
    }
}

fn execute(cli: Cli) -> Result<ExitCode> {
    let current_dir = env::current_dir()?;

    match cli.command {
        Command::Init => execute_init(&current_dir),
        Command::Modules(args) => execute_modules(args, &current_dir),
        Command::Repo(args) => execute_repo(args, &current_dir),
        Command::Skills(args) => execute_skills(args, &current_dir),
        Command::Specs(args) => execute_spec(args, &current_dir),
        Command::Lint => execute_lint(&current_dir),
    }
}

fn handle_top_level_shortcuts(args: &[std::ffi::OsString]) -> Option<ExitCode> {
    if args.len() != 2 {
        return None;
    }

    match args[1].as_os_str() {
        arg if arg == OsStr::new("help") => Some(print_top_level_help()),
        arg if arg == OsStr::new("-v") || arg == OsStr::new("--version") => {
            println!("special {}", env!("CARGO_PKG_VERSION"));
            Some(ExitCode::SUCCESS)
        }
        _ => None,
    }
}

fn print_top_level_help() -> ExitCode {
    let mut cmd = Cli::command();
    match cmd.print_help() {
        Ok(()) => {
            println!();
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}
