/**
@module SPECIAL.RENDER.HTML.SPEC
Renders spec documents into HTML pages and nodes.
*/
// @fileimplements SPECIAL.RENDER.HTML.SPEC
use crate::model::SpecDocument;
use crate::render::html_common::{SPEC_HTML_EMPTY, SPEC_HTML_STYLE};
use crate::render::projection::project_document;

#[path = "spec_templates.rs"]
mod spec_templates;

use super::format_spec_metrics_html;
use spec_templates::render_spec_page_html;

pub(in crate::render) fn render_spec_html(document: &SpecDocument, verbose: bool) -> String {
    let document = project_document(document, verbose);
    if document.nodes.is_empty() && document.metrics.is_none() {
        return format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>special specs</title><style>{}</style></head><body><main><h1>special specs</h1><p class=\"lede\">Materialized spec view for the current repository.</p>{}",
            SPEC_HTML_STYLE, SPEC_HTML_EMPTY
        );
    }

    render_spec_page_html(
        &document.nodes,
        verbose,
        document
            .metrics
            .as_ref()
            .map(format_spec_metrics_html)
            .unwrap_or_default(),
    )
}
