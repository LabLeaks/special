/**
@module SPECIAL.RENDER.TEXT.SPEC
Renders spec documents and spec trees into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT.SPEC
use crate::model::SpecDocument;
use crate::render::projection::project_document;

use super::render_spec_metrics_text;

#[path = "spec_templates.rs"]
mod spec_templates;

use spec_templates::render_spec_page_text;

pub(in crate::render) fn render_spec_text(document: &SpecDocument, verbose: bool) -> String {
    let document = project_document(document, verbose);
    if document.nodes.is_empty() && document.metrics.is_none() {
        return "No specs found.".to_string();
    }

    let rendered = render_spec_page_text(&document.nodes, verbose);
    if let Some(metrics) = &document.metrics {
        let mut output = render_spec_metrics_text(metrics);
        if !document.nodes.is_empty() {
            output.push('\n');
            output.push_str(&rendered);
        }
        output
    } else {
        rendered
    }
}
