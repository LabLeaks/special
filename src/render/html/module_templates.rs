// Askama-backed HTML template adapters for architecture module pages and nodes.
// @fileimplements SPECIAL.RENDER.HTML.MODULE
use askama::Template;

use crate::model::{ArchitectureKind, ModuleDocument, ModuleNode};
use crate::render::common::planned_badge_text;
use crate::render::html_common::{SPEC_HTML_STYLE, highlight_code_html, language_name_for_path};
use crate::render::projection::project_module_analysis_view;
use crate::render::templates::render_template;

use super::super::{
    CountsSectionHtmlTemplate, DetailSectionsHtmlTemplate, ExplanationsHtmlTemplate, HtmlCount,
    HtmlDetailSection, HtmlMetaLine, MetaLinesHtmlTemplate, ModuleVerboseHtmlTemplate,
    implementation_label, projected_count, projected_explanation, projected_meta_line,
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
        if !self.node.pattern_applications.is_empty() {
            counts.push(HtmlCount {
                label: "pattern applications",
                value: self.node.pattern_applications.len().to_string(),
            });
        }
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

pub(super) fn render_module_page_html(
    document: &ModuleDocument,
    verbose: bool,
    metrics_html: String,
) -> String {
    render_template(&ModulePageHtmlTemplate {
        document,
        verbose,
        style: SPEC_HTML_STYLE,
        metrics_html,
    })
}
