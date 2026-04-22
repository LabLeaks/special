/**
@module SPECIAL.RENDER.HTML.MODULE
Renders architecture module documents into HTML pages and nodes.
*/
// @fileimplements SPECIAL.RENDER.HTML.MODULE
use askama::Template;

use crate::model::{ArchitectureKind, ArchitectureMetricsSummary, ModuleDocument, ModuleNode};
use crate::render::common::planned_badge_text;
use crate::render::html_common::{
    MODULES_HTML_EMPTY, SPEC_HTML_STYLE, highlight_code_html, language_name_for_path,
};
use crate::render::projection::{project_module_analysis_view, project_module_document};
use crate::render::templates::render_template;

use super::{
    CountsSectionHtmlTemplate, DetailSectionsHtmlTemplate, ExplanationsHtmlTemplate, HtmlCount,
    HtmlDetailSection, HtmlMetaLine, MetaLinesHtmlTemplate, ModuleVerboseHtmlTemplate,
    implementation_label, projected_count, projected_explanation, projected_meta_line,
    render_grouped_metrics_section_html, render_metrics_section_html,
};

#[derive(Template)]
#[template(path = "render/module_page.html")]
struct ModulePageHtmlTemplate<'a> {
    document: &'a ModuleDocument,
    verbose: bool,
    style: &'static str,
    metrics_html: String,
}

impl ModulePageHtmlTemplate<'_> {
    fn tree_html(&self) -> String {
        self.document
            .nodes
            .iter()
            .map(|node| {
                render_template(&ModuleNodeHtmlTemplate {
                    node,
                    verbose: self.verbose,
                })
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/module_node.html")]
struct ModuleNodeHtmlTemplate<'a> {
    node: &'a ModuleNode,
    verbose: bool,
}

impl ModuleNodeHtmlTemplate<'_> {
    fn node_id(&self) -> String {
        self.node.id.clone()
    }

    fn node_text(&self) -> String {
        self.node.text.clone()
    }

    fn is_area(&self) -> bool {
        self.node.kind() == ArchitectureKind::Area
    }

    fn planned_badge(&self) -> String {
        planned_badge_text(self.node.planned_release())
    }

    fn declared_at(&self) -> String {
        format!(
            "{}:{}",
            self.node.location.path.display(),
            self.node.location.line
        )
    }

    fn counts_section(&self) -> String {
        let mut counts = vec![HtmlCount {
            label: "implements",
            value: self.node.implements.len().to_string(),
        }];
        if let Some(analysis) = project_module_analysis_view(self.node, self.verbose) {
            counts.extend(analysis.counts.iter().map(projected_count));
        }

        render_template(&CountsSectionHtmlTemplate { counts: &counts })
    }

    fn verbose_section(&self) -> String {
        let implementations = if self.verbose {
            self.node
                .implements
                .iter()
                .map(|implementation| HtmlDetailSection {
                    label: implementation_label(implementation),
                    location: format!(
                        "{}:{}",
                        implementation.location.path.display(),
                        implementation.location.line
                    ),
                    body_at: implementation
                        .body_location
                        .as_ref()
                        .map(|location| format!("{}:{}", location.path.display(), location.line)),
                    body_html: implementation.body.as_ref().map(|body| {
                        let language = language_name_for_path(
                            implementation
                                .body_location
                                .as_ref()
                                .map(|location| location.path.as_path())
                                .unwrap_or(implementation.location.path.as_path()),
                        );
                        highlight_code_html(body, language)
                    }),
                    language_class: language_name_for_path(
                        implementation
                            .body_location
                            .as_ref()
                            .map(|location| location.path.as_path())
                            .unwrap_or(implementation.location.path.as_path()),
                    )
                    .to_string(),
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        let mut meta_lines = if self.verbose {
            vec![HtmlMetaLine {
                label: "declared at".to_string(),
                value: self.declared_at(),
            }]
        } else {
            Vec::new()
        };
        let mut explanations = Vec::new();
        if let Some(analysis) = project_module_analysis_view(self.node, self.verbose) {
            meta_lines.extend(analysis.meta_lines.iter().map(projected_meta_line));
            explanations.extend(analysis.explanations.iter().map(projected_explanation));
        }

        render_template(&ModuleVerboseHtmlTemplate {
            implementations_html: render_template(&DetailSectionsHtmlTemplate {
                details: &implementations,
            }),
            meta_lines_html: render_template(&MetaLinesHtmlTemplate { lines: &meta_lines }),
            explanations_html: render_template(&ExplanationsHtmlTemplate {
                explanations: &explanations,
            }),
        })
    }

    fn children_section(&self) -> String {
        if self.node.children.is_empty() {
            return String::new();
        }

        let children: String = self
            .node
            .children
            .iter()
            .map(|child| {
                render_template(&ModuleNodeHtmlTemplate {
                    node: child,
                    verbose: self.verbose,
                })
            })
            .collect();
        format!("<ul>{children}</ul>")
    }
}

pub(in crate::render) fn render_module_html(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    if document.nodes.is_empty() {
        return format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>special arch</title><style>{}</style></head><body><main><h1>special arch</h1><p class=\"lede\">Materialized architecture view for the current repository.</p>{}",
            SPEC_HTML_STYLE, MODULES_HTML_EMPTY
        );
    }

    render_template(&ModulePageHtmlTemplate {
        document: &document,
        verbose,
        style: SPEC_HTML_STYLE,
        metrics_html: document
            .metrics
            .as_ref()
            .map(format_arch_metrics_html)
            .unwrap_or_default(),
    })
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
