/**
@module SPECIAL.RENDER.TEXT
Renders projected specs, modules, and lint diagnostics into human-readable text output.
*/
// @fileimplements SPECIAL.RENDER.TEXT
use std::fmt::Write;

use askama::Template;

use crate::model::{
    ArchitectureCoverageSummary, ArchitectureKind, DiagnosticSeverity, LintReport, ModuleDocument,
    ModuleNode, SpecDocument, SpecNode,
};
use crate::modules::analyze::explain::{
    MetricExplanation, MetricExplanationKey, metric_explanation,
};

use super::common::planned_badge_text;
use super::projection::{project_document, project_module_document};
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
            .map(|coverage| format_architecture_coverage(coverage, self.verbose))
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
        let mut output = String::new();
        let Some(analysis) = &self.node.analysis else {
            return output;
        };

        if let Some(coverage) = &analysis.coverage {
            writeln!(
                output,
                "{}  covered files: {}",
                indent, coverage.covered_files
            )
            .expect("string writes should succeed");
            writeln!(output, "{}  weak files: {}", indent, coverage.weak_files)
                .expect("string writes should succeed");
            writeln!(
                output,
                "{}  file-scoped implements: {}",
                indent, coverage.file_scoped_implements
            )
            .expect("string writes should succeed");
            writeln!(
                output,
                "{}  item-scoped implements: {}",
                indent, coverage.item_scoped_implements
            )
            .expect("string writes should succeed");
        }
        if let Some(metrics) = &analysis.metrics {
            writeln!(output, "{}  owned lines: {}", indent, metrics.owned_lines)
                .expect("string writes should succeed");
            writeln!(output, "{}  public items: {}", indent, metrics.public_items)
                .expect("string writes should succeed");
            writeln!(
                output,
                "{}  internal items: {}",
                indent, metrics.internal_items
            )
            .expect("string writes should succeed");
        }
        if let Some(complexity) = &analysis.complexity {
            writeln!(
                output,
                "{}  complexity functions: {}",
                indent, complexity.function_count
            )
            .expect("string writes should succeed");
            writeln!(
                output,
                "{}  cyclomatic total: {}",
                indent, complexity.total_cyclomatic
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "cyclomatic total",
                metric_explanation(MetricExplanationKey::CyclomaticTotal),
            );
            writeln!(
                output,
                "{}  cyclomatic max: {}",
                indent, complexity.max_cyclomatic
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "cyclomatic max",
                metric_explanation(MetricExplanationKey::CyclomaticMax),
            );
            writeln!(
                output,
                "{}  cognitive total: {}",
                indent, complexity.total_cognitive
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "cognitive total",
                metric_explanation(MetricExplanationKey::CognitiveTotal),
            );
            writeln!(
                output,
                "{}  cognitive max: {}",
                indent, complexity.max_cognitive
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "cognitive max",
                metric_explanation(MetricExplanationKey::CognitiveMax),
            );
        }
        if let Some(quality) = &analysis.quality {
            writeln!(
                output,
                "{}  quality public functions: {}",
                indent, quality.public_function_count
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "quality public functions",
                metric_explanation(MetricExplanationKey::QualityPublicFunctions),
            );
            writeln!(
                output,
                "{}  quality parameters: {}",
                indent, quality.parameter_count
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "quality parameters",
                metric_explanation(MetricExplanationKey::QualityParameters),
            );
            writeln!(
                output,
                "{}  quality bool params: {}",
                indent, quality.bool_parameter_count
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "quality bool params",
                metric_explanation(MetricExplanationKey::QualityBoolParameters),
            );
            writeln!(
                output,
                "{}  quality raw string params: {}",
                indent, quality.raw_string_parameter_count
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "quality raw string params",
                metric_explanation(MetricExplanationKey::QualityRawStringParameters),
            );
            writeln!(
                output,
                "{}  quality panic sites: {}",
                indent, quality.panic_site_count
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "quality panic sites",
                metric_explanation(MetricExplanationKey::QualityPanicSites),
            );
        }
        if let Some(item_signals) = &analysis.item_signals {
            writeln!(
                output,
                "{}  item signals analyzed: {}",
                indent, item_signals.analyzed_items
            )
            .expect("string writes should succeed");
            for item in &item_signals.connected_items {
                write_item_signal_line(&mut output, &indent, "connected item", item);
            }
            if !item_signals.connected_items.is_empty() {
                write_metric_explanation(
                    &mut output,
                    &indent,
                    "connected item",
                    metric_explanation(MetricExplanationKey::ConnectedItem),
                );
            }
            for item in &item_signals.outbound_heavy_items {
                write_item_signal_line(&mut output, &indent, "outbound-heavy item", item);
            }
            if !item_signals.outbound_heavy_items.is_empty() {
                write_metric_explanation(
                    &mut output,
                    &indent,
                    "outbound-heavy item",
                    metric_explanation(MetricExplanationKey::OutboundHeavyItem),
                );
            }
            for item in &item_signals.isolated_items {
                write_item_signal_line(&mut output, &indent, "isolated item", item);
            }
            if !item_signals.isolated_items.is_empty() {
                write_metric_explanation(
                    &mut output,
                    &indent,
                    "isolated item",
                    metric_explanation(MetricExplanationKey::IsolatedItem),
                );
            }
            for item in &item_signals.highest_complexity_items {
                write_item_signal_line(&mut output, &indent, "highest complexity item", item);
            }
            if !item_signals.highest_complexity_items.is_empty() {
                write_metric_explanation(
                    &mut output,
                    &indent,
                    "highest complexity item",
                    metric_explanation(MetricExplanationKey::HighestComplexityItem),
                );
            }
            for item in &item_signals.parameter_heavy_items {
                write_item_signal_line(&mut output, &indent, "parameter-heavy item", item);
            }
            if !item_signals.parameter_heavy_items.is_empty() {
                write_metric_explanation(
                    &mut output,
                    &indent,
                    "parameter-heavy item",
                    metric_explanation(MetricExplanationKey::ParameterHeavyItem),
                );
            }
            for item in &item_signals.stringly_boundary_items {
                write_item_signal_line(&mut output, &indent, "stringly boundary item", item);
            }
            if !item_signals.stringly_boundary_items.is_empty() {
                write_metric_explanation(
                    &mut output,
                    &indent,
                    "stringly boundary item",
                    metric_explanation(MetricExplanationKey::StringlyBoundaryItem),
                );
            }
            for item in &item_signals.panic_heavy_items {
                write_item_signal_line(&mut output, &indent, "panic-heavy item", item);
            }
            if !item_signals.panic_heavy_items.is_empty() {
                write_metric_explanation(
                    &mut output,
                    &indent,
                    "panic-heavy item",
                    metric_explanation(MetricExplanationKey::PanicHeavyItem),
                );
            }
        }
        if let Some(coupling) = &analysis.coupling {
            writeln!(output, "{}  fan in: {}", indent, coupling.fan_in)
                .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "fan in",
                metric_explanation(MetricExplanationKey::FanIn),
            );
            writeln!(output, "{}  fan out: {}", indent, coupling.fan_out)
                .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "fan out",
                metric_explanation(MetricExplanationKey::FanOut),
            );
            writeln!(
                output,
                "{}  afferent coupling: {}",
                indent, coupling.afferent_coupling
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "afferent coupling",
                metric_explanation(MetricExplanationKey::AfferentCoupling),
            );
            writeln!(
                output,
                "{}  efferent coupling: {}",
                indent, coupling.efferent_coupling
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "efferent coupling",
                metric_explanation(MetricExplanationKey::EfferentCoupling),
            );
            writeln!(
                output,
                "{}  instability: {:.2}",
                indent, coupling.instability
            )
            .expect("string writes should succeed");
            write_metric_explanation(
                &mut output,
                &indent,
                "instability",
                metric_explanation(MetricExplanationKey::Instability),
            );
            writeln!(
                output,
                "{}  external dependency targets: {}",
                indent, coupling.external_target_count
            )
            .expect("string writes should succeed");
            writeln!(
                output,
                "{}  unresolved internal dependency targets: {}",
                indent, coupling.unresolved_internal_target_count
            )
            .expect("string writes should succeed");
        }
        if let Some(dependencies) = &analysis.dependencies {
            writeln!(
                output,
                "{}  dependency refs: {}",
                indent, dependencies.reference_count
            )
            .expect("string writes should succeed");
            writeln!(
                output,
                "{}  dependency targets: {}",
                indent, dependencies.distinct_targets
            )
            .expect("string writes should succeed");
            for target in &dependencies.targets {
                writeln!(
                    output,
                    "{}  dependency target: {} ({})",
                    indent, target.path, target.count
                )
                .expect("string writes should succeed");
            }
        }

        output
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

        if let Some(analysis) = &self.node.analysis
            && let Some(coverage) = &analysis.coverage
        {
            for path in &coverage.covered_paths {
                writeln!(output, "{}  covered file: {}", indent, path.display())
                    .expect("string writes should succeed");
            }
            for path in &coverage.weak_paths {
                writeln!(output, "{}  weak file: {}", indent, path.display())
                    .expect("string writes should succeed");
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

fn format_architecture_coverage(coverage: &ArchitectureCoverageSummary, verbose: bool) -> String {
    let mut output = String::new();
    writeln!(output, "coverage").expect("string writes should succeed");
    writeln!(output, "  analyzed files: {}", coverage.analyzed_files)
        .expect("string writes should succeed");
    writeln!(output, "  covered files: {}", coverage.covered_files)
        .expect("string writes should succeed");
    writeln!(output, "  uncovered files: {}", coverage.uncovered_files)
        .expect("string writes should succeed");
    writeln!(output, "  weak files: {}", coverage.weak_files)
        .expect("string writes should succeed");

    if verbose {
        for path in &coverage.uncovered_paths {
            writeln!(output, "  uncovered path: {}", path.display())
                .expect("string writes should succeed");
        }
        for path in &coverage.weak_paths {
            writeln!(output, "  weak path: {}", path.display())
                .expect("string writes should succeed");
        }
    }

    output
}

fn write_block_text(output: &mut String, body: &str, depth: usize) {
    let indent = text_indent(depth);
    for line in body.lines() {
        writeln!(output, "{indent}{line}").expect("string writes should succeed");
    }
}

fn write_metric_explanation(
    output: &mut String,
    indent: &str,
    label: &str,
    explanation: MetricExplanation,
) {
    writeln!(
        output,
        "{}  {} meaning: {}",
        indent, label, explanation.plain
    )
    .expect("string writes should succeed");
    writeln!(
        output,
        "{}  {} exact: {}",
        indent, label, explanation.precise
    )
    .expect("string writes should succeed");
}

fn write_item_signal_line(
    output: &mut String,
    indent: &str,
    label: &str,
    item: &crate::model::ModuleItemSignal,
) {
    writeln!(
        output,
        "{}  {}: {} [{}; params {} (bool {}, raw string {}), internal refs {}, inbound {}, external refs {}, cyclomatic {}, cognitive {}, panic sites {}]",
        indent,
        label,
        item.name,
        item_kind_label(item.kind),
        item.parameter_count,
        item.bool_parameter_count,
        item.raw_string_parameter_count,
        item.internal_refs,
        item.inbound_internal_refs,
        item.external_refs,
        item.cyclomatic,
        item.cognitive,
        item.panic_site_count
    )
    .expect("string writes should succeed");
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

fn item_kind_label(kind: crate::model::ModuleItemKind) -> &'static str {
    match kind {
        crate::model::ModuleItemKind::Function => "function",
        crate::model::ModuleItemKind::Method => "method",
    }
}
