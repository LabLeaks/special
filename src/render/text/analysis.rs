/**
@module SPECIAL.RENDER.TEXT.ANALYSIS
Formats projected module-analysis and architecture-coverage data into human-readable text sections. This module does not traverse spec or module trees.
*/
// @fileimplements SPECIAL.RENDER.TEXT.ANALYSIS
use std::fmt::Write;

use crate::render::projection::{
    ProjectedArchitectureTraceability, ProjectedCount, ProjectedExplanation, ProjectedMetaLine,
    ProjectedModuleAnalysis, ProjectedRepoSignals,
};

pub(super) fn format_repo_signals(coverage: &ProjectedRepoSignals) -> String {
    if coverage.counts.is_empty()
        && coverage.unowned_items.is_empty()
        && coverage.duplicate_items.is_empty()
    {
        return String::new();
    }

    let mut output = String::new();
    writeln!(output, "repo-wide signals").expect("string writes should succeed");
    append_count_lines(&mut output, "  ", &coverage.counts);
    append_explanation_lines(&mut output, "  ", &coverage.explanations);
    for item in &coverage.unowned_items {
        writeln!(output, "  unowned item: {item}").expect("string writes should succeed");
    }
    for item in &coverage.duplicate_items {
        writeln!(output, "  duplicate item: {item}").expect("string writes should succeed");
    }

    output
}

pub(super) fn format_repo_traceability(traceability: &ProjectedArchitectureTraceability) -> String {
    if traceability.counts.is_empty()
        && traceability.items.is_empty()
        && traceability.explanations.is_empty()
        && traceability.unavailable_reason.is_none()
    {
        return String::new();
    }

    let mut output = String::new();
    writeln!(output, "traceability").expect("string writes should succeed");
    if let Some(reason) = &traceability.unavailable_reason {
        writeln!(output, "  unavailable: {reason}").expect("string writes should succeed");
    }
    append_count_lines(&mut output, "  ", &traceability.counts);
    append_explanation_lines(&mut output, "  ", &traceability.explanations);
    append_meta_lines(&mut output, "  ", &traceability.items);
    output
}

pub(super) fn render_projected_module_analysis(
    indent: &str,
    analysis: &ProjectedModuleAnalysis,
) -> String {
    let mut output = String::new();
    let item_indent = format!("{indent}  ");
    append_count_lines(&mut output, &item_indent, &analysis.counts);
    append_explanation_lines(&mut output, &item_indent, &analysis.explanations);
    append_meta_lines(&mut output, &item_indent, &analysis.meta_lines);
    output
}

fn append_count_lines(output: &mut String, indent: &str, counts: &[ProjectedCount]) {
    for count in counts {
        writeln!(output, "{indent}{}: {}", count.label, count.value)
            .expect("string writes should succeed");
    }
}

fn append_explanation_lines(
    output: &mut String,
    indent: &str,
    explanations: &[ProjectedExplanation],
) {
    for explanation in explanations {
        writeln!(
            output,
            "{indent}{} meaning: {}",
            explanation.label, explanation.plain
        )
        .expect("string writes should succeed");
        writeln!(
            output,
            "{indent}{} exact: {}",
            explanation.label, explanation.precise
        )
        .expect("string writes should succeed");
    }
}

fn append_meta_lines(output: &mut String, indent: &str, lines: &[ProjectedMetaLine]) {
    for line in lines {
        writeln!(output, "{indent}{}: {}", line.label, line.value)
            .expect("string writes should succeed");
    }
}
