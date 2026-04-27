/**
@module SPECIAL.MODULES.ANALYZE
Builds language-agnostic evidence-first implementation analysis over analyzable code and concrete module ownership, combining a shared item-evidence core with compile-time language-pack registration from `SPECIAL.LANGUAGE_PACKS` instead of hardcoded per-language dispatch in the analysis core. Repo-facing analysis should stay code-first, module-facing analysis should remain an ownership projection over shared evidence, and backward trace should only run when the active language pack says its required local tool is available. When backward trace does run, it should report direct, statically mediated, or currently unexplained evidence rather than over-claiming negative reachability.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    ArchitectureAnalysisSummary, ArchitectureTraceabilitySummary, ModuleAnalysisOptions,
    ModuleAnalysisSummary, ModuleComplexitySummary, ModuleDependencySummary,
    ModuleItemSignalsSummary, ModuleMetricsSummary, ModuleQualitySummary,
    ModuleTraceabilitySummary, ParsedArchitecture, ParsedRepo,
};

mod coupling;
mod coverage;
mod duplication;
pub(crate) mod explain;
mod module_summary;
mod ownership;
mod provider_merge;
mod registry;
mod repo_scope;
pub(crate) mod source_item_signals;
mod status;
pub(crate) mod traceability_core;
mod unreached_code;

pub(crate) use coupling::ModuleCouplingInput;
pub(crate) use ownership::{FileOwnership, display_path, read_owned_file_text, visit_owned_texts};
pub(crate) use provider_merge::build_dependency_summary;
pub(crate) use repo_scope::normalized_scope_paths;
pub(crate) use status::{emit_analysis_status, with_analysis_status_notifier};

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
    let file_ownership = ownership::index_file_ownership(parsed);
    let repo_contexts = if options.metrics || options.traceability {
        parsed_repo
            .map(|parsed_repo| {
                registry::build_repo_analysis_contexts(
                    root,
                    &files,
                    None,
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
    let modules = module_summary::build_module_analysis(
        root,
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
    let scope_boundary = repo_scope::build_scope_boundary(root, &all_files, scoped_paths)?;
    let scoped_files = scope_boundary
        .as_ref()
        .map(|boundary| boundary.matched_files())
        .unwrap_or(&all_files);
    if scope_boundary.is_none() {
        status::emit_analysis_status(&format!(
            "repo health analysis considers {} analyzable source files",
            all_files.len()
        ));
    } else {
        status::emit_analysis_status(&format!(
            "repo health scope matches {} of {} analyzable source files",
            scoped_files.len(),
            all_files.len()
        ));
    }
    let file_ownership = ownership::index_file_ownership(parsed);

    let mut repo_signals = coverage::build_repo_signals_summary();
    status::emit_analysis_status(&format!(
        "computing repo ownership signals from {} scoped source files",
        scoped_files.len()
    ));
    unreached_code::apply_unowned_item_summary(
        root,
        scoped_files,
        &file_ownership,
        &mut repo_signals,
    )?;
    status::emit_analysis_status("computing duplicate-logic signals across owned implementation");
    duplication::apply_duplicate_item_summary(root, parsed, &file_ownership, &mut repo_signals)?;
    if let Some(boundary) = scope_boundary.as_ref() {
        repo_scope::filter_repo_signals_to_scope(boundary, &mut repo_signals);
    }

    let traceability_context_files = scope_boundary
        .as_ref()
        .map(|boundary| boundary.traceability_candidate_files(&all_files))
        .unwrap_or_else(|| all_files.clone());
    status::emit_analysis_status(&format!(
        "building language analysis contexts from {} source files",
        traceability_context_files.len()
    ));
    let repo_contexts = registry::build_repo_analysis_contexts(
        root,
        &traceability_context_files,
        scope_boundary
            .as_ref()
            .map(|boundary| boundary.matched_files()),
        parsed_repo,
        parsed,
        &file_ownership,
        true,
    );
    let traceability = build_repo_traceability_summary(
        root,
        &traceability_context_files,
        &repo_contexts,
    )
    .map(|mut summary| {
        if let Some(boundary) = scope_boundary.as_ref() {
            status::emit_analysis_status("filtering repo traceability to the requested scope");
            repo_scope::filter_traceability_to_scope(boundary, &mut summary);
        }
        summary
    });
    let traceability_unavailable_reason =
        build_repo_traceability_unavailable_reason(&traceability_context_files, &repo_contexts);

    Ok(ArchitectureAnalysisSummary {
        repo_signals: Some(repo_signals),
        traceability,
        traceability_unavailable_reason,
    })
}

pub(crate) fn build_bounded_repo_analysis_summary(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    parsed_repo: &ParsedRepo,
    within_paths: &[PathBuf],
) -> Result<ArchitectureAnalysisSummary> {
    let all_files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let Some(boundary) = repo_scope::build_scope_boundary(root, &all_files, Some(within_paths))?
    else {
        return build_repo_analysis_summary(root, ignore_patterns, parsed, parsed_repo, None);
    };
    let corpus_files = boundary.matched_files();
    status::emit_analysis_status(&format!(
        "repo health hard analysis corpus matches {} of {} analyzable source files",
        corpus_files.len(),
        all_files.len()
    ));
    let file_ownership = ownership::index_file_ownership(parsed);

    let mut repo_signals = coverage::build_repo_signals_summary();
    status::emit_analysis_status(&format!(
        "computing repo ownership signals from {} corpus source files",
        corpus_files.len()
    ));
    unreached_code::apply_unowned_item_summary(
        root,
        corpus_files,
        &file_ownership,
        &mut repo_signals,
    )?;
    status::emit_analysis_status(
        "computing duplicate-logic signals across bounded owned implementation",
    );
    duplication::apply_duplicate_item_summary_in_files(
        root,
        parsed,
        &file_ownership,
        Some(corpus_files),
        &mut repo_signals,
    )?;

    status::emit_analysis_status(&format!(
        "building language analysis contexts from {} bounded source files",
        corpus_files.len()
    ));
    let repo_contexts = registry::build_repo_analysis_contexts(
        root,
        corpus_files,
        None,
        parsed_repo,
        parsed,
        &file_ownership,
        true,
    );
    let traceability = build_repo_traceability_summary(root, corpus_files, &repo_contexts);
    let traceability_unavailable_reason =
        build_repo_traceability_unavailable_reason(corpus_files, &repo_contexts);

    Ok(ArchitectureAnalysisSummary {
        repo_signals: Some(repo_signals),
        traceability,
        traceability_unavailable_reason,
    })
}

pub(crate) fn filter_repo_analysis_summary_to_symbol(
    symbol: &str,
    summary: &mut ArchitectureAnalysisSummary,
) {
    repo_scope::filter_repo_analysis_summary_to_symbol(symbol, summary);
}

pub(crate) fn filter_repo_analysis_summary_to_paths(
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
    let Some(boundary) = repo_scope::build_scope_boundary(root, &all_files, Some(scoped_paths))?
    else {
        return Ok(());
    };
    if let Some(repo_signals) = &mut summary.repo_signals {
        repo_scope::filter_repo_signals_to_scope(&boundary, repo_signals);
    }
    if let Some(traceability) = &mut summary.traceability {
        repo_scope::filter_traceability_to_scope(&boundary, traceability);
    }
    Ok(())
}

fn build_repo_traceability_summary(
    root: &Path,
    files: &[PathBuf],
    repo_contexts: &registry::RepoAnalysisContexts,
) -> Option<ArchitectureTraceabilitySummary> {
    let mut summary = None;

    for language in registry::languages_in_files(files) {
        provider_merge::merge_optional_repo_traceability(
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
