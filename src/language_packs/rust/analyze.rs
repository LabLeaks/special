/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE
Owns the built-in Rust implementation analysis provider, including pack-specific toolchain probing and traceability setup while depending on shared analysis core only through protocolized helpers.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleMetricsSummary, ParsedRepo,
};

use self::item_metrics::observe_rust_text;
use crate::modules::analyze::traceability_core::{TraceabilityAnalysis, TraceabilityLanguagePack};
use crate::modules::analyze::{
    FileOwnership, ProviderModuleAnalysis, emit_analysis_status, read_owned_file_text,
    visit_owned_texts,
};

#[path = "analyze/complexity.rs"]
mod complexity;
#[path = "analyze/dependencies.rs"]
mod dependencies;
#[path = "analyze/item_metrics.rs"]
mod item_metrics;
#[path = "analyze/item_signals.rs"]
pub(super) mod item_signals;
#[path = "analyze/quality.rs"]
mod quality;
#[path = "analyze/rust_analyzer.rs"]
mod rust_analyzer;
#[path = "analyze/semantic.rs"]
mod semantic;
#[path = "analyze/surface.rs"]
mod surface;
#[path = "analyze/toolchain.rs"]
mod toolchain;
#[path = "analyze/traceability.rs"]
mod traceability;
#[path = "analyze/use_tree.rs"]
mod use_tree;

pub(crate) struct RustRepoAnalysisContext {
    traceability_pack: traceability::RustTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    traceability_unavailable_reason: Option<String>,
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
    traceability::build_traceability_scope_facts(
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
    traceability::expand_traceability_closure_from_facts(
        source_files,
        scoped_source_files,
        file_ownership,
        facts,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    _parsed_architecture: &crate::model::ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> RustRepoAnalysisContext {
    let traceability_pack =
        traceability::RustTraceabilityPack::new(toolchain::probe_local_toolchain_project(root));
    if include_traceability && traceability_pack.is_parser_only() {
        emit_analysis_status(
            "Rust analyzer enrichment degraded: `rust-analyzer` is unavailable, so health traceability will use parser-resolved Rust call edges only",
        );
    }
    let (traceability, traceability_unavailable_reason) = if !include_traceability {
        (None, None)
    } else {
            let analysis = if let Some(scoped_source_files) =
                scoped_source_files.filter(|files| !files.is_empty())
            {
                traceability::build_scoped_traceability_analysis_from_cached_or_live_graph_facts(
                    root,
                    source_files,
                    scoped_source_files,
                    traceability_graph_facts,
                    parsed_repo,
                    file_ownership,
                    &traceability_pack,
                )
            } else {
                traceability::build_traceability_analysis_from_cached_or_live_graph_facts(
                    root,
                    source_files,
                    traceability_graph_facts,
                    parsed_repo,
                    file_ownership,
                    &traceability_pack,
                )
            };
            match analysis {
                Ok(traceability) => (Some(traceability), None),
                Err(error) => (None, Some(error.to_string())),
            }
    };
    RustRepoAnalysisContext {
        traceability,
        traceability_pack,
        traceability_unavailable_reason,
    }
}

pub(crate) fn analysis_environment_fingerprint(root: &Path) -> String {
    toolchain::analysis_environment_fingerprint(root)
}

// @applies ADAPTER.FACTS_TO_MODEL
pub(crate) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &RustRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut surface_summary = surface::RustSurfaceSummary::default();
    let mut complexity_summary = complexity::RustComplexitySummary::default();
    let mut quality_summary = quality::RustQualitySummary::default();
    let mut dependency_summary = dependencies::RustDependencySummary::default();
    let traceability_summary = include_traceability
        .then_some(context.traceability.as_ref())
        .flatten()
        .map(|catalog| {
            let owned_items = context.traceability_pack.owned_items_for_implementations(
                root,
                implementations,
                file_ownership,
            );
            crate::modules::analyze::traceability_core::summarize_module_traceability(
                &owned_items,
                catalog,
            )
        });

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            return Ok(());
        }
        surface_summary.observe(text);
        observe_rust_text(&mut complexity_summary, text);
        observe_rust_text(&mut quality_summary, text);
        dependency_summary.observe(root, path, text);
        Ok(())
    })?;

    let mut item_signals_summary = item_signals::RustItemSignalsSummary::default();
    for implementation in implementations {
        let path = &implementation.location.path;
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        match (&implementation.body_location, &implementation.body) {
            (Some(_), Some(body)) => item_signals_summary.observe(path, body),
            (None, _) => item_signals_summary.observe(path, &read_owned_file_text(root, path)?),
            _ => {}
        }
    }

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items: surface_summary.public_items,
            internal_items: surface_summary.internal_items,
            ..ModuleMetricsSummary::default()
        },
        complexity: Some(complexity_summary.finish()),
        quality: Some(quality_summary.finish()),
        item_signals: Some(item_signals_summary.finish()),
        traceability: traceability_summary,
        traceability_unavailable_reason: include_traceability
            .then(|| context.traceability_unavailable_reason.clone())
            .flatten(),
        coupling: Some(dependency_summary.coupling_input()),
        dependencies: Some(dependency_summary.summary()),
    })
}

pub(crate) fn summarize_repo_traceability(
    root: &Path,
    context: &RustRepoAnalysisContext,
) -> Option<ArchitectureTraceabilitySummary> {
    context
        .traceability
        .as_ref()
        .map(|catalog| traceability::summarize_repo_traceability(root, catalog))
}

pub(crate) fn traceability_unavailable_reason(context: &RustRepoAnalysisContext) -> Option<&str> {
    context.traceability_unavailable_reason.as_deref()
}
