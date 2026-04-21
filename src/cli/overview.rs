/**
@module SPECIAL.CLI.OVERVIEW
Root overview command behavior for bare `special`. This module should stay compact and direct users toward `special specs`, `special arch`, `special health`, and `special lint` rather than trying to replace them with another large projection.

@group SPECIAL.OVERVIEW
bare `special` compact overview surface.

@spec SPECIAL.OVERVIEW.COMMAND
special with no subcommand prints a compact overview covering lint, specs, architecture, and code health.

@spec SPECIAL.OVERVIEW.COMMAND.JSON
special --json emits the compact root overview as JSON.

@spec SPECIAL.OVERVIEW.COMMAND.FAILS_ON_LINT_ERRORS
special with no subcommand exits with an error status when the combined lint health in the active repo includes errors.
*/
// @fileimplements SPECIAL.CLI.OVERVIEW
use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;

use super::status::{CommandStatus, StatusStep};
use crate::cache::{format_cache_stats_summary, reset_cache_stats, with_cache_status_notifier};
use crate::config::resolve_project_root;
use crate::overview::build_overview_document;
use crate::render::{render_overview_json, render_overview_text};

pub(super) struct OverviewArgs {
    pub json: bool,
}

const OVERVIEW_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("building overview", 8),
    StatusStep::new("rendering output", 1),
];

pub(super) fn execute_overview(args: OverviewArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special", OVERVIEW_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }

    status.phase("building overview");
    let document = with_cache_status_notifier(status.notifier(), || {
        build_overview_document(
            &resolution.root,
            &resolution.ignore_patterns,
            resolution.version,
        )
    })?;
    report_cache_stats(&status);

    status.phase("rendering output");
    let rendered = if args.json {
        render_overview_json(&document)?
    } else {
        render_overview_text(&document)
    };
    println!("{rendered}");
    status.finish();

    Ok(if document.lint.errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn report_cache_stats(status: &CommandStatus) {
    if let Some(summary) = format_cache_stats_summary() {
        status.note(&summary);
    }
}
