/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.SCOPE
Owns Go scoped traceability scope-facts, exact item-kernel narrowing, and the execution-level file loading rebuilt from cached facts. The kept semantic kernel now follows the shared projected-contract shape before the later file-level execution projection.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.SCOPE
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::ParsedRepo;
use crate::modules::analyze::traceability_core::{
    TraceGraph, TraceabilityAnalysis, TraceabilityInputs, build_root_supports,
    merge_trace_graph_edges,
};
use crate::modules::analyze::{FileOwnership, emit_analysis_status};
use crate::syntax::{ParsedSourceGraph, parse_source_graph};

use super::boundary::derive_scoped_traceability_boundary;
use super::dependencies::{collect_go_import_aliases, resolve_internal_imports};
use super::traceability::{
    CachedParsedSourceGraph, CachedTraceabilityItemSupport, GoTraceabilityScopeFacts,
    build_gopls_reference_edges, build_reverse_reachable_reference_edges, build_static_call_edges,
    collect_callable_items, collect_repo_items, decode_traceability_graph_facts,
    parse_go_source_graphs,
};

pub(super) fn build_traceability_scope_facts(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
) -> Result<Vec<u8>> {
    let source_graphs = parse_go_source_graphs(root, source_files);
    let static_edges = build_static_call_edges(root, &source_graphs);
    let tool_reference_edges = build_gopls_reference_edges(root, &collect_callable_items(&source_graphs));
    let root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(path, body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let facts = GoTraceabilityScopeFacts {
        source_graphs: source_graphs
            .iter()
            .map(|(path, graph)| (path.clone(), CachedParsedSourceGraph::from_parsed(graph)))
            .collect(),
        file_adjacency: build_internal_import_adjacency(root, source_graphs.keys().cloned()),
        static_edges,
        tool_reference_edges,
        root_supports: root_supports
            .into_iter()
            .map(|(stable_id, support)| {
                (stable_id, CachedTraceabilityItemSupport::from_runtime(support))
            })
            .collect(),
    };
    Ok(serde_json::to_vec(&facts)?)
}

pub(super) fn expand_traceability_closure_from_facts(
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    facts: &[u8],
) -> Result<Vec<PathBuf>> {
    if scoped_source_files.is_empty() {
        return Ok(source_files.to_vec());
    }
    let facts: GoTraceabilityScopeFacts = serde_json::from_slice(facts)?;
    let source_graphs = facts
        .source_graphs
        .iter()
        .map(|(path, graph)| (path.clone(), graph.clone().into_parsed()))
        .collect::<BTreeMap<_, _>>();
    let known_source_paths = source_graphs.keys().cloned().collect::<Vec<_>>();
    let normalized_source_to_original = source_files
        .iter()
        .map(|path| {
            (
                normalize_go_path_for_known_sources(path, &known_source_paths),
                path.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let normalized_scoped_source_files = scoped_source_files
        .iter()
        .map(|path| normalize_go_path_for_known_sources(path, &known_source_paths))
        .collect::<Vec<_>>();
    let repo_items = collect_repo_items(&source_graphs, file_ownership);
    let boundary =
        derive_scoped_traceability_boundary(repo_items.clone(), &normalized_scoped_source_files);
    let working_files = boundary
        .context_items
        .iter()
        .map(|item| item.path.clone())
        .collect::<BTreeSet<_>>();
    let mut graph = TraceGraph {
        edges: facts.static_edges,
        root_supports: facts
            .root_supports
            .into_iter()
            .map(|(stable_id, support)| (stable_id, support.into_runtime()))
            .collect(),
    };
    merge_trace_graph_edges(&mut graph.edges, facts.tool_reference_edges);
    let reference = boundary.reference(&graph);
    let preserved_item_ids = reference
        .contract
        .projected_item_ids
        .iter()
        .cloned()
        .chain(reference.exact_reverse_closure.node_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let item_paths = collect_item_paths_by_stable_id(&source_graphs);
    let kept_files = expand_execution_file_closure(
        collect_execution_closure_files(
            &normalized_scoped_source_files,
            &preserved_item_ids,
            &graph,
            &item_paths,
        ),
        &facts.file_adjacency,
    );
    let closure_files = normalized_source_to_original
        .iter()
        .filter(|(path, _)| kept_files.contains(*path))
        .map(|(_, original)| original.clone())
        .collect::<Vec<_>>();
    emit_analysis_status(&format!(
        "go scoped exact traceability closure covers {} of {} file(s) (working closure {})",
        closure_files.len(),
        source_files.len(),
        working_files.len()
    ));
    Ok(closure_files)
}

pub(super) fn build_scoped_traceability_analysis_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityAnalysis> {
    Ok(crate::modules::analyze::traceability_core::build_traceability_analysis(
        build_scoped_traceability_inputs_from_cached_or_live_graph_facts(
            root,
            source_files,
            scoped_source_files,
            graph_facts,
            parsed_repo,
            file_ownership,
        )?,
    ))
}

pub(super) fn build_scoped_traceability_inputs_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityInputs> {
    let (source_graphs, static_edges) = match decode_traceability_graph_facts(graph_facts) {
        Ok(Some(decoded)) => decoded,
        Ok(None) | Err(_) => {
            let source_graphs = parse_go_source_graphs(root, source_files);
            let static_edges = build_static_call_edges(root, &source_graphs);
            (source_graphs, static_edges)
        }
    };
    let normalized_scoped_source_files = scoped_source_files
        .iter()
        .map(|path| normalize_go_path(root, path))
        .collect::<Vec<_>>();
    let boundary = derive_scoped_traceability_boundary(
        collect_repo_items(&source_graphs, file_ownership),
        &normalized_scoped_source_files,
    );
    let scoped_seed_ids = boundary.seed_ids.clone();
    emit_analysis_status(&format!(
        "go scoped traceability targets {} projected item(s), walks reverse callers from {} seed(s), across {} file(s)",
        boundary.projected_item_ids.len(),
        scoped_seed_ids.len(),
        boundary
            .context_items
            .iter()
            .map(|item| item.path.clone())
            .collect::<BTreeSet<_>>()
            .len()
    ));

    let mut edges = static_edges;
    let reverse_seed_edges = edges.clone();
    merge_trace_graph_edges(
        &mut edges,
        build_reverse_reachable_reference_edges(
            root,
            &collect_callable_items(&source_graphs),
            &scoped_seed_ids,
            &reverse_seed_edges,
        ),
    );
    let mut graph = TraceGraph {
        edges,
        root_supports: BTreeMap::new(),
    };
    graph.root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(&normalize_go_path(root, path), body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let retained_context_items = boundary.context_items.clone();

    Ok(narrow_scoped_traceability_inputs_for_go(
        boundary,
        TraceabilityInputs {
            repo_items: retained_context_items.clone(),
            context_items: retained_context_items,
            graph,
        },
    ))
}

pub(super) fn normalize_go_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

pub(super) fn normalize_go_path_for_known_sources(
    path: &Path,
    known_source_paths: &[PathBuf],
) -> PathBuf {
    crate::modules::analyze::traceability_core::normalize_path_for_known_sources(
        path,
        known_source_paths,
    )
}

fn narrow_scoped_traceability_inputs_for_go(
    boundary: super::boundary::ScopedTraceabilityBoundary,
    inputs: TraceabilityInputs,
) -> TraceabilityInputs {
    let reference = boundary.reference(&inputs.graph);
    let projected_item_ids = &reference.contract.projected_item_ids;
    let preserved_item_ids =
        crate::modules::analyze::traceability_core::preserved_graph_item_ids_for_reference(
            &reference,
        );
    let repo_items = inputs
        .repo_items
        .into_iter()
        .filter(|item| projected_item_ids.contains(&item.stable_id))
        .collect::<Vec<_>>();
    let context_items = inputs
        .context_items
        .into_iter()
        .filter(|item| preserved_item_ids.contains(&item.stable_id))
        .collect::<Vec<_>>();
    let graph = TraceGraph {
        edges: inputs
            .graph
            .edges
            .into_iter()
            .filter(|(caller, _)| preserved_item_ids.contains(caller))
            .map(|(caller, callees)| {
                (
                    caller,
                    callees
                        .into_iter()
                        .filter(|callee| preserved_item_ids.contains(callee))
                        .collect(),
                )
            })
            .collect(),
        root_supports: inputs
            .graph
            .root_supports
            .into_iter()
            .filter(|(item_id, _)| preserved_item_ids.contains(item_id))
            .collect(),
    };

    TraceabilityInputs {
        repo_items,
        context_items,
        graph,
    }
}

fn collect_item_paths_by_stable_id(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, PathBuf> {
    source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            graph.items
                .iter()
                .map(move |item| (item.stable_id.clone(), path.clone()))
        })
        .collect()
}

fn collect_execution_closure_files(
    scoped_source_files: &[PathBuf],
    preserved_item_ids: &BTreeSet<String>,
    graph: &TraceGraph,
    item_paths_by_id: &BTreeMap<String, PathBuf>,
) -> BTreeSet<PathBuf> {
    let direct_callee_ids = preserved_item_ids
        .iter()
        .flat_map(|item_id| graph.edges.get(item_id).into_iter().flatten().cloned())
        .collect::<BTreeSet<_>>();

    scoped_source_files
        .iter()
        .cloned()
        .chain(
            preserved_item_ids
                .iter()
                .chain(direct_callee_ids.iter())
                .filter_map(|item_id| item_paths_by_id.get(item_id).cloned()),
        )
        .collect()
}

fn build_internal_import_adjacency(
    root: &Path,
    source_files: impl IntoIterator<Item = PathBuf>,
) -> BTreeMap<PathBuf, BTreeSet<PathBuf>> {
    let mut adjacency = BTreeMap::new();
    for path in source_files {
        let Ok(text) = crate::modules::analyze::read_owned_file_text(root, &path) else {
            continue;
        };
        let imported_files = collect_go_import_aliases(&text)
            .into_values()
            .flat_map(|import_path| resolve_internal_imports(root, &import_path))
            .map(|resolved| {
                resolved
                    .strip_prefix(root)
                    .unwrap_or(&resolved)
                    .to_path_buf()
            })
            .collect::<BTreeSet<_>>();
        adjacency.insert(path, imported_files);
    }
    adjacency
}

fn expand_execution_file_closure(
    initial_files: BTreeSet<PathBuf>,
    adjacency: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
) -> BTreeSet<PathBuf> {
    let mut closure = initial_files.clone();
    let mut pending = initial_files.into_iter().collect::<Vec<_>>();
    while let Some(path) = pending.pop() {
        let Some(imported) = adjacency.get(&path) else {
            continue;
        };
        for imported_path in imported {
            if closure.insert(imported_path.clone()) {
                pending.push(imported_path.clone());
            }
        }
    }
    closure
}
