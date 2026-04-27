// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleAnalysisSummary;
use crate::modules::analyze::explain::MetricExplanationKey;

use super::super::{ProjectedCount, ProjectedExplanation, count, explanation};

pub(super) fn append_complexity(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(complexity) = &analysis.complexity else {
        return;
    };

    counts.push(count("complexity functions", complexity.function_count));
    counts.push(count("cyclomatic total", complexity.total_cyclomatic));
    counts.push(count("cyclomatic max", complexity.max_cyclomatic));
    counts.push(count("cognitive total", complexity.total_cognitive));
    counts.push(count("cognitive max", complexity.max_cognitive));
    if verbose {
        explanations.push(explanation(
            "cyclomatic total",
            MetricExplanationKey::CyclomaticTotal,
        ));
        explanations.push(explanation(
            "cyclomatic max",
            MetricExplanationKey::CyclomaticMax,
        ));
        explanations.push(explanation(
            "cognitive total",
            MetricExplanationKey::CognitiveTotal,
        ));
        explanations.push(explanation(
            "cognitive max",
            MetricExplanationKey::CognitiveMax,
        ));
    }
}
