/**
@module SPECIAL.RENDER.TEXT
Renders projected specs, modules, and lint diagnostics into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT
mod analysis;
mod attachments;

use std::fmt::Write;

use askama::Template;

use crate::model::{
    ArchitectureKind, DiagnosticSeverity, LintReport, ModuleDocument, ModuleNode, SpecDocument,
    SpecNode,
};

use self::analysis::{format_architecture_coverage, render_projected_module_analysis};
use self::attachments::{
    render_attest_section, render_implementation_section, render_verify_section,
};
use super::common::planned_badge_text;
use super::projection::{
    project_architecture_coverage_view, project_document, project_module_analysis_view,
    project_module_document,
};
use super::templates::{render_template, text_indent};

#[derive(Template)]
#[template(path = "render/spec_page.txt", escape = "none")]
struct SpecPageTextTemplate<'a> {
    nodes: &'a [SpecNode],
    verbose: bool,
}

impl SpecPageTextTemplate<'_> {
    fn render_nodes(&self) -> String {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let rendered = render_template(&SpecNodeTextTemplate {
                    node,
                    depth: 0,
                    verbose: self.verbose,
                });
                if index == 0 {
                    rendered
                } else {
                    format!("\n{rendered}")
                }
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/spec_node.txt", escape = "none")]
struct SpecNodeTextTemplate<'a> {
    node: &'a SpecNode,
    depth: usize,
    verbose: bool,
}

impl SpecNodeTextTemplate<'_> {
    fn indent(&self) -> String {
        text_indent(self.depth)
    }

    fn is_group(&self) -> bool {
        self.node.kind() == crate::model::NodeKind::Group
    }

    fn planned_badge(&self) -> String {
        planned_badge_text(self.node.planned_release())
    }

    fn verbose_section(&self) -> String {
        if !self.verbose {
            return String::new();
        }

        let indent = self.indent();
        let mut output = String::new();
        writeln!(
            output,
            "{}  declared at: {}:{}",
            indent,
            self.node.location.path.display(),
            self.node.location.line
        )
        .expect("string writes should succeed");

        render_verify_section(&mut output, &indent, self.depth, &self.node.verifies);
        render_attest_section(&mut output, &indent, self.depth, &self.node.attests);

        output
    }

    fn children_section(&self) -> String {
        self.node
            .children
            .iter()
            .map(|child| {
                render_template(&SpecNodeTextTemplate {
                    node: child,
                    depth: self.depth + 1,
                    verbose: self.verbose,
                })
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/module_page.txt", escape = "none")]
struct ModulePageTextTemplate<'a> {
    document: &'a ModuleDocument,
    verbose: bool,
}

impl ModulePageTextTemplate<'_> {
    fn coverage_section(&self) -> String {
        self.document
            .analysis
            .as_ref()
            .and_then(|analysis| analysis.coverage.as_ref())
            .map(|coverage| {
                format_architecture_coverage(&project_architecture_coverage_view(
                    coverage,
                    self.verbose,
                ))
            })
            .unwrap_or_default()
    }

    fn render_nodes(&self) -> String {
        self.document
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let rendered = render_template(&ModuleNodeTextTemplate {
                    node,
                    depth: 0,
                    verbose: self.verbose,
                });
                if index == 0 {
                    rendered
                } else {
                    format!("\n{rendered}")
                }
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/module_node.txt", escape = "none")]
struct ModuleNodeTextTemplate<'a> {
    node: &'a ModuleNode,
    depth: usize,
    verbose: bool,
}

impl ModuleNodeTextTemplate<'_> {
    fn indent(&self) -> String {
        text_indent(self.depth)
    }

    fn is_area(&self) -> bool {
        self.node.kind() == ArchitectureKind::Area
    }

    fn planned_badge(&self) -> String {
        planned_badge_text(self.node.planned_release())
    }

    fn analysis_section(&self) -> String {
        let indent = self.indent();
        project_module_analysis_view(self.node, self.verbose)
            .map(|analysis| render_projected_module_analysis(&indent, &analysis))
            .unwrap_or_default()
    }

    fn verbose_section(&self) -> String {
        if !self.verbose {
            return String::new();
        }

        let indent = self.indent();
        let mut output = String::new();
        writeln!(
            output,
            "{}  declared at: {}:{}",
            indent,
            self.node.location.path.display(),
            self.node.location.line
        )
        .expect("string writes should succeed");

        render_implementation_section(&mut output, &indent, self.depth, &self.node.implements);

        output
    }

    fn children_section(&self) -> String {
        self.node
            .children
            .iter()
            .map(|child| {
                render_template(&ModuleNodeTextTemplate {
                    node: child,
                    depth: self.depth + 1,
                    verbose: self.verbose,
                })
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/lint.txt", escape = "none")]
struct LintTextTemplate<'a> {
    report: &'a LintReport,
}

impl LintTextTemplate<'_> {
    fn severity_label(&self, severity: &DiagnosticSeverity) -> &'static str {
        match severity {
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Error => "error",
        }
    }
}

pub(super) fn render_spec_text(document: &SpecDocument, verbose: bool) -> String {
    let document = project_document(document, verbose);
    if document.nodes.is_empty() {
        return "No specs found.".to_string();
    }

    render_template(&SpecPageTextTemplate {
        nodes: &document.nodes,
        verbose,
    })
}

pub(super) fn render_module_text(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    if document.nodes.is_empty() {
        return "No modules found.".to_string();
    }

    render_template(&ModulePageTextTemplate {
        document: &document,
        verbose,
    })
}

pub(super) fn render_lint_text(report: &LintReport) -> String {
    if report.diagnostics.is_empty() {
        return "Lint clean.".to_string();
    }

    render_template(&LintTextTemplate { report })
        .trim_end()
        .to_string()
}
