/**
@module SPECIAL.CLI.PATTERNS
Pattern command behavior.

*/
// @fileimplements SPECIAL.CLI.PATTERNS
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::common::{report_cache_stats, resolve_cli_paths};
use super::status::{CommandStatus, StatusStep};
use crate::cache::{reset_cache_stats, with_cache_status_notifier};
use crate::config::resolve_project_root;
use crate::model::PatternFilter;
use crate::patterns::build_pattern_document;
use crate::render::{render_pattern_json, render_pattern_text};

#[derive(Debug, Args)]
pub(super) struct PatternsArgs {
    pattern_id: Option<String>,

    #[arg(
        short = 'm',
        long = "metrics",
        help = "Show pattern definition and source-application metrics"
    )]
    metrics: bool,

    #[arg(
        long = "target",
        value_name = "PATH",
        requires = "metrics",
        help = "Limit advisory pattern metric candidates to one file or subtree"
    )]
    targets: Vec<PathBuf>,

    #[arg(
        long = "within",
        value_name = "PATH",
        requires = "metrics",
        help = "Limit advisory pattern metric comparison to one file or subtree"
    )]
    within: Vec<PathBuf>,

    #[arg(
        long = "symbol",
        value_name = "SYMBOL",
        requires = "metrics",
        help = "Limit advisory pattern metric candidates to one source item name"
    )]
    symbol: Option<String>,

    #[arg(long = "json", help = "Render the view as JSON")]
    json: bool,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Show pattern definitions and application bodies"
    )]
    verbose: bool,
}

const PATTERN_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("building pattern view", 8),
    StatusStep::new("rendering output", 1),
];

// @applies COMMAND.PROJECTION_PIPELINE
pub(super) fn execute_patterns(args: PatternsArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special patterns", PATTERN_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    status.phase("building pattern view");
    let (document, lint) = with_cache_status_notifier(status.notifier(), || {
        build_pattern_document(
            &resolution.root,
            &resolution.ignore_patterns,
            PatternFilter {
                scope: args.pattern_id.clone(),
                metrics: args.metrics,
                target_paths: resolve_cli_paths(current_dir, &args.targets),
                comparison_paths: resolve_cli_paths(current_dir, &args.within),
                symbol: args.symbol.clone(),
            },
            resolution.pattern_benchmarks,
        )
    })?;
    report_cache_stats(&status);

    if !lint.diagnostics.is_empty() {
        let rendered_lint = crate::render::render_lint_text(&lint);
        eprintln!("{rendered_lint}");
    }

    status.phase("rendering output");
    let rendered = if args.json {
        render_pattern_json(&document, args.verbose)?
    } else {
        render_pattern_text(&document, args.verbose)
    };
    println!("{rendered}");
    status.finish();

    Ok(if lint.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}
