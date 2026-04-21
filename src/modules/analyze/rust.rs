/**
@module SPECIAL.MODULES.ANALYZE.RUST
Applies the built-in Rust language pack to analyzable Rust implementation while keeping Rust-specific parsing and resolution out of the language-agnostic analysis core. Rust backward trace should only run when `rust-analyzer` is installed, and until then this adapter should say plainly that Rust backward trace is unavailable instead of guessing from weaker analysis.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleMetricsSummary, ParsedRepo,
};

use self::item_metrics::observe_rust_text;
use super::traceability_core::{
    TraceabilityAnalysis, TraceabilityLanguagePack, build_traceability_analysis,
};
use super::{FileOwnership, ProviderModuleAnalysis, read_owned_file_text, visit_owned_texts};

mod complexity;
mod dependencies;
mod item_metrics;
pub(super) mod item_signals;
mod quality;
mod rust_analyzer;
mod semantic;
mod surface;
mod toolchain;
mod traceability;
mod use_tree;

pub(crate) struct RustRepoAnalysisContext {
    traceability_pack: traceability::RustTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    traceability_unavailable_reason: Option<String>,
}

pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    parsed_architecture: &crate::model::ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> RustRepoAnalysisContext {
    let traceability_pack =
        traceability::RustTraceabilityPack::new(toolchain::probe_local_toolchain_project(root));
    let traceability_unavailable_reason = traceability_pack
        .backward_trace_availability()
        .unavailable_reason()
        .map(ToString::to_string);
    let traceability =
        (include_traceability && traceability_unavailable_reason.is_none()).then(|| {
            build_traceability_analysis(traceability_pack.build_inputs(
                root,
                source_files,
                parsed_repo,
                parsed_architecture,
                file_ownership,
            ))
        });
    RustRepoAnalysisContext {
        traceability,
        traceability_pack,
        traceability_unavailable_reason,
    }
}

pub(crate) fn analysis_environment_fingerprint(root: &Path) -> String {
    toolchain::analysis_environment_fingerprint(root)
}

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
            super::traceability_core::summarize_module_traceability(&owned_items, catalog)
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
