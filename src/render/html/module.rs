/**
@module SPECIAL.RENDER.HTML.MODULE
Renders architecture module documents into HTML pages and nodes.
*/
// @fileimplements SPECIAL.RENDER.HTML.MODULE
use crate::model::{ArchitectureMetricsSummary, ModuleDocument};
use crate::render::html_common::{MODULES_HTML_EMPTY, SPEC_HTML_STYLE};
use crate::render::projection::project_module_document;

use super::{HtmlCount, render_grouped_metrics_section_html, render_metrics_section_html};

#[path = "module_templates.rs"]
mod module_templates;

use module_templates::render_module_page_html;

pub(in crate::render) fn render_module_html(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    if document.nodes.is_empty() {
        return format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>special arch</title><style>{}</style></head><body><main><h1>special arch</h1><p class=\"lede\">Materialized architecture view for the current repository.</p>{}",
            SPEC_HTML_STYLE, MODULES_HTML_EMPTY
        );
    }

    render_module_page_html(
        &document,
        verbose,
        document
            .metrics
            .as_ref()
            .map(format_arch_metrics_html)
            .unwrap_or_default(),
    )
}

fn format_arch_metrics_html(metrics: &ArchitectureMetricsSummary) -> String {
    let mut html = render_metrics_section_html(
        "special arch metrics",
        &[
            HtmlCount {
                label: "total modules",
                value: metrics.total_modules.to_string(),
            },
            HtmlCount {
                label: "total areas",
                value: metrics.total_areas.to_string(),
            },
            HtmlCount {
                label: "unimplemented modules",
                value: metrics.unimplemented_modules.to_string(),
            },
            HtmlCount {
                label: "file-scoped implements",
                value: metrics.file_scoped_implements.to_string(),
            },
            HtmlCount {
                label: "item-scoped implements",
                value: metrics.item_scoped_implements.to_string(),
            },
            HtmlCount {
                label: "owned lines",
                value: metrics.owned_lines.to_string(),
            },
            HtmlCount {
                label: "public items",
                value: metrics.public_items.to_string(),
            },
            HtmlCount {
                label: "internal items",
                value: metrics.internal_items.to_string(),
            },
        ],
    );
    html.push_str(&render_metrics_section_html(
        "complexity totals",
        &[
            HtmlCount {
                label: "complexity functions",
                value: metrics.complexity_functions.to_string(),
            },
            HtmlCount {
                label: "total cyclomatic",
                value: metrics.total_cyclomatic.to_string(),
            },
            HtmlCount {
                label: "max cyclomatic",
                value: metrics.max_cyclomatic.to_string(),
            },
            HtmlCount {
                label: "total cognitive",
                value: metrics.total_cognitive.to_string(),
            },
            HtmlCount {
                label: "max cognitive",
                value: metrics.max_cognitive.to_string(),
            },
        ],
    ));
    html.push_str(&render_metrics_section_html(
        "quality totals",
        &[
            HtmlCount {
                label: "quality public functions",
                value: metrics.quality_public_functions.to_string(),
            },
            HtmlCount {
                label: "quality parameters",
                value: metrics.quality_parameters.to_string(),
            },
            HtmlCount {
                label: "quality bool params",
                value: metrics.quality_bool_params.to_string(),
            },
            HtmlCount {
                label: "quality raw string params",
                value: metrics.quality_raw_string_params.to_string(),
            },
            HtmlCount {
                label: "quality panic sites",
                value: metrics.quality_panic_sites.to_string(),
            },
            HtmlCount {
                label: "unreached items",
                value: metrics.unreached_items.to_string(),
            },
        ],
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "modules by area",
        &metrics.modules_by_area,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "owned lines by module",
        &metrics.owned_lines_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "max cyclomatic by module",
        &metrics.max_cyclomatic_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "max cognitive by module",
        &metrics.max_cognitive_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "panic sites by module",
        &metrics.panic_sites_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "unreached items by module",
        &metrics.unreached_items_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "fan in by module",
        &metrics.fan_in_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "fan out by module",
        &metrics.fan_out_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "ambiguous internal dependency targets by module",
        &metrics.ambiguous_internal_targets_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "unresolved internal dependency targets by module",
        &metrics.unresolved_internal_targets_by_module,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "external dependency targets by module",
        &metrics.external_dependency_targets_by_module,
    ));
    html
}
