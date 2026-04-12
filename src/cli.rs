use std::env;
use std::process::ExitCode;

use anyhow::Result;
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
}

#[derive(Debug, Args)]
struct SpecArgs {
    #[arg(long = "all")]
    include_all: bool,

    #[arg(long = "unsupported")]
    unsupported_only: bool,

    #[arg(long = "json", conflicts_with = "html")]
    json: bool,

    #[arg(long = "html", conflicts_with = "json")]
    html: bool,
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
    let resolution = resolve_project_root(&current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }
    let root = resolution.root;

    match cli.command {
        Command::Spec(args) => {
            let (document, lint) = build_spec_document(
                &root,
                SpecFilter {
                    include_planned: args.include_all,
                    unsupported_only: args.unsupported_only,
                },
            )?;

            if !lint.diagnostics.is_empty() {
                eprintln!("{}", render_lint_text(&lint));
            }

            if args.json {
                println!("{}", render_spec_json(&document)?);
            } else if args.html {
                println!("{}", render_spec_html(&document));
            } else {
                println!("{}", render_spec_text(&document));
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Lint => {
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
