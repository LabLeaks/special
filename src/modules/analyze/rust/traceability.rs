/**
@module SPECIAL.MODULES.ANALYZE.RUST.TRACEABILITY
Builds conservative Rust implementation traceability from owned items through Rust tests to resolved spec lifecycle state without leaking parser-specific details into higher analysis layers.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.TRACEABILITY
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::model::{
    ImplementRef, ModuleItemKind, ModuleTraceabilityItem, ModuleTraceabilitySummary,
    ParsedArchitecture, ParsedRepo,
};
use crate::syntax::{ParsedSourceGraph, SourceCall, SourceInvocation, parse_source_graph};

use super::super::{FileOwnership, read_owned_file_text};

#[derive(Debug, Clone, Default)]
pub(super) struct RustTraceabilityCatalog {
    tests: Vec<RustTestTrace>,
}

#[derive(Debug, Clone)]
struct RustTestTrace {
    name: String,
    reachable_item_ids: BTreeSet<String>,
    has_item_scoped_support: bool,
    has_file_scoped_support: bool,
    live_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct OwnedRustItem {
    stable_id: String,
    kind: ModuleItemKind,
    name: String,
}

#[derive(Debug, Clone)]
struct SourceCallableItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    path: PathBuf,
    calls: Vec<SourceCall>,
    invocations: Vec<SourceInvocation>,
}

#[derive(Debug, Clone, Default)]
struct CallableIndexes {
    global_name_counts: BTreeMap<String, usize>,
    same_file_name_counts: BTreeMap<(PathBuf, String), usize>,
    global_qualified_name_counts: BTreeMap<String, usize>,
    same_file_qualified_name_counts: BTreeMap<(PathBuf, String), usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpecLifecycle {
    Live,
    Planned,
    Deprecated,
}

#[derive(Debug, Clone, Default)]
struct SpecStateBuckets {
    live_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
}

pub(super) fn build_traceability_catalog(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    _parsed_architecture: &ParsedArchitecture,
    _file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> RustTraceabilityCatalog {
    let source_graphs = parse_rust_source_graphs(root, source_files);
    let callable_items = collect_callable_items(&source_graphs);
    let callable_indexes = build_callable_indexes(&callable_items);
    let cargo_binary_entrypoints = collect_cargo_binary_entrypoints(root, &source_graphs);
    let call_edges = build_call_edges(
        &callable_items,
        &callable_indexes,
        &cargo_binary_entrypoints,
    );

    let spec_states = parsed_repo
        .specs
        .iter()
        .map(|spec| {
            let state = if spec.is_planned() {
                SpecLifecycle::Planned
            } else if spec.is_deprecated() {
                SpecLifecycle::Deprecated
            } else {
                SpecLifecycle::Live
            };
            (spec.id.clone(), state)
        })
        .collect::<BTreeMap<_, _>>();
    let (verify_by_item, verify_by_file) = build_verify_indexes(parsed_repo, &spec_states);

    let mut tests = Vec::new();
    for path in source_files {
        let Some(graph) = source_graphs.get(path) else {
            continue;
        };

        let file_specs = verify_by_file.get(path).cloned().unwrap_or_default();
        tests.extend(
            graph
                .items
                .iter()
                .filter(|item| item.is_test)
                .cloned()
                .map(|item| {
                    let reachable_item_ids =
                        reachable_items_from_test(&item.stable_id, &call_edges);
                    let item_specs = verify_by_item
                        .get(&(path.clone(), item.span.start_line))
                        .cloned()
                        .unwrap_or_default();
                    RustTestTrace {
                        name: item.name,
                        reachable_item_ids,
                        has_item_scoped_support: !item_specs.live_specs.is_empty()
                            || !item_specs.planned_specs.is_empty()
                            || !item_specs.deprecated_specs.is_empty(),
                        has_file_scoped_support: !file_specs.live_specs.is_empty()
                            || !file_specs.planned_specs.is_empty()
                            || !file_specs.deprecated_specs.is_empty(),
                        live_specs: item_specs
                            .live_specs
                            .union(&file_specs.live_specs)
                            .cloned()
                            .collect(),
                        planned_specs: item_specs
                            .planned_specs
                            .union(&file_specs.planned_specs)
                            .cloned()
                            .collect(),
                        deprecated_specs: item_specs
                            .deprecated_specs
                            .union(&file_specs.deprecated_specs)
                            .cloned()
                            .collect(),
                    }
                }),
        );
    }
    RustTraceabilityCatalog { tests }
}

pub(super) fn summarize_module_traceability(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    catalog: &RustTraceabilityCatalog,
) -> ModuleTraceabilitySummary {
    let owned_items = collect_owned_items(root, implementations, file_ownership, true);
    let mut live_spec_items = Vec::new();
    let mut planned_only_items = Vec::new();
    let mut deprecated_only_items = Vec::new();
    let mut file_scoped_only_items = Vec::new();
    let mut unverified_test_items = Vec::new();
    let mut unknown_items = Vec::new();

    for item in &owned_items {
        let mut verifying_tests = BTreeSet::new();
        let mut unverified_tests = BTreeSet::new();
        let mut live_specs = BTreeSet::new();
        let mut planned_specs = BTreeSet::new();
        let mut deprecated_specs = BTreeSet::new();
        let mut has_item_scoped_support = false;
        let mut has_file_scoped_support = false;

        for test in &catalog.tests {
            if !test_reaches_item(test, item, catalog) {
                continue;
            }

            if test.live_specs.is_empty()
                && test.planned_specs.is_empty()
                && test.deprecated_specs.is_empty()
            {
                unverified_tests.insert(test.name.clone());
                continue;
            }

            verifying_tests.insert(test.name.clone());
            live_specs.extend(test.live_specs.iter().cloned());
            planned_specs.extend(test.planned_specs.iter().cloned());
            deprecated_specs.extend(test.deprecated_specs.iter().cloned());
            has_item_scoped_support |= test.has_item_scoped_support;
            has_file_scoped_support |= test.has_file_scoped_support;
        }

        let traceability_item = ModuleTraceabilityItem {
            name: item.name.clone(),
            kind: item.kind,
            verifying_tests: verifying_tests.into_iter().collect(),
            unverified_tests: unverified_tests.into_iter().collect(),
            live_specs: live_specs.into_iter().collect(),
            planned_specs: planned_specs.into_iter().collect(),
            deprecated_specs: deprecated_specs.into_iter().collect(),
        };

        if has_file_scoped_support && !has_item_scoped_support {
            file_scoped_only_items.push(traceability_item.clone());
        }

        if !traceability_item.live_specs.is_empty() {
            live_spec_items.push(traceability_item);
        } else if !traceability_item.planned_specs.is_empty() {
            planned_only_items.push(traceability_item);
        } else if !traceability_item.deprecated_specs.is_empty() {
            deprecated_only_items.push(traceability_item);
        } else if !traceability_item.unverified_tests.is_empty() {
            unverified_test_items.push(traceability_item);
        } else {
            unknown_items.push(traceability_item);
        }
    }

    sort_traceability_items(&mut live_spec_items);
    sort_traceability_items(&mut planned_only_items);
    sort_traceability_items(&mut deprecated_only_items);
    sort_traceability_items(&mut file_scoped_only_items);
    sort_traceability_items(&mut unverified_test_items);
    sort_traceability_items(&mut unknown_items);

    ModuleTraceabilitySummary {
        analyzed_items: owned_items.len(),
        live_spec_items,
        planned_only_items,
        deprecated_only_items,
        file_scoped_only_items,
        unverified_test_items,
        unknown_items,
    }
}

fn build_verify_indexes(
    parsed_repo: &ParsedRepo,
    spec_states: &BTreeMap<String, SpecLifecycle>,
) -> (
    BTreeMap<(PathBuf, usize), SpecStateBuckets>,
    BTreeMap<PathBuf, SpecStateBuckets>,
) {
    let mut by_item = BTreeMap::new();
    let mut by_file = BTreeMap::new();

    for verify in &parsed_repo.verifies {
        let Some(state) = spec_states.get(&verify.spec_id).copied() else {
            continue;
        };
        if let Some(body_location) = &verify.body_location {
            let resolved_line = verify
                .body
                .as_ref()
                .and_then(|body| parse_source_graph(&body_location.path, body))
                .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
                .map(|start_line| body_location.line + start_line - 1)
                .unwrap_or(body_location.line);
            for target_line in
                body_location.line.min(resolved_line)..=body_location.line.max(resolved_line)
            {
                accumulate_spec_state(
                    by_item
                        .entry((body_location.path.clone(), target_line))
                        .or_default(),
                    &verify.spec_id,
                    state,
                );
            }
        } else {
            accumulate_spec_state(
                by_file.entry(verify.location.path.clone()).or_default(),
                &verify.spec_id,
                state,
            );
        }
    }

    (by_item, by_file)
}

fn collect_owned_items(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    dedupe: bool,
) -> Vec<OwnedRustItem> {
    let mut items = Vec::new();
    let mut seen = BTreeSet::new();
    for implementation in implementations {
        if implementation
            .location
            .path
            .extension()
            .and_then(|ext| ext.to_str())
            != Some("rs")
        {
            continue;
        }
        for item in owned_items_from_implementation(root, implementation, file_ownership) {
            let key = item.stable_id.clone();
            if !dedupe || seen.insert(key) {
                items.push(item);
            }
        }
    }
    items
}

fn owned_items_from_implementation(
    root: &Path,
    implementation: &ImplementRef,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<OwnedRustItem> {
    let Some(graph) = parse_owned_implementation_graph(root, implementation, file_ownership) else {
        return Vec::new();
    };

    graph
        .items
        .into_iter()
        .filter(|item| !item.is_test)
        .map(|item| OwnedRustItem {
            stable_id: item.stable_id,
            name: item.name,
            kind: source_item_kind(item.kind),
        })
        .collect()
}

fn parse_owned_implementation_graph(
    root: &Path,
    implementation: &ImplementRef,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Option<ParsedSourceGraph> {
    if let Some(body) = &implementation.body {
        return parse_source_graph(&implementation.location.path, body);
    }

    let ownership = file_ownership.get(&implementation.location.path)?;
    if !ownership.item_scoped.is_empty() {
        return None;
    }

    let text = read_owned_file_text(root, &implementation.location.path);
    parse_source_graph(&implementation.location.path, &text)
}

fn test_reaches_item(
    test: &RustTestTrace,
    item: &OwnedRustItem,
    _catalog: &RustTraceabilityCatalog,
) -> bool {
    test.reachable_item_ids.contains(&item.stable_id)
}

fn parse_rust_source_graphs(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, ParsedSourceGraph> {
    source_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .filter_map(|path| {
            let text = read_owned_file_text(root, path);
            parse_source_graph(path, &text).map(|graph| (path.clone(), graph))
        })
        .collect()
}

fn collect_callable_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Vec<SourceCallableItem> {
    let mut items = Vec::new();
    for (path, graph) in source_graphs {
        items.extend(graph.items.iter().cloned().map(|item| SourceCallableItem {
            stable_id: item.stable_id,
            name: item.name,
            qualified_name: item.qualified_name,
            module_path: item.module_path,
            container_path: item.container_path,
            path: path.clone(),
            calls: item.calls,
            invocations: item.invocations,
        }));
    }
    items
}

fn build_callable_indexes(items: &[SourceCallableItem]) -> CallableIndexes {
    let mut indexes = CallableIndexes::default();
    for item in items {
        *indexes
            .global_name_counts
            .entry(item.name.clone())
            .or_default() += 1;
        *indexes
            .same_file_name_counts
            .entry((item.path.clone(), item.name.clone()))
            .or_default() += 1;
        *indexes
            .global_qualified_name_counts
            .entry(item.qualified_name.clone())
            .or_default() += 1;
        *indexes
            .same_file_qualified_name_counts
            .entry((item.path.clone(), item.qualified_name.clone()))
            .or_default() += 1;
    }
    indexes
}

fn build_call_edges(
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
    cargo_binary_entrypoints: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut edges = BTreeMap::new();
    for item in items {
        let mut callees = item
            .calls
            .iter()
            .filter_map(|call| resolve_call_target(item, call, items, indexes))
            .collect::<BTreeSet<_>>();
        for invocation in &item.invocations {
            for entrypoint_id in resolve_invocation_targets(invocation, cargo_binary_entrypoints) {
                callees.insert(entrypoint_id);
            }
        }
        edges.insert(item.stable_id.clone(), callees);
    }
    edges
}

fn resolve_call_target(
    caller: &SourceCallableItem,
    call: &SourceCall,
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
) -> Option<String> {
    if let Some(qualifier) = call.qualifier.as_ref() {
        for qualified_name in qualified_name_candidates(caller, qualifier, &call.name) {
            if indexes
                .global_qualified_name_counts
                .get(&qualified_name)
                .copied()
                .unwrap_or(0)
                == 1
            {
                return items
                    .iter()
                    .find(|item| item.qualified_name == qualified_name)
                    .map(|item| item.stable_id.clone());
            }

            if indexes
                .same_file_qualified_name_counts
                .get(&(caller.path.clone(), qualified_name.clone()))
                .copied()
                .unwrap_or(0)
                == 1
            {
                return items
                    .iter()
                    .find(|item| item.path == caller.path && item.qualified_name == qualified_name)
                    .map(|item| item.stable_id.clone());
            }
        }
    }

    if indexes
        .same_file_name_counts
        .get(&(caller.path.clone(), call.name.clone()))
        .copied()
        .unwrap_or(0)
        == 1
    {
        return items
            .iter()
            .find(|item| item.path == caller.path && item.name == call.name)
            .map(|item| item.stable_id.clone());
    }

    if indexes
        .global_name_counts
        .get(&call.name)
        .copied()
        .unwrap_or(0)
        == 1
    {
        return items
            .iter()
            .find(|item| item.name == call.name)
            .map(|item| item.stable_id.clone());
    }

    None
}

fn resolve_invocation_targets(
    invocation: &SourceInvocation,
    cargo_binary_entrypoints: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    match &invocation.kind {
        crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name } => {
            cargo_binary_entrypoints
                .get(binary_name)
                .cloned()
                .unwrap_or_default()
        }
    }
}

fn reachable_items_from_test(
    test_stable_id: &str,
    call_edges: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut visited = BTreeSet::new();
    let mut pending = call_edges
        .get(test_stable_id)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();

    while let Some(item_id) = pending.pop() {
        if !visited.insert(item_id.clone()) {
            continue;
        }
        if let Some(next) = call_edges.get(&item_id) {
            pending.extend(next.iter().cloned());
        }
    }

    visited
}

fn collect_cargo_binary_entrypoints(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut entrypoints = BTreeMap::new();
    let cargo_toml_path = root.join("Cargo.toml");
    let Ok(cargo_toml_text) = std::fs::read_to_string(cargo_toml_path) else {
        return entrypoints;
    };
    let Ok(cargo_toml) = cargo_toml_text.parse::<toml::Value>() else {
        return entrypoints;
    };
    let Some(bin_entries) = cargo_toml.get("bin").and_then(|value| value.as_array()) else {
        return entrypoints;
    };

    for bin_entry in bin_entries {
        let Some(bin_name) = bin_entry.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(bin_path) = bin_entry.get("path").and_then(|value| value.as_str()) else {
            continue;
        };
        let bin_path = PathBuf::from(bin_path);
        let Some((_, graph)) = source_graphs.iter().find(|(path, _)| {
            *path == &bin_path || *path == &root.join(&bin_path) || path.ends_with(&bin_path)
        }) else {
            continue;
        };
        let main_ids = graph
            .items
            .iter()
            .filter(|item| item.name == "main")
            .map(|item| item.stable_id.clone())
            .collect::<BTreeSet<_>>();
        if !main_ids.is_empty() {
            entrypoints.insert(bin_name.to_string(), main_ids);
        }
    }

    entrypoints
}

fn qualified_name_candidates(
    caller: &SourceCallableItem,
    qualifier: &str,
    call_name: &str,
) -> Vec<String> {
    let prefixes = qualifier_prefix_candidates(caller, qualifier);
    let mut seen = BTreeSet::new();
    prefixes
        .into_iter()
        .filter_map(|mut segments| {
            segments.push(call_name.to_string());
            let qualified_name = segments.join("::");
            seen.insert(qualified_name.clone())
                .then_some(qualified_name)
        })
        .collect()
}

fn qualifier_prefix_candidates(caller: &SourceCallableItem, qualifier: &str) -> Vec<Vec<String>> {
    let segments = qualifier
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return Vec::new();
    }

    if segments.first().map(String::as_str) == Some("crate") {
        return vec![segments.into_iter().skip(1).collect()];
    }

    if segments.first().map(String::as_str) == Some("self") {
        if segments.len() == 1 && !caller.container_path.is_empty() {
            return vec![
                caller
                    .module_path
                    .iter()
                    .cloned()
                    .chain(caller.container_path.iter().cloned())
                    .collect(),
            ];
        }
        return vec![
            caller
                .module_path
                .iter()
                .cloned()
                .chain(segments.into_iter().skip(1))
                .collect(),
        ];
    }

    if segments.first().map(String::as_str) == Some("Self") {
        if caller.container_path.is_empty() {
            return Vec::new();
        }
        return vec![
            caller
                .module_path
                .iter()
                .cloned()
                .chain(caller.container_path.iter().cloned())
                .chain(segments.into_iter().skip(1))
                .collect(),
        ];
    }

    if segments.first().map(String::as_str) == Some("super") {
        let mut ancestor_depth = 0usize;
        while segments.get(ancestor_depth).map(String::as_str) == Some("super") {
            ancestor_depth += 1;
        }
        if ancestor_depth > caller.module_path.len() {
            return Vec::new();
        }
        let base_len = caller.module_path.len().saturating_sub(ancestor_depth);
        return vec![
            caller.module_path[..base_len]
                .iter()
                .cloned()
                .chain(segments.into_iter().skip(ancestor_depth))
                .collect(),
        ];
    }

    let mut candidates = Vec::new();
    for ancestor_len in (0..=caller.module_path.len()).rev() {
        candidates.push(
            caller.module_path[..ancestor_len]
                .iter()
                .cloned()
                .chain(segments.iter().cloned())
                .collect(),
        );
    }
    candidates
}

fn source_item_kind(kind: crate::syntax::SourceItemKind) -> ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => ModuleItemKind::Method,
    }
}

fn accumulate_spec_state(buckets: &mut SpecStateBuckets, spec_id: &str, state: SpecLifecycle) {
    match state {
        SpecLifecycle::Live => {
            buckets.live_specs.insert(spec_id.to_string());
        }
        SpecLifecycle::Planned => {
            buckets.planned_specs.insert(spec_id.to_string());
        }
        SpecLifecycle::Deprecated => {
            buckets.deprecated_specs.insert(spec_id.to_string());
        }
    }
}

fn sort_traceability_items(items: &mut [ModuleTraceabilityItem]) {
    items.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.kind.cmp(&right.kind))
    });
}
