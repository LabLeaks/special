/**
@module SPECIAL.RENDER.JSON
Renders projected specs and modules into structured JSON output.
*/
// @implements SPECIAL.RENDER.JSON
use anyhow::Result;

use crate::model::{ModuleDocument, SpecDocument};

use super::projection::{project_document, project_module_document};

pub(super) fn render_spec_json(document: &SpecDocument, verbose: bool) -> Result<String> {
    let document = project_document(document, verbose);
    Ok(serde_json::to_string_pretty(&document)?)
}

pub(super) fn render_module_json(document: &ModuleDocument, verbose: bool) -> Result<String> {
    let document = project_module_document(document, verbose);
    Ok(serde_json::to_string_pretty(&document)?)
}
