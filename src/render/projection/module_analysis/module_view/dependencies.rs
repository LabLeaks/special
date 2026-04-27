// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::ModuleAnalysisSummary;

use super::super::{ProjectedCount, ProjectedMetaLine, count};

pub(super) fn append_dependencies(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    meta_lines: &mut Vec<ProjectedMetaLine>,
) {
    let Some(dependencies) = &analysis.dependencies else {
        return;
    };

    counts.push(count("dependency refs", dependencies.reference_count));
    counts.push(count("dependency targets", dependencies.distinct_targets));
    if verbose {
        meta_lines.extend(dependencies.targets.iter().map(|target| ProjectedMetaLine {
            label: "dependency target",
            value: format!("{} ({})", target.path, target.count),
        }));
    }
}
