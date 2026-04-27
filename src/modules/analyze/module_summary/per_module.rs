// @fileimplements SPECIAL.MODULES.ANALYZE.MODULE_SUMMARY
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ModuleAnalysisOptions, ModuleAnalysisSummary, ModuleCouplingSummary, ModuleDecl,
    ParsedArchitecture,
};

use super::super::{FileOwnership, coupling, registry};
use super::{coverage, provider};

pub(super) fn build_module_summary(
    root: &Path,
    module: &ModuleDecl,
    parsed: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    repo_contexts: &registry::RepoAnalysisContexts,
    options: ModuleAnalysisOptions,
) -> Result<
    Option<(
        String,
        ModuleAnalysisSummary,
        Option<coupling::ModuleCouplingInput>,
    )>,
> {
    let implementations = coverage::implementations_for_module(parsed, &module.id);
    if implementations.is_empty() && !options.any() {
        return Ok(None);
    }

    let coverage = options
        .coverage
        .then(|| coverage::summarize_coverage(&implementations));

    let (
        metrics,
        complexity,
        quality,
        item_signals,
        traceability,
        traceability_unavailable_reason,
        coupling_input,
        dependencies,
    ) = if options.metrics {
        let provider = provider::build_provider_analysis(
            root,
            &implementations,
            file_ownership,
            repo_contexts,
            options,
        )?;

        (
            Some(provider.metrics),
            provider.complexity,
            provider.quality,
            provider.item_signals,
            provider.traceability,
            provider.traceability_unavailable_reason,
            provider.coupling,
            provider.dependencies,
        )
    } else {
        (None, None, None, None, None, None, None, None)
    };

    Ok(Some((
        module.id.clone(),
        ModuleAnalysisSummary {
            coverage,
            metrics,
            complexity,
            quality,
            item_signals,
            traceability,
            traceability_unavailable_reason,
            coupling: coupling_input
                .as_ref()
                .map(|_| ModuleCouplingSummary::default()),
            dependencies,
        },
        coupling_input,
    )))
}
