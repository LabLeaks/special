/**
@module SPECIAL.CLI.REPO
Code-health command boundary. This module resolves the active project root and renders code-first health signals without requiring module ownership as the primary lens. Traceability belongs here by default because this is the code line-item surface, and it should say plainly when a required language tool is not installed.

@spec SPECIAL.HEALTH_COMMAND.METRICS
special health --metrics surfaces deeper code-health analysis for the current view.
*/
// @fileimplements SPECIAL.CLI.REPO
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::Args;

use super::common::{report_cache_stats, resolve_cli_paths};
use super::status::{CommandStatus, StatusStep};
use crate::cache::{reset_cache_stats, with_cache_status_notifier};
use crate::config::resolve_project_root;
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::modules::{
    RepoDocumentOptions, analyze::with_analysis_status_notifier, build_repo_document,
};
use crate::render::{render_lint_text, render_repo_html, render_repo_json, render_repo_text};

#[derive(Debug, Args)]
pub(super) struct HealthArgs {
    #[arg(value_name = "PATH", hide = true)]
    positional_paths: Vec<PathBuf>,

    #[arg(
        long = "target",
        value_name = "PATH",
        help = "Limit the current health view to one file or subtree"
    )]
    targets: Vec<PathBuf>,

    #[arg(
        long = "within",
        value_name = "PATH",
        help = "Limit the analysis corpus used to build the current health view"
    )]
    within: Vec<PathBuf>,

    #[arg(
        long = "symbol",
        value_name = "SYMBOL",
        help = "Narrow the current health view to one symbol in the scoped file"
    )]
    symbol: Option<String>,

    #[arg(
        short = 'm',
        long = "metrics",
        help = "Show deeper analysis for the current health view with grouped counts first"
    )]
    metrics: bool,

    #[arg(
        long = "json",
        conflicts_with = "html",
        help = "Render the view as JSON"
    )]
    json: bool,

    #[arg(
        long = "html",
        conflicts_with = "json",
        help = "Render the view as HTML"
    )]
    html: bool,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Show more item-level detail within the current health view"
    )]
    verbose: bool,
}

const HEALTH_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("discovering analyzable files", 2),
    StatusStep::new("building health view", 6),
    StatusStep::new("rendering output", 1),
];

// @applies COMMAND.PROJECTION_PIPELINE
pub(super) fn execute_health(args: HealthArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special health", HEALTH_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    if !args.positional_paths.is_empty() {
        bail!(
            "health path scopes must use --target PATH; try `special health --target {}`",
            args.positional_paths[0].display()
        );
    }
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    let root = resolution.root.clone();
    let target_paths = resolve_cli_paths(current_dir, &args.targets);
    let within_paths = resolve_cli_paths(current_dir, &args.within);
    let view_scope_paths = if target_paths.is_empty() && !within_paths.is_empty() {
        within_paths.clone()
    } else {
        target_paths.clone()
    };
    if args.symbol.is_some() && target_paths.len() != 1 {
        bail!("--symbol requires exactly one --target path");
    }

    status.phase("discovering analyzable files");
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &resolution.ignore_patterns,
    })?;
    status.note(&format!(
        "discovered {} source files and {} markdown files",
        discovered.source_files.len(),
        discovered.markdown_files.len()
    ));
    if !view_scope_paths.is_empty() {
        let scoped_file_count = discovered
            .source_files
            .iter()
            .filter(|path| scope_matches(path, &view_scope_paths))
            .count();
        let symbol_suffix = args
            .symbol
            .as_deref()
            .map(|symbol| format!(", symbol `{symbol}`"))
            .unwrap_or_default();
        status.note(&format!(
            "target covers {} source files across {} path(s){}",
            scoped_file_count,
            view_scope_paths.len(),
            symbol_suffix
        ));
    }
    if !within_paths.is_empty() {
        let within_file_count = discovered
            .source_files
            .iter()
            .filter(|path| scope_matches(path, &within_paths))
            .count();
        status.note(&format!(
            "analysis corpus covers {} source files across {} path(s)",
            within_file_count,
            within_paths.len()
        ));
    }

    status.phase("building health view");
    let cache_notifier = status.notifier();
    let analysis_notifier = status.notifier();
    let (document, lint) = with_cache_status_notifier(cache_notifier, || {
        with_analysis_status_notifier(analysis_notifier, || {
            build_repo_document(
                &root,
                &resolution.ignore_patterns,
                resolution.version,
                RepoDocumentOptions {
                    metrics: args.metrics,
                    health_ignore_unexplained_patterns: &resolution
                        .health_ignore_unexplained_patterns,
                    target_scope_paths: (!target_paths.is_empty())
                        .then_some(target_paths.as_slice()),
                    within_scope_paths: (!within_paths.is_empty())
                        .then_some(within_paths.as_slice()),
                    symbol: args.symbol.as_deref(),
                },
            )
        })
    })?;
    report_cache_stats(&status);

    if !lint.diagnostics.is_empty() {
        let rendered_lint = render_lint_text(&lint);
        eprintln!("{rendered_lint}");
    }

    status.phase("rendering output");
    let rendered = if args.json {
        render_repo_json(&document, args.verbose)?
    } else if args.html {
        render_repo_html(&document, args.verbose)
    } else {
        render_repo_text(&document, args.verbose)
    };
    println!("{rendered}");
    status.finish();

    Ok(if lint.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn scope_matches(path: &Path, scope_paths: &[PathBuf]) -> bool {
    scope_paths.iter().any(|scope| {
        if scope.is_dir() {
            path.starts_with(scope)
        } else {
            path == scope || path.starts_with(scope)
        }
    })
}
