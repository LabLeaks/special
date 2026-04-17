/**
@module SPECIAL.CLI.MODULES
Architecture module command boundary. This module resolves the active project root, selects module filters and analysis options from CLI flags, and renders the resulting module view without owning module parsing or analysis rules.
*/
// @fileimplements SPECIAL.CLI.MODULES
use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::config::resolve_project_root;
use crate::model::{ModuleAnalysisOptions, ModuleFilter};
use crate::modules::build_module_document;
use crate::render::{render_module_html, render_module_json, render_module_text};

#[derive(Debug, Args)]
pub(super) struct ModulesArgs {
    module_id: Option<String>,

    #[arg(long = "all")]
    include_planned: bool,

    #[arg(long = "unsupported")]
    unsupported_only: bool,

    #[arg(long = "json", conflicts_with = "html")]
    json: bool,

    #[arg(long = "html", conflicts_with = "json")]
    html: bool,

    #[arg(long = "verbose")]
    verbose: bool,

    #[arg(long = "metrics")]
    metrics: bool,
}

pub(super) fn execute_modules(args: ModulesArgs, current_dir: &Path) -> Result<ExitCode> {
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    let root = resolution.root.clone();
    let (document, lint) = build_module_document(
        &root,
        &resolution.ignore_patterns,
        resolution.version,
        ModuleFilter {
            include_planned: args.include_planned,
            unsupported_only: args.unsupported_only,
            scope: args.module_id,
        },
        ModuleAnalysisOptions {
            coverage: args.metrics,
            metrics: args.metrics,
            experimental: false,
        },
    )?;

    if !lint.diagnostics.is_empty() {
        eprintln!("{}", crate::render::render_lint_text(&lint));
    }

    if args.json {
        println!("{}", render_module_json(&document, args.verbose)?);
    } else if args.html {
        println!("{}", render_module_html(&document, args.verbose));
    } else {
        println!("{}", render_module_text(&document, args.verbose));
    }

    Ok(if lint.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
