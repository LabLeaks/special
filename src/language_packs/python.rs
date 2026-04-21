/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON
Registers the built-in Python language pack with the shared compile-time pack registry. This pack lives under `src/language_packs/python/`, uses Python's own `ast` as its parser baseline, and should only surface backward trace when the local `pyright-langserver` tool is available so Python traceability stays on the same union-graph contract as the other built-in packs.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{LanguagePackAnalysisContext, LanguagePackDescriptor};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ModuleItemKind,
    ModuleMetricsSummary, ParsedArchitecture, ParsedRepo,
};
use crate::modules::analyze::{
    FileOwnership, ProviderModuleAnalysis, read_owned_file_text,
    source_item_signals::summarize_source_item_signals,
    traceability_core::{
        BackwardTraceAvailability, TraceGraph, TraceabilityAnalysis, TraceabilityInputs,
        TraceabilityLanguagePack, TraceabilityOwnedItem, build_root_supports,
        build_traceability_analysis, merge_trace_graph_edges, owned_module_ids_for_path,
        summarize_module_traceability,
        summarize_repo_traceability as summarize_shared_repo_traceability,
    },
    visit_owned_texts,
};
use crate::syntax::{ParsedSourceGraph, SourceCall, SourceLanguage};

#[path = "python/ast_bridge.rs"]
mod ast_bridge;
#[path = "python/pyright_langserver.rs"]
mod pyright_langserver;

pub(crate) const DESCRIPTOR: LanguagePackDescriptor = LanguagePackDescriptor {
    language: SourceLanguage::new("python"),
    matches_path: is_python_path,
    parse_source_graph: parse_source_graph,
    build_repo_analysis_context: build_repo_analysis_context,
    analysis_environment_fingerprint: analysis_environment_fingerprint,
};

pub(crate) struct PythonRepoAnalysisContext {
    traceability_pack: PythonTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    traceability_unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct PythonTraceabilityPack;

impl LanguagePackAnalysisContext for PythonRepoAnalysisContext {
    fn summarize_repo_traceability(
        &self,
        root: &Path,
    ) -> Option<ArchitectureTraceabilitySummary> {
        self.traceability
            .as_ref()
            .map(|analysis| summarize_shared_repo_traceability(root, analysis))
    }

    fn traceability_unavailable_reason(&self) -> Option<String> {
        self.traceability_unavailable_reason.clone()
    }

    fn analyze_module(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
        options: ModuleAnalysisOptions,
    ) -> Result<ProviderModuleAnalysis> {
        analyze_module(
            root,
            implementations,
            file_ownership,
            self,
            options.traceability,
        )
    }
}

impl TraceabilityLanguagePack for PythonTraceabilityPack {
    fn backward_trace_availability(&self) -> BackwardTraceAvailability {
        if pyright_langserver_available() {
            BackwardTraceAvailability::default()
        } else {
            BackwardTraceAvailability::unavailable(
                "Python backward trace is unavailable because `pyright-langserver` is not installed",
            )
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
        let source_graphs = parse_python_source_graphs(root, source_files);
        let repo_items = collect_repo_items(&source_graphs, file_ownership);
        let mut graph = TraceGraph {
            edges: build_parser_call_edges(&source_graphs),
            root_supports: BTreeMap::new(),
        };
        merge_trace_graph_edges(
            &mut graph.edges,
            build_tool_call_edges(root, &source_graphs).unwrap_or_default(),
        );
        graph.root_supports =
            build_root_supports(parsed_repo, &source_graphs, |path, body| {
                parse_source_graph(path, body).and_then(|graph| {
                    graph.items.first().map(|item| item.span.start_line)
                })
            });
        TraceabilityInputs { repo_items, graph }
    }

    fn owned_items_for_implementations(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> Vec<TraceabilityOwnedItem> {
        collect_owned_items(root, implementations, file_ownership)
    }
}

pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> Box<dyn LanguagePackAnalysisContext> {
    let traceability_pack = PythonTraceabilityPack;
    let traceability_unavailable_reason = traceability_pack
        .backward_trace_availability()
        .unavailable_reason()
        .map(ToString::to_string);
    let (traceability, traceability_unavailable_reason) =
        if include_traceability && traceability_unavailable_reason.is_none() {
            match build_traceability_analysis_for_python(
                root,
                source_files,
                parsed_repo,
                parsed_architecture,
                file_ownership,
            ) {
                Ok(analysis) => (Some(analysis), None),
                Err(error) => (
                    None,
                    Some(format!(
                        "Python backward trace is unavailable because syntax or tool-backed trace setup failed: {error}"
                    )),
                ),
            }
        } else {
            (None, traceability_unavailable_reason)
        };
    Box::new(PythonRepoAnalysisContext {
        traceability_pack,
        traceability,
        traceability_unavailable_reason,
    })
}

fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &PythonRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut owned_items = Vec::new();
    let mut public_items = 0;
    let mut internal_items = 0;

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_python_path(path) {
            return Ok(());
        }
        if let Some(graph) = parse_source_graph(path, text) {
            for item in &graph.items {
                if item.public {
                    public_items += 1;
                } else {
                    internal_items += 1;
                }
            }
            owned_items.extend(graph.items);
        }
        Ok(())
    })?;

    let traceability_summary = include_traceability
        .then_some(context.traceability.as_ref())
        .flatten()
        .map(|analysis| {
            let owned_items = context.traceability_pack.owned_items_for_implementations(
                root,
                implementations,
                file_ownership,
            );
            summarize_module_traceability(&owned_items, analysis)
        });

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items,
            internal_items,
            ..ModuleMetricsSummary::default()
        },
        item_signals: Some(summarize_source_item_signals(&owned_items)),
        traceability: traceability_summary,
        traceability_unavailable_reason: include_traceability
            .then(|| context.traceability_unavailable_reason.clone())
            .flatten(),
        ..ProviderModuleAnalysis::default()
    })
}

fn is_python_path(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("py")
}

fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    ast_bridge::parse_source_graph(path, text)
}

fn pyright_langserver_available() -> bool {
    pyright_langserver::available()
}

fn analysis_environment_fingerprint(_root: &Path) -> String {
    pyright_langserver::environment_fingerprint()
}

#[derive(Debug, Clone, Default)]
struct CallableIndexes {
    global_name_counts: BTreeMap<String, usize>,
    same_file_name_counts: BTreeMap<(PathBuf, String), usize>,
    global_qualified_name_counts: BTreeMap<String, usize>,
}

fn parse_python_source_graphs(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, ParsedSourceGraph> {
    source_files
        .iter()
        .filter(|path| is_python_path(path))
        .filter_map(|path| {
            let text = read_owned_file_text(root, path).ok()?;
            parse_source_graph(path, &text).map(|graph| (path.clone(), graph))
        })
        .collect()
}

fn parse_python_source_graphs_result(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<BTreeMap<PathBuf, ParsedSourceGraph>> {
    let mut graphs = BTreeMap::new();
    for path in source_files.iter().filter(|path| is_python_path(path)) {
        let text = read_owned_file_text(root, path)?;
        let graph = ast_bridge::parse_source_graph_result(path, &text)?;
        graphs.insert(path.clone(), graph);
    }
    Ok(graphs)
}

fn collect_repo_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
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
                    review_surface: item.public && !test_file,
                    test_file,
                    module_ids: module_ids.clone(),
                    mediated_reason: None,
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

fn collect_owned_items(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = Vec::new();
    let mut seen = BTreeSet::new();
    for implementation in implementations {
        if !is_python_path(&implementation.location.path) {
            continue;
        }
        let Some(graph) = parse_owned_implementation_graph(root, implementation, file_ownership)
        else {
            continue;
        };
        let test_file = is_test_file_path(&implementation.location.path);
        for item in graph.items.into_iter().filter(|item| !item.is_test) {
            if !seen.insert(item.stable_id.clone()) {
                continue;
            }
            items.push(TraceabilityOwnedItem {
                stable_id: item.stable_id,
                name: item.name,
                kind: source_item_kind(item.kind),
                path: implementation.location.path.clone(),
                public: item.public,
                review_surface: item.public && !test_file,
                test_file,
                module_ids: vec![implementation.module_id.clone()],
                mediated_reason: None,
            });
        }
    }
    items
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

fn build_parser_call_edges(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    let callable_items = collect_callable_items(source_graphs);
    let indexes = build_callable_indexes(&callable_items);
    let mut edges = BTreeMap::new();

    for item in &callable_items {
        let callees = item
            .calls
            .iter()
            .flat_map(|call| resolve_call_targets(call, item, &callable_items, &indexes))
            .collect::<BTreeSet<_>>();
        if !callees.is_empty() {
            edges.insert(item.stable_id.clone(), callees);
        }
    }

    edges
}

fn build_tool_call_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let callable_items = collect_callable_items(source_graphs);
    if callable_items.is_empty() {
        return Ok(BTreeMap::new());
    }
    pyright_langserver::build_reachable_call_edges(root, &callable_items)
}

fn build_traceability_analysis_for_python(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityAnalysis> {
    let source_graphs = parse_python_source_graphs_result(root, source_files)?;
    let repo_items = collect_repo_items(&source_graphs, file_ownership);
    let mut graph = TraceGraph {
        edges: build_parser_call_edges(&source_graphs),
        root_supports: BTreeMap::new(),
    };
    merge_trace_graph_edges(
        &mut graph.edges,
        build_tool_call_edges(root, &source_graphs)?,
    );
    graph.root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(path, body).and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let _ = parsed_architecture;
    Ok(build_traceability_analysis(TraceabilityInputs { repo_items, graph }))
}

fn collect_callable_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Vec<pyright_langserver::PyrightCallableItem> {
    let mut items = source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            graph.items.iter().map(move |item| pyright_langserver::PyrightCallableItem {
                stable_id: item.stable_id.clone(),
                name: item.name.clone(),
                qualified_name: item.qualified_name.clone(),
                path: path.clone(),
                span: item.span,
                calls: item.calls.clone(),
                is_test: item.is_test,
            })
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.span.start_line.cmp(&right.span.start_line))
            .then_with(|| left.name.cmp(&right.name))
    });
    items
}

fn build_callable_indexes(items: &[pyright_langserver::PyrightCallableItem]) -> CallableIndexes {
    let mut indexes = CallableIndexes::default();
    for item in items {
        *indexes.global_name_counts.entry(item.name.clone()).or_default() += 1;
        *indexes
            .same_file_name_counts
            .entry((item.path.clone(), item.name.clone()))
            .or_default() += 1;
        *indexes
            .global_qualified_name_counts
            .entry(item.qualified_name.clone())
            .or_default() += 1;
    }
    indexes
}

fn resolve_call_targets(
    call: &SourceCall,
    caller: &pyright_langserver::PyrightCallableItem,
    items: &[pyright_langserver::PyrightCallableItem],
    indexes: &CallableIndexes,
) -> Vec<String> {
    if let Some(target) = resolve_exact_qualified_target(call, items, indexes) {
        return vec![target];
    }

    let same_file_matches = items
        .iter()
        .filter(|item| {
            item.path == caller.path
                && item.name == call.name
                && item.stable_id != caller.stable_id
                && indexes
                    .same_file_name_counts
                    .get(&(item.path.clone(), item.name.clone()))
                    .copied()
                    == Some(1)
        })
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();
    if same_file_matches.len() == 1 {
        return same_file_matches;
    }

    let global_matches = items
        .iter()
        .filter(|item| {
            item.name == call.name
                && item.stable_id != caller.stable_id
                && indexes.global_name_counts.get(&item.name).copied() == Some(1)
        })
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();
    if global_matches.len() == 1 {
        return global_matches;
    }

    Vec::new()
}

fn resolve_exact_qualified_target(
    call: &SourceCall,
    items: &[pyright_langserver::PyrightCallableItem],
    indexes: &CallableIndexes,
) -> Option<String> {
    let qualifier = call.qualifier.as_deref()?;
    let qualified = normalize_qualified_target(qualifier, &call.name);
    if indexes.global_qualified_name_counts.get(&qualified).copied() != Some(1) {
        return None;
    }
    items.iter()
        .find(|item| item.qualified_name == qualified)
        .map(|item| item.stable_id.clone())
}

fn normalize_qualified_target(qualifier: &str, name: &str) -> String {
    qualifier
        .split('.')
        .chain(std::iter::once(name))
        .collect::<Vec<_>>()
        .join("::")
}

fn is_test_file_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("test_") || name.ends_with("_test.py"))
}

fn source_item_kind(kind: crate::syntax::SourceItemKind) -> ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => ModuleItemKind::Method,
    }
}
