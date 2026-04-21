/**
@module SPECIAL.MODULES.ANALYZE
Builds language-agnostic evidence-first implementation analysis over analyzable code and concrete module ownership, combining a shared item-evidence core with compile-time language-pack registration from `SPECIAL.LANGUAGE_PACKS` instead of hardcoded per-language dispatch in the analysis core. Repo-facing analysis should stay code-first, module-facing analysis should remain an ownership projection over shared evidence, and backward trace should only run when the active language pack says its required local tool is available. When backward trace does run, it should report direct, statically mediated, or currently unexplained evidence rather than over-claiming negative reachability.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    ArchitectureAnalysisSummary, ArchitectureRepoSignalsSummary, ArchitectureTraceabilityItem,
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ModuleAnalysisSummary,
    ModuleComplexitySummary, ModuleCouplingSummary, ModuleCoverageSummary, ModuleDependencySummary,
    ModuleDependencyTargetSummary, ModuleItemSignal, ModuleItemSignalsSummary,
    ModuleMetricsSummary, ModuleQualitySummary, ModuleTraceabilitySummary, ParsedArchitecture,
    ParsedRepo,
};
use crate::syntax::SourceLanguage;

mod coupling;
mod coverage;
mod duplication;
pub(crate) mod explain;
pub(crate) mod go;
mod registry;
pub(crate) mod rust;
pub(crate) mod source_item_signals;
pub(crate) mod traceability_core;
pub(crate) mod typescript;
mod unreached_code;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ArchitectureAnalysis {
    pub modules: BTreeMap<String, ModuleAnalysisSummary>,
}

#[derive(Debug, Default)]
pub(crate) struct ProviderModuleAnalysis {
    pub metrics: ModuleMetricsSummary,
    pub complexity: Option<ModuleComplexitySummary>,
    pub quality: Option<ModuleQualitySummary>,
    pub item_signals: Option<ModuleItemSignalsSummary>,
    pub traceability: Option<ModuleTraceabilitySummary>,
    pub traceability_unavailable_reason: Option<String>,
    pub coupling: Option<coupling::ModuleCouplingInput>,
    pub dependencies: Option<ModuleDependencySummary>,
}

pub(crate) fn build_dependency_summary(
    targets: &BTreeMap<String, usize>,
) -> ModuleDependencySummary {
    ModuleDependencySummary {
        reference_count: targets.values().sum(),
        distinct_targets: targets.len(),
        targets: targets
            .iter()
            .map(|(path, count)| ModuleDependencyTargetSummary {
                path: path.clone(),
                count: *count,
            })
            .collect(),
    }
}

pub(crate) fn build_architecture_analysis(
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
    let repo_contexts = if options.metrics || options.traceability {
        parsed_repo
            .map(|parsed_repo| {
                registry::build_repo_analysis_contexts(
                    root,
                    &files,
                    parsed_repo,
                    parsed,
                    &file_ownership,
                    options.traceability,
                )
            })
            .unwrap_or_default()
    } else {
        BTreeMap::new()
    };
    let modules = build_module_analysis(
        root,
        &files,
        parsed,
        &file_ownership,
        &repo_contexts,
        options,
    )?;

    Ok(ArchitectureAnalysis { modules })
}

pub(crate) fn analysis_environment_fingerprint(root: &Path, files: &[PathBuf]) -> String {
    registry::analysis_environment_fingerprint(root, files)
}

pub(crate) fn build_repo_analysis_summary(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    parsed_repo: &ParsedRepo,
    scoped_paths: Option<&[PathBuf]>,
) -> Result<ArchitectureAnalysisSummary> {
    let all_files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let scoped_files = scope_source_files(root, &all_files, scoped_paths)?;
    let file_ownership = index_file_ownership(parsed);

    let mut repo_signals = coverage::build_repo_signals_summary();
    unreached_code::apply_unowned_item_summary(
        root,
        &scoped_files,
        &file_ownership,
        &mut repo_signals,
    )?;
    duplication::apply_duplicate_item_summary(root, parsed, &file_ownership, &mut repo_signals)?;
    if scoped_files.len() != all_files.len() {
        filter_repo_signals_to_scope(root, &scoped_files, &mut repo_signals);
    }

    let repo_contexts = registry::build_repo_analysis_contexts(
        root,
        &all_files,
        parsed_repo,
        parsed,
        &file_ownership,
        true,
    );
    let traceability =
        build_repo_traceability_summary(root, &all_files, &repo_contexts).map(|mut summary| {
            if scoped_files.len() != all_files.len() {
                filter_traceability_to_scope(root, &scoped_files, &mut summary);
            }
            summary
        });
    let traceability_unavailable_reason =
        build_repo_traceability_unavailable_reason(&scoped_files, &repo_contexts);

    Ok(ArchitectureAnalysisSummary {
        repo_signals: Some(repo_signals),
        traceability,
        traceability_unavailable_reason,
    })
}

pub(crate) fn filter_repo_analysis_summary_to_scope(
    root: &Path,
    ignore_patterns: &[String],
    scoped_paths: &[PathBuf],
    summary: &mut ArchitectureAnalysisSummary,
) -> Result<()> {
    let all_files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let scoped_files = scope_source_files(root, &all_files, Some(scoped_paths))?;
    if let Some(repo_signals) = &mut summary.repo_signals {
        filter_repo_signals_to_scope(root, &scoped_files, repo_signals);
    }
    if let Some(traceability) = &mut summary.traceability {
        filter_traceability_to_scope(root, &scoped_files, traceability);
    }
    Ok(())
}

pub(crate) fn filter_repo_analysis_summary_to_symbol(
    symbol: &str,
    summary: &mut ArchitectureAnalysisSummary,
) {
    if let Some(repo_signals) = &mut summary.repo_signals {
        repo_signals
            .unowned_item_details
            .retain(|item| item.name == symbol);
        repo_signals.unowned_items = repo_signals.unowned_item_details.len();
        repo_signals
            .duplicate_item_details
            .retain(|item| item.name == symbol);
        repo_signals.duplicate_items = repo_signals.duplicate_item_details.len();
    }
    if let Some(traceability) = &mut summary.traceability {
        retain_traceability_items_by_symbol(symbol, &mut traceability.current_spec_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.planned_only_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.deprecated_only_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.file_scoped_only_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.unverified_test_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.statically_mediated_items);
        retain_traceability_items_by_symbol(symbol, &mut traceability.unexplained_items);
        traceability.analyzed_items = traceability.current_spec_items.len()
            + traceability.planned_only_items.len()
            + traceability.deprecated_only_items.len()
            + traceability.file_scoped_only_items.len()
            + traceability.unverified_test_items.len()
            + traceability.statically_mediated_items.len()
            + traceability.unexplained_items.len();
        traceability.sort_items();
    }
}

fn build_repo_traceability_summary(
    root: &Path,
    files: &[PathBuf],
    repo_contexts: &registry::RepoAnalysisContexts,
) -> Option<ArchitectureTraceabilitySummary> {
    let mut summary = None;

    for language in registry::languages_in_files(files) {
        merge_optional_repo_traceability(
            &mut summary,
            registry::summarize_repo_traceability(language, root, repo_contexts),
        );
    }

    if let Some(summary) = &mut summary {
        summary.sort_items();
    }

    summary
}

fn build_repo_traceability_unavailable_reason(
    files: &[PathBuf],
    repo_contexts: &registry::RepoAnalysisContexts,
) -> Option<String> {
    for language in registry::languages_in_files(files) {
        if let Some(reason) = registry::traceability_unavailable_reason(language, repo_contexts) {
            return Some(reason);
        }
    }
    None
}

fn scope_source_files(
    root: &Path,
    all_files: &[PathBuf],
    scoped_paths: Option<&[PathBuf]>,
) -> Result<Vec<PathBuf>> {
    let Some(scoped_paths) = scoped_paths else {
        return Ok(all_files.to_vec());
    };
    if scoped_paths.is_empty() {
        return Ok(all_files.to_vec());
    }

    let scope_roots = scoped_paths
        .iter()
        .map(|path| normalize_scope_path(root, path))
        .collect::<Vec<_>>();

    let scoped_files = all_files
        .iter()
        .filter(|path| {
            let candidate = normalize_scope_path(root, path);
            scope_roots
                .iter()
                .any(|scope| candidate == *scope || candidate.starts_with(scope))
        })
        .cloned()
        .collect::<Vec<_>>();

    if scoped_files.is_empty() {
        anyhow::bail!("repo scope did not match any analyzable source files");
    }

    Ok(scoped_files)
}

fn normalize_scope_path(root: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    fs::canonicalize(&joined).unwrap_or(joined)
}

fn filter_repo_signals_to_scope(
    root: &Path,
    scoped_files: &[PathBuf],
    summary: &mut ArchitectureRepoSignalsSummary,
) {
    let matcher = RepoScopeMatcher::new(root, scoped_files);
    summary
        .unowned_item_details
        .retain(|item| matcher.matches_display_path(&item.path));
    summary.unowned_items = summary.unowned_item_details.len();
    summary
        .duplicate_item_details
        .retain(|item| matcher.matches_display_path(&item.path));
    summary.duplicate_items = summary.duplicate_item_details.len();
}

fn filter_traceability_to_scope(
    root: &Path,
    scoped_files: &[PathBuf],
    summary: &mut ArchitectureTraceabilitySummary,
) {
    let matcher = RepoScopeMatcher::new(root, scoped_files);
    retain_traceability_items(&matcher, &mut summary.current_spec_items);
    retain_traceability_items(&matcher, &mut summary.planned_only_items);
    retain_traceability_items(&matcher, &mut summary.deprecated_only_items);
    retain_traceability_items(&matcher, &mut summary.file_scoped_only_items);
    retain_traceability_items(&matcher, &mut summary.unverified_test_items);
    retain_traceability_items(&matcher, &mut summary.statically_mediated_items);
    retain_traceability_items(&matcher, &mut summary.unexplained_items);
    summary.analyzed_items = summary.current_spec_items.len()
        + summary.planned_only_items.len()
        + summary.deprecated_only_items.len()
        + summary.file_scoped_only_items.len()
        + summary.unverified_test_items.len()
        + summary.statically_mediated_items.len()
        + summary.unexplained_items.len();
    summary.sort_items();
}

fn retain_traceability_items(
    matcher: &RepoScopeMatcher,
    items: &mut Vec<ArchitectureTraceabilityItem>,
) {
    items.retain(|item| matcher.matches_display_path(&item.path));
}

fn retain_traceability_items_by_symbol(
    symbol: &str,
    items: &mut Vec<ArchitectureTraceabilityItem>,
) {
    items.retain(|item| item.name == symbol);
}

struct RepoScopeMatcher {
    root: PathBuf,
    scoped_files: BTreeSet<PathBuf>,
}

impl RepoScopeMatcher {
    fn new(root: &Path, scoped_files: &[PathBuf]) -> Self {
        Self {
            root: root.to_path_buf(),
            scoped_files: scoped_files
                .iter()
                .map(|path| normalize_scope_path(root, path))
                .collect(),
        }
    }

    fn matches_display_path(&self, display_path: &Path) -> bool {
        let candidate = normalize_scope_path(&self.root, display_path);
        self.scoped_files.contains(&candidate)
    }
}

#[derive(Debug, Default)]
pub(crate) struct FileOwnership<'a> {
    pub(crate) file_scoped: Vec<&'a ImplementRef>,
    pub(crate) item_scoped: Vec<&'a ImplementRef>,
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
    _files: &[PathBuf],
    parsed: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    repo_contexts: &registry::RepoAnalysisContexts,
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

        let (
            metrics,
            complexity,
            quality,
            item_signals,
            traceability,
            traceability_unavailable_reason,
            coupling,
            dependencies,
        ) = if options.metrics {
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
                    repo_contexts,
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
                traceability_unavailable_reason,
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
                traceability_unavailable_reason,
                coupling_inputs
                    .contains_key(&module.id)
                    .then_some(ModuleCouplingSummary::default()),
                dependencies,
            )
        } else {
            (None, None, None, None, None, None, None, None)
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
                traceability_unavailable_reason,
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
    repo_contexts: &registry::RepoAnalysisContexts,
    options: ModuleAnalysisOptions,
) -> Result<ProviderModuleAnalysis> {
    registry::analyze_module_language(
        language,
        root,
        implementations,
        file_ownership,
        repo_contexts,
        options,
    )
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
    if summary.traceability_unavailable_reason.is_none() {
        summary.traceability_unavailable_reason = delta.traceability_unavailable_reason;
    }
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
}

fn merge_optional_traceability(
    summary: &mut Option<ModuleTraceabilitySummary>,
    delta: Option<ModuleTraceabilitySummary>,
) {
    if summary.is_none() {
        *summary = delta;
    }
}

fn merge_optional_repo_traceability(
    summary: &mut Option<ArchitectureTraceabilitySummary>,
    delta: Option<ArchitectureTraceabilitySummary>,
) {
    let Some(delta) = delta else {
        return;
    };
    let target = summary.get_or_insert_with(ArchitectureTraceabilitySummary::default);
    target.extend_from(delta);
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
        Ok(())
    })?;
    Ok(summary)
}

pub(crate) fn visit_owned_texts<F>(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    mut visit: F,
) -> Result<()>
where
    F: FnMut(&Path, &str) -> Result<()>,
{
    for implementation in implementations {
        let path = &implementation.location.path;
        match &implementation.body {
            Some(body) => visit(path, body)?,
            None => {
                let Some(ownership) = file_ownership.get(path) else {
                    continue;
                };
                if !ownership.item_scoped.is_empty() {
                    continue;
                }
                let content = read_owned_file_text(root, path)?;
                visit(path, &content)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn read_owned_file_text(root: &Path, path: &Path) -> Result<String> {
    let full_path = root.join(path);
    fs::read_to_string(&full_path)
        .or_else(|_| fs::read_to_string(path))
        .with_context(|| {
            format!(
                "failed to read owned file {}",
                display_path(root, path).display()
            )
        })
}

fn line_count(text: &str) -> usize {
    text.lines().count().max(1)
}
