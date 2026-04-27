/**
@module SPECIAL.MODULES.ANALYZE.REPO_SCOPE
Owns repo-scope path matching and summary filtering for repo-wide analysis views.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.REPO_SCOPE
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{
    ArchitectureAnalysisSummary, ArchitectureRepoSignalsSummary, ArchitectureTraceabilityItem,
    ArchitectureTraceabilitySummary,
};
use crate::source_paths::normalize_existing_or_joined_path;
use crate::syntax::SourceLanguage;

pub(crate) struct RepoScopeBoundary {
    root: PathBuf,
    request_roots: Vec<PathBuf>,
    matched_files: Vec<PathBuf>,
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

pub(super) fn build_scope_boundary(
    root: &Path,
    all_files: &[PathBuf],
    scoped_paths: Option<&[PathBuf]>,
) -> Result<Option<RepoScopeBoundary>> {
    let Some(scoped_paths) = scoped_paths else {
        return Ok(None);
    };
    if scoped_paths.is_empty() {
        return Ok(None);
    }

    let boundary = RepoScopeBoundary::new(root, scoped_paths, all_files);

    if boundary.matched_files.is_empty() {
        anyhow::bail!("repo scope did not match any analyzable source files");
    }

    Ok(Some(boundary))
}

pub(super) fn filter_repo_signals_to_scope(
    boundary: &RepoScopeBoundary,
    summary: &mut ArchitectureRepoSignalsSummary,
) {
    summary
        .unowned_item_details
        .retain(|item| boundary.matches_display_path(&item.path));
    summary.unowned_items = summary.unowned_item_details.len();
    summary
        .duplicate_item_details
        .retain(|item| boundary.matches_display_path(&item.path));
    summary.duplicate_items = summary.duplicate_item_details.len();
}

pub(super) fn filter_traceability_to_scope(
    boundary: &RepoScopeBoundary,
    summary: &mut ArchitectureTraceabilitySummary,
) {
    retain_traceability_items(boundary, &mut summary.current_spec_items);
    retain_traceability_items(boundary, &mut summary.planned_only_items);
    retain_traceability_items(boundary, &mut summary.deprecated_only_items);
    retain_traceability_items(boundary, &mut summary.file_scoped_only_items);
    retain_traceability_items(boundary, &mut summary.unverified_test_items);
    retain_traceability_items(boundary, &mut summary.statically_mediated_items);
    retain_traceability_items(boundary, &mut summary.unexplained_items);
    summary.analyzed_items = summary.current_spec_items.len()
        + summary.planned_only_items.len()
        + summary.deprecated_only_items.len()
        + summary.file_scoped_only_items.len()
        + summary.unverified_test_items.len()
        + summary.statically_mediated_items.len()
        + summary.unexplained_items.len();
    summary.sort_items();
}

fn normalize_scope_path(root: &Path, path: &Path) -> PathBuf {
    normalize_existing_or_joined_path(root, path)
}

pub(crate) fn normalized_scope_paths(root: &Path, scoped_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut normalized = scoped_paths
        .iter()
        .map(|path| normalize_scope_path(root, path))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn retain_traceability_items(
    boundary: &RepoScopeBoundary,
    items: &mut Vec<ArchitectureTraceabilityItem>,
) {
    items.retain(|item| boundary.matches_display_path(&item.path));
}

fn retain_traceability_items_by_symbol(
    symbol: &str,
    items: &mut Vec<ArchitectureTraceabilityItem>,
) {
    items.retain(|item| item.name == symbol);
}

impl RepoScopeBoundary {
    fn new(root: &Path, scoped_paths: &[PathBuf], all_files: &[PathBuf]) -> Self {
        let request_roots = normalized_scope_paths(root, scoped_paths);
        let matched_files = all_files
            .iter()
            .filter(|path| {
                let candidate = normalize_scope_path(root, path);
                request_roots
                    .iter()
                    .any(|scope| candidate == *scope || candidate.starts_with(scope))
            })
            .cloned()
            .collect();

        Self {
            root: root.to_path_buf(),
            request_roots,
            matched_files,
        }
    }

    pub(super) fn matched_files(&self) -> &[PathBuf] {
        &self.matched_files
    }

    pub(super) fn traceability_candidate_files(&self, all_files: &[PathBuf]) -> Vec<PathBuf> {
        let scoped_languages = self
            .matched_files
            .iter()
            .filter_map(|path| SourceLanguage::from_path(path))
            .collect::<BTreeSet<_>>();
        if scoped_languages.is_empty() {
            return all_files.to_vec();
        }

        all_files
            .iter()
            .filter(|path| {
                SourceLanguage::from_path(path)
                    .is_some_and(|language| scoped_languages.contains(&language))
            })
            .cloned()
            .collect()
    }

    fn matches_display_path(&self, display_path: &Path) -> bool {
        let candidate = normalize_scope_path(&self.root, display_path);
        self.request_roots
            .iter()
            .any(|scope| candidate == *scope || candidate.starts_with(scope))
    }
}
