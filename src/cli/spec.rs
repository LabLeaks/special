/**
@module SPECIAL.CLI.SPEC
Spec and lint command behavior.

@spec SPECIAL.SPEC_COMMAND
special specs materializes the current live spec view from parsed annotations.

@spec SPECIAL.SPEC_COMMAND.FAILS_ON_ERRORS
special specs exits with an error status when annotation diagnostics include errors, even if it still prints diagnostics and best-effort rendered output.

@spec SPECIAL.SPEC_COMMAND.SINGULAR_ALIAS
special spec remains accepted as an alias for special specs.

@spec SPECIAL.SPEC_COMMAND.LIVE_ONLY
special specs excludes planned items by default.

@spec SPECIAL.SPEC_COMMAND.ALL
special specs --all includes planned items.

@spec SPECIAL.SPEC_COMMAND.PLANNED_RELEASE_METADATA
when a planned spec declares release metadata, special specs surfaces that release string in text, json, and html output.

@spec SPECIAL.SPEC_COMMAND.DEPRECATED_METADATA
when a deprecated spec declares release metadata, special specs surfaces that release string in text, json, and html output.

@spec SPECIAL.SPEC_COMMAND.ID_SCOPE
special specs SPEC.ID scopes the materialized view to the matching spec or group node and its descendants.

@spec SPECIAL.SPEC_COMMAND.UNSUPPORTED
special specs --unsupported shows live items with zero verifies and zero attests.

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

@spec SPECIAL.LINT_COMMAND.UNSUPPORTED_EXCLUDED
special lint does not report unsupported live specs.

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

use crate::config::{RootResolution, RootSource, resolve_project_root};
use crate::index::{build_lint_report, build_spec_document};
use crate::model::{Diagnostic, DiagnosticSeverity, LintReport, SpecFilter};
use crate::modules::build_module_lint_report;
use crate::render::{render_lint_text, render_spec_html, render_spec_json, render_spec_text};

#[derive(Debug, Args)]
pub(super) struct SpecArgs {
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

pub(super) fn execute_spec(args: SpecArgs, current_dir: &Path) -> Result<ExitCode> {
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }
    let root = resolution.root.clone();
    let (document, mut lint) = build_spec_document(
        &root,
        &resolution.ignore_patterns,
        resolution.version,
        SpecFilter {
            include_planned: args.include_all,
            unsupported_only: args.unsupported_only,
            scope: args.spec_id,
        },
    )?;
    add_config_lint_warnings(&mut lint, &resolution);

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
    Ok(if lint.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

pub(super) fn execute_lint(current_dir: &Path) -> Result<ExitCode> {
    let resolution = resolve_project_root(current_dir)?;
    if let Some(warning) = resolution.warning() {
        eprintln!("{warning}");
    }
    let root = resolution.root.clone();
    let mut report = build_lint_report(&root, &resolution.ignore_patterns, resolution.version)?;
    report
        .diagnostics
        .extend(build_module_lint_report(&root, &resolution.ignore_patterns)?.diagnostics);
    add_config_lint_warnings(&mut report, &resolution);
    normalize_report(&mut report);
    let clean = !report.has_errors();
    println!("{}", render_lint_text(&report));
    Ok(if clean {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
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
