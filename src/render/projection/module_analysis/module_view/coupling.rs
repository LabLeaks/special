// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleAnalysisSummary;
use crate::modules::analyze::explain::MetricExplanationKey;

use super::super::{ProjectedCount, ProjectedExplanation, count, explanation};

pub(super) fn append_coupling(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(coupling) = &analysis.coupling else {
        return;
    };

    counts.push(count("fan in", coupling.fan_in));
    counts.push(count("fan out", coupling.fan_out));
    counts.push(count("afferent coupling", coupling.afferent_coupling));
    counts.push(count("efferent coupling", coupling.efferent_coupling));
    counts.push(ProjectedCount {
        label: "instability",
        value: format!("{:.2}", coupling.instability),
    });
    counts.push(count(
        "external dependency targets",
        coupling.external_target_count,
    ));
    counts.push(count(
        "ambiguous internal dependency targets",
        coupling.ambiguous_internal_target_count,
    ));
    counts.push(count(
        "unresolved internal dependency targets",
        coupling.unresolved_internal_target_count,
    ));
    if verbose {
        explanations.push(explanation("fan in", MetricExplanationKey::FanIn));
        explanations.push(explanation("fan out", MetricExplanationKey::FanOut));
        explanations.push(explanation(
            "afferent coupling",
            MetricExplanationKey::AfferentCoupling,
        ));
        explanations.push(explanation(
            "efferent coupling",
            MetricExplanationKey::EfferentCoupling,
        ));
        explanations.push(explanation(
            "instability",
            MetricExplanationKey::Instability,
        ));
    }
}
