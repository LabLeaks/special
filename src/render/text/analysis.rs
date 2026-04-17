/**
@module SPECIAL.RENDER.TEXT.ANALYSIS
Formats projected module-analysis and architecture-coverage data into human-readable text sections. This module does not traverse spec or module trees.
*/
// @fileimplements SPECIAL.RENDER.TEXT.ANALYSIS
use std::fmt::Write;

use crate::render::projection::{
    ProjectedArchitectureTraceability, ProjectedModuleAnalysis, ProjectedRepoSignals,
};

pub(super) fn format_repo_signals(coverage: &ProjectedRepoSignals) -> String {
    if coverage.counts.is_empty()
        && coverage.unowned_unreached_items.is_empty()
        && coverage.duplicate_items.is_empty()
    {
        return String::new();
    }

    let mut output = String::new();
    writeln!(output, "repo-wide signals").expect("string writes should succeed");
    for count in &coverage.counts {
        writeln!(output, "  {}: {}", count.label, count.value)
            .expect("string writes should succeed");
    }
    for explanation in &coverage.explanations {
        writeln!(
            output,
            "  {} meaning: {}",
            explanation.label, explanation.plain
        )
        .expect("string writes should succeed");
        writeln!(
            output,
            "  {} exact: {}",
            explanation.label, explanation.precise
        )
        .expect("string writes should succeed");
    }
    for item in &coverage.unowned_unreached_items {
        writeln!(output, "  unowned unreached item: {item}").expect("string writes should succeed");
    }
    for item in &coverage.duplicate_items {
        writeln!(output, "  duplicate item: {item}").expect("string writes should succeed");
    }

    output
}

pub(super) fn format_repo_traceability(traceability: &ProjectedArchitectureTraceability) -> String {
    if traceability.counts.is_empty() && traceability.items.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    writeln!(output, "experimental traceability").expect("string writes should succeed");
    for count in &traceability.counts {
        writeln!(output, "  {}: {}", count.label, count.value)
            .expect("string writes should succeed");
    }
    for item in &traceability.items {
        writeln!(output, "  {}: {}", item.label, item.value).expect("string writes should succeed");
    }
    output
}

pub(super) fn render_projected_module_analysis(
    indent: &str,
    analysis: &ProjectedModuleAnalysis,
) -> String {
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
