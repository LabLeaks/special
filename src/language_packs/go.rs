/**
@module SPECIAL.LANGUAGE_PACKS.GO
Registers the built-in Go language pack with the shared compile-time pack registry, delegating parsing and implementation analysis to Go-owned provider code without reintroducing hardcoded dispatch into syntax or analysis cores.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{LanguagePackAnalysisContext, LanguagePackDescriptor};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::modules::analyze::{self, FileOwnership, ProviderModuleAnalysis};
use crate::syntax::{ParsedSourceGraph, SourceLanguage};

pub(crate) const DESCRIPTOR: LanguagePackDescriptor = LanguagePackDescriptor {
    language: SourceLanguage::new("go"),
    matches_path: is_go_path,
    parse_source_graph: parse_source_graph,
    build_repo_analysis_context: build_repo_analysis_context,
    analysis_environment_fingerprint: analysis_environment_fingerprint,
};

impl LanguagePackAnalysisContext for analyze::go::GoRepoAnalysisContext {
    fn summarize_repo_traceability(&self, root: &Path) -> Option<ArchitectureTraceabilitySummary> {
        analyze::go::summarize_repo_traceability(root, self)
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
        analyze::go::analyze_module(
            root,
            implementations,
            file_ownership,
            self,
            options.traceability,
        )
    }
}

fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> Box<dyn LanguagePackAnalysisContext> {
    Box::new(analyze::go::build_repo_analysis_context(
        root,
        source_files,
        parsed_repo,
        parsed_architecture,
        file_ownership,
        include_traceability,
    ))
}

fn analysis_environment_fingerprint(_root: &Path) -> String {
    analyze::go::analysis_environment_fingerprint()
}

fn is_go_path(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("go")
}

fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    crate::syntax::go::parse_source_graph(path, text)
}
