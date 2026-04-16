/**
@module SPECIAL.RENDER.TEXT.ANALYSIS
Formats projected module-analysis and architecture-coverage data into human-readable text sections. This module does not traverse spec or module trees.
*/
// @fileimplements SPECIAL.RENDER.TEXT.ANALYSIS
use std::fmt::Write;

use crate::render::projection::{ProjectedArchitectureCoverage, ProjectedModuleAnalysis};

pub(super) fn format_architecture_coverage(coverage: &ProjectedArchitectureCoverage) -> String {
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
