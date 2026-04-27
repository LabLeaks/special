/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE
Owns the built-in Go implementation analysis provider, including pack-specific traceability setup and tool discovery while depending on shared analysis core only through protocolized helpers.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleMetricsSummary, ParsedArchitecture,
    ParsedRepo,
};

use self::dependencies::{GoDependencyResolver, GoDependencySummary};
use self::surface::{GoSurfaceSummary, is_go_path};
use self::traceability::GoTraceabilityPack;
use crate::modules::analyze::source_item_signals::summarize_source_item_signals_with_metrics;
use crate::modules::analyze::traceability_core::{
    TraceabilityAnalysis, TraceabilityLanguagePack,
};
use crate::modules::analyze::{
    FileOwnership, ProviderModuleAnalysis, emit_analysis_status, visit_owned_texts,
};

#[path = "analyze/dependencies.rs"]
mod dependencies;
#[path = "analyze/boundary.rs"]
mod boundary;
#[path = "analyze/scope.rs"]
mod scope;
#[path = "analyze/quality.rs"]
mod quality;
#[cfg(test)]
#[path = "analyze/tests.rs"]
mod scoped_tests;
#[path = "analyze/surface.rs"]
mod surface;
#[path = "analyze/toolchain.rs"]
mod toolchain;
#[path = "analyze/traceability.rs"]
mod traceability;

// @applies ADAPTER.FACTS_TO_MODEL
pub(crate) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &GoRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut surface = GoSurfaceSummary::default();
    let mut quality = quality::GoQualitySummary::default();
    let mut owned_items = Vec::new();
    let mut dependencies = GoDependencySummary::default();
    let traceability_summary = include_traceability
        .then_some(context.traceability.as_ref())
        .flatten()
        .map(|traceability| {
            let owned_items = context.traceability_pack.owned_items_for_implementations(
                root,
                implementations,
                file_ownership,
            );
            crate::modules::analyze::traceability_core::summarize_module_traceability(
                &owned_items,
                traceability,
            )
        });

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_go_path(path) {
            return Ok(());
        }
        if let Some(graph) = crate::syntax::parse_source_graph(path, text) {
            surface.observe(&graph.items);
            quality.observe(path, text, &graph);
            owned_items.extend(graph.items);
        }
        dependencies.observe(root, text, &context.dependency_resolver);
        Ok(())
    })?;

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items: surface.public_items,
            internal_items: surface.internal_items,
            ..ModuleMetricsSummary::default()
        },
        complexity: Some(quality.finish_complexity()),
        item_signals: Some(summarize_source_item_signals_with_metrics(
            &owned_items,
            quality.item_metrics(),
        )),
        quality: Some(quality.finish()),
        traceability: traceability_summary,
        traceability_unavailable_reason: include_traceability
            .then(|| context.traceability_unavailable_reason.clone())
            .flatten(),
        coupling: Some(dependencies.coupling_input()),
        dependencies: Some(dependencies.summary()),
    })
}

pub(crate) struct GoRepoAnalysisContext {
    traceability_pack: GoTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    dependency_resolver: GoDependencyResolver,
    pub(crate) traceability_unavailable_reason: Option<String>,
}

pub(crate) fn build_traceability_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<Vec<u8>> {
    traceability::build_traceability_graph_facts(root, source_files)
}

pub(crate) fn build_traceability_scope_facts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<Vec<u8>> {
    scope::build_traceability_scope_facts(
        root,
        source_files,
        scoped_source_files,
        parsed_repo,
        file_ownership,
    )
}

pub(crate) fn expand_traceability_closure_from_facts(
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    facts: &[u8],
) -> Result<Vec<PathBuf>> {
    scope::expand_traceability_closure_from_facts(source_files, scoped_source_files, file_ownership, facts)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> GoRepoAnalysisContext {
    let traceability_pack = GoTraceabilityPack;
    let dependency_resolver = GoDependencyResolver::from_project(root);
    if !source_files.is_empty() && !dependency_resolver.is_tool_backed() {
        emit_analysis_status(
            "Go analyzer enrichment degraded: declare Go through special.toml, mise.toml, or .tool-versions and provide a working `go` tool to enable go-list-backed dependency and coupling metrics",
        );
    }
    let base_traceability_unavailable_reason = toolchain::go_backward_trace_unavailable_reason(root);
    let (traceability, traceability_unavailable_reason) =
        if !include_traceability || base_traceability_unavailable_reason.is_some() {
            (None, base_traceability_unavailable_reason)
        } else {
            let analysis = if let Some(scoped_source_files) =
                scoped_source_files.filter(|files| !files.is_empty())
            {
                scope::build_scoped_traceability_analysis_from_cached_or_live_graph_facts(
                    root,
                    source_files,
                    scoped_source_files,
                    traceability_graph_facts,
                    parsed_repo,
                    file_ownership,
                )
            } else {
                traceability::build_traceability_analysis_from_cached_or_live_graph_facts(
                    root,
                    source_files,
                    traceability_graph_facts,
                    parsed_repo,
                    parsed_architecture,
                    file_ownership,
                    &traceability_pack,
                )
            };
            match analysis {
                Ok(traceability) => (Some(traceability), None),
                Err(error) => (
                    None,
                    Some(format!(
                        "Go backward trace is unavailable because gopls trace collection failed: {error:#}"
                    )),
                ),
            }
        };
    GoRepoAnalysisContext {
        traceability_pack,
        traceability,
        dependency_resolver,
        traceability_unavailable_reason,
    }
}

pub(crate) fn summarize_repo_traceability(
    root: &Path,
    context: &GoRepoAnalysisContext,
) -> Option<ArchitectureTraceabilitySummary> {
    context
        .traceability
        .as_ref()
        .map(|traceability| traceability::summarize_repo_traceability(root, traceability))
}

pub(crate) fn analysis_environment_fingerprint(root: &Path) -> String {
    toolchain::analysis_environment_fingerprint(root)
}
