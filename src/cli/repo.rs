/**
@module SPECIAL.CLI.REPO
Repo-wide quality command boundary. This module resolves the active project root and renders repo-level quality signals without materializing the architecture tree.

@spec SPECIAL.REPO_COMMAND.EXPERIMENTAL
special repo --experimental opts into experimental repo-wide analysis surfaces that are not shown in the default repo view.
*/
// @fileimplements SPECIAL.CLI.REPO
use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::config::resolve_project_root;
use crate::modules::build_repo_document;
use crate::render::{render_lint_text, render_repo_html, render_repo_json, render_repo_text};

#[derive(Debug, Args)]
pub(super) struct RepoArgs {
    #[arg(long = "json", conflicts_with = "html")]
    json: bool,

    #[arg(long = "html", conflicts_with = "json")]
    html: bool,

    #[arg(long = "verbose")]
    verbose: bool,

    #[arg(long = "experimental")]
    experimental: bool,
}

pub(super) fn execute_repo(args: RepoArgs, current_dir: &Path) -> Result<ExitCode> {
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    let root = resolution.root.clone();
    let (document, lint) = build_repo_document(
        &root,
        &resolution.ignore_patterns,
        resolution.version,
        args.experimental,
    )?;

    if !lint.diagnostics.is_empty() {
        eprintln!("{}", render_lint_text(&lint));
    }

    if args.json {
        println!("{}", render_repo_json(&document, args.verbose)?);
    } else if args.html {
        println!("{}", render_repo_html(&document, args.verbose));
    } else {
        println!("{}", render_repo_text(&document, args.verbose));
    }

    Ok(if lint.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
