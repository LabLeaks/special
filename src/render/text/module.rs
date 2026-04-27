/**
@module SPECIAL.RENDER.TEXT.MODULE
Renders architecture module documents and module trees into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT.MODULE
use crate::model::ModuleDocument;
use crate::render::projection::project_module_document;

use super::render_arch_metrics_text;

#[path = "module_templates.rs"]
mod module_templates;

use module_templates::render_module_page_text;

pub(in crate::render) fn render_module_text(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    if document.nodes.is_empty() {
        return "No modules found.".to_string();
    }

    let rendered = render_module_page_text(&document, verbose);
    if let Some(metrics) = &document.metrics {
        let mut output = render_arch_metrics_text(metrics);
        if !document.nodes.is_empty() {
            output.push('\n');
            output.push_str(&rendered);
        }
        output
    } else {
        rendered
    }
}
