/**
@module SPECIAL.MODULES.ANALYZE.DUPLICATION
Surfaces repo-wide duplicate-logic signals from owned implementation so substantively similar code can be reviewed across files and module boundaries without relying on embeddings.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.DUPLICATION
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{ArchitectureDuplicateItem, ArchitectureRepoSignalsSummary, ModuleItemKind};
use crate::source_paths::{looks_like_test_path, normalize_existing_or_joined_path};
use crate::syntax::{
    CallSyntaxKind, SourceInvocationKind, SourceItem, SourceItemKind, parse_source_graph,
};

use super::{FileOwnership, display_path, read_owned_file_text};

const MIN_DUPLICATE_SHAPE_NODES: usize = 24;
const MIN_DUPLICATE_SUBSTANTIVE_SCORE: usize = 4;

pub(super) fn apply_duplicate_item_summary(
    root: &Path,
    parsed: &crate::model::ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    coverage: &mut ArchitectureRepoSignalsSummary,
) -> Result<()> {
    apply_duplicate_item_summary_in_files(root, parsed, file_ownership, None, coverage)
}

pub(super) fn apply_duplicate_item_summary_in_files(
    root: &Path,
    parsed: &crate::model::ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    source_files: Option<&[PathBuf]>,
    coverage: &mut ArchitectureRepoSignalsSummary,
) -> Result<()> {
    let mut items = Vec::new();
    let source_files = source_files.map(|paths| {
        paths
            .iter()
            .map(|path| normalize_file_path(root, path))
            .collect::<BTreeSet<_>>()
    });

    for module in &parsed.modules {
        let implementations = parsed
            .implements
            .iter()
            .filter(|implementation| implementation.module_id == module.id)
            .collect::<Vec<_>>();
        let mut module_items = collect_owned_items(
            root,
            &implementations,
            file_ownership,
            source_files.as_ref(),
        )?
        .into_iter()
        .filter_map(|item| {
            let duplicate_key = duplication_key(&item)?;
            Some(OwnedDuplicateItem {
                module_id: module.id.clone(),
                source_path: item.source_path,
                name: item.name,
                kind: match item.kind {
                    SourceItemKind::Function => ModuleItemKind::Function,
                    SourceItemKind::Method => ModuleItemKind::Method,
                },
                duplicate_key,
            })
        })
        .collect::<Vec<_>>();
        items.append(&mut module_items);
    }

    let mut groups: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        groups
            .entry(item.duplicate_key.as_str())
            .or_default()
            .push(index);
    }

    let mut duplicates = Vec::new();
    for indexes in groups.values() {
        if indexes.len() < 2 {
            continue;
        }
        for index in indexes {
            let item = &items[*index];
            duplicates.push(ArchitectureDuplicateItem {
                module_id: item.module_id.clone(),
                path: display_path(root, Path::new(&item.source_path)),
                name: item.name.clone(),
                kind: item.kind,
                duplicate_peer_count: indexes.len() - 1,
            });
        }
    }

    duplicates.sort_by(|left, right| {
        right
            .duplicate_peer_count
            .cmp(&left.duplicate_peer_count)
            .then_with(|| left.module_id.cmp(&right.module_id))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.name.cmp(&right.name))
    });

    coverage.duplicate_items = duplicates.len();
    coverage.duplicate_item_details = duplicates;
    Ok(())
}

fn collect_owned_items(
    root: &Path,
    implementations: &[&crate::model::ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    source_files: Option<&BTreeSet<PathBuf>>,
) -> Result<Vec<SourceItem>> {
    let mut items = Vec::new();
    let mut seen_file_scoped = BTreeSet::new();
    for implementation in implementations {
        let path = &implementation.location.path;
        if source_files.is_some_and(|allowed| !path_matches_allowed(root, path, allowed)) {
            continue;
        }
        if let Some(body) = &implementation.body {
            if let Some(graph) = parse_source_graph(path, body) {
                items.extend(graph.items);
            }
            continue;
        }

        let Some(ownership) = file_ownership.get(path) else {
            continue;
        };
        if !ownership.item_scoped.is_empty() || !seen_file_scoped.insert(path.clone()) {
            continue;
        }
        let text = read_owned_file_text(root, path)?;
        if let Some(graph) = parse_source_graph(path, &text) {
            items.extend(graph.items);
        }
    }

    Ok(items)
}

fn path_matches_allowed(root: &Path, path: &Path, allowed: &BTreeSet<PathBuf>) -> bool {
    allowed.contains(&normalize_file_path(root, path))
}

fn normalize_file_path(root: &Path, path: &Path) -> PathBuf {
    normalize_existing_or_joined_path(root, path)
}

struct OwnedDuplicateItem {
    module_id: String,
    source_path: String,
    name: String,
    kind: ModuleItemKind,
    duplicate_key: String,
}

fn duplication_key(item: &SourceItem) -> Option<String> {
    if item.is_test || looks_like_test_path(&item.source_path) {
        return None;
    }
    if item.shape_node_count < MIN_DUPLICATE_SHAPE_NODES {
        return None;
    }
    if !has_substantive_structure(item) {
        return None;
    }

    let call_profile = item
        .calls
        .iter()
        .map(|call| match &call.qualifier {
            Some(qualifier) => format!("{qualifier}::{}", call.name),
            None => call.name.clone(),
        })
        .collect::<Vec<_>>()
        .join("|");
    let invocation_profile = item
        .invocations
        .iter()
        .map(|invocation| match &invocation.kind {
            SourceInvocationKind::LocalCargoBinary { binary_name } => {
                format!("cargo-bin:{binary_name}")
            }
        })
        .collect::<Vec<_>>()
        .join("|");

    Some(format!(
        "{}#calls:{}#invoke:{}",
        item.shape_fingerprint, call_profile, invocation_profile
    ))
}

fn has_substantive_structure(item: &SourceItem) -> bool {
    let score = substantive_operation_score(item);
    score >= MIN_DUPLICATE_SUBSTANTIVE_SCORE && has_stateful_structure(item)
}

fn substantive_operation_score(item: &SourceItem) -> usize {
    let call_score = item.calls.len() + item.invocations.len();
    let nontrivial_call_bonus =
        item.calls.iter().any(|call| {
            call.qualifier.is_some() || !matches!(call.syntax, CallSyntaxKind::Identifier)
        }) as usize;
    let control_flow_score = score_occurrences(
        &item.shape_fingerprint,
        &[
            "if_expression",
            "match_expression",
            "conditional_type",
            "switch_statement",
            "if_statement",
        ],
    );
    let loop_score = score_occurrences(
        &item.shape_fingerprint,
        &[
            "for_expression",
            "while_expression",
            "loop_expression",
            "for_statement",
        ],
    );
    let dataflow_score = score_occurrences(
        &item.shape_fingerprint,
        &[
            "let_declaration",
            "lexical_declaration",
            "variable_declarator",
            "short_var_declaration",
            "assignment_expression",
            "augmented_assignment_expression",
        ],
    );
    let closure_score = score_occurrences(
        &item.shape_fingerprint,
        &[
            "closure_expression",
            "arrow_function",
            "func_literal",
            "anonymous_function",
        ],
    );

    call_score
        + nontrivial_call_bonus
        + control_flow_score
        + loop_score
        + dataflow_score
        + closure_score
}

fn has_stateful_structure(item: &SourceItem) -> bool {
    score_occurrences(
        &item.shape_fingerprint,
        &[
            "for_expression",
            "while_expression",
            "loop_expression",
            "for_statement",
            "let_declaration",
            "lexical_declaration",
            "variable_declarator",
            "short_var_declaration",
            "assignment_expression",
            "augmented_assignment_expression",
            "closure_expression",
            "arrow_function",
            "func_literal",
            "anonymous_function",
        ],
    ) > 0
}

fn score_occurrences(shape_fingerprint: &str, markers: &[&str]) -> usize {
    markers
        .iter()
        .map(|marker| shape_fingerprint.matches(marker).count())
        .sum()
}
