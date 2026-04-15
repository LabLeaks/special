/**
@module SPECIAL.MODULES
Coordinates architecture parsing, lint, and tree materialization over discovered source-local and markdown-backed `@module`/`@area` declarations.
*/
// @fileimplements SPECIAL.MODULES
use std::path::Path;

use anyhow::Result;

use crate::model::{
    LintReport, ModuleAnalysisOptions, ModuleDocument, ModuleFilter, ParsedArchitecture,
};

pub(crate) mod analyze;
mod lint;
mod materialize;
mod parse;
mod parse_markdown;

pub fn build_module_document(
    root: &Path,
    ignore_patterns: &[String],
    filter: ModuleFilter,
    analysis_options: ModuleAnalysisOptions,
) -> Result<(ModuleDocument, LintReport)> {
    let parsed = parse_architecture(root, ignore_patterns)?;
    let lint = lint::build_module_lint_report(&parsed);
    let analysis = if analysis_options.any() {
        Some(analyze::build_architecture_analysis(
            root,
            ignore_patterns,
            &parsed,
            analysis_options,
        )?)
    } else {
        None
    };
    let document = materialize::build_module_document(&parsed, filter, analysis.as_ref());
    Ok((document, lint))
}

pub fn build_module_lint_report(root: &Path, ignore_patterns: &[String]) -> Result<LintReport> {
    let parsed = parse_architecture(root, ignore_patterns)?;
    Ok(lint::build_module_lint_report(&parsed))
}

fn parse_architecture(root: &Path, ignore_patterns: &[String]) -> Result<ParsedArchitecture> {
    parse::parse_architecture(root, ignore_patterns)
}
