/**
@module SPECIAL.MODULES.ANALYZE
Builds language-agnostic evidence-first implementation analysis for concrete modules, combining shared ownership coverage with built-in language providers without inventing architecture verdicts.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    ArchitectureAnalysisSummary, ImplementRef, ModuleAnalysisOptions, ModuleAnalysisSummary,
    ModuleComplexitySummary, ModuleCouplingSummary, ModuleCoverageSummary, ModuleDependencySummary,
    ModuleItemSignalsSummary, ModuleMetricsSummary, ModuleQualitySummary, ParsedArchitecture,
};

mod coupling;
mod coverage;
pub(crate) mod explain;
mod rust;

#[derive(Debug, Clone)]
pub(super) struct ArchitectureAnalysis {
    pub summary: ArchitectureAnalysisSummary,
    pub modules: BTreeMap<String, ModuleAnalysisSummary>,
}

pub(super) fn build_architecture_analysis(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    options: ModuleAnalysisOptions,
) -> Result<ArchitectureAnalysis> {
    let files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let file_ownership = index_file_ownership(parsed);
    let coverage = options
        .coverage
        .then(|| coverage::build_coverage_summary(root, &files, &file_ownership));
    let modules = build_module_analysis(root, parsed, &file_ownership, options)?;

    Ok(ArchitectureAnalysis {
        summary: ArchitectureAnalysisSummary { coverage },
        modules,
    })
}

#[derive(Debug, Default)]
pub(super) struct FileOwnership<'a> {
    file_scoped: Vec<&'a ImplementRef>,
    item_scoped: Vec<&'a ImplementRef>,
}

fn index_file_ownership<'a>(
    parsed: &'a ParsedArchitecture,
) -> BTreeMap<PathBuf, FileOwnership<'a>> {
    let mut files: BTreeMap<PathBuf, FileOwnership<'a>> = BTreeMap::new();
    for implementation in &parsed.implements {
        let entry = files
            .entry(implementation.location.path.clone())
            .or_default();
        if implementation.body_location.is_some() {
            entry.item_scoped.push(implementation);
        } else {
            entry.file_scoped.push(implementation);
        }
    }
    files
}

fn build_module_analysis(
    root: &Path,
    parsed: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    options: ModuleAnalysisOptions,
) -> Result<BTreeMap<String, ModuleAnalysisSummary>> {
    let mut modules = BTreeMap::new();
    let mut coupling_inputs: BTreeMap<String, coupling::ModuleCouplingInput> = BTreeMap::new();

    for module in &parsed.modules {
        let implementations: Vec<&ImplementRef> = parsed
            .implements
            .iter()
            .filter(|implementation| implementation.module_id == module.id)
            .collect();

        if implementations.is_empty() && !options.any() {
            continue;
        }

        let coverage = options.coverage.then(|| {
            let covered_paths: BTreeSet<PathBuf> = implementations
                .iter()
                .map(|implementation| display_path(root, &implementation.location.path))
                .collect();
            let weak_paths: BTreeSet<PathBuf> = implementations
                .iter()
                .filter(|implementation| implementation.body_location.is_none())
                .map(|implementation| display_path(root, &implementation.location.path))
                .collect();

            ModuleCoverageSummary {
                covered_files: covered_paths.len(),
                weak_files: weak_paths.len(),
                file_scoped_implements: implementations
                    .iter()
                    .filter(|implementation| implementation.body_location.is_none())
                    .count(),
                item_scoped_implements: implementations
                    .iter()
                    .filter(|implementation| implementation.body_location.is_some())
                    .count(),
                covered_paths: covered_paths.into_iter().collect(),
                weak_paths: weak_paths.into_iter().collect(),
            }
        });

        let (metrics, complexity, quality, item_signals, coupling, dependencies) =
            if options.metrics {
                let mut metrics = build_module_metrics(root, &implementations, file_ownership)?;
                let mut complexity = ModuleComplexitySummary::default();
                let mut quality = ModuleQualitySummary::default();
                let mut item_signals = ModuleItemSignalsSummary::default();
                let mut coupling = coupling::ModuleCouplingInput::default();
                let mut dependencies = ModuleDependencySummary::default();
                rust::apply_module_analysis(
                    root,
                    &implementations,
                    file_ownership,
                    rust::RustModuleAnalysisOutputs {
                        metrics: &mut metrics,
                        complexity: &mut complexity,
                        quality: &mut quality,
                        item_signals: &mut item_signals,
                        coupling: &mut coupling,
                        dependencies: &mut dependencies,
                    },
                )?;
                coupling_inputs.insert(module.id.clone(), coupling);
                (
                    Some(metrics),
                    Some(complexity),
                    Some(quality),
                    Some(item_signals),
                    Some(ModuleCouplingSummary::default()),
                    Some(dependencies),
                )
            } else {
                (None, None, None, None, None, None)
            };

        modules.insert(
            module.id.clone(),
            ModuleAnalysisSummary {
                coverage,
                metrics,
                complexity,
                quality,
                item_signals,
                coupling,
                dependencies,
            },
        );
    }

    if options.metrics {
        coupling::apply_module_coupling(parsed, &coupling_inputs, &mut modules);
    }

    Ok(modules)
}

fn display_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn build_module_metrics(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<ModuleMetricsSummary> {
    let mut summary = ModuleMetricsSummary::default();
    visit_owned_texts(root, implementations, file_ownership, |_path, text| {
        summary.owned_lines += line_count(text);
    })?;
    Ok(summary)
}

pub(super) fn visit_owned_texts<F>(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    mut visit: F,
) -> Result<()>
where
    F: FnMut(&Path, &str),
{
    for implementation in implementations {
        let path = &implementation.location.path;
        match &implementation.body {
            Some(body) => visit(path, body),
            None => {
                let Some(ownership) = file_ownership.get(path) else {
                    continue;
                };
                if !ownership.item_scoped.is_empty() {
                    continue;
                }
                let content = read_owned_file_text(root, path);
                visit(path, &content);
            }
        }
    }
    Ok(())
}

pub(super) fn read_owned_file_text(root: &Path, path: &Path) -> String {
    let full_path = root.join(path);
    fs::read_to_string(&full_path)
        .or_else(|_| fs::read_to_string(path))
        .unwrap_or_default()
}

fn line_count(text: &str) -> usize {
    text.lines().count().max(1)
}
