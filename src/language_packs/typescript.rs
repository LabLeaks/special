/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT
Registers the built-in TypeScript language pack with the shared compile-time pack registry, delegating parsing and implementation analysis to TypeScript-owned provider code without reintroducing hardcoded dispatch into syntax or analysis cores.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{
    LanguagePackAnalysisContext, LanguagePackDescriptor, ProjectToolRequirement,
    ProjectToolingDescriptor, ScopedTraceabilityPreparation, TraceabilityGraphFactsDescriptor,
    TraceabilityScopeFactsDescriptor,
};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::modules::analyze::{FileOwnership, ProviderModuleAnalysis};
use crate::source_paths::has_extension;
use crate::syntax::{ParsedSourceGraph, SourceLanguage};

#[path = "typescript/analyze.rs"]
mod analyze;

pub(crate) const DESCRIPTOR: LanguagePackDescriptor = LanguagePackDescriptor {
    language: SourceLanguage::new("typescript"),
    matches_path: is_typescript_path,
    parse_source_graph,
    build_repo_analysis_context,
    analysis_environment_fingerprint,
    project_tooling: Some(&PROJECT_TOOLING),
    traceability_scope_facts: Some(&TRACEABILITY_SCOPE_FACTS),
    traceability_graph_facts: Some(&TRACEABILITY_GRAPH_FACTS),
    scoped_traceability_preparation: ScopedTraceabilityPreparation::ScopedGraphDiscovery,
};

const PROJECT_TOOLING: ProjectToolingDescriptor = ProjectToolingDescriptor {
    requirements: &[
        ProjectToolRequirement {
            tool: "node",
            probe_args: &["--version"],
        },
    ],
};

const TRACEABILITY_SCOPE_FACTS: TraceabilityScopeFactsDescriptor = TraceabilityScopeFactsDescriptor {
    build_facts: build_traceability_scope_facts,
    expand_closure: expand_traceability_closure_from_facts,
};

const TRACEABILITY_GRAPH_FACTS: TraceabilityGraphFactsDescriptor = TraceabilityGraphFactsDescriptor {
    build_facts: build_traceability_graph_facts,
};

impl LanguagePackAnalysisContext for analyze::TypeScriptRepoAnalysisContext {
    fn summarize_repo_traceability(&self, root: &Path) -> Option<ArchitectureTraceabilitySummary> {
        analyze::summarize_repo_traceability(root, self)
    }

    fn traceability_unavailable_reason(&self) -> Option<String> {
        self.traceability_unavailable_reason.clone()
    }

    fn analyze_module(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
        options: ModuleAnalysisOptions,
    ) -> Result<ProviderModuleAnalysis> {
        analyze::analyze_module(root, implementations, file_ownership, self, options.traceability)
    }
}

#[allow(clippy::too_many_arguments)]
fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> Box<dyn LanguagePackAnalysisContext> {
    Box::new(analyze::build_repo_analysis_context(
        root,
        source_files,
        scoped_source_files,
        traceability_graph_facts,
        parsed_repo,
        parsed_architecture,
        file_ownership,
        include_traceability,
    ))
}

fn analysis_environment_fingerprint(root: &Path) -> String {
    analyze::analysis_environment_fingerprint(root)
}

fn build_traceability_scope_facts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<Vec<u8>> {
    analyze::build_traceability_scope_facts(
        root,
        source_files,
        scoped_source_files,
        parsed_repo,
        file_ownership,
    )
}

fn expand_traceability_closure_from_facts(
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    facts: &[u8],
) -> Result<Vec<PathBuf>> {
    analyze::expand_traceability_closure_from_facts(
        source_files,
        scoped_source_files,
        file_ownership,
        facts,
    )
}

fn build_traceability_graph_facts(root: &Path, source_files: &[PathBuf]) -> Result<Vec<u8>> {
    analyze::build_traceability_graph_facts(root, source_files)
}

fn is_typescript_path(path: &Path) -> bool {
    has_extension(path, &["ts", "tsx"])
}

fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    crate::syntax::typescript::parse_source_graph(path, text)
}
