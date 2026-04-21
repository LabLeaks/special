/**
@module SPECIAL.CLI.SPEC
Spec and lint command behavior.

@spec SPECIAL.SPEC_COMMAND
special specs materializes the declared spec view from parsed annotations.

@spec SPECIAL.SPEC_COMMAND.FAILS_ON_ERRORS
special specs exits with an error status when annotation diagnostics include errors, even if it still prints diagnostics and best-effort rendered output.

@spec SPECIAL.SPEC_COMMAND.DEFAULT_ALL
special specs includes both current and planned items by default.

@spec SPECIAL.SPEC_COMMAND.CURRENT_ONLY
special specs --current excludes planned items.

@spec SPECIAL.SPEC_COMMAND.PLANNED_ONLY
special specs --planned shows only planned items.

@spec SPECIAL.SPEC_COMMAND.PLANNED_RELEASE_METADATA
when a planned spec declares release metadata, special specs surfaces that release string in text, json, and html output.

@spec SPECIAL.SPEC_COMMAND.DEPRECATED_METADATA
when a deprecated spec declares release metadata, special specs surfaces that release string in text, json, and html output.

@spec SPECIAL.SPEC_COMMAND.ID_SCOPE
special specs SPEC.ID scopes the materialized view to the matching spec or group node and its descendants.

@spec SPECIAL.SPEC_COMMAND.UNVERIFIED
special specs --unverified shows current items with zero verifies and zero attests.

@spec SPECIAL.SPEC_COMMAND.METRICS
special specs --metrics surfaces deeper contract analysis for the declared spec view.

@spec SPECIAL.SPEC_COMMAND.JSON
special specs --json emits the materialized spec as JSON.

@spec SPECIAL.SPEC_COMMAND.HTML
special specs --html emits the materialized spec as HTML.

@spec SPECIAL.SPEC_COMMAND.VERBOSE
special specs --verbose shows the attached verifies and attests bodies for review.

@spec SPECIAL.SPEC_COMMAND.VERBOSE.JSON
special specs --json --verbose includes attached verifies and attests bodies in JSON output.

@spec SPECIAL.SPEC_COMMAND.VERBOSE.HTML
special specs --html --verbose includes attached verifies and attests in collapsed detail blocks.

@spec SPECIAL.SPEC_COMMAND.VERBOSE.HTML.CODE_HIGHLIGHTING
special specs --html --verbose renders attached code blocks with best-effort language-sensitive highlighting.

@spec SPECIAL.LINT_COMMAND
special lint reports annotation parsing and reference errors.

@spec SPECIAL.LINT_COMMAND.UNKNOWN_VERIFY_REFS
special lint reports @verifies references to unknown spec ids.

@spec SPECIAL.LINT_COMMAND.UNKNOWN_ATTEST_REFS
special lint reports @attests and @fileattests references to unknown spec ids.

@spec SPECIAL.LINT_COMMAND.INTERMEDIATE_SPECS
special lint reports missing intermediate spec ids in dot-path hierarchies.

@spec SPECIAL.LINT_COMMAND.DUPLICATE_IDS
special lint reports duplicate node ids.

@spec SPECIAL.LINT_COMMAND.PLANNED_SCOPE
special lint reports invalid `@planned` ownership, including floating and non-adjacent markers under versioned parsing rules.

@spec SPECIAL.LINT_COMMAND.WARNINGS_DO_NOT_FAIL
special lint reports warnings without failing the command.

@spec SPECIAL.LINT_COMMAND.UNVERIFIED_EXCLUDED
special lint does not report unverified current specs.

@spec SPECIAL.LINT_COMMAND.ORPHAN_VERIFIES
special lint reports @verifies blocks that do not attach to a supported owned item.

@spec SPECIAL.LINT_COMMAND.UNKNOWN_IMPLEMENTS_REFS
special lint reports `@implements` references to unknown module ids.

@spec SPECIAL.LINT_COMMAND.INTERMEDIATE_MODULES
special lint reports missing intermediate module ids in dot-path hierarchies.
*/
// @fileimplements SPECIAL.CLI.SPEC
use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use super::status::{CommandStatus, StatusStep};
use crate::cache::{format_cache_stats_summary, reset_cache_stats, with_cache_status_notifier};
use crate::config::{RootResolution, RootSource, resolve_project_root};
use crate::index::{build_lint_report, build_spec_document};
use crate::model::{DeclaredStateFilter, Diagnostic, DiagnosticSeverity, LintReport, SpecFilter};
use crate::modules::build_module_lint_report;
use crate::render::{render_lint_text, render_spec_html, render_spec_json, render_spec_text};

#[derive(Debug, Args)]
pub(super) struct SpecArgs {
    spec_id: Option<String>,

    #[arg(
        long = "current",
        conflicts_with = "planned_only",
        help = "Show only current specs"
    )]
    current_only: bool,

    #[arg(
        long = "planned",
        conflicts_with = "current_only",
        help = "Show only planned specs"
    )]
    planned_only: bool,

    #[arg(
        short = 'u',
        long = "unverified",
        conflicts_with = "planned_only",
        help = "Show only current specs with no verifies and no attests"
    )]
    unverified_only: bool,

    #[arg(
        short = 'm',
        long = "metrics",
        help = "Show deeper contract analysis for the declared spec view"
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
        help = "Show more attached support detail within the current view"
    )]
    verbose: bool,
}

const SPEC_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("building spec view", 8),
    StatusStep::new("rendering output", 1),
];

const LINT_PLAN: &[StatusStep] = &[
    StatusStep::new("resolving project root", 1),
    StatusStep::new("building lint report", 8),
    StatusStep::new("rendering output", 1),
];

pub(super) fn execute_spec(args: SpecArgs, current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special specs", SPEC_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }
    let root = resolution.root.clone();
    let state = args.state_filter();
    let unverified_only = args.unverified_only;
    let scope = args.spec_id.clone();
    let metrics = args.metrics;
    let verbose = args.verbose;
    status.phase("building spec view");
    let (document, mut lint) = with_cache_status_notifier(status.notifier(), || {
        build_spec_document(
            &root,
            &resolution.ignore_patterns,
            resolution.version,
            SpecFilter {
                state,
                unverified_only,
                scope,
            },
            metrics,
        )
    })?;
    report_cache_stats(&status);
    add_config_lint_warnings(&mut lint, &resolution);

    if !lint.diagnostics.is_empty() {
        let rendered_lint = render_lint_text(&lint);
        eprintln!("{rendered_lint}");
    }

    status.phase("rendering output");
    let rendered = if args.json {
        render_spec_json(&document, args.verbose)?
    } else if args.html {
        render_spec_html(&document, args.verbose)
    } else {
        render_spec_text(&document, verbose)
    };
    println!("{rendered}");
    status.finish();
    Ok(if lint.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

impl SpecArgs {
    fn state_filter(&self) -> DeclaredStateFilter {
        if self.planned_only {
            DeclaredStateFilter::Planned
        } else if self.current_only || self.unverified_only {
            DeclaredStateFilter::Current
        } else {
            DeclaredStateFilter::All
        }
    }
}

pub(super) fn execute_lint(current_dir: &Path) -> Result<ExitCode> {
    let status = CommandStatus::with_plan("special lint", LINT_PLAN);
    reset_cache_stats();
    status.phase("resolving project root");
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }
    let root = resolution.root.clone();
    status.phase("building lint report");
    let mut report = with_cache_status_notifier(status.notifier(), || {
        build_lint_report(&root, &resolution.ignore_patterns, resolution.version)
    })?;
    report.diagnostics.extend(
        with_cache_status_notifier(status.notifier(), || {
            build_module_lint_report(&root, &resolution.ignore_patterns)
        })?
        .diagnostics,
    );
    report_cache_stats(&status);
    add_config_lint_warnings(&mut report, &resolution);
    normalize_report(&mut report);
    let clean = !report.has_errors();
    status.phase("rendering output");
    let rendered = render_lint_text(&report);
    println!("{rendered}");
    status.finish();
    Ok(if clean {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn report_cache_stats(status: &CommandStatus) {
    if let Some(summary) = format_cache_stats_summary() {
        status.note(&summary);
    }
}

fn add_config_lint_warnings(report: &mut LintReport, resolution: &RootResolution) {
    if resolution.version_explicit {
        return;
    }

    let (path, line, message) = match (&resolution.config_path, resolution.source) {
        (Some(path), RootSource::SpecialToml) => (
            path.clone(),
            1,
            "missing `version` in special.toml; using compatibility parsing rules; set `version = \"1\"` to use the current rules".to_string(),
        ),
        _ => (
            resolution.root.join("special.toml"),
            1,
            "no special.toml found; using compatibility parsing rules; run `special init` to create config with the current rules".to_string(),
        ),
    };

    report.diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Warning,
        path,
        line,
        message,
    });
}

fn normalize_report(report: &mut LintReport) {
    report.diagnostics.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then(left.path.cmp(&right.path))
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });
    report.diagnostics.dedup_by(|left, right| {
        left.severity == right.severity
            && left.path == right.path
            && left.line == right.line
            && left.message == right.message
    });
}
