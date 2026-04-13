use std::env;
use std::fs;
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};

use crate::config::resolve_project_root;
use crate::index::{build_lint_report, build_spec_document};
use crate::model::SpecFilter;
use crate::render::{render_lint_text, render_spec_html, render_spec_json, render_spec_text};

#[derive(Debug, Parser)]
#[command(
    name = "special",
    bin_name = "special",
    about = "Repo-native claim-and-support materializer",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Spec(SpecArgs),
    Lint,
    Init,
}

#[derive(Debug, Args)]
struct SpecArgs {
    spec_id: Option<String>,

    #[arg(long = "all")]
    include_all: bool,

    #[arg(long = "unsupported")]
    unsupported_only: bool,

    #[arg(long = "json", conflicts_with = "html")]
    json: bool,

    #[arg(long = "html", conflicts_with = "json")]
    html: bool,

    #[arg(long = "verbose")]
    verbose: bool,
}

pub fn run_from_env() -> ExitCode {
    let cli = match Cli::try_parse() {
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
        Command::Init => {
            let config_path = current_dir.join("special.toml");
            if config_path.exists() {
                bail!("special.toml already exists at `{}`", config_path.display());
            }

            fs::write(&config_path, "root = \".\"\n")?;
            println!("Created {}", config_path.display());
            Ok(ExitCode::SUCCESS)
        }
        Command::Spec(args) => {
            let resolution = resolve_project_root(&current_dir)?;
            if let Some(warning) = resolution.warning() {
                eprintln!("{warning}");
            }
            let root = resolution.root;
            let (document, lint) = build_spec_document(
                &root,
                SpecFilter {
                    include_planned: args.include_all,
                    unsupported_only: args.unsupported_only,
                    scope: args.spec_id,
                },
            )?;

            if !lint.diagnostics.is_empty() {
                eprintln!("{}", render_lint_text(&lint));
            }

            if args.json {
                println!("{}", render_spec_json(&document, args.verbose)?);
            } else if args.html {
                println!("{}", render_spec_html(&document, args.verbose));
            } else {
                println!("{}", render_spec_text(&document, args.verbose));
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Lint => {
            let resolution = resolve_project_root(&current_dir)?;
            if let Some(warning) = resolution.warning() {
                eprintln!("{warning}");
            }
            let root = resolution.root;
            let report = build_lint_report(&root)?;
            let clean = report.diagnostics.is_empty();
            println!("{}", render_lint_text(&report));
            Ok(if clean {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            })
        }
    }
}
