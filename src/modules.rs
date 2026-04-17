use std::collections::BTreeSet;
/**
@module SPECIAL.MODULES
Coordinates architecture parsing, lint, and tree materialization over discovered source-local and markdown-backed `@module`/`@area` declarations.
*/
// @fileimplements SPECIAL.MODULES
use std::path::Path;

use anyhow::Result;

use crate::config::SpecialVersion;
use crate::model::{
    LintReport, ModuleAnalysisOptions, ModuleDocument, ModuleFilter, ParsedArchitecture,
    RepoDocument,
};
use crate::parser::ParseDialect;

pub(crate) mod analyze;
mod lint;
mod materialize;
mod parse;
mod parse_markdown;

pub fn build_module_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    filter: ModuleFilter,
    analysis_options: ModuleAnalysisOptions,
) -> Result<(ModuleDocument, LintReport)> {
    let analysis_options = analysis_options.normalized();
    let parsed = parse_architecture(root, ignore_patterns)?;
    let lint = lint::build_module_lint_report(&parsed);
    let parsed_repo = analysis_options
        .metrics
        .then(|| crate::parser::parse_repo(root, ignore_patterns, parse_dialect(version)))
        .transpose()?;
    let analysis = if analysis_options.any() {
        Some(analyze::build_architecture_analysis(
            root,
            ignore_patterns,
            &parsed,
            parsed_repo.as_ref(),
            analysis_options,
        )?)
    } else {
        None
    };
    let document = materialize::build_module_document(&parsed, filter, analysis.as_ref());
    Ok((document, lint))
}

pub fn build_repo_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    experimental: bool,
) -> Result<(RepoDocument, LintReport)> {
    let parsed = parse_architecture(root, ignore_patterns)?;
    let lint = lint::build_module_lint_report(&parsed);
    let parsed_repo = crate::parser::parse_repo(root, ignore_patterns, parse_dialect(version))?;
    let analysis = analyze::build_architecture_analysis(
        root,
        ignore_patterns,
        &parsed,
        Some(&parsed_repo),
        ModuleAnalysisOptions {
            coverage: true,
            metrics: true,
            experimental,
        },
    )?;

    Ok((
        RepoDocument {
            analysis: Some(analysis.summary),
        },
        lint,
    ))
}

pub fn build_module_lint_report(root: &Path, ignore_patterns: &[String]) -> Result<LintReport> {
    let parsed = parse_architecture(root, ignore_patterns)?;
    Ok(lint::build_module_lint_report(&parsed))
}

fn parse_architecture(root: &Path, ignore_patterns: &[String]) -> Result<ParsedArchitecture> {
    parse::parse_architecture(root, ignore_patterns)
}

fn parse_dialect(version: SpecialVersion) -> ParseDialect {
    match version {
        SpecialVersion::V0 => ParseDialect::CompatibilityV0,
        SpecialVersion::V1 => ParseDialect::CurrentV1,
    }
}

pub(super) fn immediate_parent_id(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(parent, _)| parent)
}

pub(super) fn nearest_visible_parent_id(
    id: &str,
    visible_ids: &BTreeSet<String>,
) -> Option<String> {
    let mut parent = immediate_parent_id(id);
    while let Some(candidate) = parent {
        if visible_ids.contains(candidate) {
            return Some(candidate.to_string());
        }
        parent = immediate_parent_id(candidate);
    }
    None
}
