/**
@module SPECIAL.LANGUAGE_PACKS
Owns compile-time language-pack registration and the shared descriptor boundary between syntax parsing, implementation analysis, and pack-specific local-tool enrichers. Adding a built-in pack should reduce to adding one pack entry file under this directory plus its own implementation files, while the shared core consumes the generated pack registry without hardcoded per-language match arms.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::modules::analyze::{FileOwnership, ProviderModuleAnalysis};
use crate::syntax::{ParsedSourceGraph, SourceLanguage};

pub(crate) trait LanguagePackAnalysisContext {
    fn summarize_repo_traceability(&self, root: &Path) -> Option<ArchitectureTraceabilitySummary>;

    fn traceability_unavailable_reason(&self) -> Option<String>;

    fn analyze_module(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
        options: ModuleAnalysisOptions,
    ) -> Result<ProviderModuleAnalysis>;
}

pub(crate) type BuildRepoAnalysisContextFn = for<'a> fn(
    &Path,
    &[PathBuf],
    &ParsedRepo,
    &ParsedArchitecture,
    &BTreeMap<PathBuf, FileOwnership<'a>>,
    bool,
) -> Box<dyn LanguagePackAnalysisContext>;

pub(crate) struct LanguagePackDescriptor {
    pub(crate) language: SourceLanguage,
    pub(crate) matches_path: fn(&Path) -> bool,
    pub(crate) parse_source_graph: fn(&Path, &str) -> Option<ParsedSourceGraph>,
    pub(crate) build_repo_analysis_context: BuildRepoAnalysisContextFn,
    pub(crate) analysis_environment_fingerprint: fn(&Path) -> String,
}

include!(concat!(env!("OUT_DIR"), "/language_pack_registry.rs"));

pub(crate) fn descriptors() -> &'static [&'static LanguagePackDescriptor] {
    REGISTERED_LANGUAGE_PACKS
}
