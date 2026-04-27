// @fileimplements SPECIAL.MODULES.ANALYZE.MODULE_SUMMARY
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{ImplementRef, ModuleAnalysisOptions};
use crate::syntax::SourceLanguage;

use super::super::{
    FileOwnership, ProviderModuleAnalysis, ownership::build_module_metrics,
    provider_merge::merge_provider_module_analysis, registry,
};

pub(super) fn build_provider_analysis(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    repo_contexts: &registry::RepoAnalysisContexts,
    options: ModuleAnalysisOptions,
) -> Result<ProviderModuleAnalysis> {
    let mut provider = ProviderModuleAnalysis {
        metrics: build_module_metrics(root, implementations, file_ownership)?,
        ..ProviderModuleAnalysis::default()
    };

    for language in languages_for_implementations(implementations) {
        let delta = registry::analyze_module_language(
            language,
            root,
            implementations,
            file_ownership,
            repo_contexts,
            options,
        )?;
        merge_provider_module_analysis(&mut provider, delta);
    }

    Ok(provider)
}

fn languages_for_implementations(implementations: &[&ImplementRef]) -> BTreeSet<SourceLanguage> {
    implementations
        .iter()
        .filter_map(|implementation| SourceLanguage::from_path(&implementation.location.path))
        .collect()
}
