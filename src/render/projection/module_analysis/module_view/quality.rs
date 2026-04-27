// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleAnalysisSummary;
use crate::modules::analyze::explain::MetricExplanationKey;

use super::super::{ProjectedCount, ProjectedExplanation, count, explanation};

pub(super) fn append_quality(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(quality) = &analysis.quality else {
        return;
    };

    counts.push(count(
        "quality public functions",
        quality.public_function_count,
    ));
    counts.push(count("quality parameters", quality.parameter_count));
    counts.push(count("quality bool params", quality.bool_parameter_count));
    counts.push(count(
        "quality raw string params",
        quality.raw_string_parameter_count,
    ));
    counts.push(count("quality panic sites", quality.panic_site_count));
    if verbose {
        explanations.push(explanation(
            "quality public functions",
            MetricExplanationKey::QualityPublicFunctions,
        ));
        explanations.push(explanation(
            "quality parameters",
            MetricExplanationKey::QualityParameters,
        ));
        explanations.push(explanation(
            "quality bool params",
            MetricExplanationKey::QualityBoolParameters,
        ));
        explanations.push(explanation(
            "quality raw string params",
            MetricExplanationKey::QualityRawStringParameters,
        ));
        explanations.push(explanation(
            "quality panic sites",
            MetricExplanationKey::QualityPanicSites,
        ));
    }
}
