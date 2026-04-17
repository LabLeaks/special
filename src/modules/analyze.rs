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
    ArchitectureAnalysisSummary, ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary,
    ImplementRef, ModuleAnalysisOptions, ModuleAnalysisSummary, ModuleComplexitySummary,
    ModuleCouplingSummary, ModuleCoverageSummary, ModuleDependencySummary,
    ModuleDependencyTargetSummary, ModuleItemSignal, ModuleItemSignalsSummary,
    ModuleMetricsSummary, ModuleQualitySummary, ModuleTraceabilityItem, ModuleTraceabilitySummary,
    ParsedArchitecture, ParsedRepo,
};
use crate::syntax::SourceLanguage;

mod coupling;
mod coverage;
mod duplication;
pub(crate) mod explain;
mod go;
mod rust;
mod source_item_signals;
mod typescript;
mod unreached_code;

#[derive(Debug, Clone)]
pub(super) struct ArchitectureAnalysis {
    pub summary: ArchitectureAnalysisSummary,
    pub modules: BTreeMap<String, ModuleAnalysisSummary>,
}

#[derive(Debug, Default)]
pub(super) struct ProviderModuleAnalysis {
    pub metrics: ModuleMetricsSummary,
    pub complexity: Option<ModuleComplexitySummary>,
    pub quality: Option<ModuleQualitySummary>,
    pub item_signals: Option<ModuleItemSignalsSummary>,
    pub traceability: Option<ModuleTraceabilitySummary>,
    pub coupling: Option<coupling::ModuleCouplingInput>,
    pub dependencies: Option<ModuleDependencySummary>,
}

pub(super) fn build_architecture_analysis(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    parsed_repo: Option<&ParsedRepo>,
    options: ModuleAnalysisOptions,
) -> Result<ArchitectureAnalysis> {
    let files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let file_ownership = index_file_ownership(parsed);
    let coverage = options.coverage.then(coverage::build_repo_signals_summary);
    let mut coverage = coverage;
    if options.metrics
        && let Some(coverage) = &mut coverage
    {
        unreached_code::apply_unowned_unreached_summary(root, &files, &file_ownership, coverage);
        duplication::apply_duplicate_item_summary(root, parsed, &file_ownership, coverage);
    }
    let modules =
        build_module_analysis(root, &files, parsed, &file_ownership, parsed_repo, options)?;

    Ok(ArchitectureAnalysis {
        summary: ArchitectureAnalysisSummary {
            repo_signals: coverage,
            traceability: options
                .experimental
                .then(|| build_repo_traceability_summary(&modules)),
        },
        modules,
    })
}

fn build_repo_traceability_summary(
    modules: &BTreeMap<String, ModuleAnalysisSummary>,
) -> ArchitectureTraceabilitySummary {
    let mut summary = ArchitectureTraceabilitySummary::default();

    for (module_id, analysis) in modules {
        let Some(traceability) = &analysis.traceability else {
            continue;
        };
        summary.analyzed_items += traceability.analyzed_items;
        summary.live_spec_items.extend(
            traceability
                .live_spec_items
                .iter()
                .cloned()
                .map(|item| architecture_traceability_item(module_id, item)),
        );
        summary.planned_only_items.extend(
            traceability
                .planned_only_items
                .iter()
                .cloned()
                .map(|item| architecture_traceability_item(module_id, item)),
        );
        summary.deprecated_only_items.extend(
            traceability
                .deprecated_only_items
                .iter()
                .cloned()
                .map(|item| architecture_traceability_item(module_id, item)),
        );
        summary.file_scoped_only_items.extend(
            traceability
                .file_scoped_only_items
                .iter()
                .cloned()
                .map(|item| architecture_traceability_item(module_id, item)),
        );
        summary.unverified_test_items.extend(
            traceability
                .unverified_test_items
                .iter()
                .cloned()
                .map(|item| architecture_traceability_item(module_id, item)),
        );
        summary.unknown_items.extend(
            traceability
                .unknown_items
                .iter()
                .cloned()
                .map(|item| architecture_traceability_item(module_id, item)),
        );
    }

    summary
}

fn architecture_traceability_item(
    module_id: &str,
    item: ModuleTraceabilityItem,
) -> ArchitectureTraceabilityItem {
    ArchitectureTraceabilityItem {
        module_id: module_id.to_string(),
        name: item.name,
        kind: item.kind,
        verifying_tests: item.verifying_tests,
        unverified_tests: item.unverified_tests,
        live_specs: item.live_specs,
        planned_specs: item.planned_specs,
        deprecated_specs: item.deprecated_specs,
    }
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
    files: &[PathBuf],
    parsed: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    parsed_repo: Option<&ParsedRepo>,
    options: ModuleAnalysisOptions,
) -> Result<BTreeMap<String, ModuleAnalysisSummary>> {
    let mut modules = BTreeMap::new();
    let mut coupling_inputs: BTreeMap<String, coupling::ModuleCouplingInput> = BTreeMap::new();
    let rust_context = parsed_repo.map(|parsed_repo| {
        rust::build_repo_analysis_context(root, files, parsed_repo, parsed, file_ownership)
    });

    for module in &parsed.modules {
        let implementations: Vec<&ImplementRef> = parsed
            .implements
            .iter()
            .filter(|implementation| implementation.module_id == module.id)
            .collect();

        if implementations.is_empty() && !options.any() {
            continue;
        }

        let coverage = options.coverage.then(|| ModuleCoverageSummary {
            file_scoped_implements: implementations
                .iter()
                .filter(|implementation| implementation.body_location.is_none())
                .count(),
            item_scoped_implements: implementations
                .iter()
                .filter(|implementation| implementation.body_location.is_some())
                .count(),
        });

        let (metrics, complexity, quality, item_signals, traceability, coupling, dependencies) =
            if options.metrics {
                let mut provider = ProviderModuleAnalysis {
                    metrics: build_module_metrics(root, &implementations, file_ownership)?,
                    ..ProviderModuleAnalysis::default()
                };

                for language in languages_for_implementations(&implementations) {
                    let delta = analyze_provider_language(
                        language,
                        root,
                        &implementations,
                        file_ownership,
                        rust_context.as_ref(),
                        options,
                    )?;
                    merge_provider_module_analysis(&mut provider, delta);
                }

                let ProviderModuleAnalysis {
                    metrics,
                    complexity,
                    quality,
                    item_signals,
                    traceability,
                    coupling,
                    dependencies,
                } = provider;

                if let Some(coupling) = coupling {
                    coupling_inputs.insert(module.id.clone(), coupling);
                }
                (
                    Some(metrics),
                    complexity,
                    quality,
                    item_signals,
                    traceability,
                    coupling_inputs
                        .contains_key(&module.id)
                        .then_some(ModuleCouplingSummary::default()),
                    dependencies,
                )
            } else {
                (None, None, None, None, None, None, None)
            };

        modules.insert(
            module.id.clone(),
            ModuleAnalysisSummary {
                coverage,
                metrics,
                complexity,
                quality,
                item_signals,
                traceability,
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

fn languages_for_implementations(implementations: &[&ImplementRef]) -> BTreeSet<SourceLanguage> {
    implementations
        .iter()
        .filter_map(|implementation| SourceLanguage::from_path(&implementation.location.path))
        .collect()
}

fn analyze_provider_language(
    language: SourceLanguage,
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    rust_context: Option<&rust::RustRepoAnalysisContext>,
    options: ModuleAnalysisOptions,
) -> Result<ProviderModuleAnalysis> {
    match language {
        SourceLanguage::Go => go::analyze_module(root, implementations, file_ownership),
        SourceLanguage::Rust => rust::analyze_module(
            root,
            implementations,
            file_ownership,
            rust_context.expect("metrics analysis should prepare a Rust repo context"),
            options.experimental,
        ),
        SourceLanguage::TypeScript => {
            typescript::analyze_module(root, implementations, file_ownership)
        }
    }
}

fn merge_provider_module_analysis(
    summary: &mut ProviderModuleAnalysis,
    delta: ProviderModuleAnalysis,
) {
    merge_metrics(&mut summary.metrics, delta.metrics);
    merge_optional_complexity(&mut summary.complexity, delta.complexity);
    merge_optional_quality(&mut summary.quality, delta.quality);
    merge_optional_item_signals(&mut summary.item_signals, delta.item_signals);
    merge_optional_traceability(&mut summary.traceability, delta.traceability);
    merge_optional_coupling_input(&mut summary.coupling, delta.coupling);
    merge_optional_dependencies(&mut summary.dependencies, delta.dependencies);
}

fn display_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn merge_metrics(summary: &mut ModuleMetricsSummary, delta: ModuleMetricsSummary) {
    summary.public_items += delta.public_items;
    summary.internal_items += delta.internal_items;
}

fn merge_optional_complexity(
    summary: &mut Option<ModuleComplexitySummary>,
    delta: Option<ModuleComplexitySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ModuleComplexitySummary::default);
    target.function_count += delta.function_count;
    target.total_cyclomatic += delta.total_cyclomatic;
    target.max_cyclomatic = target.max_cyclomatic.max(delta.max_cyclomatic);
    target.total_cognitive += delta.total_cognitive;
    target.max_cognitive = target.max_cognitive.max(delta.max_cognitive);
}

fn merge_optional_quality(
    summary: &mut Option<ModuleQualitySummary>,
    delta: Option<ModuleQualitySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ModuleQualitySummary::default);
    target.public_function_count += delta.public_function_count;
    target.parameter_count += delta.parameter_count;
    target.bool_parameter_count += delta.bool_parameter_count;
    target.raw_string_parameter_count += delta.raw_string_parameter_count;
    target.panic_site_count += delta.panic_site_count;
}

fn merge_optional_item_signals(
    summary: &mut Option<ModuleItemSignalsSummary>,
    delta: Option<ModuleItemSignalsSummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ModuleItemSignalsSummary::default);
    target.analyzed_items += delta.analyzed_items;
    target.unreached_item_count += delta.unreached_item_count;
    merge_item_signal_group(&mut target.connected_items, delta.connected_items);
    merge_item_signal_group(&mut target.outbound_heavy_items, delta.outbound_heavy_items);
    merge_item_signal_group(&mut target.isolated_items, delta.isolated_items);
    merge_item_signal_group(&mut target.unreached_items, delta.unreached_items);
    merge_item_signal_group(
        &mut target.highest_complexity_items,
        delta.highest_complexity_items,
    );
    merge_item_signal_group(
        &mut target.parameter_heavy_items,
        delta.parameter_heavy_items,
    );
    merge_item_signal_group(
        &mut target.stringly_boundary_items,
        delta.stringly_boundary_items,
    );
    merge_item_signal_group(&mut target.panic_heavy_items, delta.panic_heavy_items);
}

fn merge_item_signal_group(target: &mut Vec<ModuleItemSignal>, mut delta: Vec<ModuleItemSignal>) {
    target.append(&mut delta);
    target.truncate(5);
}

fn merge_optional_traceability(
    summary: &mut Option<ModuleTraceabilitySummary>,
    delta: Option<ModuleTraceabilitySummary>,
) {
    if summary.is_none() {
        *summary = delta;
    }
}

fn merge_optional_coupling_input(
    summary: &mut Option<coupling::ModuleCouplingInput>,
    delta: Option<coupling::ModuleCouplingInput>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(coupling::ModuleCouplingInput::default);
    target.internal_files.extend(delta.internal_files);
    target.external_targets.extend(delta.external_targets);
}

fn merge_optional_dependencies(
    summary: &mut Option<ModuleDependencySummary>,
    delta: Option<ModuleDependencySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let mut merged = summary
        .take()
        .map(|summary| {
            summary
                .targets
                .into_iter()
                .map(|target| (target.path, target.count))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    for target in delta.targets {
        *merged.entry(target.path).or_default() += target.count;
    }
    let reference_count = merged.values().sum();
    let distinct_targets = merged.len();
    *summary = Some(ModuleDependencySummary {
        reference_count,
        distinct_targets,
        targets: merged
            .into_iter()
            .map(|(path, count)| ModuleDependencyTargetSummary { path, count })
            .collect(),
    });
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
