/**
@module SPECIAL.MODULES.ANALYZE.RUST
Applies the built-in Rust analysis provider to owned Rust implementation while keeping Rust parsing out of the language-agnostic analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{ImplementRef, ModuleMetricsSummary, ParsedRepo};

use super::{FileOwnership, ProviderModuleAnalysis, read_owned_file_text, visit_owned_texts};

mod complexity;
mod dependencies;
mod item_metrics;
pub(super) mod item_signals;
mod quality;
mod surface;
mod traceability;

pub(super) struct RustRepoAnalysisContext {
    traceability: traceability::RustTraceabilityCatalog,
}

pub(super) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    parsed_architecture: &crate::model::ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> RustRepoAnalysisContext {
    RustRepoAnalysisContext {
        traceability: traceability::build_traceability_catalog(
            root,
            source_files,
            parsed_repo,
            parsed_architecture,
            file_ownership,
        ),
    }
}

pub(super) fn analyze_module(
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
    let traceability_summary = include_traceability.then(|| {
        traceability::summarize_module_traceability(
            root,
            implementations,
            file_ownership,
            &context.traceability,
        )
    });

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            return;
        }
        surface_summary.observe(text);
        complexity_summary.observe(text);
        quality_summary.observe(text);
        dependency_summary.observe(root, path, text);
    })?;

    let mut item_signals_summary = item_signals::RustItemSignalsSummary::default();
    for implementation in implementations {
        let path = &implementation.location.path;
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        match (&implementation.body_location, &implementation.body) {
            (Some(_), Some(body)) => item_signals_summary.observe(path, body),
            (None, _) => item_signals_summary.observe(path, &read_owned_file_text(root, path)),
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
        coupling: Some(dependency_summary.coupling_input()),
        dependencies: Some(dependency_summary.summary()),
    })
}
