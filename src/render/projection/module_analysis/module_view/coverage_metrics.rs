// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleAnalysisSummary;

use super::super::{ProjectedCount, count};

pub(super) fn append_coverage(analysis: &ModuleAnalysisSummary, counts: &mut Vec<ProjectedCount>) {
    let Some(coverage) = &analysis.coverage else {
        return;
    };

    counts.push(count(
        "file-scoped implements",
        coverage.file_scoped_implements,
    ));
    counts.push(count(
        "item-scoped implements",
        coverage.item_scoped_implements,
    ));
}

pub(super) fn append_metrics(analysis: &ModuleAnalysisSummary, counts: &mut Vec<ProjectedCount>) {
    let Some(metrics) = &analysis.metrics else {
        return;
    };

    counts.push(count("owned lines", metrics.owned_lines));
    counts.push(count("public items", metrics.public_items));
    counts.push(count("internal items", metrics.internal_items));
}
