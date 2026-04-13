/**
@module SPECIAL.RENDER.HTML
Renders projected specs and modules into HTML views with shared styling and best-effort code highlighting.
*/
// @implements SPECIAL.RENDER.HTML
use crate::model::{ArchitectureKind, ModuleDocument, ModuleNode, SpecDocument, SpecNode};

use super::common::{
    MODULES_HTML_EMPTY, SPEC_HTML_CLOSE, SPEC_HTML_EMPTY, SPEC_HTML_STYLE, SPEC_HTML_TREE_OPEN,
    escape_html, highlight_code_html, language_name_for_path, planned_badge_text,
};
use super::projection::{project_document, project_module_document};

pub(super) fn render_spec_html(document: &SpecDocument, verbose: bool) -> String {
    let document = project_document(document, verbose);
    let mut html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>special specs</title><style>{}</style></head><body><main><h1>special specs</h1><p class=\"lede\">Materialized semantic spec view for the current repository.</p>",
        SPEC_HTML_STYLE
    );

    if document.nodes.is_empty() {
        html.push_str(SPEC_HTML_EMPTY);
        return html;
    }

    html.push_str(SPEC_HTML_TREE_OPEN);
    for node in &document.nodes {
        render_node_html(node, verbose, &mut html);
    }
    html.push_str(SPEC_HTML_CLOSE);
    html
}

pub(super) fn render_module_html(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    let mut html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>special modules</title><style>{}</style></head><body><main><h1>special modules</h1><p class=\"lede\">Materialized architecture module view for the current repository.</p>",
        SPEC_HTML_STYLE
    );

    if document.nodes.is_empty() {
        html.push_str(MODULES_HTML_EMPTY);
        return html;
    }

    html.push_str(SPEC_HTML_TREE_OPEN);
    for node in &document.nodes {
        render_module_node_html(node, verbose, &mut html);
    }
    html.push_str(SPEC_HTML_CLOSE);
    html
}

fn render_node_html(node: &SpecNode, verbose: bool, html: &mut String) {
    html.push_str("<li><div class=\"node\"><div class=\"node-header\"><span class=\"node-id\">");
    html.push_str(&escape_html(&node.id));
    html.push_str("</span>");
    if node.kind() == crate::model::NodeKind::Group {
        html.push_str("<span class=\"badge badge-group\">group</span>");
    }
    if node.is_planned() {
        html.push_str("<span class=\"badge badge-planned\">");
        html.push_str(&escape_html(&planned_badge_text(node.planned_release())));
        html.push_str("</span>");
    }
    if node.is_unsupported() {
        html.push_str("<span class=\"badge badge-unsupported\">unsupported</span>");
    }
    html.push_str("</div>");

    if !node.text.is_empty() {
        html.push_str("<div class=\"node-text\">");
        html.push_str(&escape_html(&node.text));
        html.push_str("</div>");
    }

    html.push_str("<div class=\"meta counts\">");
    html.push_str(&format!("<span>verifies: {}</span>", node.verifies.len()));
    html.push_str(&format!("<span>attests: {}</span>", node.attests.len()));
    html.push_str("</div>");

    if verbose {
        html.push_str("<div class=\"meta\">declared at ");
        html.push_str(&escape_html(&format!(
            "{}:{}",
            node.location.path.display(),
            node.location.line
        )));
        html.push_str("</div>");

        for verify in &node.verifies {
            html.push_str("<details><summary>@verifies ");
            html.push_str(&escape_html(&format!(
                "{}:{}",
                verify.location.path.display(),
                verify.location.line
            )));
            html.push_str("</summary>");
            if let Some(body_location) = &verify.body_location {
                html.push_str("<div class=\"meta\">body at ");
                html.push_str(&escape_html(&format!(
                    "{}:{}",
                    body_location.path.display(),
                    body_location.line
                )));
                html.push_str("</div>");
            }
            if let Some(body) = &verify.body {
                let language = language_name_for_path(
                    verify
                        .body_location
                        .as_ref()
                        .map(|location| location.path.as_path())
                        .unwrap_or(verify.location.path.as_path()),
                );
                html.push_str("<pre class=\"code-block\"><code class=\"language-");
                html.push_str(language);
                html.push_str("\">");
                html.push_str(&highlight_code_html(body, language));
                html.push_str("</code></pre>");
            }
            html.push_str("</details>");
        }

        for attest in &node.attests {
            html.push_str("<details><summary>@attests ");
            html.push_str(&escape_html(&format!(
                "{}:{}",
                attest.location.path.display(),
                attest.location.line
            )));
            html.push_str("</summary>");
            if let Some(body) = &attest.body {
                html.push_str("<pre class=\"code-block\"><code class=\"language-text\">");
                html.push_str(&escape_html(body));
                html.push_str("</code></pre>");
            }
            html.push_str("</details>");
        }
    }

    if !node.children.is_empty() {
        html.push_str("<ul>");
        for child in &node.children {
            render_node_html(child, verbose, html);
        }
        html.push_str("</ul>");
    }

    html.push_str("</div></li>");
}

fn render_module_node_html(node: &ModuleNode, verbose: bool, html: &mut String) {
    html.push_str("<li><div class=\"node\"><div class=\"node-header\"><span class=\"node-id\">");
    html.push_str(&escape_html(&node.id));
    html.push_str("</span>");
    if node.kind() == ArchitectureKind::Area {
        html.push_str("<span class=\"badge badge-group\">area</span>");
    }
    if node.is_planned() {
        html.push_str("<span class=\"badge badge-planned\">");
        html.push_str(&escape_html(&planned_badge_text(node.planned_release())));
        html.push_str("</span>");
    }
    if node.is_unsupported() {
        html.push_str("<span class=\"badge badge-unsupported\">unsupported</span>");
    }
    html.push_str("</div>");

    if !node.text.is_empty() {
        html.push_str("<div class=\"node-text\">");
        html.push_str(&escape_html(&node.text));
        html.push_str("</div>");
    }

    html.push_str("<div class=\"meta counts\">");
    html.push_str(&format!(
        "<span>implements: {}</span>",
        node.implements.len()
    ));
    html.push_str("</div>");

    if verbose {
        html.push_str("<div class=\"meta\">declared at ");
        html.push_str(&escape_html(&format!(
            "{}:{}",
            node.location.path.display(),
            node.location.line
        )));
        html.push_str("</div>");

        for implementation in &node.implements {
            html.push_str("<details><summary>@implements ");
            html.push_str(&escape_html(&format!(
                "{}:{}",
                implementation.location.path.display(),
                implementation.location.line
            )));
            html.push_str("</summary>");
            if let Some(body_location) = &implementation.body_location {
                html.push_str("<div class=\"meta\">body at ");
                html.push_str(&escape_html(&format!(
                    "{}:{}",
                    body_location.path.display(),
                    body_location.line
                )));
                html.push_str("</div>");
            }
            if let Some(body) = &implementation.body {
                let language = language_name_for_path(
                    implementation
                        .body_location
                        .as_ref()
                        .map(|location| location.path.as_path())
                        .unwrap_or(implementation.location.path.as_path()),
                );
                html.push_str("<pre class=\"code-block\"><code class=\"language-");
                html.push_str(language);
                html.push_str("\">");
                html.push_str(&highlight_code_html(body, language));
                html.push_str("</code></pre>");
            }
            html.push_str("</details>");
        }
    }

    if !node.children.is_empty() {
        html.push_str("<ul>");
        for child in &node.children {
            render_module_node_html(child, verbose, html);
        }
        html.push_str("</ul>");
    }

    html.push_str("</div></li>");
}
