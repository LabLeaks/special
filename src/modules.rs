use std::collections::BTreeMap;
/**
@module SPECIAL.MODULES
Coordinates architecture parsing, lint, and projection over discovered source-local and markdown-backed `@module`/`@area` declarations. This subsystem owns the architecture tree and its ownership joins. It may project shared code-analysis evidence onto modules, but repo-wide code analysis must not require module ownership before code becomes visible.
*/
// @fileimplements SPECIAL.MODULES
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cache::{
    load_or_build_architecture_analysis, load_or_build_repo_analysis_summary,
    load_or_parse_architecture, load_or_parse_repo,
};
use crate::config::SpecialVersion;
use crate::model::{
    ArchitectureKind, ArchitectureMetricsSummary, GroupedCount, LintReport, ModuleAnalysisOptions,
    ModuleDocument, ModuleFilter, ModuleNode, ParsedArchitecture, RepoDocument, RepoMetricsSummary,
    RepoTraceabilityMetrics,
};

pub(crate) mod analyze;
mod lint;
mod materialize;
mod parse;
mod parse_markdown;

pub fn build_module_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    filter: ModuleFilter,
    analysis_options: ModuleAnalysisOptions,
) -> Result<(ModuleDocument, LintReport)> {
    let analysis_options = analysis_options.normalized();
    let parsed = load_or_parse_architecture(root, ignore_patterns)?;
    let lint = lint::build_module_lint_report(&parsed);
    let parsed_repo = analysis_options
        .any()
        .then(|| load_or_parse_repo(root, ignore_patterns, version))
        .transpose()?;
    let analysis = if analysis_options.any() {
        Some(load_or_build_architecture_analysis(
            root,
            ignore_patterns,
            version,
            &parsed,
            parsed_repo.as_ref(),
            analysis_options,
        )?)
    } else {
        None
    };
    let mut document =
        materialize::build_module_document(&parsed, filter, analysis.as_ref().map(|a| &a.modules));
    if analysis_options.metrics {
        document.metrics = Some(build_architecture_metrics(&document.nodes));
    }
    Ok((document, lint))
}

pub(crate) fn build_module_document_from_parsed(
    parsed: &ParsedArchitecture,
    filter: ModuleFilter,
) -> ModuleDocument {
    materialize::build_module_document(parsed, filter, None)
}

pub fn build_repo_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    metrics: bool,
    scoped_paths: Option<&[PathBuf]>,
    symbol: Option<&str>,
) -> Result<(RepoDocument, LintReport)> {
    let parsed = load_or_parse_architecture(root, ignore_patterns)?;
    let lint = lint::build_module_lint_report(&parsed);
    let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
    let mut summary =
        load_or_build_repo_analysis_summary(root, ignore_patterns, version, &parsed, &parsed_repo)?;
    if let Some(scoped_paths) = scoped_paths {
        analyze::filter_repo_analysis_summary_to_scope(
            root,
            ignore_patterns,
            scoped_paths,
            &mut summary,
        )?;
    }
    if let Some(symbol) = symbol {
        analyze::filter_repo_analysis_summary_to_symbol(symbol, &mut summary);
    }

    Ok((
        RepoDocument {
            metrics: metrics.then(|| build_repo_metrics(&summary)),
            analysis: Some(summary),
        },
        lint,
    ))
}

pub(crate) fn build_repo_document_from_parsed(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed: &ParsedArchitecture,
    parsed_repo: &crate::model::ParsedRepo,
    metrics: bool,
    scoped_paths: Option<&[PathBuf]>,
    symbol: Option<&str>,
) -> Result<RepoDocument> {
    let mut summary =
        load_or_build_repo_analysis_summary(root, ignore_patterns, version, parsed, parsed_repo)?;
    if let Some(scoped_paths) = scoped_paths {
        analyze::filter_repo_analysis_summary_to_scope(
            root,
            ignore_patterns,
            scoped_paths,
            &mut summary,
        )?;
    }
    if let Some(symbol) = symbol {
        analyze::filter_repo_analysis_summary_to_symbol(symbol, &mut summary);
    }

    Ok(RepoDocument {
        metrics: metrics.then(|| build_repo_metrics(&summary)),
        analysis: Some(summary),
    })
}

fn build_repo_metrics(summary: &crate::model::ArchitectureAnalysisSummary) -> RepoMetricsSummary {
    let duplicate_items_by_file = summary
        .repo_signals
        .as_ref()
        .map(|signals| {
            grouped_counts(
                signals
                    .duplicate_item_details
                    .iter()
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let unowned_items_by_file = summary
        .repo_signals
        .as_ref()
        .map(|signals| {
            grouped_counts(
                signals
                    .unowned_item_details
                    .iter()
                    .map(|item| item.path.display().to_string()),
            )
        })
        .unwrap_or_default();
    let traceability = summary
        .traceability
        .as_ref()
        .map(|traceability| RepoTraceabilityMetrics {
            analyzed_items: traceability.analyzed_items,
            current_spec_items: traceability.current_spec_items.len(),
            statically_mediated_items: traceability.statically_mediated_items.len(),
            unverified_test_items: traceability.unverified_test_items.len(),
            unexplained_items: traceability.unexplained_items.len(),
            unexplained_review_surface_items: traceability.unexplained_review_surface_items(),
            unexplained_public_items: traceability.unexplained_public_items(),
            unexplained_internal_items: traceability.unexplained_internal_items(),
            unexplained_module_backed_items: traceability.unexplained_module_backed_items(),
            unexplained_module_connected_items: traceability.unexplained_module_connected_items(),
            unexplained_module_isolated_items: traceability.unexplained_module_isolated_items(),
            unexplained_items_by_file: grouped_counts(
                traceability
                    .unexplained_items
                    .iter()
                    .map(|item| item.path.display().to_string()),
            ),
            unexplained_review_surface_items_by_file: grouped_counts(
                traceability
                    .unexplained_items
                    .iter()
                    .filter(|item| item.review_surface)
                    .map(|item| item.path.display().to_string()),
            ),
        });

    RepoMetricsSummary {
        duplicate_items: summary
            .repo_signals
            .as_ref()
            .map(|signals| signals.duplicate_items)
            .unwrap_or_default(),
        unowned_items: summary
            .repo_signals
            .as_ref()
            .map(|signals| signals.unowned_items)
            .unwrap_or_default(),
        duplicate_items_by_file,
        unowned_items_by_file,
        traceability,
    }
}

fn build_architecture_metrics(nodes: &[ModuleNode]) -> ArchitectureMetricsSummary {
    let mut summary = ArchitectureMetricsSummary {
        total_modules: 0,
        total_areas: 0,
        unimplemented_modules: 0,
        file_scoped_implements: 0,
        item_scoped_implements: 0,
        owned_lines: 0,
        public_items: 0,
        internal_items: 0,
        complexity_functions: 0,
        total_cyclomatic: 0,
        max_cyclomatic: 0,
        total_cognitive: 0,
        max_cognitive: 0,
        quality_public_functions: 0,
        quality_parameters: 0,
        quality_bool_params: 0,
        quality_raw_string_params: 0,
        quality_panic_sites: 0,
        unreached_items: 0,
        modules_by_area: Vec::new(),
        owned_lines_by_module: Vec::new(),
        max_cyclomatic_by_module: Vec::new(),
        max_cognitive_by_module: Vec::new(),
        panic_sites_by_module: Vec::new(),
        unreached_items_by_module: Vec::new(),
        external_dependency_targets_by_module: Vec::new(),
    };
    let mut modules_by_area: BTreeMap<String, usize> = BTreeMap::new();
    let mut owned_lines_by_module: BTreeMap<String, usize> = BTreeMap::new();
    let mut max_cyclomatic_by_module: BTreeMap<String, usize> = BTreeMap::new();
    let mut max_cognitive_by_module: BTreeMap<String, usize> = BTreeMap::new();
    let mut panic_sites_by_module: BTreeMap<String, usize> = BTreeMap::new();
    let mut unreached_items_by_module: BTreeMap<String, usize> = BTreeMap::new();
    let mut external_dependency_targets_by_module: BTreeMap<String, usize> = BTreeMap::new();

    append_architecture_metrics(
        nodes,
        &mut summary,
        &mut modules_by_area,
        &mut owned_lines_by_module,
        &mut max_cyclomatic_by_module,
        &mut max_cognitive_by_module,
        &mut panic_sites_by_module,
        &mut unreached_items_by_module,
        &mut external_dependency_targets_by_module,
    );

    summary.modules_by_area = grouped_count_map(modules_by_area);
    summary.owned_lines_by_module = grouped_count_map(owned_lines_by_module);
    summary.max_cyclomatic_by_module = grouped_count_map(max_cyclomatic_by_module);
    summary.max_cognitive_by_module = grouped_count_map(max_cognitive_by_module);
    summary.panic_sites_by_module = grouped_count_map(panic_sites_by_module);
    summary.unreached_items_by_module = grouped_count_map(unreached_items_by_module);
    summary.external_dependency_targets_by_module =
        grouped_count_map(external_dependency_targets_by_module);

    summary
}

fn append_architecture_metrics(
    nodes: &[ModuleNode],
    summary: &mut ArchitectureMetricsSummary,
    modules_by_area: &mut BTreeMap<String, usize>,
    owned_lines_by_module: &mut BTreeMap<String, usize>,
    max_cyclomatic_by_module: &mut BTreeMap<String, usize>,
    max_cognitive_by_module: &mut BTreeMap<String, usize>,
    panic_sites_by_module: &mut BTreeMap<String, usize>,
    unreached_items_by_module: &mut BTreeMap<String, usize>,
    external_dependency_targets_by_module: &mut BTreeMap<String, usize>,
) {
    for node in nodes {
        match node.kind() {
            ArchitectureKind::Area => {
                summary.total_areas += 1;
                let descendant_modules = count_descendant_modules(&node.children);
                if descendant_modules > 0 {
                    modules_by_area.insert(node.id.clone(), descendant_modules);
                }
            }
            ArchitectureKind::Module => {
                summary.total_modules += 1;
                if node.is_unimplemented() {
                    summary.unimplemented_modules += 1;
                }
            }
        }

        if let Some(analysis) = &node.analysis {
            if let Some(coverage) = &analysis.coverage {
                summary.file_scoped_implements += coverage.file_scoped_implements;
                summary.item_scoped_implements += coverage.item_scoped_implements;
            }
            if let Some(metrics) = &analysis.metrics {
                summary.owned_lines += metrics.owned_lines;
                summary.public_items += metrics.public_items;
                summary.internal_items += metrics.internal_items;
                if node.kind() == ArchitectureKind::Module && metrics.owned_lines > 0 {
                    owned_lines_by_module.insert(node.id.clone(), metrics.owned_lines);
                }
            }
            if let Some(complexity) = &analysis.complexity {
                summary.complexity_functions += complexity.function_count;
                summary.total_cyclomatic += complexity.total_cyclomatic;
                summary.max_cyclomatic = summary.max_cyclomatic.max(complexity.max_cyclomatic);
                summary.total_cognitive += complexity.total_cognitive;
                summary.max_cognitive = summary.max_cognitive.max(complexity.max_cognitive);
                if node.kind() == ArchitectureKind::Module {
                    if complexity.max_cyclomatic > 0 {
                        max_cyclomatic_by_module.insert(node.id.clone(), complexity.max_cyclomatic);
                    }
                    if complexity.max_cognitive > 0 {
                        max_cognitive_by_module.insert(node.id.clone(), complexity.max_cognitive);
                    }
                }
            }
            if let Some(quality) = &analysis.quality {
                summary.quality_public_functions += quality.public_function_count;
                summary.quality_parameters += quality.parameter_count;
                summary.quality_bool_params += quality.bool_parameter_count;
                summary.quality_raw_string_params += quality.raw_string_parameter_count;
                summary.quality_panic_sites += quality.panic_site_count;
                if node.kind() == ArchitectureKind::Module && quality.panic_site_count > 0 {
                    panic_sites_by_module.insert(node.id.clone(), quality.panic_site_count);
                }
            }
            if let Some(item_signals) = &analysis.item_signals {
                summary.unreached_items += item_signals.unreached_item_count;
                if node.kind() == ArchitectureKind::Module && item_signals.unreached_item_count > 0
                {
                    unreached_items_by_module
                        .insert(node.id.clone(), item_signals.unreached_item_count);
                }
            }
            if let Some(coupling) = &analysis.coupling
                && node.kind() == ArchitectureKind::Module
                && coupling.external_target_count > 0
            {
                external_dependency_targets_by_module
                    .insert(node.id.clone(), coupling.external_target_count);
            }
        }

        append_architecture_metrics(
            &node.children,
            summary,
            modules_by_area,
            owned_lines_by_module,
            max_cyclomatic_by_module,
            max_cognitive_by_module,
            panic_sites_by_module,
            unreached_items_by_module,
            external_dependency_targets_by_module,
        );
    }
}

fn count_descendant_modules(nodes: &[ModuleNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            usize::from(node.kind() == ArchitectureKind::Module)
                + count_descendant_modules(&node.children)
        })
        .sum()
}

fn grouped_counts(values: impl Iterator<Item = String>) -> Vec<GroupedCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(value, count)| GroupedCount { value, count })
        .collect()
}

fn grouped_count_map(counts: BTreeMap<String, usize>) -> Vec<GroupedCount> {
    counts
        .into_iter()
        .map(|(value, count)| GroupedCount { value, count })
        .collect()
}

pub fn build_module_lint_report(root: &Path, ignore_patterns: &[String]) -> Result<LintReport> {
    let parsed = parse_architecture(root, ignore_patterns)?;
    Ok(lint::build_module_lint_report(&parsed))
}

pub(crate) fn build_module_lint_report_from_parsed(parsed: &ParsedArchitecture) -> LintReport {
    lint::build_module_lint_report(parsed)
}

pub(crate) fn parse_architecture(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<ParsedArchitecture> {
    parse::parse_architecture(root, ignore_patterns)
}
