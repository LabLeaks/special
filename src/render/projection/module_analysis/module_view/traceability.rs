// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleAnalysisSummary;

use super::super::ProjectedMetaLine;
use super::support::push_traceability_group;

pub(super) fn append_traceability(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    meta_lines: &mut Vec<ProjectedMetaLine>,
) {
    if !verbose {
        return;
    }
    if let Some(reason) = &analysis.traceability_unavailable_reason {
        meta_lines.push(ProjectedMetaLine {
            label: "Rust backward trace unavailable",
            value: reason.clone(),
        });
        return;
    }

    let Some(traceability) = &analysis.traceability else {
        return;
    };

    meta_lines.push(ProjectedMetaLine {
        label: "traceability items analyzed",
        value: traceability.analyzed_items.to_string(),
    });
    push_traceability_group(
        meta_lines,
        "current spec item",
        &traceability.current_spec_items,
    );
    push_traceability_group(
        meta_lines,
        "planned-only item",
        &traceability.planned_only_items,
    );
    push_traceability_group(
        meta_lines,
        "deprecated-only item",
        &traceability.deprecated_only_items,
    );
    push_traceability_group(
        meta_lines,
        "file-scoped-only item",
        &traceability.file_scoped_only_items,
    );
    push_traceability_group(
        meta_lines,
        "unverified-test item",
        &traceability.unverified_test_items,
    );
    push_traceability_group(
        meta_lines,
        "statically mediated item",
        &traceability.statically_mediated_items,
    );
    push_traceability_group(
        meta_lines,
        "unexplained item",
        &traceability.unexplained_items,
    );
}
