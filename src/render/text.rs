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
    ArchitectureKind, ArchitectureMetricsSummary, DiagnosticSeverity, GroupedCount, LintReport,
    ModuleDocument, ModuleNode, OverviewDocument, RepoDocument, RepoMetricsSummary,
    RepoTraceabilityMetrics, SpecDocument, SpecMetricsSummary, SpecNode,
};

use self::analysis::{
    format_repo_signals, format_repo_traceability, render_projected_module_analysis,
};
use self::attachments::{
    render_attest_section, render_implementation_section, render_verify_section,
};
use super::common::{deprecated_badge_text, planned_badge_text};
use super::projection::{
    project_document, project_module_analysis_view, project_module_document,
    project_repo_signals_view, project_repo_traceability_view,
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

    fn deprecated_badge(&self) -> String {
        deprecated_badge_text(self.node.deprecated_release())
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
    if document.nodes.is_empty() && document.metrics.is_none() {
        return "No specs found.".to_string();
    }

    let rendered = render_template(&SpecPageTextTemplate {
        nodes: &document.nodes,
        verbose,
    });
    if let Some(metrics) = &document.metrics {
        let mut output = render_spec_metrics_text(metrics);
        if !document.nodes.is_empty() {
            output.push('\n');
            output.push_str(&rendered);
        }
        output
    } else {
        rendered
    }
}

pub(super) fn render_module_text(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    if document.nodes.is_empty() {
        return "No modules found.".to_string();
    }

    let rendered = render_template(&ModulePageTextTemplate {
        document: &document,
        verbose,
    });
    if let Some(metrics) = &document.metrics {
        let mut output = render_arch_metrics_text(metrics);
        if !document.nodes.is_empty() {
            output.push('\n');
            output.push_str(&rendered);
        }
        output
    } else {
        rendered
    }
}

pub(super) fn render_repo_text(document: &RepoDocument, verbose: bool) -> String {
    let document = super::projection::project_repo_document(document, verbose);
    let mut output = String::from("special health\n");
    if let Some(metrics) = &document.metrics {
        output.push_str(&render_repo_metrics_text(metrics));
    }
    if let Some(repo_signals) = document
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.repo_signals.as_ref())
    {
        output.push_str(&format_repo_signals(&project_repo_signals_view(
            repo_signals,
            verbose,
        )));
    }
    if let Some(analysis) = document.analysis.as_ref() {
        output.push_str(&format_repo_traceability(&project_repo_traceability_view(
            analysis.traceability.as_ref(),
            analysis.traceability_unavailable_reason.as_deref(),
        )));
    }
    output
}

fn render_spec_metrics_text(metrics: &SpecMetricsSummary) -> String {
    let mut output = String::from("special specs metrics\n");
    output.push_str(&format!("  total specs: {}\n", metrics.total_specs));
    output.push_str(&format!(
        "  unverified specs: {}\n",
        metrics.unverified_specs
    ));
    output.push_str(&format!("  planned specs: {}\n", metrics.planned_specs));
    output.push_str(&format!(
        "  deprecated specs: {}\n",
        metrics.deprecated_specs
    ));
    output.push_str(&format!("  verified specs: {}\n", metrics.verified_specs));
    output.push_str(&format!("  attested specs: {}\n", metrics.attested_specs));
    output.push_str(&format!(
        "  specs with both supports: {}\n",
        metrics.specs_with_both_supports
    ));
    output.push_str(&format!("  verifies: {}\n", metrics.verifies));
    output.push_str(&format!(
        "    item-scoped verifies: {}\n",
        metrics.item_scoped_verifies
    ));
    output.push_str(&format!(
        "    file-scoped verifies: {}\n",
        metrics.file_scoped_verifies
    ));
    output.push_str(&format!(
        "    unattached verifies: {}\n",
        metrics.unattached_verifies
    ));
    output.push_str(&format!("  attests: {}\n", metrics.attests));
    output.push_str(&format!("    block attests: {}\n", metrics.block_attests));
    output.push_str(&format!("    file attests: {}\n", metrics.file_attests));
    append_grouped_counts_text(&mut output, "specs by file", &metrics.specs_by_file);
    append_grouped_counts_text(
        &mut output,
        "current specs by top-level id",
        &metrics.current_specs_by_top_level_id,
    );
    output
}

fn render_repo_metrics_text(metrics: &RepoMetricsSummary) -> String {
    let mut output = String::from("special health metrics\n");
    output.push_str(&format!("  duplicate items: {}\n", metrics.duplicate_items));
    output.push_str(&format!("  unowned items: {}\n", metrics.unowned_items));
    append_grouped_counts_text(
        &mut output,
        "duplicate items by file",
        &metrics.duplicate_items_by_file,
    );
    append_grouped_counts_text(
        &mut output,
        "unowned items by file",
        &metrics.unowned_items_by_file,
    );
    if let Some(traceability) = &metrics.traceability {
        append_repo_traceability_metrics_text(&mut output, traceability);
    }
    output
}

fn render_arch_metrics_text(metrics: &ArchitectureMetricsSummary) -> String {
    let mut output = String::from("special arch metrics\n");
    output.push_str(&format!("  total modules: {}\n", metrics.total_modules));
    output.push_str(&format!("  total areas: {}\n", metrics.total_areas));
    output.push_str(&format!(
        "  unimplemented modules: {}\n",
        metrics.unimplemented_modules
    ));
    output.push_str(&format!(
        "  file-scoped implements: {}\n",
        metrics.file_scoped_implements
    ));
    output.push_str(&format!(
        "  item-scoped implements: {}\n",
        metrics.item_scoped_implements
    ));
    output.push_str(&format!("  owned lines: {}\n", metrics.owned_lines));
    output.push_str(&format!("  public items: {}\n", metrics.public_items));
    output.push_str(&format!("  internal items: {}\n", metrics.internal_items));
    output.push_str(&format!(
        "  complexity functions: {}\n",
        metrics.complexity_functions
    ));
    output.push_str(&format!(
        "  total cyclomatic: {}\n",
        metrics.total_cyclomatic
    ));
    output.push_str(&format!("  max cyclomatic: {}\n", metrics.max_cyclomatic));
    output.push_str(&format!("  total cognitive: {}\n", metrics.total_cognitive));
    output.push_str(&format!("  max cognitive: {}\n", metrics.max_cognitive));
    output.push_str(&format!(
        "  quality public functions: {}\n",
        metrics.quality_public_functions
    ));
    output.push_str(&format!(
        "  quality parameters: {}\n",
        metrics.quality_parameters
    ));
    output.push_str(&format!(
        "  quality bool params: {}\n",
        metrics.quality_bool_params
    ));
    output.push_str(&format!(
        "  quality raw string params: {}\n",
        metrics.quality_raw_string_params
    ));
    output.push_str(&format!(
        "  quality panic sites: {}\n",
        metrics.quality_panic_sites
    ));
    output.push_str(&format!("  unreached items: {}\n", metrics.unreached_items));
    append_grouped_counts_text(&mut output, "modules by area", &metrics.modules_by_area);
    append_grouped_counts_text(
        &mut output,
        "owned lines by module",
        &metrics.owned_lines_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "max cyclomatic by module",
        &metrics.max_cyclomatic_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "max cognitive by module",
        &metrics.max_cognitive_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "panic sites by module",
        &metrics.panic_sites_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "unreached items by module",
        &metrics.unreached_items_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "external dependency targets by module",
        &metrics.external_dependency_targets_by_module,
    );
    output
}

fn append_repo_traceability_metrics_text(output: &mut String, metrics: &RepoTraceabilityMetrics) {
    output.push_str("  traceability\n");
    output.push_str(&format!("    analyzed items: {}\n", metrics.analyzed_items));
    output.push_str(&format!(
        "    current spec items: {}\n",
        metrics.current_spec_items
    ));
    output.push_str(&format!(
        "    statically mediated items: {}\n",
        metrics.statically_mediated_items
    ));
    output.push_str(&format!(
        "    unverified test items: {}\n",
        metrics.unverified_test_items
    ));
    output.push_str(&format!(
        "    unexplained items: {}\n",
        metrics.unexplained_items
    ));
    output.push_str(&format!(
        "    unexplained review-surface items: {}\n",
        metrics.unexplained_review_surface_items
    ));
    output.push_str(&format!(
        "    unexplained public items: {}\n",
        metrics.unexplained_public_items
    ));
    output.push_str(&format!(
        "    unexplained internal items: {}\n",
        metrics.unexplained_internal_items
    ));
    output.push_str(&format!(
        "    unexplained module-backed items: {}\n",
        metrics.unexplained_module_backed_items
    ));
    output.push_str(&format!(
        "    unexplained module-connected items: {}\n",
        metrics.unexplained_module_connected_items
    ));
    output.push_str(&format!(
        "    unexplained module-isolated items: {}\n",
        metrics.unexplained_module_isolated_items
    ));
    append_grouped_counts_text(
        output,
        "unexplained items by file",
        &metrics.unexplained_items_by_file,
    );
    append_grouped_counts_text(
        output,
        "unexplained review-surface items by file",
        &metrics.unexplained_review_surface_items_by_file,
    );
}

fn append_grouped_counts_text(output: &mut String, label: &str, counts: &[GroupedCount]) {
    if counts.is_empty() {
        return;
    }
    output.push_str(&format!("  {label}\n"));
    counts.iter().for_each(|group| {
        output.push_str(&format!("    {}: {}\n", group.value, group.count));
    });
}

pub(super) fn render_overview_text(document: &OverviewDocument) -> String {
    let mut output = String::from("special\n");
    output.push_str("  lint\n");
    output.push_str(&format!("    errors: {}\n", document.lint.errors));
    output.push_str(&format!("    warnings: {}\n", document.lint.warnings));
    output.push_str("  specs\n");
    output.push_str(&format!(
        "    total specs: {}\n",
        document.specs.total_specs
    ));
    output.push_str(&format!(
        "    unverified specs: {}\n",
        document.specs.unverified_specs
    ));
    output.push_str(&format!(
        "    planned specs: {}\n",
        document.specs.planned_specs
    ));
    output.push_str(&format!(
        "    deprecated specs: {}\n",
        document.specs.deprecated_specs
    ));
    output.push_str("  arch\n");
    output.push_str(&format!("    modules: {}\n", document.arch.total_modules));
    output.push_str(&format!("    areas: {}\n", document.arch.total_areas));
    output.push_str(&format!(
        "    unimplemented modules: {}\n",
        document.arch.unimplemented_modules
    ));
    output.push_str("  health\n");
    output.push_str(&format!(
        "    duplicate items: {}\n",
        document.health.duplicate_items
    ));
    output.push_str(&format!(
        "    unowned items: {}\n",
        document.health.unowned_items
    ));

    output.push_str("  look next\n");
    output.push_str("    special lint\n");
    output.push_str("    special specs\n");
    output.push_str("    special specs --metrics\n");
    output.push_str("    special specs --unverified\n");
    output.push_str("    special arch\n");
    output.push_str("    special arch --metrics\n");
    output.push_str("    special arch --unimplemented\n");
    output.push_str("    special health\n");
    output.push_str("    special health --metrics\n");

    output
}

pub(super) fn render_lint_text(report: &LintReport) -> String {
    if report.diagnostics.is_empty() {
        return "Lint clean.".to_string();
    }

    render_template(&LintTextTemplate { report })
        .trim_end()
        .to_string()
}
