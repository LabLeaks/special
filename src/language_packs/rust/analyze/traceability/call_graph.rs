/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.CALL_GRAPH
Builds Rust traceability call edges from parser-visible calls, cargo binary entrypoints, import aliasing, and rust-analyzer reverse reachability.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.CALL_GRAPH
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::syntax::{ParsedSourceGraph, SourceCall, SourceInvocation, SourceSpan};

use super::super::rust_analyzer::RustAnalyzerCallableItem;
use super::super::toolchain::RustToolchainProject;
use super::super::use_tree::collect_use_aliases;

#[derive(Debug, Clone)]
struct SourceCallableItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    path: PathBuf,
    span: SourceSpan,
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

pub(super) fn build_parser_call_edges_with_toolchain(
    root: &Path,
    source_files: &[PathBuf],
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    toolchain_project: Option<&RustToolchainProject>,
) -> BTreeMap<String, BTreeSet<String>> {
    let cargo_binary_entrypoints = toolchain_project
        .map(|project| collect_toolchain_binary_entrypoints(project, source_graphs))
        .unwrap_or_else(|| collect_cargo_binary_entrypoints(root, source_graphs));
    let crate_root_aliases = toolchain_project
        .map(RustToolchainProject::crate_root_aliases)
        .unwrap_or_else(|| collect_crate_root_aliases(root));

    build_parser_call_edges(
        root,
        source_files,
        source_graphs,
        cargo_binary_entrypoints,
        crate_root_aliases,
    )
}

pub(super) fn build_rust_analyzer_call_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    cargo_binary_entrypoints: BTreeMap<String, BTreeSet<String>>,
    parser_call_edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let callable_items = collect_callable_items(source_graphs);
    let seed_ids = source_graphs
        .values()
        .flat_map(|graph| graph.items.iter().filter(|item| item.is_test))
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    let items = callable_items
        .iter()
        .map(|item| RustAnalyzerCallableItem {
            stable_id: item.stable_id.clone(),
            name: item.name.clone(),
            path: root.join(&item.path),
            span: item.span,
            calls: item.calls.clone(),
            invocation_targets: item
                .invocations
                .iter()
                .flat_map(|invocation| {
                    resolve_invocation_targets(invocation, &cargo_binary_entrypoints)
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    super::super::rust_analyzer::build_reachable_call_edges(root, &items, &seed_ids, parser_call_edges)
}

pub(super) fn collect_rust_analyzer_reference_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Vec<RustAnalyzerCallableItem> {
    collect_callable_items(source_graphs)
        .into_iter()
        .map(|item| RustAnalyzerCallableItem {
            stable_id: item.stable_id,
            name: item.name,
            path: item.path,
            span: item.span,
            calls: item.calls,
            invocation_targets: BTreeSet::new(),
        })
        .collect()
}

fn build_parser_call_edges(
    root: &Path,
    source_files: &[PathBuf],
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    cargo_binary_entrypoints: BTreeMap<String, BTreeSet<String>>,
    crate_root_aliases: BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let callable_items = collect_callable_items(source_graphs);
    let callable_indexes = build_callable_indexes(&callable_items);
    let imported_call_aliases = collect_imported_call_aliases(root, source_files);
    build_call_edges(
        &callable_items,
        &callable_indexes,
        &cargo_binary_entrypoints,
        &crate_root_aliases,
        &imported_call_aliases,
    )
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
            span: item.span,
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
    crate_root_aliases: &BTreeSet<String>,
    imported_call_aliases: &BTreeMap<PathBuf, BTreeMap<String, Vec<String>>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut edges = BTreeMap::new();
    for item in items {
        let mut callees = item
            .calls
            .iter()
            .filter_map(|call| {
                resolve_call_target(
                    item,
                    call,
                    items,
                    indexes,
                    crate_root_aliases,
                    imported_call_aliases,
                )
            })
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
    crate_root_aliases: &BTreeSet<String>,
    imported_call_aliases: &BTreeMap<PathBuf, BTreeMap<String, Vec<String>>>,
) -> Option<String> {
    if let Some(qualifier) = call.qualifier.as_ref() {
        for qualified_name in
            qualified_name_candidates(caller, qualifier, &call.name, crate_root_aliases)
        {
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

    if let Some(imports) = imported_call_aliases
        .get(&caller.path)
        .and_then(|aliases| aliases.get(&call.name))
    {
        for imported_path in imports {
            let Some((qualifier, imported_name)) = imported_path.rsplit_once("::") else {
                continue;
            };
            for qualified_name in
                qualified_name_candidates(caller, qualifier, imported_name, crate_root_aliases)
            {
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
                        .find(|item| {
                            item.path == caller.path && item.qualified_name == qualified_name
                        })
                        .map(|item| item.stable_id.clone());
                }
            }
        }
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

pub(super) fn collect_cargo_binary_entrypoints(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    let cargo_toml_path = root.join("Cargo.toml");
    let Ok(cargo_toml_text) = std::fs::read_to_string(cargo_toml_path) else {
        return BTreeMap::new();
    };
    let Ok(cargo_toml) = cargo_toml_text.parse::<toml::Value>() else {
        return BTreeMap::new();
    };
    let Some(bin_entries) = cargo_toml.get("bin").and_then(|value| value.as_array()) else {
        return BTreeMap::new();
    };

    let binary_sources = bin_entries.iter().filter_map(|bin_entry| {
        let bin_name = bin_entry.get("name").and_then(|value| value.as_str())?;
        let bin_path = bin_entry.get("path").and_then(|value| value.as_str())?;
        Some((bin_name.to_string(), PathBuf::from(bin_path)))
    });

    collect_binary_entrypoints_for_sources(source_graphs, binary_sources)
}

pub(super) fn collect_toolchain_binary_entrypoints(
    project: &RustToolchainProject,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    collect_binary_entrypoints_for_sources(
        source_graphs,
        project
            .binary_target_sources()
            .into_iter()
            .flat_map(|(bin_name, paths)| {
                paths
                    .into_iter()
                    .map(move |path| (bin_name.clone(), path))
                    .collect::<Vec<_>>()
            }),
    )
}

fn collect_binary_entrypoints_for_sources<I>(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    binary_sources: I,
) -> BTreeMap<String, BTreeSet<String>>
where
    I: IntoIterator<Item = (String, PathBuf)>,
{
    let mut entrypoints: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (bin_name, bin_path) in binary_sources {
        let Some(graph) = find_source_graph_for_path(source_graphs, &bin_path) else {
            continue;
        };
        let main_ids = graph
            .items
            .iter()
            .filter(|item| item.name == "main")
            .map(|item| item.stable_id.clone())
            .collect::<BTreeSet<_>>();
        if !main_ids.is_empty() {
            entrypoints.entry(bin_name).or_default().extend(main_ids);
        }
    }

    entrypoints
}

fn find_source_graph_for_path<'a>(
    source_graphs: &'a BTreeMap<PathBuf, ParsedSourceGraph>,
    target_path: &Path,
) -> Option<&'a ParsedSourceGraph> {
    source_graphs
        .iter()
        .find(|(path, _)| *path == target_path || path.ends_with(target_path))
        .map(|(_, graph)| graph)
}

fn collect_crate_root_aliases(root: &Path) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    let cargo_toml_path = root.join("Cargo.toml");
    let Ok(cargo_toml_text) = std::fs::read_to_string(cargo_toml_path) else {
        return aliases;
    };
    let Ok(cargo_toml) = cargo_toml_text.parse::<toml::Value>() else {
        return aliases;
    };

    if let Some(lib_name) = cargo_toml
        .get("lib")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
    {
        aliases.insert(lib_name.to_string());
    }

    if let Some(package_name) = cargo_toml
        .get("package")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
    {
        aliases.insert(package_name.replace('-', "_"));
    }

    aliases
}

fn collect_imported_call_aliases(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, BTreeMap<String, Vec<String>>> {
    let mut aliases_by_file = BTreeMap::new();

    for path in source_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
    {
        let Ok(text) = crate::modules::analyze::read_owned_file_text(root, path) else {
            continue;
        };
        let Ok(file) = syn::parse_file(&text) else {
            continue;
        };
        let mut aliases = BTreeMap::<String, Vec<String>>::new();
        for item in &file.items {
            let syn::Item::Use(item_use) = item else {
                continue;
            };
            for (alias, targets) in collect_use_aliases(&item_use.tree) {
                aliases.entry(alias).or_default().extend(targets);
            }
        }
        if !aliases.is_empty() {
            aliases_by_file.insert(path.clone(), aliases);
        }
    }

    aliases_by_file
}

fn qualified_name_candidates(
    caller: &SourceCallableItem,
    qualifier: &str,
    call_name: &str,
    crate_root_aliases: &BTreeSet<String>,
) -> Vec<String> {
    let prefixes = qualifier_prefix_candidates(caller, qualifier, crate_root_aliases);
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

fn qualifier_prefix_candidates(
    caller: &SourceCallableItem,
    qualifier: &str,
    crate_root_aliases: &BTreeSet<String>,
) -> Vec<Vec<String>> {
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

    if segments
        .first()
        .is_some_and(|segment| crate_root_aliases.contains(segment))
    {
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
