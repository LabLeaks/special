// @fileimplements SPECIAL.MODULES.ANALYZE.MODULE_SUMMARY
use crate::model::{ImplementRef, ModuleCoverageSummary};

pub(super) fn implementations_for_module<'a>(
    parsed: &'a crate::model::ParsedArchitecture,
    module_id: &str,
) -> Vec<&'a ImplementRef> {
    parsed
        .implements
        .iter()
        .filter(|implementation| implementation.module_id == module_id)
        .collect()
}

pub(super) fn summarize_coverage(implementations: &[&ImplementRef]) -> ModuleCoverageSummary {
    ModuleCoverageSummary {
        file_scoped_implements: implementations
            .iter()
            .filter(|implementation| implementation.body_location.is_none())
            .count(),
        item_scoped_implements: implementations
            .iter()
            .filter(|implementation| implementation.body_location.is_some())
            .count(),
    }
}
