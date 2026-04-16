/**
@module SPECIAL.RENDER.HTML
Renders projected specs and modules into HTML views with shared styling and best-effort code highlighting.
*/
// @fileimplements SPECIAL.RENDER.HTML
use askama::Template;

use crate::model::{ArchitectureKind, ModuleDocument, ModuleNode, SpecDocument, SpecNode};

use super::common::{
    MODULES_HTML_EMPTY, SPEC_HTML_EMPTY, SPEC_HTML_STYLE, escape_html, highlight_code_html,
    language_name_for_path, planned_badge_text,
};
use super::projection::{
    ProjectedArchitectureCoverage, ProjectedCount, ProjectedExplanation, ProjectedMetaLine,
    project_architecture_coverage_view, project_document, project_module_analysis_view,
    project_module_document,
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
    label: &'static str,
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
    verbose: bool,
    uncovered_paths: &'a [String],
    weak_paths: &'a [String],
}

#[derive(Template)]
#[template(path = "render/spec_page.html")]
struct SpecPageHtmlTemplate<'a> {
    nodes: &'a [SpecNode],
    verbose: bool,
    style: &'static str,
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
}

impl ModulePageHtmlTemplate<'_> {
    fn coverage_section(&self) -> String {
        self.document
            .analysis
            .as_ref()
            .and_then(|analysis| analysis.coverage.as_ref())
            .map(|coverage| {
                format_architecture_coverage_html(&project_architecture_coverage_view(
                    coverage,
                    self.verbose,
                ))
            })
            .unwrap_or_default()
    }

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
                label: "declared at",
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
    if document.nodes.is_empty() {
        return format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>special specs</title><style>{}</style></head><body><main><h1>special specs</h1><p class=\"lede\">Materialized semantic spec view for the current repository.</p>{}",
            SPEC_HTML_STYLE, SPEC_HTML_EMPTY
        );
    }

    render_template(&SpecPageHtmlTemplate {
        nodes: &document.nodes,
        verbose,
        style: SPEC_HTML_STYLE,
    })
}

pub(super) fn render_module_html(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    if document.nodes.is_empty() {
        return format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>special modules</title><style>{}</style></head><body><main><h1>special modules</h1><p class=\"lede\">Materialized architecture module view for the current repository.</p>{}",
            SPEC_HTML_STYLE, MODULES_HTML_EMPTY
        );
    }

    render_template(&ModulePageHtmlTemplate {
        document: &document,
        verbose,
        style: SPEC_HTML_STYLE,
    })
}

fn format_architecture_coverage_html(coverage: &ProjectedArchitectureCoverage) -> String {
    render_template(&CoverageSectionHtmlTemplate {
        counts_html: render_template(&CountsSectionHtmlTemplate {
            counts: &coverage
                .counts
                .iter()
                .map(projected_count)
                .collect::<Vec<_>>(),
        }),
        verbose: !(coverage.uncovered_paths.is_empty() && coverage.weak_paths.is_empty()),
        uncovered_paths: &coverage.uncovered_paths,
        weak_paths: &coverage.weak_paths,
    })
}

fn projected_count(count: &ProjectedCount) -> HtmlCount {
    HtmlCount {
        label: count.label,
        value: count.value.clone(),
    }
}

fn projected_meta_line(line: &ProjectedMetaLine) -> HtmlMetaLine {
    HtmlMetaLine {
        label: line.label,
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
