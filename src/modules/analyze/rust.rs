/**
@module SPECIAL.MODULES.ANALYZE.RUST
Applies the built-in Rust analysis provider to owned Rust implementation while keeping Rust parsing out of the language-agnostic analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ImplementRef, ModuleComplexitySummary, ModuleDependencySummary, ModuleItemSignalsSummary,
    ModuleMetricsSummary, ModuleQualitySummary,
};

use super::{
    FileOwnership, coupling::ModuleCouplingInput, read_owned_file_text, visit_owned_texts,
};

mod complexity;
mod dependencies;
mod item_metrics;
mod item_signals;
mod quality;
mod surface;

pub(super) struct RustModuleAnalysisOutputs<'a> {
    pub metrics: &'a mut ModuleMetricsSummary,
    pub complexity: &'a mut ModuleComplexitySummary,
    pub quality: &'a mut ModuleQualitySummary,
    pub item_signals: &'a mut ModuleItemSignalsSummary,
    pub coupling: &'a mut ModuleCouplingInput,
    pub dependencies: &'a mut ModuleDependencySummary,
}

pub(super) fn apply_module_analysis(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    outputs: RustModuleAnalysisOutputs<'_>,
) -> Result<()> {
    let mut surface_summary = surface::RustSurfaceSummary::default();
    let mut complexity_summary = complexity::RustComplexitySummary::default();
    let mut quality_summary = quality::RustQualitySummary::default();
    let mut dependency_summary = dependencies::RustDependencySummary::default();

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
            (Some(_), Some(body)) => item_signals_summary.observe(body),
            (None, _) => item_signals_summary.observe(&read_owned_file_text(root, path)),
            _ => {}
        }
    }

    outputs.metrics.public_items += surface_summary.public_items;
    outputs.metrics.internal_items += surface_summary.internal_items;
    *outputs.complexity = complexity_summary.finish();
    *outputs.quality = quality_summary.finish();
    *outputs.item_signals = item_signals_summary.finish();
    *outputs.coupling = dependency_summary.coupling_input();
    *outputs.dependencies = dependency_summary.summary();
    Ok(())
}
