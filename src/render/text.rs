/**
@module SPECIAL.RENDER.TEXT
Renders projected specs, modules, and lint diagnostics into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT
use std::fmt::Write;

use askama::Template;

use crate::model::{
    ArchitectureKind, DiagnosticSeverity, LintReport, ModuleDocument, ModuleNode, SpecDocument,
    SpecNode,
};

use super::common::planned_badge_text;
use super::projection::{
    ProjectedArchitectureCoverage, ProjectedModuleAnalysis, project_architecture_coverage_view,
    project_document, project_module_analysis_view, project_module_document,
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

        for verify in &self.node.verifies {
            writeln!(
                output,
                "{}  {} {}:{}",
                indent,
                verify_label(verify),
                verify.location.path.display(),
                verify.location.line
            )
            .expect("string writes should succeed");
            if let Some(body_location) = &verify.body_location {
                writeln!(
                    output,
                    "{}    body at: {}:{}",
                    indent,
                    body_location.path.display(),
                    body_location.line
                )
                .expect("string writes should succeed");
            }
            if let Some(body) = &verify.body {
                write_block_text(&mut output, body, self.depth + 2);
            }
        }

        for attest in &self.node.attests {
            writeln!(
                output,
                "{}  @attests {}:{}",
                indent,
                attest.location.path.display(),
                attest.location.line
            )
            .expect("string writes should succeed");
            if let Some(body) = &attest.body {
                write_block_text(&mut output, body, self.depth + 2);
            }
        }

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

        for implementation in &self.node.implements {
            writeln!(
                output,
                "{}  {} {}:{}",
                indent,
                implementation_label(implementation),
                implementation.location.path.display(),
                implementation.location.line
            )
            .expect("string writes should succeed");
            if let Some(body_location) = &implementation.body_location {
                writeln!(
                    output,
                    "{}    body at: {}:{}",
                    indent,
                    body_location.path.display(),
                    body_location.line
                )
                .expect("string writes should succeed");
            }
            if let Some(body) = &implementation.body {
                write_block_text(&mut output, body, self.depth + 2);
            }
        }

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

fn format_architecture_coverage(coverage: &ProjectedArchitectureCoverage) -> String {
    let mut output = String::new();
    writeln!(output, "coverage").expect("string writes should succeed");
    for count in &coverage.counts {
        writeln!(output, "  {}: {}", count.label, count.value)
            .expect("string writes should succeed");
    }
    for path in &coverage.uncovered_paths {
        writeln!(output, "  uncovered path: {path}").expect("string writes should succeed");
    }
    for path in &coverage.weak_paths {
        writeln!(output, "  weak path: {path}").expect("string writes should succeed");
    }

    output
}

fn write_block_text(output: &mut String, body: &str, depth: usize) {
    let indent = text_indent(depth);
    for line in body.lines() {
        writeln!(output, "{indent}{line}").expect("string writes should succeed");
    }
}

fn render_projected_module_analysis(indent: &str, analysis: &ProjectedModuleAnalysis) -> String {
    let mut output = String::new();
    for count in &analysis.counts {
        writeln!(output, "{}  {}: {}", indent, count.label, count.value)
            .expect("string writes should succeed");
    }
    for explanation in &analysis.explanations {
        writeln!(
            output,
            "{}  {} meaning: {}",
            indent, explanation.label, explanation.plain
        )
        .expect("string writes should succeed");
        writeln!(
            output,
            "{}  {} exact: {}",
            indent, explanation.label, explanation.precise
        )
        .expect("string writes should succeed");
    }
    for line in &analysis.meta_lines {
        writeln!(output, "{}  {}: {}", indent, line.label, line.value)
            .expect("string writes should succeed");
    }
    output
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
