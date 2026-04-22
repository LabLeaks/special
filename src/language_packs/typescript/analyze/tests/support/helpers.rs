/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.HELPERS
Shared TypeScript scoped traceability test fixtures, identity helpers, and closure-local utilities.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.HELPERS
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::language_packs::typescript::analyze as analyze;
use crate::model::{ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary};
use crate::modules::analyze::FileOwnership;
use crate::modules::parse_architecture;
use crate::parser::{ParseDialect, parse_repo};

pub(crate) type TypeScriptFixtureContext = (
    PathBuf,
    crate::model::ParsedRepo,
    crate::model::ParsedArchitecture,
    Vec<PathBuf>,
    BTreeMap<PathBuf, FileOwnership<'static>>,
);

pub(crate) fn build_typescript_fixture_context(
    fixture_name: &str,
    fixture_writer: fn(&Path),
) -> Option<TypeScriptFixtureContext> {
    let root = temp_repo_dir(fixture_name);
    fixture_writer(&root);

    if analyze::typescript_runtime(&root).is_none() {
        fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
        return None;
    }

    let parsed_architecture = parse_architecture(&root, &[]).expect("architecture should parse");
    let parsed_repo =
        parse_repo(&root, &[], ParseDialect::CurrentV1).expect("repo annotations should parse");
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let file_ownership = index_file_ownership_for_test_owned(&parsed_architecture);

    Some((
        root,
        parsed_repo,
        parsed_architecture,
        discovered.source_files,
        file_ownership,
    ))
}

pub(crate) fn build_typescript_summary_from_closure(
    root: &Path,
    closure_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    parsed_repo: &crate::model::ParsedRepo,
    parsed_architecture: &crate::model::ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'static>>,
) -> Option<ArchitectureTraceabilitySummary> {
    let graph_facts = match analyze::build_traceability_graph_facts(root, closure_files) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => return None,
        Err(error) => panic!("graph facts should build: {error}"),
    };
    let analysis = match analyze::build_traceability_analysis_for_typescript(
        root,
        closure_files,
        scoped_source_files,
        Some(&graph_facts),
        parsed_repo,
        parsed_architecture,
        file_ownership,
    ) {
        Ok(analysis) => analysis,
        Err(error) if is_typescript_tooling_unavailable(&error) => return None,
        Err(error) => panic!("typescript analysis should build: {error}"),
    };
    Some(analyze::summarize_shared_repo_traceability(root, &analysis))
}

pub(crate) fn filter_summary_to_display_path(
    mut summary: ArchitectureTraceabilitySummary,
    scoped_path: &str,
) -> ArchitectureTraceabilitySummary {
    let scoped_path = Path::new(scoped_path);
    for items in [
        &mut summary.current_spec_items,
        &mut summary.planned_only_items,
        &mut summary.deprecated_only_items,
        &mut summary.file_scoped_only_items,
        &mut summary.unverified_test_items,
        &mut summary.statically_mediated_items,
        &mut summary.unexplained_items,
    ] {
        items.retain(|item| item.path == scoped_path);
    }
    summary.analyzed_items = summary
        .current_spec_items
        .iter()
        .chain(summary.planned_only_items.iter())
        .chain(summary.deprecated_only_items.iter())
        .chain(summary.file_scoped_only_items.iter())
        .chain(summary.unverified_test_items.iter())
        .chain(summary.statically_mediated_items.iter())
        .chain(summary.unexplained_items.iter())
        .map(identity_key)
        .collect::<BTreeSet<_>>()
        .len();
    summary.sort_items();
    summary
}

pub(crate) fn identity_key(item: &ArchitectureTraceabilityItem) -> String {
    serde_json::to_string(item).expect("traceability item identity should serialize")
}

pub(crate) fn summary_identity(summary: &ArchitectureTraceabilitySummary) -> String {
    serde_json::to_string(summary).expect("traceability summary should serialize")
}

pub(crate) fn index_file_ownership_for_test_owned(
    parsed: &crate::model::ParsedArchitecture,
) -> BTreeMap<PathBuf, FileOwnership<'static>> {
    let mut files: BTreeMap<PathBuf, FileOwnership<'static>> = BTreeMap::new();
    for implementation in &parsed.implements {
        let leaked = Box::leak(Box::new(implementation.clone()));
        let entry = files.entry(leaked.location.path.clone()).or_default();
        if leaked.body_location.is_some() {
            entry.item_scoped.push(leaked);
        } else {
            entry.file_scoped.push(leaked);
        }
    }
    files
}

pub(crate) fn temp_repo_dir(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{}-{timestamp}", std::process::id()));
    fs::create_dir_all(&path).expect("temp repo dir should be created");
    path
}

pub(crate) fn resolve_scoped_source_file(
    candidate_files: &[PathBuf],
    root: &Path,
    scoped_path: &str,
) -> PathBuf {
    let scoped_path = Path::new(scoped_path);
    candidate_files
        .iter()
        .find(|candidate| candidate.ends_with(scoped_path))
        .cloned()
        .unwrap_or_else(|| root.join(scoped_path))
}

pub(crate) fn relativize_contract_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> BTreeSet<PathBuf> {
    paths.into_iter()
        .map(|path| {
            path.components()
                .position(|component| {
                    matches!(
                        component.as_os_str().to_str(),
                        Some("src" | "app" | "components")
                    )
                })
                .map(|index| path.components().skip(index).collect())
                .unwrap_or(path)
        })
        .collect()
}

pub(crate) fn contract_contains_path(paths: &[PathBuf], expected: &str) -> bool {
    relativize_contract_paths(paths.iter().cloned()).contains(&PathBuf::from(expected))
}

pub(crate) fn is_typescript_tooling_unavailable(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("required Node runtime is not installed")
        || message.contains("Cannot find module")
        || message.contains("MODULE_NOT_FOUND")
}
