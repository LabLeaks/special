/**
@module SPECIAL.LANGUAGE_PACKS
Owns compile-time language-pack registration and the shared descriptor boundary between syntax parsing, implementation analysis, scoped traceability preparation, and pack-specific local-tool enrichers. Adding a built-in pack should reduce to adding one pack entry file under this directory plus its own implementation files, while the shared core consumes the generated pack registry without hardcoded per-language match arms.
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
    Option<&[PathBuf]>,
    Option<&[u8]>,
    &ParsedRepo,
    &ParsedArchitecture,
    &BTreeMap<PathBuf, FileOwnership<'a>>,
    bool,
) -> Box<dyn LanguagePackAnalysisContext>;

pub(crate) type BuildTraceabilityScopeFactsFn = for<'a> fn(
    &Path,
    &[PathBuf],
    &[PathBuf],
    &ParsedRepo,
    &BTreeMap<PathBuf, FileOwnership<'a>>,
) -> Result<Vec<u8>>;

pub(crate) type ExpandTraceabilityClosureFromFactsFn = fn(
    &[PathBuf],
    &[PathBuf],
    &BTreeMap<PathBuf, FileOwnership<'_>>,
    &[u8],
) -> Result<Vec<PathBuf>>;

pub(crate) struct TraceabilityScopeFactsDescriptor {
    pub(crate) build_facts: BuildTraceabilityScopeFactsFn,
    pub(crate) expand_closure: ExpandTraceabilityClosureFromFactsFn,
}

pub(crate) type BuildTraceabilityGraphFactsFn = fn(&Path, &[PathBuf]) -> Result<Vec<u8>>;

pub(crate) struct TraceabilityGraphFactsDescriptor {
    pub(crate) build_facts: BuildTraceabilityGraphFactsFn,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ScopedTraceabilityPreparation {
    /// Shared orchestration precomputes scoped source facts and graph facts
    /// before the provider builds its repo analysis context.
    #[default]
    EagerFacts,
    /// The provider receives the requested scope plus the full source set and
    /// discovers only the traceability graph needed to preserve scoped output.
    ScopedGraphDiscovery,
}

pub(crate) struct ProjectToolRequirement {
    pub(crate) tool: &'static str,
    pub(crate) probe_args: &'static [&'static str],
}

pub(crate) struct ProjectToolingDescriptor {
    pub(crate) requirements: &'static [ProjectToolRequirement],
}

pub(crate) struct LanguagePackDescriptor {
    pub(crate) language: SourceLanguage,
    pub(crate) matches_path: fn(&Path) -> bool,
    pub(crate) parse_source_graph: fn(&Path, &str) -> Option<ParsedSourceGraph>,
    pub(crate) build_repo_analysis_context: BuildRepoAnalysisContextFn,
    pub(crate) analysis_environment_fingerprint: fn(&Path) -> String,
    pub(crate) project_tooling: Option<&'static ProjectToolingDescriptor>,
    pub(crate) traceability_scope_facts: Option<&'static TraceabilityScopeFactsDescriptor>,
    pub(crate) traceability_graph_facts: Option<&'static TraceabilityGraphFactsDescriptor>,
    pub(crate) scoped_traceability_preparation: ScopedTraceabilityPreparation,
}

include!(concat!(env!("OUT_DIR"), "/language_pack_registry.rs"));

// @applies REGISTRY.PROVIDER_DESCRIPTOR
pub(crate) fn descriptors() -> &'static [&'static LanguagePackDescriptor] {
    REGISTERED_LANGUAGE_PACKS
}
