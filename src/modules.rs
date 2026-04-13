/**
@module SPECIAL.MODULES
Coordinates architecture parsing, lint, and tree materialization over source-local `@module`/`@area` declarations plus `_project/ARCHITECTURE.md`.
*/
// @implements SPECIAL.MODULES
use std::path::Path;

use anyhow::Result;

use crate::model::{LintReport, ModuleDocument, ModuleFilter, ParsedArchitecture};

mod lint;
mod materialize;
mod parse;

pub fn build_module_document(
    root: &Path,
    filter: ModuleFilter,
) -> Result<(ModuleDocument, LintReport)> {
    let parsed = parse_architecture(root)?;
    let lint = lint::build_module_lint_report(&parsed);
    let document = materialize::build_module_document(&parsed, filter);
    Ok((document, lint))
}

pub fn build_module_lint_report(root: &Path) -> Result<LintReport> {
    let parsed = parse_architecture(root)?;
    Ok(lint::build_module_lint_report(&parsed))
}

fn parse_architecture(root: &Path) -> Result<ParsedArchitecture> {
    parse::parse_architecture(root.join(ARCHITECTURE_DOC_PATH), root)
}

const ARCHITECTURE_DOC_PATH: &str = "_project/ARCHITECTURE.md";
