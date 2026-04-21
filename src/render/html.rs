/**
@module SPECIAL.RENDER.HTML
Renders projected specs and modules into HTML views with shared styling and best-effort code highlighting.
*/
// @fileimplements SPECIAL.RENDER.HTML
use askama::Template;

use crate::model::{
    ArchitectureKind, ArchitectureMetricsSummary, GroupedCount, ModuleDocument, ModuleNode,
    RepoDocument, RepoMetricsSummary, RepoTraceabilityMetrics, SpecDocument, SpecMetricsSummary,
    SpecNode,
};

use super::common::{
    MODULES_HTML_EMPTY, SPEC_HTML_EMPTY, SPEC_HTML_STYLE, deprecated_badge_text, escape_html,
    highlight_code_html, language_name_for_path, planned_badge_text,
};
use super::projection::{
    ProjectedArchitectureTraceability, ProjectedCount, ProjectedExplanation, ProjectedMetaLine,
    ProjectedRepoSignals, project_document, project_module_analysis_view, project_module_document,
    project_repo_signals_view, project_repo_traceability_view,
};
use super::templates::render_template;

#[derive(Clone)]
struct HtmlCount {
    label: &'static str,
    value: String,
}

#[derive(Clone)]
struct HtmlDetailSection {
    label: &'static str,
    location: String,
    body_at: Option<String>,
    body_html: Option<String>,
    language_class: String,
}

#[derive(Clone)]
struct HtmlMetaLine {
    label: String,
    value: String,
}

#[derive(Clone)]
struct HtmlExplanationSection {
    label: &'static str,
    plain: String,
    precise: String,
}

#[derive(Template)]
#[template(path = "render/counts_section.html")]
struct CountsSectionHtmlTemplate<'a> {
    counts: &'a [HtmlCount],
}

#[derive(Template)]
#[template(path = "render/detail_sections.html")]
struct DetailSectionsHtmlTemplate<'a> {
    details: &'a [HtmlDetailSection],
}

#[derive(Template)]
#[template(path = "render/meta_lines.html")]
struct MetaLinesHtmlTemplate<'a> {
    lines: &'a [HtmlMetaLine],
}

#[derive(Template)]
#[template(path = "render/explanations.html")]
struct ExplanationsHtmlTemplate<'a> {
    explanations: &'a [HtmlExplanationSection],
}

#[derive(Template)]
#[template(path = "render/spec_verbose.html")]
struct SpecVerboseHtmlTemplate {
    declared_at: String,
    verifies_html: String,
    attests_html: String,
}

#[derive(Template)]
#[template(path = "render/module_verbose.html")]
struct ModuleVerboseHtmlTemplate {
    implementations_html: String,
    meta_lines_html: String,
    explanations_html: String,
}

#[derive(Template)]
#[template(path = "render/coverage_section.html")]
struct CoverageSectionHtmlTemplate<'a> {
    counts_html: String,
    explanations_html: String,
    verbose: bool,
    unowned_items: &'a [String],
    duplicate_items: &'a [String],
}

#[derive(Template)]
#[template(path = "render/spec_page.html")]
struct SpecPageHtmlTemplate<'a> {
    nodes: &'a [SpecNode],
    verbose: bool,
    style: &'static str,
    metrics_html: String,
}

impl SpecPageHtmlTemplate<'_> {
    fn tree_html(&self) -> String {
        self.nodes
            .iter()
            .map(|node| {
                render_template(&SpecNodeHtmlTemplate {
                    node,
                    verbose: self.verbose,
                })
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/spec_node.html")]
struct SpecNodeHtmlTemplate<'a> {
    node: &'a SpecNode,
    verbose: bool,
}

impl SpecNodeHtmlTemplate<'_> {
    fn node_id(&self) -> String {
        self.node.id.clone()
    }

    fn node_text(&self) -> String {
        self.node.text.clone()
    }

    fn is_group(&self) -> bool {
        self.node.kind() == crate::model::NodeKind::Group
    }

    fn planned_badge(&self) -> String {
        planned_badge_text(self.node.planned_release())
    }

    fn deprecated_badge(&self) -> String {
        deprecated_badge_text(self.node.deprecated_release())
    }

    fn declared_at(&self) -> String {
        format!(
            "{}:{}",
            self.node.location.path.display(),
            self.node.location.line
        )
    }

    fn verbose_section(&self) -> String {
        if !self.verbose {
            return String::new();
        }
        let verifies = self
            .node
            .verifies
            .iter()
            .map(|verify| HtmlDetailSection {
                label: verify_label(verify),
                location: format!(
                    "{}:{}",
                    verify.location.path.display(),
                    verify.location.line
                ),
                body_at: verify
                    .body_location
                    .as_ref()
                    .map(|location| format!("{}:{}", location.path.display(), location.line)),
                body_html: verify.body.as_ref().map(|body| {
                    let language = language_name_for_path(
                        verify
                            .body_location
                            .as_ref()
                            .map(|location| location.path.as_path())
                            .unwrap_or(verify.location.path.as_path()),
                    );
                    highlight_code_html(body, language)
                }),
                language_class: language_name_for_path(
                    verify
                        .body_location
                        .as_ref()
                        .map(|location| location.path.as_path())
                        .unwrap_or(verify.location.path.as_path()),
                )
                .to_string(),
            })
            .collect::<Vec<_>>();
        let attests = self
            .node
            .attests
            .iter()
            .map(|attest| HtmlDetailSection {
                label: attest_label(attest),
                location: format!(
                    "{}:{}",
                    attest.location.path.display(),
                    attest.location.line
                ),
                body_at: None,
                body_html: attest.body.as_ref().map(|body| escape_html(body)),
                language_class: "text".to_string(),
            })
            .collect::<Vec<_>>();

        render_template(&SpecVerboseHtmlTemplate {
            declared_at: self.declared_at(),
            verifies_html: render_template(&DetailSectionsHtmlTemplate { details: &verifies }),
            attests_html: render_template(&DetailSectionsHtmlTemplate { details: &attests }),
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
                render_template(&SpecNodeHtmlTemplate {
                    node: child,
                    verbose: self.verbose,
                })
            })
            .collect();
        format!("<ul>{children}</ul>")
    }
}

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

pub(super) fn render_spec_html(document: &SpecDocument, verbose: bool) -> String {
    let document = project_document(document, verbose);
    if document.nodes.is_empty() && document.metrics.is_none() {
        return format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>special specs</title><style>{}</style></head><body><main><h1>special specs</h1><p class=\"lede\">Materialized spec view for the current repository.</p>{}",
            SPEC_HTML_STYLE, SPEC_HTML_EMPTY
        );
    }

    render_template(&SpecPageHtmlTemplate {
        nodes: &document.nodes,
        verbose,
        style: SPEC_HTML_STYLE,
        metrics_html: document
            .metrics
            .as_ref()
            .map(format_spec_metrics_html)
            .unwrap_or_default(),
    })
}

pub(super) fn render_module_html(document: &ModuleDocument, verbose: bool) -> String {
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

pub(super) fn render_repo_html(document: &RepoDocument, verbose: bool) -> String {
    let document = super::projection::project_repo_document(document, verbose);
    let metrics_html = document
        .metrics
        .as_ref()
        .map(format_repo_metrics_html)
        .unwrap_or_default();
    let repo_signals_html = document
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.repo_signals.as_ref())
        .map(|signals| format_repo_signals_html(&project_repo_signals_view(signals, verbose)))
        .unwrap_or_default();
    let traceability_html = document
        .analysis
        .as_ref()
        .map(|analysis| {
            format_repo_traceability_html(&project_repo_traceability_view(
                analysis.traceability.as_ref(),
                analysis.traceability_unavailable_reason.as_deref(),
            ))
        })
        .unwrap_or_default();
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>special health</title><style>{}</style></head><body><main><h1>special health</h1><p class=\"lede\">Repo-wide quality signals for the current repository.</p>{}{}</main></body></html>",
        SPEC_HTML_STYLE,
        format!("{metrics_html}{repo_signals_html}"),
        traceability_html
    )
}

fn format_spec_metrics_html(metrics: &SpecMetricsSummary) -> String {
    let mut html = render_metrics_section_html(
        "special specs metrics",
        &[
            HtmlCount {
                label: "total specs",
                value: metrics.total_specs.to_string(),
            },
            HtmlCount {
                label: "unverified specs",
                value: metrics.unverified_specs.to_string(),
            },
            HtmlCount {
                label: "planned specs",
                value: metrics.planned_specs.to_string(),
            },
            HtmlCount {
                label: "deprecated specs",
                value: metrics.deprecated_specs.to_string(),
            },
            HtmlCount {
                label: "verifies",
                value: metrics.verifies.to_string(),
            },
            HtmlCount {
                label: "attests",
                value: metrics.attests.to_string(),
            },
        ],
    );
    html.push_str(&render_metrics_section_html(
        "spec support buckets",
        &[
            HtmlCount {
                label: "verified specs",
                value: metrics.verified_specs.to_string(),
            },
            HtmlCount {
                label: "attested specs",
                value: metrics.attested_specs.to_string(),
            },
            HtmlCount {
                label: "specs with both supports",
                value: metrics.specs_with_both_supports.to_string(),
            },
            HtmlCount {
                label: "item-scoped verifies",
                value: metrics.item_scoped_verifies.to_string(),
            },
            HtmlCount {
                label: "file-scoped verifies",
                value: metrics.file_scoped_verifies.to_string(),
            },
            HtmlCount {
                label: "unattached verifies",
                value: metrics.unattached_verifies.to_string(),
            },
            HtmlCount {
                label: "block attests",
                value: metrics.block_attests.to_string(),
            },
            HtmlCount {
                label: "file attests",
                value: metrics.file_attests.to_string(),
            },
        ],
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "specs by file",
        &metrics.specs_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "current specs by top-level id",
        &metrics.current_specs_by_top_level_id,
    ));
    html
}

fn format_repo_metrics_html(metrics: &RepoMetricsSummary) -> String {
    let mut html = render_metrics_section_html(
        "special health metrics",
        &[
            HtmlCount {
                label: "duplicate items",
                value: metrics.duplicate_items.to_string(),
            },
            HtmlCount {
                label: "unowned items",
                value: metrics.unowned_items.to_string(),
            },
        ],
    );
    html.push_str(&render_grouped_metrics_section_html(
        "duplicate items by file",
        &metrics.duplicate_items_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "unowned items by file",
        &metrics.unowned_items_by_file,
    ));
    if let Some(traceability) = &metrics.traceability {
        html.push_str(&format_repo_traceability_metrics_html(traceability));
    }
    html
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
        "external dependency targets by module",
        &metrics.external_dependency_targets_by_module,
    ));
    html
}

fn render_metrics_section_html(title: &str, counts: &[HtmlCount]) -> String {
    let counts_html = render_template(&CountsSectionHtmlTemplate { counts });
    format!(
        "<section class=\"node\"><div class=\"node-header\"><span class=\"node-id\">{}</span></div><div class=\"meta counts\">{}</div></section>",
        title, counts_html
    )
}

fn render_grouped_metrics_section_html(title: &str, counts: &[GroupedCount]) -> String {
    if counts.is_empty() {
        return String::new();
    }
    let lines = counts
        .iter()
        .map(|group| HtmlMetaLine {
            label: group.value.clone(),
            value: group.count.to_string(),
        })
        .collect::<Vec<_>>();
    let lines_html = render_template(&MetaLinesHtmlTemplate { lines: &lines });
    format!(
        "<section class=\"node\"><div class=\"node-header\"><span class=\"node-id\">{}</span></div>{}</section>",
        title, lines_html
    )
}

fn format_repo_traceability_metrics_html(metrics: &RepoTraceabilityMetrics) -> String {
    let mut html = render_metrics_section_html(
        "special health traceability metrics",
        &[
            HtmlCount {
                label: "analyzed items",
                value: metrics.analyzed_items.to_string(),
            },
            HtmlCount {
                label: "current spec items",
                value: metrics.current_spec_items.to_string(),
            },
            HtmlCount {
                label: "statically mediated items",
                value: metrics.statically_mediated_items.to_string(),
            },
            HtmlCount {
                label: "unverified test items",
                value: metrics.unverified_test_items.to_string(),
            },
            HtmlCount {
                label: "unexplained items",
                value: metrics.unexplained_items.to_string(),
            },
            HtmlCount {
                label: "unexplained review-surface items",
                value: metrics.unexplained_review_surface_items.to_string(),
            },
            HtmlCount {
                label: "unexplained public items",
                value: metrics.unexplained_public_items.to_string(),
            },
            HtmlCount {
                label: "unexplained internal items",
                value: metrics.unexplained_internal_items.to_string(),
            },
            HtmlCount {
                label: "unexplained module-backed items",
                value: metrics.unexplained_module_backed_items.to_string(),
            },
            HtmlCount {
                label: "unexplained module-connected items",
                value: metrics.unexplained_module_connected_items.to_string(),
            },
            HtmlCount {
                label: "unexplained module-isolated items",
                value: metrics.unexplained_module_isolated_items.to_string(),
            },
        ],
    );
    html.push_str(&render_grouped_metrics_section_html(
        "unexplained items by file",
        &metrics.unexplained_items_by_file,
    ));
    html.push_str(&render_grouped_metrics_section_html(
        "unexplained review-surface items by file",
        &metrics.unexplained_review_surface_items_by_file,
    ));
    html
}

fn format_repo_signals_html(coverage: &ProjectedRepoSignals) -> String {
    if coverage.counts.is_empty()
        && coverage.unowned_items.is_empty()
        && coverage.duplicate_items.is_empty()
    {
        return String::new();
    }

    render_template(&CoverageSectionHtmlTemplate {
        counts_html: render_template(&CountsSectionHtmlTemplate {
            counts: &coverage
                .counts
                .iter()
                .map(projected_count)
                .collect::<Vec<_>>(),
        }),
        explanations_html: render_template(&ExplanationsHtmlTemplate {
            explanations: &coverage
                .explanations
                .iter()
                .map(projected_explanation)
                .collect::<Vec<_>>(),
        }),
        verbose: !coverage.unowned_items.is_empty() || !coverage.duplicate_items.is_empty(),
        unowned_items: &coverage.unowned_items,
        duplicate_items: &coverage.duplicate_items,
    })
}

fn format_repo_traceability_html(traceability: &ProjectedArchitectureTraceability) -> String {
    if traceability.counts.is_empty()
        && traceability.items.is_empty()
        && traceability.explanations.is_empty()
        && traceability.unavailable_reason.is_none()
    {
        return String::new();
    }

    let counts_html = render_template(&CountsSectionHtmlTemplate {
        counts: &traceability
            .counts
            .iter()
            .map(projected_count)
            .collect::<Vec<_>>(),
    });
    let explanations_html = render_template(&ExplanationsHtmlTemplate {
        explanations: &traceability
            .explanations
            .iter()
            .map(projected_explanation)
            .collect::<Vec<_>>(),
    });
    let details_html = traceability
        .items
        .iter()
        .map(|item| {
            format!(
                "<li><strong>{}</strong>: {}</li>",
                item.label,
                escape_html(&item.value)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let unavailable_html = traceability
        .unavailable_reason
        .as_ref()
        .map(|reason| {
            format!(
                "<p><strong>unavailable</strong>: {}</p>",
                escape_html(reason)
            )
        })
        .unwrap_or_default();
    format!(
        "<section class=\"coverage\"><h2>traceability</h2>{unavailable_html}{counts_html}{explanations_html}<details><summary>traceability detail</summary><ul>{details_html}</ul></details></section>"
    )
}

fn projected_count(count: &ProjectedCount) -> HtmlCount {
    HtmlCount {
        label: count.label,
        value: count.value.clone(),
    }
}

fn projected_meta_line(line: &ProjectedMetaLine) -> HtmlMetaLine {
    HtmlMetaLine {
        label: line.label.to_string(),
        value: line.value.clone(),
    }
}

fn projected_explanation(explanation: &ProjectedExplanation) -> HtmlExplanationSection {
    HtmlExplanationSection {
        label: explanation.label,
        plain: explanation.plain.to_string(),
        precise: explanation.precise.to_string(),
    }
}

fn verify_label(verify: &crate::model::VerifyRef) -> &'static str {
    if verify.body_location.is_none() && verify.body.is_some() {
        "@fileverifies"
    } else {
        "@verifies"
    }
}

fn implementation_label(implementation: &crate::model::ImplementRef) -> &'static str {
    if implementation.body_location.is_none() {
        "@fileimplements"
    } else {
        "@implements"
    }
}

fn attest_label(attest: &crate::model::AttestRef) -> &'static str {
    attest.scope.as_annotation()
}
