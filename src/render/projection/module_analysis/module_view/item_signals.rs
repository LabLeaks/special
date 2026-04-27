// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleAnalysisSummary;
use crate::modules::analyze::explain::MetricExplanationKey;

use super::super::{ProjectedCount, ProjectedExplanation, ProjectedMetaLine, count, explanation};
use super::support::push_item_group;

pub(super) fn append_item_signals(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    meta_lines: &mut Vec<ProjectedMetaLine>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(item_signals) = &analysis.item_signals else {
        return;
    };

    if item_signals.unreached_item_count > 0 {
        counts.push(count("unreached items", item_signals.unreached_item_count));
        if verbose {
            explanations.push(explanation(
                "unreached items",
                MetricExplanationKey::UnreachedItems,
            ));
        }
    }
    if verbose {
        meta_lines.push(ProjectedMetaLine {
            label: "item signals analyzed",
            value: item_signals.analyzed_items.to_string(),
        });
    }
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "connected item",
        MetricExplanationKey::ConnectedItem,
        &item_signals.connected_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "outbound-heavy item",
        MetricExplanationKey::OutboundHeavyItem,
        &item_signals.outbound_heavy_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "isolated item",
        MetricExplanationKey::IsolatedItem,
        &item_signals.isolated_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "unreached item",
        MetricExplanationKey::UnreachedItem,
        &item_signals.unreached_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "highest complexity item",
        MetricExplanationKey::HighestComplexityItem,
        &item_signals.highest_complexity_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "parameter-heavy item",
        MetricExplanationKey::ParameterHeavyItem,
        &item_signals.parameter_heavy_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "stringly boundary item",
        MetricExplanationKey::StringlyBoundaryItem,
        &item_signals.stringly_boundary_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "panic-heavy item",
        MetricExplanationKey::PanicHeavyItem,
        &item_signals.panic_heavy_items,
    );
}
