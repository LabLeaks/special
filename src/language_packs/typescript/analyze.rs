/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE
Owns the built-in TypeScript implementation analysis provider, including pack-specific traceability setup, tool-edge enrichment, and runtime discovery while depending on shared analysis core only through protocolized helpers.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleDependencySummary, ModuleItemKind,
    ModuleMetricsSummary, ParsedArchitecture, ParsedRepo,
};
use crate::syntax::{ParsedSourceGraph, SourceCall, parse_source_graph};
use crate::config::{ProjectToolchain, supported_project_toolchain_contracts};

use crate::modules::analyze::source_item_signals::summarize_source_item_signals;
use crate::modules::analyze::traceability_core::{
    TraceGraph, TraceabilityAnalysis, TraceabilityInputs, TraceabilityLanguagePack,
    TraceabilityItemSupport, TraceabilityOwnedItem, build_root_supports,
    build_traceability_analysis, merge_trace_graph_edges, owned_module_ids_for_path,
    summarize_module_traceability, summarize_repo_traceability as summarize_shared_repo_traceability,
};
use crate::modules::analyze::{
    FileOwnership, ModuleCouplingInput, ProviderModuleAnalysis, build_dependency_summary,
    emit_analysis_status,
    read_owned_file_text, visit_owned_texts,
};

#[path = "analyze/boundary.rs"]
mod boundary;
#[cfg(test)]
#[path = "analyze/tests.rs"]
mod scoped_tests;

use boundary::{derive_projected_traceability_boundary, derive_scoped_traceability_boundary};

pub(crate) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &TypeScriptRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut surface = TypeScriptSurfaceSummary::default();
    let mut owned_items = Vec::new();
    let mut dependencies = TypeScriptDependencySummary::default();
    let traceability_summary = include_traceability
        .then_some(context.traceability.as_ref())
        .flatten()
        .map(|traceability| {
            let owned_items = context.traceability_pack.owned_items_for_implementations(
                root,
                implementations,
                file_ownership,
            );
            summarize_module_traceability(&owned_items, traceability)
        });

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_typescript_path(path) {
            return Ok(());
        }
        if let Some(graph) = parse_source_graph(path, text) {
            surface.observe(&graph.items);
            owned_items.extend(graph.items);
        }
        dependencies.observe(root, path, text);
        Ok(())
    })?;

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items: surface.public_items,
            internal_items: surface.internal_items,
            ..ModuleMetricsSummary::default()
        },
        item_signals: Some(summarize_source_item_signals(&owned_items)),
        traceability: traceability_summary,
        traceability_unavailable_reason: include_traceability
            .then(|| context.traceability_unavailable_reason.clone())
            .flatten(),
        coupling: Some(dependencies.coupling_input()),
        dependencies: Some(dependencies.summary()),
        ..ProviderModuleAnalysis::default()
    })
}

pub(crate) struct TypeScriptRepoAnalysisContext {
    traceability_pack: TypeScriptTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    pub(crate) traceability_unavailable_reason: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    _scoped_source_files: Option<&[PathBuf]>,
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> TypeScriptRepoAnalysisContext {
    let traceability_pack = TypeScriptTraceabilityPack;
    let traceability_unavailable_reason = typescript_backward_trace_unavailable_reason(root);
    let (traceability, traceability_unavailable_reason) = if include_traceability
        && traceability_unavailable_reason.is_none()
    {
        match build_traceability_analysis_for_typescript(
            root,
            source_files,
            _scoped_source_files,
            traceability_graph_facts,
            parsed_repo,
            parsed_architecture,
            file_ownership,
        ) {
            Ok(analysis) => (Some(analysis), None),
            Err(error) => (
                None,
                Some(format!(
                    "TypeScript backward trace is unavailable because tool-edge resolution failed: {error}"
                )),
            ),
        }
    } else {
        (None, traceability_unavailable_reason)
    };
    TypeScriptRepoAnalysisContext {
        traceability_pack,
        traceability,
        traceability_unavailable_reason,
    }
}

pub(crate) fn summarize_repo_traceability(
    root: &Path,
    context: &TypeScriptRepoAnalysisContext,
) -> Option<ArchitectureTraceabilitySummary> {
    context
        .traceability
        .as_ref()
        .map(|traceability| summarize_shared_repo_traceability(root, traceability))
}

#[derive(Debug, Clone, Copy)]
struct TypeScriptTraceabilityPack;

impl TraceabilityLanguagePack for TypeScriptTraceabilityPack {
    fn backward_trace_availability(
        &self,
    ) -> crate::modules::analyze::traceability_core::BackwardTraceAvailability {
        crate::modules::analyze::traceability_core::BackwardTraceAvailability::unavailable(
            "TypeScript backward trace availability must be resolved from the analyzed project root",
        )
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

fn build_traceability_analysis_for_typescript(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityAnalysis> {
    let inputs = build_traceability_inputs_for_typescript(
        root,
        source_files,
        traceability_graph_facts,
        parsed_repo,
        parsed_architecture,
        file_ownership,
    )?;
    Ok(build_traceability_analysis(
        narrow_scoped_traceability_inputs_for_typescript(source_files, scoped_source_files, inputs),
    ))
}

fn build_traceability_inputs_for_typescript(
    root: &Path,
    source_files: &[PathBuf],
    traceability_graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<TraceabilityInputs> {
    let (source_graphs, tool_call_edges) =
        match decode_traceability_graph_facts(traceability_graph_facts) {
            Some(Ok(decoded)) => decoded,
            None | Some(Err(_)) => {
                let source_graphs = parse_typescript_source_graphs(root, source_files);
                let tool_call_edges = build_tool_call_edges(root, &source_graphs)?;
                (source_graphs, tool_call_edges)
            }
        };
    let repo_items = collect_repo_items(&source_graphs, file_ownership);
    let context_items = collect_context_items(&source_graphs, file_ownership);
    let mut graph = TraceGraph {
        edges: build_parser_call_edges(&source_graphs),
        root_supports: BTreeMap::new(),
    };
    merge_trace_graph_edges(&mut graph.edges, tool_call_edges);
    graph.root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(path, body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let _ = parsed_architecture;
    Ok(TraceabilityInputs {
        repo_items,
        context_items,
        graph,
    })
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TypeScriptTraceabilityScopeFacts {
    pub(crate) adjacency: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    source_graphs: BTreeMap<PathBuf, CachedParsedSourceGraph>,
    tool_call_edges: BTreeMap<String, BTreeSet<String>>,
    root_supports: BTreeMap<String, CachedTraceabilityItemSupport>,
}

pub(crate) fn build_traceability_scope_facts(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
) -> Result<Vec<u8>> {
    let source_graphs = parse_typescript_source_graphs(root, source_files);
    let tool_call_edges = build_tool_call_edges(root, &source_graphs)?;
    let root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(path, body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });
    let facts = TypeScriptTraceabilityScopeFacts {
        adjacency: build_tool_module_graph(root, source_files)?,
        source_graphs: source_graphs
            .iter()
            .map(|(path, graph)| (path.clone(), CachedParsedSourceGraph::from_parsed(graph)))
            .collect(),
        tool_call_edges,
        root_supports: root_supports
            .into_iter()
            .map(|(stable_id, support)| (stable_id, CachedTraceabilityItemSupport::from_runtime(support)))
            .collect(),
    };
    Ok(serde_json::to_vec(&facts)?)
}

fn narrow_scoped_traceability_inputs_for_typescript(
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    inputs: TraceabilityInputs,
) -> TraceabilityInputs {
    let Some(scoped_source_files) = scoped_source_files else {
        return inputs;
    };
    if scoped_source_files.is_empty() {
        return inputs;
    }

    let boundary = derive_projected_traceability_boundary(source_files, scoped_source_files);
    let reference = boundary.reference(source_files, &inputs);
    let projected_item_ids = &reference.contract.projected_item_ids;
    let preserved_item_ids = &reference.contract.preserved_item_ids;
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

pub(crate) fn expand_traceability_closure_from_facts(
    source_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    facts: &[u8],
) -> Result<Vec<PathBuf>> {
    if scoped_source_files.is_empty() {
        return Ok(source_files.to_vec());
    }
    let facts: TypeScriptTraceabilityScopeFacts = serde_json::from_slice(facts)?;
    let boundary =
        derive_scoped_traceability_boundary(source_files, scoped_source_files, &facts.adjacency);
    let source_graphs = facts
        .source_graphs
        .iter()
        .map(|(path, graph)| (path.clone(), graph.clone().into_parsed()))
        .collect::<BTreeMap<_, _>>();
    let repo_items = collect_repo_items(&source_graphs, file_ownership);
    let context_items = collect_context_items(&source_graphs, file_ownership);
    let mut graph = TraceGraph {
        edges: build_parser_call_edges(&source_graphs),
        root_supports: facts
            .root_supports
            .into_iter()
            .map(|(stable_id, support)| (stable_id, support.into_runtime()))
            .collect(),
    };
    merge_trace_graph_edges(&mut graph.edges, facts.tool_call_edges);
    let full_inputs = TraceabilityInputs {
        repo_items,
        context_items,
        graph,
    };
    // TypeScript scope facts now carry enough cached graph material to rebuild
    // the exact scoped contract at live narrowing time rather than widening to
    // the broad working file closure.
    let working_contract = boundary.working_contract();
    let exact_contract = boundary.exact_contract(source_files, &full_inputs);
    let closure_files = exact_contract.preserved_file_closure;
    emit_analysis_status(&format!(
        "typescript scoped exact traceability closure covers {} of {} file(s) (working closure {})",
        closure_files.len(),
        source_files.len(),
        working_contract.preserved_file_closure.len()
    ));
    Ok(closure_files)
}

pub(crate) fn build_traceability_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<Vec<u8>> {
    let source_graphs = parse_typescript_source_graphs(root, source_files);
    let facts = TypeScriptTraceabilityGraphFacts {
        source_graphs: source_graphs
            .iter()
            .map(|(path, graph)| (path.clone(), CachedParsedSourceGraph::from_parsed(graph)))
            .collect(),
        tool_call_edges: build_tool_call_edges(root, &source_graphs)?,
    };
    Ok(serde_json::to_vec(&facts)?)
}

fn is_typescript_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx")
    )
}

#[derive(Default)]
struct TypeScriptSurfaceSummary {
    public_items: usize,
    internal_items: usize,
}

impl TypeScriptSurfaceSummary {
    fn observe(&mut self, items: &[crate::syntax::SourceItem]) {
        for item in items {
            if item.public {
                self.public_items += 1;
            } else {
                self.internal_items += 1;
            }
        }
    }
}

#[derive(Default)]
struct TypeScriptDependencySummary {
    targets: BTreeMap<String, usize>,
    internal_files: BTreeSet<PathBuf>,
    external_targets: BTreeSet<String>,
}

impl TypeScriptDependencySummary {
    fn observe(&mut self, root: &Path, source_path: &Path, text: &str) {
        let mut parser = Parser::new();
        if parser
            .set_language(
                &match source_path.extension().and_then(|ext| ext.to_str()) {
                    Some("tsx") => tree_sitter_typescript::LANGUAGE_TSX,
                    _ => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
                }
                .into(),
            )
            .is_err()
        {
            return;
        }
        let Some(tree) = parser.parse(text, None) else {
            return;
        };
        let mut imports = Vec::new();
        collect_import_sources(tree.root_node(), text.as_bytes(), &mut imports);
        for target in imports {
            *self.targets.entry(target.clone()).or_default() += 1;
            if let Some(file) = resolve_internal_import(root, source_path, &target) {
                self.internal_files.insert(file);
            } else if !target.starts_with('.') {
                self.external_targets.insert(target);
            }
        }
    }

    fn summary(&self) -> ModuleDependencySummary {
        build_dependency_summary(&self.targets)
    }

    fn coupling_input(&self) -> ModuleCouplingInput {
        ModuleCouplingInput {
            internal_files: self.internal_files.clone(),
            external_targets: self.external_targets.clone(),
        }
    }
}

fn collect_import_sources(node: Node<'_>, source: &[u8], imports: &mut Vec<String>) {
    if node.kind() == "import_statement"
        && let Some(import_source) = node.child_by_field_name("source")
        && let Ok(text) = import_source.utf8_text(source)
    {
        imports.push(text.trim_matches('"').trim_matches('\'').to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_sources(child, source, imports);
    }
}

fn resolve_internal_import(root: &Path, source_path: &Path, target: &str) -> Option<PathBuf> {
    if !target.starts_with('.') {
        return None;
    }

    let source_dir = normalize_path(
        &root
            .join(source_path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.to_path_buf()),
    );
    let candidate_base = source_dir.join(target);
    let candidates = [
        candidate_base.with_extension("ts"),
        candidate_base.with_extension("tsx"),
        candidate_base.join("index.ts"),
        candidate_base.join("index.tsx"),
    ];

    candidates
        .into_iter()
        .map(|candidate| normalize_path(&candidate))
        .find(|candidate| candidate.exists())
}

fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[derive(Debug, Clone)]
struct SourceCallableItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    path: PathBuf,
    calls: Vec<SourceCall>,
    start_line: usize,
    end_line: usize,
    start_column: usize,
}

#[derive(Debug, Clone, Default)]
struct CallableIndexes {
    global_name_counts: BTreeMap<String, usize>,
    same_file_name_counts: BTreeMap<(PathBuf, String), usize>,
    global_qualified_name_counts: BTreeMap<String, usize>,
}

fn parse_typescript_source_graphs(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, ParsedSourceGraph> {
    source_files
        .iter()
        .filter(|path| is_typescript_path(path))
        .filter_map(|path| {
            let text = read_owned_file_text(root, path).ok()?;
            parse_source_graph(path, &text).map(|graph| (path.clone(), graph))
        })
        .collect()
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
                    review_surface: is_review_surface(
                        item.public,
                        &item.name,
                        item.kind,
                        test_file,
                    ),
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

fn collect_context_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            let module_ids = owned_module_ids_for_path(file_ownership, path);
            let test_file = is_test_file_path(path);
            graph.items.iter().map(move |item| TraceabilityOwnedItem {
                stable_id: item.stable_id.clone(),
                name: item.name.clone(),
                kind: source_item_kind(item.kind),
                path: path.clone(),
                public: item.public,
                review_surface: is_review_surface(item.public, &item.name, item.kind, test_file),
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
        if !is_typescript_path(&implementation.location.path) {
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
            let review_surface = is_review_surface(item.public, &item.name, item.kind, test_file);
            items.push(TraceabilityOwnedItem {
                stable_id: item.stable_id,
                name: item.name,
                kind: source_item_kind(item.kind),
                path: implementation.location.path.clone(),
                public: item.public,
                review_surface,
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
            .filter_map(|call| resolve_call_target(item, call, &callable_items, &indexes))
            .collect::<BTreeSet<_>>();
        edges.insert(item.stable_id.clone(), callees);
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

    let Some(runtime) = typescript_runtime(root) else {
        return Ok(BTreeMap::new());
    };
    let Some(script) = write_embedded_tool_script(
        "special-typescript-trace-edges.cjs",
        include_str!("assets/typescript_trace_edges.cjs"),
    ) else {
        return Err(anyhow!("failed to write embedded TypeScript trace helper"));
    };

    let input = ToolTraceInput {
        mode: ToolRequestMode::TraceEdges,
        root: root.display().to_string(),
        source_files: source_graphs
            .keys()
            .map(|path| root.join(path).display().to_string())
            .collect(),
        items: callable_items
            .iter()
            .map(|item| ToolTraceItemInput {
                stable_id: item.stable_id.clone(),
                name: item.name.clone(),
                path: root.join(&item.path).display().to_string(),
                start_line: item.start_line,
                end_line: item.end_line,
                start_column: item.start_column,
            })
            .collect(),
    };

    let json_input = serde_json::to_vec(&input)?;

    let mut child = runtime
        .toolchain
        .command("node")
        .arg(script.path().to_string_lossy().as_ref())
        .arg(runtime.typescript_entry.to_string_lossy().as_ref())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut()
        && stdin.write_all(&json_input).is_err()
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err(anyhow!("failed to write input to TypeScript trace helper"));
    }
    let _ = child.stdin.take();

    let output = match child.wait_with_output() {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(anyhow!(
                "TypeScript trace helper exited with status {}{}",
                output.status,
                if stderr.is_empty() {
                    String::new()
                } else {
                    format!(": {stderr}")
                }
            ));
        }
        Err(error) => return Err(error.into()),
    };

    let tool_output: ToolTraceOutput = serde_json::from_slice(&output.stdout)?;

    let callable_ids = callable_items
        .iter()
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    let mut edges = BTreeMap::new();
    for edge in tool_output.edges {
        if !callable_ids.contains(&edge.caller) || !callable_ids.contains(&edge.callee) {
            continue;
        }
        edges
            .entry(edge.caller)
            .or_insert_with(BTreeSet::new)
            .insert(edge.callee);
    }
    Ok(edges)
}

fn build_tool_module_graph(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<BTreeMap<PathBuf, BTreeSet<PathBuf>>> {
    if source_files.is_empty() {
        return Ok(BTreeMap::new());
    }

    let Some(runtime) = typescript_runtime(root) else {
        return Ok(BTreeMap::new());
    };
    let Some(script) = write_embedded_tool_script(
        "special-typescript-trace-edges.cjs",
        include_str!("assets/typescript_trace_edges.cjs"),
    ) else {
        return Err(anyhow!("failed to write embedded TypeScript trace helper"));
    };

    let absolute_files = source_files
        .iter()
        .map(|path| root.join(path))
        .collect::<Vec<_>>();
    let absolute_index = absolute_files
        .iter()
        .zip(source_files.iter())
        .map(|(absolute, relative)| (normalize_path(absolute), relative.clone()))
        .collect::<BTreeMap<_, _>>();
    let input = ToolTraceInput {
        mode: ToolRequestMode::ModuleGraph,
        root: root.display().to_string(),
        source_files: absolute_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        items: Vec::new(),
    };
    let json_input = serde_json::to_vec(&input)?;

    let mut child = runtime
        .toolchain
        .command("node")
        .arg(script.path().to_string_lossy().as_ref())
        .arg(runtime.typescript_entry.to_string_lossy().as_ref())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut()
        && let Err(error) = stdin.write_all(&json_input)
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error.into());
    }
    let _ = child.stdin.take();

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!(
            "TypeScript trace helper exited with status {}{}",
            output.status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        ));
    }

    let tool_output: ToolTraceOutput = serde_json::from_slice(&output.stdout)?;
    let mut graph: BTreeMap<PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
    for edge in tool_output.file_edges {
        let Some(from) = absolute_index.get(&normalize_path(Path::new(&edge.from))).cloned() else {
            continue;
        };
        let Some(to) = absolute_index.get(&normalize_path(Path::new(&edge.to))).cloned() else {
            continue;
        };
        if from == to {
            continue;
        }
        graph.entry(from.clone()).or_default().insert(to.clone());
        graph.entry(to).or_default().insert(from);
    }
    Ok(graph)
}

#[derive(Clone, Serialize, Deserialize)]
struct TypeScriptTraceabilityGraphFacts {
    source_graphs: BTreeMap<PathBuf, CachedParsedSourceGraph>,
    tool_call_edges: BTreeMap<String, BTreeSet<String>>,
}

type TypeScriptGraphFactsDecoded = Result<(
    BTreeMap<PathBuf, ParsedSourceGraph>,
    BTreeMap<String, BTreeSet<String>>,
)>;

fn decode_traceability_graph_facts(
    facts: Option<&[u8]>,
) -> Option<TypeScriptGraphFactsDecoded> {
    let facts = facts?;
    Some(
        serde_json::from_slice::<TypeScriptTraceabilityGraphFacts>(facts)
            .map_err(anyhow::Error::from)
            .map(|facts| {
            (
                facts
                    .source_graphs
                    .into_iter()
                    .map(|(path, graph)| (path, graph.into_parsed()))
                    .collect(),
                facts.tool_call_edges,
            )
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
struct CachedParsedSourceGraph {
    items: Vec<CachedSourceItem>,
}

impl CachedParsedSourceGraph {
    fn from_parsed(graph: &ParsedSourceGraph) -> Self {
        Self {
            items: graph.items.iter().map(CachedSourceItem::from_parsed).collect(),
        }
    }

    fn into_parsed(self) -> ParsedSourceGraph {
        ParsedSourceGraph {
            language: crate::syntax::SourceLanguage::new("typescript"),
            items: self
                .items
                .into_iter()
                .map(CachedSourceItem::into_parsed)
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct CachedSourceItem {
    source_path: String,
    stable_id: String,
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    shape_fingerprint: String,
    shape_node_count: usize,
    kind: CachedSourceItemKind,
    span: CachedSourceSpan,
    public: bool,
    root_visible: bool,
    is_test: bool,
    calls: Vec<CachedSourceCall>,
}

impl CachedSourceItem {
    fn from_parsed(item: &crate::syntax::SourceItem) -> Self {
        Self {
            source_path: item.source_path.clone(),
            stable_id: item.stable_id.clone(),
            name: item.name.clone(),
            qualified_name: item.qualified_name.clone(),
            module_path: item.module_path.clone(),
            container_path: item.container_path.clone(),
            shape_fingerprint: item.shape_fingerprint.clone(),
            shape_node_count: item.shape_node_count,
            kind: CachedSourceItemKind::from_parsed(item.kind),
            span: CachedSourceSpan::from_parsed(item.span),
            public: item.public,
            root_visible: item.root_visible,
            is_test: item.is_test,
            calls: item.calls.iter().map(CachedSourceCall::from_parsed).collect(),
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceItem {
        crate::syntax::SourceItem {
            source_path: self.source_path,
            stable_id: self.stable_id,
            name: self.name,
            qualified_name: self.qualified_name,
            module_path: self.module_path,
            container_path: self.container_path,
            shape_fingerprint: self.shape_fingerprint,
            shape_node_count: self.shape_node_count,
            kind: self.kind.into_parsed(),
            span: self.span.into_parsed(),
            public: self.public,
            root_visible: self.root_visible,
            is_test: self.is_test,
            calls: self
                .calls
                .into_iter()
                .map(CachedSourceCall::into_parsed)
                .collect(),
            invocations: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum CachedSourceItemKind {
    Function,
    Method,
}

impl CachedSourceItemKind {
    fn from_parsed(kind: crate::syntax::SourceItemKind) -> Self {
        match kind {
            crate::syntax::SourceItemKind::Function => Self::Function,
            crate::syntax::SourceItemKind::Method => Self::Method,
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceItemKind {
        match self {
            Self::Function => crate::syntax::SourceItemKind::Function,
            Self::Method => crate::syntax::SourceItemKind::Method,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct CachedSourceSpan {
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,
    start_byte: usize,
    end_byte: usize,
}

impl CachedSourceSpan {
    fn from_parsed(span: crate::syntax::SourceSpan) -> Self {
        Self {
            start_line: span.start_line,
            end_line: span.end_line,
            start_column: span.start_column,
            end_column: span.end_column,
            start_byte: span.start_byte,
            end_byte: span.end_byte,
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceSpan {
        crate::syntax::SourceSpan {
            start_line: self.start_line,
            end_line: self.end_line,
            start_column: self.start_column,
            end_column: self.end_column,
            start_byte: self.start_byte,
            end_byte: self.end_byte,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct CachedSourceCall {
    name: String,
    qualifier: Option<String>,
    syntax: CachedCallSyntaxKind,
    span: CachedSourceSpan,
}

impl CachedSourceCall {
    fn from_parsed(call: &crate::syntax::SourceCall) -> Self {
        Self {
            name: call.name.clone(),
            qualifier: call.qualifier.clone(),
            syntax: CachedCallSyntaxKind::from_parsed(&call.syntax),
            span: CachedSourceSpan::from_parsed(call.span),
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceCall {
        crate::syntax::SourceCall {
            name: self.name,
            qualifier: self.qualifier,
            syntax: self.syntax.into_parsed(),
            span: self.span.into_parsed(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum CachedCallSyntaxKind {
    Identifier,
    ScopedIdentifier,
    Field,
}

impl CachedCallSyntaxKind {
    fn from_parsed(kind: &crate::syntax::CallSyntaxKind) -> Self {
        match kind {
            crate::syntax::CallSyntaxKind::Identifier => Self::Identifier,
            crate::syntax::CallSyntaxKind::ScopedIdentifier => Self::ScopedIdentifier,
            crate::syntax::CallSyntaxKind::Field => Self::Field,
        }
    }

    fn into_parsed(self) -> crate::syntax::CallSyntaxKind {
        match self {
            Self::Identifier => crate::syntax::CallSyntaxKind::Identifier,
            Self::ScopedIdentifier => crate::syntax::CallSyntaxKind::ScopedIdentifier,
            Self::Field => crate::syntax::CallSyntaxKind::Field,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct CachedTraceabilityItemSupport {
    name: String,
    has_item_scoped_support: bool,
    has_file_scoped_support: bool,
    current_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
}

impl CachedTraceabilityItemSupport {
    fn from_runtime(support: TraceabilityItemSupport) -> Self {
        Self {
            name: support.name,
            has_item_scoped_support: support.has_item_scoped_support,
            has_file_scoped_support: support.has_file_scoped_support,
            current_specs: support.current_specs,
            planned_specs: support.planned_specs,
            deprecated_specs: support.deprecated_specs,
        }
    }

    fn into_runtime(self) -> TraceabilityItemSupport {
        TraceabilityItemSupport {
            name: self.name,
            has_item_scoped_support: self.has_item_scoped_support,
            has_file_scoped_support: self.has_file_scoped_support,
            current_specs: self.current_specs,
            planned_specs: self.planned_specs,
            deprecated_specs: self.deprecated_specs,
        }
    }
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
            path: path.clone(),
            calls: item.calls,
            start_line: item.span.start_line,
            end_line: item.span.end_line,
            start_column: item.span.start_column,
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
    }
    indexes
}

fn resolve_call_target(
    caller: &SourceCallableItem,
    call: &SourceCall,
    items: &[SourceCallableItem],
    indexes: &CallableIndexes,
) -> Option<String> {
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

fn is_test_file_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with(".test.ts")
                    || name.ends_with(".test.tsx")
                    || name.ends_with(".spec.ts")
                    || name.ends_with(".spec.tsx")
            })
}

fn is_process_entrypoint_name(name: &str, kind: crate::syntax::SourceItemKind) -> bool {
    kind == crate::syntax::SourceItemKind::Function && name == "main"
}

fn is_review_surface(
    public: bool,
    name: &str,
    kind: crate::syntax::SourceItemKind,
    test_file: bool,
) -> bool {
    !test_file && (public || is_process_entrypoint_name(name, kind))
}

fn source_item_kind(kind: crate::syntax::SourceItemKind) -> ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => ModuleItemKind::Method,
    }
}

struct TypeScriptRuntime {
    toolchain: ProjectToolchain,
    typescript_entry: PathBuf,
}

fn typescript_runtime(root: &Path) -> Option<TypeScriptRuntime> {
    let toolchain = ProjectToolchain::discover(root).ok().flatten()?;
    let node_ok = toolchain.command("node").arg("--version").output().ok()?;
    if !node_ok.status.success() {
        return None;
    }
    let typescript_entry = resolve_typescript_entry(root, &toolchain)?;
    typescript_entry.exists().then_some(TypeScriptRuntime {
        toolchain,
        typescript_entry,
    })
}

fn resolve_typescript_entry(root: &Path, toolchain: &ProjectToolchain) -> Option<PathBuf> {
    let output = toolchain
        .command("node")
        .args([
            "-e",
            "process.stdout.write(require.resolve('typescript'))",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        let resolved = String::from_utf8(output.stdout).ok()?;
        let path = PathBuf::from(resolved.trim());
        if path.exists() {
            return Some(path);
        }
    }

    let npm_root = toolchain
        .command("npm")
        .args(["root", "-g"])
        .current_dir(root)
        .output()
        .ok()?;
    if !npm_root.status.success() {
        return None;
    }
    let global_root = PathBuf::from(String::from_utf8(npm_root.stdout).ok()?.trim());
    let global_entry = global_root.join("typescript/lib/typescript.js");
    global_entry.exists().then_some(global_entry)
}

fn typescript_backward_trace_unavailable_reason(root: &Path) -> Option<String> {
    typescript_runtime(root).is_none().then(|| {
        format!(
            "TypeScript backward trace is unavailable because the analyzed project does not declare a supported {} contract with a resolvable `typescript` package",
            supported_project_toolchain_contracts()
        )
    })
}

pub(crate) fn analysis_environment_fingerprint(root: &Path) -> String {
    typescript_runtime(root)
        .map(|runtime| {
            let node = tool_version_fingerprint(&runtime.toolchain, "node", &["--version"]);
            let typescript = typescript_entry_fingerprint(&runtime.typescript_entry);
            format!(
                "tool_manager={};node={};typescript={}",
                runtime.toolchain.launcher_label("node"),
                node,
                typescript
            )
        })
        .unwrap_or_else(|| "project_toolchain_or_typescript=unavailable".to_string())
}

fn tool_version_fingerprint(
    toolchain: &ProjectToolchain,
    tool: &str,
    version_args: &[&str],
) -> String {
    let available = toolchain.tool_available(tool, version_args);
    let output = toolchain.command(tool).args(version_args).output();
    output
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let version = if !stdout.is_empty() {
                stdout
            } else if !stderr.is_empty() {
                stderr
            } else {
                "available".to_string()
            };
            version.replace(['\n', '\r'], " ")
        })
        .unwrap_or_else(|| available.to_string())
}

fn typescript_entry_fingerprint(entry: &Path) -> String {
    let bytes = fs::read(entry).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    entry.hash(&mut hasher);
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

static EMBEDDED_TOOL_SCRIPT_COUNTER: AtomicU64 = AtomicU64::new(0);

struct EmbeddedToolScript {
    path: PathBuf,
}

impl EmbeddedToolScript {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for EmbeddedToolScript {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn write_embedded_tool_script(file_name: &str, contents: &str) -> Option<EmbeddedToolScript> {
    let unique = EMBEDDED_TOOL_SCRIPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = match file_name.rsplit_once('.') {
        Some((stem, ext)) => {
            std::env::temp_dir().join(format!("{stem}-{}-{unique}.{ext}", std::process::id()))
        }
        None => std::env::temp_dir().join(format!("{file_name}-{}-{unique}", std::process::id())),
    };
    std::fs::write(&path, contents).ok()?;
    Some(EmbeddedToolScript { path })
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum ToolRequestMode {
    TraceEdges,
    ModuleGraph,
}

#[derive(Serialize)]
struct ToolTraceInput {
    mode: ToolRequestMode,
    root: String,
    source_files: Vec<String>,
    items: Vec<ToolTraceItemInput>,
}

#[derive(Serialize)]
struct ToolTraceItemInput {
    stable_id: String,
    name: String,
    path: String,
    start_line: usize,
    end_line: usize,
    start_column: usize,
}

#[derive(Deserialize)]
struct ToolTraceOutput {
    #[serde(default)]
    edges: Vec<ToolTraceEdge>,
    #[serde(default)]
    file_edges: Vec<ToolFileEdge>,
}

#[derive(Deserialize)]
struct ToolTraceEdge {
    caller: String,
    callee: String,
}

#[derive(Deserialize)]
struct ToolFileEdge {
    from: String,
    to: String,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{typescript_entry_fingerprint, write_embedded_tool_script};

    #[test]
    fn embedded_tool_script_uses_unique_paths_and_cleans_up_on_drop() {
        let first = write_embedded_tool_script("special-test-helper.cjs", "console.log('a');")
            .expect("first helper should be written");
        let second = write_embedded_tool_script("special-test-helper.cjs", "console.log('b');")
            .expect("second helper should be written");

        let first_path = first.path().to_path_buf();
        let second_path = second.path().to_path_buf();

        assert_ne!(first_path, second_path);
        assert_eq!(
            fs::read_to_string(&first_path).expect("first helper should be readable"),
            "console.log('a');"
        );
        assert_eq!(
            fs::read_to_string(&second_path).expect("second helper should be readable"),
            "console.log('b');"
        );

        drop(first);
        assert!(!first_path.exists());
        assert!(second_path.exists());

        drop(second);
        assert!(!second_path.exists());
    }

    #[test]
    fn typescript_entry_fingerprint_changes_with_file_contents() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("special-typescript-entry-{unique}.js"));
        fs::write(&path, "export const version = 'a';").expect("fixture should be written");
        let first = typescript_entry_fingerprint(&path);

        fs::write(&path, "export const version = 'b';").expect("fixture should be rewritten");
        let second = typescript_entry_fingerprint(&path);

        assert_ne!(first, second);

        let _ = fs::remove_file(&path);
    }
}
