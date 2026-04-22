/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY
Builds conservative Rust implementation traceability from analyzable Rust source items through verifying Rust tests to resolved spec lifecycle state without leaking parser-specific details into higher analysis layers. This adapter should refuse to run backward trace unless `rust-analyzer` is available, contribute one combined Rust trace graph from parser and tool-backed edges, and let repo and module projections consume that shared graph instead of redefining separate walks or over-claiming negative proofs.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::model::{ArchitectureTraceabilitySummary, ImplementRef, ParsedRepo};
use crate::syntax::{ParsedSourceGraph, parse_source_graph, rust::file_module_segments};

use crate::modules::analyze::{
    FileOwnership, read_owned_file_text,
    traceability_core::{
        BackwardTraceAvailability, TraceGraph, TraceabilityAnalysis, TraceabilityInputs,
        TraceabilityLanguagePack, TraceabilityOwnedItem, build_root_supports,
        merge_trace_graph_edges, preserved_graph_item_ids_for_reference,
        preserved_item_ids_for_reference,
        summarize_repo_traceability as summarize_shared_repo_traceability,
    },
};
use super::semantic::{RustSemanticFactSourceKind, selected_semantic_fact_source};
use super::toolchain::RustToolchainProject;

#[path = "traceability/boundary.rs"]
mod boundary;
#[path = "traceability/call_graph.rs"]
mod call_graph;
#[path = "traceability/facts.rs"]
mod facts;
#[cfg(test)]
#[path = "traceability/tests.rs"]
mod tests;

use boundary::{
    collect_repo_items, derive_scoped_traceability_boundary, is_review_surface, is_test_file_path,
    source_item_kind,
};
#[cfg(test)]
use boundary::ScopedTraceabilityBoundary;
use call_graph::{
    build_parser_call_edges_with_toolchain, build_rust_analyzer_call_edges,
    collect_cargo_binary_entrypoints, collect_rust_analyzer_reference_items,
    collect_toolchain_binary_entrypoints,
};
use facts::{
    CachedRustMediatedReason, CachedParsedSourceGraph, CachedTraceabilityItemSupport,
    RustTraceabilityGraphFacts, RustTraceabilityScopeFacts,
    decode_traceability_graph_facts,
};

pub(super) struct RustTraceabilityPack {
    toolchain_project: Option<RustToolchainProject>,
    semantic_fact_source: Option<RustSemanticFactSourceKind>,
}

impl RustTraceabilityPack {
    pub(super) fn new(toolchain_project: Option<RustToolchainProject>) -> Self {
        let semantic_fact_source = selected_semantic_fact_source(toolchain_project.as_ref());
        Self {
            toolchain_project,
            semantic_fact_source,
        }
    }
}

impl TraceabilityLanguagePack for RustTraceabilityPack {
    fn backward_trace_availability(&self) -> BackwardTraceAvailability {
        match self.semantic_fact_source {
            None => BackwardTraceAvailability::unavailable(
                "Rust backward trace is unavailable because `rust-analyzer` is not installed",
            ),
            Some(_) => BackwardTraceAvailability::default(),
        }
    }

    fn owned_items_for_implementations(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> Vec<TraceabilityOwnedItem> {
        collect_owned_items(root, implementations, file_ownership, true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustMediatedReason {
    TraitImplEntrypoint,
}

impl RustMediatedReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::TraitImplEntrypoint => "trait impl entrypoint",
        }
    }
}

fn build_traceability_inputs_from_parts(
    parsed_repo: &ParsedRepo,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    mut graph: TraceGraph,
    mediated_reasons: &BTreeMap<String, RustMediatedReason>,
) -> TraceabilityInputs {
    let repo_items = collect_repo_items(source_graphs, file_ownership, mediated_reasons);

    graph.root_supports = build_root_supports(parsed_repo, source_graphs, |path, body| {
        parse_source_graph(path, body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    TraceabilityInputs {
        repo_items,
        context_items: Vec::new(),
        graph,
    }
}

pub(super) fn summarize_repo_traceability(
    root: &Path,
    analysis: &TraceabilityAnalysis,
) -> ArchitectureTraceabilitySummary {
    summarize_shared_repo_traceability(root, analysis)
}

pub(super) fn build_traceability_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<Vec<u8>> {
    let source_graphs = parse_rust_source_graphs(root, source_files);
    let toolchain_project = super::toolchain::probe_local_toolchain_project(root);
    let parser_edges = build_parser_call_edges_with_toolchain(
        root,
        source_files,
        &source_graphs,
        toolchain_project.as_ref(),
    );
    let facts = RustTraceabilityGraphFacts {
        source_graphs: source_graphs
            .iter()
            .map(|(path, graph)| (path.clone(), CachedParsedSourceGraph::from_parsed(graph)))
            .collect(),
        parser_edges,
        mediated_reasons: collect_mediated_reasons(root, source_files, &source_graphs)
            .into_iter()
            .map(|(stable_id, reason)| (stable_id, CachedRustMediatedReason::from_parsed(reason)))
            .collect(),
    };
    Ok(serde_json::to_vec(&facts)?)
}

pub(super) fn build_traceability_scope_facts(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
) -> Result<Vec<u8>> {
    let toolchain_project = super::toolchain::probe_local_toolchain_project(root);
    let semantic_fact_source = selected_semantic_fact_source(toolchain_project.as_ref());
    if semantic_fact_source.is_none() {
        return Err(anyhow!(
            "Rust backward trace is unavailable because `rust-analyzer` is not installed"
        ));
    }
    let source_graphs = parse_rust_source_graphs(root, source_files);
    let parser_edges = build_parser_call_edges_with_toolchain(
        root,
        source_files,
        &source_graphs,
        toolchain_project.as_ref(),
    );
    let edges = build_full_trace_edges(
        root,
        &source_graphs,
        toolchain_project.as_ref(),
        semantic_fact_source,
        &parser_edges,
    )?;
    let root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(path, body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let facts = RustTraceabilityScopeFacts {
        source_graphs: source_graphs
            .iter()
            .map(|(path, graph)| (path.clone(), CachedParsedSourceGraph::from_parsed(graph)))
            .collect(),
        edges,
        mediated_reasons: collect_mediated_reasons(root, source_files, &source_graphs)
            .into_iter()
            .map(|(stable_id, reason)| (stable_id, CachedRustMediatedReason::from_parsed(reason)))
            .collect(),
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
    let RustTraceabilityScopeFacts {
        source_graphs,
        edges,
        mediated_reasons,
        root_supports,
    } = serde_json::from_slice(facts)?;
    let source_graphs = source_graphs
        .into_iter()
        .map(|(path, graph)| (path, graph.into_parsed()))
        .collect::<BTreeMap<_, _>>();
    let repo_items = collect_repo_items(
        &source_graphs,
        file_ownership,
        &mediated_reasons
            .into_iter()
            .map(|(stable_id, reason)| (stable_id, reason.into_parsed()))
            .collect(),
    );
    let boundary = derive_scoped_traceability_boundary(repo_items, scoped_source_files);
    let working_files = boundary
        .context_items
        .iter()
        .map(|item| item.path.clone())
        .collect::<BTreeSet<_>>();
    let graph = TraceGraph {
        edges,
        root_supports: root_supports
            .into_iter()
            .map(|(stable_id, support)| (stable_id, support.into_runtime()))
            .collect(),
    };
    let reference = boundary.reference(&graph);
    let preserved_graph_item_ids = preserved_graph_item_ids_for_reference(&reference);
    let item_paths = collect_item_paths_by_stable_id(&source_graphs);
    let kept_files = source_files
        .iter()
        .filter(|path| {
            scoped_source_files.contains(path)
                || item_paths
                    .iter()
                    .any(|(stable_id, item_path)| {
                        preserved_graph_item_ids.contains(stable_id) && item_path == *path
                    })
        })
        .cloned()
        .collect::<Vec<_>>();
    crate::modules::analyze::emit_analysis_status(&format!(
        "rust scoped exact traceability closure covers {} of {} file(s) (working closure {})",
        kept_files.len(),
        source_files.len(),
        working_files.len()
    ));
    Ok(kept_files)
}

pub(super) fn build_traceability_analysis_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
    graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    traceability_pack: &RustTraceabilityPack,
) -> Result<TraceabilityAnalysis> {
    Ok(crate::modules::analyze::traceability_core::build_traceability_analysis(
        build_traceability_inputs_from_cached_or_live_graph_facts(
            root,
            source_files,
            graph_facts,
            parsed_repo,
            file_ownership,
            traceability_pack,
        )?,
    ))
}

fn build_traceability_inputs_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
    graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    traceability_pack: &RustTraceabilityPack,
) -> Result<TraceabilityInputs> {
    let (source_graphs, parser_edges, mediated_reasons) =
        match decode_traceability_graph_facts(graph_facts) {
            Ok(Some(decoded)) => decoded,
            Ok(None) | Err(_) => {
                let source_graphs = parse_rust_source_graphs(root, source_files);
                let parser_edges = build_parser_call_edges_with_toolchain(
                    root,
                    source_files,
                    &source_graphs,
                    traceability_pack.toolchain_project.as_ref(),
                );
                let mediated_reasons = collect_mediated_reasons(root, source_files, &source_graphs);
                (source_graphs, parser_edges, mediated_reasons)
            }
        };
    let edges = build_full_trace_edges(
        root,
        &source_graphs,
        traceability_pack.toolchain_project.as_ref(),
        traceability_pack.semantic_fact_source,
        &parser_edges,
    )?;
    Ok(build_traceability_inputs_from_parts(
        parsed_repo,
        &source_graphs,
        file_ownership,
        TraceGraph {
            edges,
            root_supports: BTreeMap::new(),
        },
        &mediated_reasons,
    ))
}

pub(super) fn build_scoped_traceability_analysis_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    traceability_pack: &RustTraceabilityPack,
) -> Result<TraceabilityAnalysis> {
    Ok(crate::modules::analyze::traceability_core::build_traceability_analysis(
        build_scoped_traceability_inputs_from_cached_or_live_graph_facts(
            root,
            source_files,
            scoped_source_files,
            graph_facts,
            parsed_repo,
            file_ownership,
            traceability_pack,
        )?,
    ))
}

fn build_scoped_traceability_inputs_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    traceability_pack: &RustTraceabilityPack,
) -> Result<TraceabilityInputs> {
    let (source_graphs, parser_edges, mediated_reasons) =
        match decode_traceability_graph_facts(graph_facts) {
            Ok(Some(decoded)) => decoded,
            Ok(None) | Err(_) => {
                let source_graphs = parse_rust_source_graphs(root, source_files);
                let parser_edges = build_parser_call_edges_with_toolchain(
                    root,
                    source_files,
                    &source_graphs,
                    traceability_pack.toolchain_project.as_ref(),
                );
                let mediated_reasons = collect_mediated_reasons(root, source_files, &source_graphs);
                (source_graphs, parser_edges, mediated_reasons)
            }
        };
    let scoped_boundary = derive_scoped_traceability_boundary(
        collect_repo_items(&source_graphs, file_ownership, &mediated_reasons),
        scoped_source_files,
    );
    let working_contract = scoped_boundary.working_contract();
    crate::modules::analyze::emit_analysis_status(&format!(
        "rust scoped traceability targets {} projected item(s), walks reverse callers from {} seed(s), across {} file(s)",
        working_contract.projected_item_ids.len(),
        working_contract.preserved_reverse_closure_target_ids.len(),
        scoped_boundary
            .context_items
            .iter()
            .map(|item| item.path.clone())
            .collect::<BTreeSet<_>>()
            .len()
    ));
    let mut edges = parser_edges.clone();
    if matches!(
        traceability_pack.semantic_fact_source,
        Some(RustSemanticFactSourceKind::RustAnalyzer)
    ) {
        let semantic_edges = super::rust_analyzer::build_reverse_reachable_call_edges(
            root,
            &collect_rust_analyzer_reference_items(&source_graphs),
            &scoped_boundary.seed_ids,
            &parser_edges,
        )
        .map_err(rust_backward_trace_failure)?;
        merge_trace_graph_edges(&mut edges, semantic_edges);
    }
    let mut graph = TraceGraph {
        edges,
        root_supports: BTreeMap::new(),
    };
    graph.root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(path, body).and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let reference = scoped_boundary.reference(&graph);
    let projected_item_ids = &reference.contract.projected_item_ids;
    let preserved_graph_item_ids = preserved_graph_item_ids_for_reference(&reference);
    let owned_item_ids = scoped_boundary
        .context_items
        .iter()
        .map(|item| item.stable_id.clone());
    let preserved_context_item_ids = preserved_item_ids_for_reference(&reference, owned_item_ids);
    let repo_items = scoped_boundary
        .context_items
        .iter()
        .filter(|item| projected_item_ids.contains(&item.stable_id))
        .cloned()
        .collect::<Vec<_>>();
    let context_items = scoped_boundary
        .context_items
        .into_iter()
        .filter(|item| preserved_context_item_ids.contains(&item.stable_id))
        .collect::<Vec<_>>();
    let graph = TraceGraph {
        edges: graph
            .edges
            .into_iter()
            .filter(|(caller, _)| preserved_graph_item_ids.contains(caller))
            .map(|(caller, callees)| {
                (
                    caller,
                    callees
                        .into_iter()
                        .filter(|callee| preserved_graph_item_ids.contains(callee))
                        .collect(),
                )
            })
            .collect(),
        root_supports: graph
            .root_supports
            .into_iter()
            .filter(|(item_id, _)| preserved_graph_item_ids.contains(item_id))
            .collect(),
    };
    Ok(TraceabilityInputs {
        repo_items,
        context_items,
        graph,
    })
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

fn build_full_trace_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    toolchain_project: Option<&RustToolchainProject>,
    semantic_fact_source: Option<RustSemanticFactSourceKind>,
    parser_edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let cargo_binary_entrypoints = toolchain_project
        .map(|project| collect_toolchain_binary_entrypoints(project, source_graphs))
        .unwrap_or_else(|| collect_cargo_binary_entrypoints(root, source_graphs));

    if matches!(
        semantic_fact_source,
        Some(RustSemanticFactSourceKind::RustAnalyzer)
    ) {
        build_rust_analyzer_call_edges(root, source_graphs, cargo_binary_entrypoints, parser_edges)
            .map_err(rust_backward_trace_failure)
    } else {
        Ok(parser_edges.clone())
    }
}

fn rust_backward_trace_failure(error: anyhow::Error) -> anyhow::Error {
    anyhow!("Rust backward trace is unavailable because `rust-analyzer` trace collection failed: {error:#}")
}

fn collect_owned_items(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    dedupe: bool,
) -> Vec<TraceabilityOwnedItem> {
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
) -> Vec<TraceabilityOwnedItem> {
    let Some(graph) = parse_owned_implementation_graph(root, implementation, file_ownership) else {
        return Vec::new();
    };
    let source_text = if let Some(body) = &implementation.body {
        body.clone()
    } else {
        let Ok(text) = read_owned_file_text(root, &implementation.location.path) else {
            return Vec::new();
        };
        text
    };
    let mediated_reasons =
        collect_mediated_reasons_in_graph(&implementation.location.path, &source_text, &graph);

    graph
        .items
        .into_iter()
        .filter(|item| !item.is_test)
        .map(|item| {
            let test_file = is_test_file_path(&implementation.location.path);
            TraceabilityOwnedItem {
                review_surface: is_review_surface(&item, test_file),
                mediated_reason: mediated_reasons
                    .get(&item.stable_id)
                    .map(|reason| reason.as_str()),
                stable_id: item.stable_id,
                name: item.name,
                kind: source_item_kind(item.kind),
                path: implementation.location.path.clone(),
                public: item.public,
                test_file,
                module_ids: vec![implementation.module_id.clone()],
            }
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

    let text = read_owned_file_text(root, &implementation.location.path).ok()?;
    parse_source_graph(&implementation.location.path, &text)
}

fn parse_rust_source_graphs(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, ParsedSourceGraph> {
    source_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .filter_map(|path| {
            let text = read_owned_file_text(root, path).ok()?;
            parse_source_graph(path, &text).map(|graph| (path.clone(), graph))
        })
        .collect()
}

fn collect_mediated_reasons(
    root: &Path,
    source_files: &[PathBuf],
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, RustMediatedReason> {
    let mut reasons = BTreeMap::new();

    for path in source_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
    {
        let Some(graph) = source_graphs.get(path) else {
            continue;
        };
        let Ok(text) = read_owned_file_text(root, path) else {
            continue;
        };
        reasons.extend(collect_mediated_reasons_in_graph(path, &text, graph));
    }

    reasons
}

fn collect_mediated_reasons_in_graph(
    path: &Path,
    text: &str,
    graph: &ParsedSourceGraph,
) -> BTreeMap<String, RustMediatedReason> {
    let Ok(file) = syn::parse_file(text) else {
        return BTreeMap::new();
    };
    let trait_methods =
        collect_trait_impl_method_qualified_names(&file.items, &file_module_segments(path));
    if trait_methods.is_empty() {
        return BTreeMap::new();
    }

    let graph_items_by_name = graph
        .items
        .iter()
        .map(|item| (item.qualified_name.as_str(), &item.stable_id))
        .collect::<BTreeMap<_, _>>();
    let mut reasons = BTreeMap::new();
    for qualified_name in trait_methods {
        if let Some(stable_id) = graph_items_by_name.get(qualified_name.as_str()) {
            reasons.insert(
                (*stable_id).clone(),
                RustMediatedReason::TraitImplEntrypoint,
            );
        }
    }
    reasons
}

fn collect_trait_impl_method_qualified_names<'a>(
    items: impl IntoIterator<Item = &'a syn::Item>,
    module_path: &[String],
) -> BTreeSet<String> {
    let mut qualified_names = BTreeSet::new();

    for item in items {
        match item {
            syn::Item::Impl(item_impl) if item_impl.trait_.is_some() => {
                let Some(type_name) = impl_self_type_name(&item_impl.self_ty) else {
                    continue;
                };
                for impl_item in &item_impl.items {
                    let syn::ImplItem::Fn(method) = impl_item else {
                        continue;
                    };
                    qualified_names.insert(build_local_qualified_name(
                        module_path,
                        std::slice::from_ref(&type_name),
                        &method.sig.ident.to_string(),
                    ));
                }
            }
            syn::Item::Mod(item_mod) => {
                let Some((_, nested)) = &item_mod.content else {
                    continue;
                };
                let mut nested_path = module_path.to_vec();
                nested_path.push(item_mod.ident.to_string());
                qualified_names.extend(collect_trait_impl_method_qualified_names(
                    nested.iter(),
                    &nested_path,
                ));
            }
            _ => {}
        }
    }

    qualified_names
}

fn impl_self_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => Some(type_path.path.segments.last()?.ident.to_string()),
        _ => None,
    }
}

fn build_local_qualified_name(
    module_path: &[String],
    container_path: &[String],
    name: &str,
) -> String {
    let mut segments = module_path.to_vec();
    segments.extend(container_path.iter().cloned());
    segments.push(name.to_string());
    segments.join("::")
}
