/**
@module SPECIAL.MODULES.ANALYZE.DUPLICATION
Surfaces repo-wide duplicate-logic signals from owned implementation so substantively similar code can be reviewed across files and module boundaries without relying on embeddings.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.DUPLICATION
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{ArchitectureDuplicateItem, ArchitectureRepoSignalsSummary, ModuleItemKind};
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
    let mut items = Vec::new();

    for module in &parsed.modules {
        let implementations = parsed
            .implements
            .iter()
            .filter(|implementation| implementation.module_id == module.id)
            .collect::<Vec<_>>();
        let mut module_items = collect_owned_items(root, &implementations, file_ownership)?
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
) -> Result<Vec<SourceItem>> {
    let mut items = Vec::new();
    let mut seen_file_scoped = BTreeSet::new();
    for implementation in implementations {
        let path = &implementation.location.path;
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

fn looks_like_test_path(path: &str) -> bool {
    let path = Path::new(path);
    if path
        .components()
        .any(|component| component.as_os_str() == "tests" || component.as_os_str() == "__tests__")
    {
        return true;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with("_test.go")
                || name.ends_with(".test.ts")
                || name.ends_with(".test.tsx")
                || name.ends_with(".spec.ts")
                || name.ends_with(".spec.tsx")
        })
}
