/**
@module SPECIAL.MODULES.ANALYZE.REGISTRY
Projects the shared `SPECIAL.LANGUAGE_PACKS` registry onto implementation analysis so shared analysis flow can build repo contexts, module analysis, and repo traceability without hardcoding one dispatch branch per language in the analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.REGISTRY
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::language_packs::{self, LanguagePackAnalysisContext};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::syntax::SourceLanguage;

use super::{FileOwnership, ProviderModuleAnalysis};

pub(super) type RepoAnalysisContexts =
    BTreeMap<SourceLanguage, Box<dyn LanguagePackAnalysisContext>>;

pub(super) fn build_repo_analysis_contexts(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> RepoAnalysisContexts {
    let available_languages = languages_in_files(source_files);
    let mut contexts = BTreeMap::new();
    for descriptor in language_packs::descriptors() {
        if !available_languages.contains(&descriptor.language) {
            continue;
        }
        contexts.insert(
            descriptor.language,
            (descriptor.build_repo_analysis_context)(
                root,
                source_files,
                parsed_repo,
                parsed_architecture,
                file_ownership,
                include_traceability,
            ),
        );
    }
    contexts
}

pub(super) fn languages_in_files(files: &[PathBuf]) -> BTreeSet<SourceLanguage> {
    files
        .iter()
        .filter_map(|path| SourceLanguage::from_path(path))
        .collect()
}

pub(super) fn analysis_environment_fingerprint(root: &Path, files: &[PathBuf]) -> String {
    let languages = languages_in_files(files);
    let mut parts = Vec::new();
    for descriptor in language_packs::descriptors() {
        if !languages.contains(&descriptor.language) {
            continue;
        }
        parts.push(format!(
            "{}={}",
            descriptor.language.id(),
            (descriptor.analysis_environment_fingerprint)(root)
        ));
    }
    parts.join("|")
}

pub(super) fn summarize_repo_traceability(
    language: SourceLanguage,
    root: &Path,
    contexts: &RepoAnalysisContexts,
) -> Option<ArchitectureTraceabilitySummary> {
    contexts.get(&language)?.summarize_repo_traceability(root)
}

pub(super) fn traceability_unavailable_reason(
    language: SourceLanguage,
    contexts: &RepoAnalysisContexts,
) -> Option<String> {
    contexts.get(&language)?.traceability_unavailable_reason()
}

pub(super) fn analyze_module_language(
    language: SourceLanguage,
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    contexts: &RepoAnalysisContexts,
    options: ModuleAnalysisOptions,
) -> Result<ProviderModuleAnalysis> {
    let context = contexts.get(&language).ok_or_else(|| {
        anyhow::anyhow!(
            "metrics analysis expected a {} repo context but none was prepared",
            language.id()
        )
    })?;
    context.analyze_module(root, implementations, file_ownership, options)
}
