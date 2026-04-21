/**
@module SPECIAL.MODULES.ANALYZE.RUST.TRACEABILITY
Builds conservative Rust implementation traceability from analyzable Rust source items through verifying Rust tests to resolved spec lifecycle state without leaking parser-specific details into higher analysis layers. This adapter should refuse to run backward trace unless `rust-analyzer` is available, contribute one combined Rust trace graph from parser and tool-backed edges, and let repo and module projections consume that shared graph instead of redefining separate walks or over-claiming negative proofs.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.TRACEABILITY
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleItemKind, ParsedArchitecture, ParsedRepo,
};
use crate::syntax::{
    ParsedSourceGraph, SourceCall, SourceInvocation, parse_source_graph, rust::file_module_segments,
};

use super::super::{
    FileOwnership, read_owned_file_text,
    traceability_core::{
        BackwardTraceAvailability, TraceGraph, TraceabilityAnalysis, TraceabilityInputs,
        TraceabilityLanguagePack, TraceabilityOwnedItem, build_root_supports,
        owned_module_ids_for_path,
        summarize_repo_traceability as summarize_shared_repo_traceability,
    },
};
use super::rust_analyzer::RustAnalyzerCallableItem;
use super::semantic::{RustSemanticFactSourceKind, selected_semantic_fact_source};
use super::toolchain::RustToolchainProject;
use super::use_tree::collect_use_aliases;

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

    fn build_inputs(
        &self,
        root: &Path,
        source_files: &[PathBuf],
        parsed_repo: &ParsedRepo,
        _parsed_architecture: &ParsedArchitecture,
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> TraceabilityInputs {
        let source_graphs = parse_rust_source_graphs(root, source_files);
        let graph = build_rust_trace_graph(
            root,
            source_files,
            &source_graphs,
            self.toolchain_project.as_ref(),
            self.semantic_fact_source,
        );
        build_traceability_inputs(
            root,
            source_files,
            parsed_repo,
            &source_graphs,
            file_ownership,
            graph,
        )
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

#[derive(Debug, Clone)]
struct SourceCallableItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    path: PathBuf,
    span: crate::syntax::SourceSpan,
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

fn build_rust_trace_graph(
    root: &Path,
    source_files: &[PathBuf],
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    toolchain_project: Option<&RustToolchainProject>,
    semantic_fact_source: Option<RustSemanticFactSourceKind>,
) -> TraceGraph {
    let cargo_binary_entrypoints = toolchain_project
        .map(|project| collect_toolchain_binary_entrypoints(project, source_graphs))
        .unwrap_or_else(|| collect_cargo_binary_entrypoints(root, source_graphs));
    let crate_root_aliases = toolchain_project
        .map(RustToolchainProject::crate_root_aliases)
        .unwrap_or_else(|| collect_crate_root_aliases(root));

    let parser_edges = build_parser_call_edges(
        root,
        source_files,
        source_graphs,
        cargo_binary_entrypoints.clone(),
        crate_root_aliases,
    );

    let edges = if matches!(
        semantic_fact_source,
        Some(RustSemanticFactSourceKind::RustAnalyzer)
    ) {
        build_rust_analyzer_call_edges(root, source_graphs, cargo_binary_entrypoints, &parser_edges)
            .unwrap_or_else(|error| {
                eprintln!("rust-analyzer backward trace failed: {error:#}");
                parser_edges
            })
    } else {
        parser_edges
    };

    TraceGraph {
        edges,
        root_supports: BTreeMap::new(),
    }
}

fn build_traceability_inputs(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    mut graph: TraceGraph,
) -> TraceabilityInputs {
    let mediated_reasons = collect_mediated_reasons(root, source_files, source_graphs);
    let repo_items = collect_repo_items(source_graphs, file_ownership, &mediated_reasons);

    graph.root_supports = build_root_supports(parsed_repo, source_graphs, |path, body| {
        parse_source_graph(path, body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    TraceabilityInputs { repo_items, graph }
}

pub(super) fn summarize_repo_traceability(
    root: &Path,
    analysis: &TraceabilityAnalysis,
) -> ArchitectureTraceabilitySummary {
    summarize_shared_repo_traceability(root, analysis)
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

fn collect_repo_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    mediated_reasons: &BTreeMap<String, RustMediatedReason>,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            let module_ids = owned_module_ids_for_path(file_ownership, path);
            let test_file = is_test_file_path(path);
            graph
                .items
                .iter()
                .filter(|item| !item.is_test)
                .map(move |item| TraceabilityOwnedItem {
                    stable_id: item.stable_id.clone(),
                    name: item.name.clone(),
                    kind: source_item_kind(item.kind),
                    path: path.clone(),
                    public: item.public,
                    review_surface: is_review_surface(item, test_file),
                    test_file,
                    module_ids: module_ids.clone(),
                    mediated_reason: mediated_reasons
                        .get(&item.stable_id)
                        .map(|reason| reason.as_str()),
                })
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    items
}

fn is_test_file_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
        || path.file_stem().and_then(|stem| stem.to_str()) == Some("tests")
}

fn is_process_entrypoint_name(name: &str, kind: crate::syntax::SourceItemKind) -> bool {
    kind == crate::syntax::SourceItemKind::Function && name == "main"
}

fn is_review_surface(item: &crate::syntax::SourceItem, test_file: bool) -> bool {
    !test_file && (item.public || is_process_entrypoint_name(&item.name, item.kind))
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

fn build_rust_analyzer_call_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    cargo_binary_entrypoints: BTreeMap<String, BTreeSet<String>>,
    parser_call_edges: &BTreeMap<String, BTreeSet<String>>,
) -> anyhow::Result<BTreeMap<String, BTreeSet<String>>> {
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
    super::rust_analyzer::build_reachable_call_edges(root, &items, &seed_ids, parser_call_edges)
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

fn collect_cargo_binary_entrypoints(
    _root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    let cargo_toml_path = _root.join("Cargo.toml");
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

fn collect_toolchain_binary_entrypoints(
    project: &RustToolchainProject,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    collect_binary_entrypoints_for_sources(source_graphs, project.binary_target_sources())
}

fn collect_binary_entrypoints_for_sources<I>(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    binary_sources: I,
) -> BTreeMap<String, BTreeSet<String>>
where
    I: IntoIterator<Item = (String, PathBuf)>,
{
    let mut entrypoints = BTreeMap::new();

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
            entrypoints.insert(bin_name, main_ids);
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
        let Ok(text) = read_owned_file_text(root, path) else {
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
                        &[type_name.clone()],
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

fn source_item_kind(kind: crate::syntax::SourceItemKind) -> ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => ModuleItemKind::Method,
    }
}
