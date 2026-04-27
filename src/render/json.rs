/**
@module SPECIAL.RENDER.JSON
Renders projected specs and modules into structured JSON output.
*/
// @fileimplements SPECIAL.RENDER.JSON
use anyhow::Result;

use crate::model::{ModuleDocument, OverviewDocument, PatternDocument, RepoDocument, SpecDocument};

use super::projection::{
    project_document, project_module_document, project_pattern_document, project_repo_document_json,
};

pub(super) fn render_spec_json(document: &SpecDocument, verbose: bool) -> Result<String> {
    let document = project_document(document, verbose);
    Ok(serde_json::to_string_pretty(&document)?)
}

pub(super) fn render_module_json(document: &ModuleDocument, verbose: bool) -> Result<String> {
    let document = project_module_document(document, verbose);
    Ok(serde_json::to_string_pretty(&document)?)
}

pub(super) fn render_repo_json(document: &RepoDocument, verbose: bool) -> Result<String> {
    let document = project_repo_document_json(document, verbose);
    Ok(serde_json::to_string_pretty(&document)?)
}

pub(super) fn render_pattern_json(document: &PatternDocument, verbose: bool) -> Result<String> {
    let document = project_pattern_document(document, verbose);
    Ok(serde_json::to_string_pretty(&document)?)
}

pub(super) fn render_overview_json(document: &OverviewDocument) -> Result<String> {
    Ok(serde_json::to_string_pretty(document)?)
}
