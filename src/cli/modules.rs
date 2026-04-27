/**
@module SPECIAL.CLI.MODULES
Architecture module command boundary. This module resolves the active project root, selects module filters and analysis options from CLI flags, and renders the resulting ownership view without owning module parsing or analysis rules. This surface stays annotation-first: module metrics should expose owned implementation evidence such as coverage, complexity, quality, and dependencies without turning `special arch` into a repo-trace view.

*/
// @fileimplements SPECIAL.CLI.MODULES
use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::common::report_cache_stats;
use super::status::{CommandStatus, StatusStep};
use crate::cache::{reset_cache_stats, with_cache_status_notifier};
use crate::config::resolve_project_root;
use crate::model::{DeclaredStateFilter, ModuleAnalysisOptions, ModuleFilter};
use crate::modules::analyze::with_analysis_status_notifier;
use crate::modules::build_module_document;
use crate::render::{render_module_html, render_module_json, render_module_text};

#[derive(Debug, Args)]
pub(super) struct ModulesArgs {
    module_id: Option<String>,

    #[arg(
        long = "current",
        conflicts_with = "planned_only",
        help = "Show only current modules"
    )]
    current_only: bool,

    #[arg(
        long = "planned",
        conflicts_with = "current_only",
        help = "Show only planned modules"
    )]
    planned_only: bool,

    #[arg(
        short = 'u',
        long = "unimplemented",
        conflicts_with = "planned_only",
        help = "Show only current modules with no direct @implements attachments"
    )]
    unimplemented_only: bool,

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
        help = "Show more implementation detail within the current view"
    )]
    verbose: bool,

    #[arg(
        short = 'm',
        long = "metrics",
        help = "Show deeper implementation analysis for the current architecture view; unscoped runs summarize first"
    )]
    metrics: bool,
}

const ARCH_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("building architecture view", 8),
    StatusStep::new("rendering output", 1),
];

// @applies COMMAND.PROJECTION_PIPELINE
pub(super) fn execute_modules(args: ModulesArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special arch", ARCH_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    let root = resolution.root.clone();
    let state = args.state_filter();
    let unimplemented_only = args.unimplemented_only;
    let scope = args.module_id.clone();
    let verbose = args.verbose;
    let metrics = args.metrics;
    status.phase("building architecture view");
    let analysis_notifier = status.notifier();
    let (document, lint) = with_cache_status_notifier(status.notifier(), || {
        with_analysis_status_notifier(analysis_notifier, || {
            build_module_document(
                &root,
                &resolution.ignore_patterns,
                resolution.version,
                ModuleFilter {
                    state,
                    unimplemented_only,
                    scope,
                },
                ModuleAnalysisOptions {
                    coverage: metrics,
                    metrics,
                    traceability: false,
                },
            )
        })
    })?;
    report_cache_stats(&status);

    if !lint.diagnostics.is_empty() {
        let rendered_lint = crate::render::render_lint_text(&lint);
        eprintln!("{rendered_lint}");
    }

    status.phase("rendering output");
    let rendered = if args.json {
        render_module_json(&document, args.verbose)?
    } else if args.html {
        render_module_html(&document, args.verbose)
    } else {
        render_module_text(&document, verbose)
    };
    println!("{rendered}");
    status.finish();

    Ok(if lint.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

impl ModulesArgs {
    fn state_filter(&self) -> DeclaredStateFilter {
        if self.planned_only {
            DeclaredStateFilter::Planned
        } else if self.current_only || self.unimplemented_only {
            DeclaredStateFilter::Current
        } else {
            DeclaredStateFilter::All
        }
    }
}
