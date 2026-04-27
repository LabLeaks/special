/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TRACEABILITY
Builds Go traceability inputs and tool-assisted call edges for the built-in Go pack analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TRACEABILITY
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

use crate::config::ProjectToolchain;
use crate::model::{ArchitectureTraceabilitySummary, ImplementRef, ParsedArchitecture, ParsedRepo};
use crate::syntax::{CallSyntaxKind, ParsedSourceGraph, SourceCall, parse_source_graph};

use crate::modules::analyze::traceability_core::{
    TraceGraph, TraceabilityAnalysis, TraceabilityInputs, TraceabilityLanguagePack,
    TraceabilityItemSupport, TraceabilityOwnedItem, build_root_supports,
    merge_trace_graph_edges, owned_module_ids_for_path,
    summarize_repo_traceability as summarize_shared_repo_traceability,
};
use crate::modules::analyze::{FileOwnership, emit_analysis_status, read_owned_file_text};
use super::scope;
use super::dependencies::collect_go_import_aliases;
use super::surface::{is_go_path, is_review_surface, is_test_file_path, source_item_kind};
use super::toolchain::{TempDirGuard, create_temp_dir, go_list_packages};

#[derive(Debug, Clone, Copy)]
pub(super) struct GoTraceabilityPack;

impl TraceabilityLanguagePack for GoTraceabilityPack {
    fn owned_items_for_implementations(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> Vec<TraceabilityOwnedItem> {
        collect_owned_items(root, implementations, file_ownership)
    }
}

pub(super) fn summarize_repo_traceability(
    root: &Path,
    traceability: &TraceabilityAnalysis,
) -> ArchitectureTraceabilitySummary {
    summarize_shared_repo_traceability(root, traceability)
}

pub(super) fn build_traceability_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
) -> Result<Vec<u8>> {
    let source_graphs = parse_go_source_graphs(root, source_files);
    let static_edges = build_static_call_edges(root, &source_graphs);
    let facts = GoTraceabilityGraphFacts {
        source_graphs: source_graphs
            .iter()
            .map(|(path, graph)| (path.clone(), CachedParsedSourceGraph::from_parsed(graph)))
            .collect(),
        static_edges,
    };
    Ok(serde_json::to_vec(&facts)?)
}

pub(super) fn build_traceability_analysis_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
    graph_facts: Option<&[u8]>,
    parsed_repo: &ParsedRepo,
    _parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    _traceability_pack: &GoTraceabilityPack,
) -> Result<TraceabilityAnalysis> {
    Ok(crate::modules::analyze::traceability_core::build_traceability_analysis(
        build_traceability_inputs_from_cached_or_live_graph_facts(
            root,
            source_files,
            graph_facts,
            parsed_repo,
            file_ownership,
        )?,
    ))
}

pub(super) fn build_traceability_inputs_from_cached_or_live_graph_facts(
    root: &Path,
    source_files: &[PathBuf],
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
    let repo_items = collect_repo_items(&source_graphs, file_ownership);
    let mut graph = TraceGraph {
        edges: static_edges,
        root_supports: BTreeMap::new(),
    };
    merge_trace_graph_edges(
        &mut graph.edges,
        build_gopls_reference_edges(root, &collect_callable_items(&source_graphs))?,
    );
    graph.root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
        parse_source_graph(&scope::normalize_go_path(root, path), body)
            .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
    });

    Ok(TraceabilityInputs {
        repo_items,
        context_items: Vec::new(),
        graph,
    })
}

pub(super) fn parse_go_source_graphs(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, ParsedSourceGraph> {
    source_files
        .iter()
        .filter(|path| is_go_path(path))
        .filter_map(|path| {
            let repo_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_path_buf();
            let text = read_owned_file_text(root, &repo_path).ok()?;
            parse_source_graph(&repo_path, &text).map(|graph| (repo_path, graph))
        })
        .collect()
}

// @applies ADAPTER.FACTS_TO_MODEL.TRACEABILITY_ITEMS
pub(super) fn collect_repo_items(
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

fn collect_owned_items(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = Vec::new();
    let mut seen = BTreeSet::new();
    for implementation in implementations {
        if !is_go_path(&implementation.location.path) {
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

#[cfg(test)]
fn build_tool_call_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    build_static_call_edges(root, source_graphs)
}

pub(super) fn build_static_call_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> BTreeMap<String, BTreeSet<String>> {
    let callable_items = collect_callable_items(source_graphs);
    let mut edges = build_parser_call_edges(source_graphs);
    if callable_items.is_empty() {
        return edges;
    }
    merge_trace_graph_edges(
        &mut edges,
        build_go_list_package_edges(root, source_graphs, &callable_items),
    );
    edges
}

fn build_go_list_package_edges(
    root: &Path,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    callable_items: &[SourceCallableItem],
) -> BTreeMap<String, BTreeSet<String>> {
    let Some(packages) = go_list_packages(root) else {
        return BTreeMap::new();
    };
    let canonical_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let mut file_package_paths = BTreeMap::<PathBuf, String>::new();
    for package in &packages {
        for file_name in &package.go_files {
            let full_path = package.dir.join(file_name);
            let canonical_full_path = fs::canonicalize(&full_path).unwrap_or(full_path.clone());
            file_package_paths.insert(full_path.clone(), package.import_path.clone());
            file_package_paths.insert(canonical_full_path.clone(), package.import_path.clone());
            if let Ok(relative) = canonical_full_path
                .strip_prefix(&canonical_root)
                .or_else(|_| full_path.strip_prefix(root))
            {
                file_package_paths.insert(relative.to_path_buf(), package.import_path.clone());
            }
        }
    }

    let mut package_name_to_ids = BTreeMap::<(String, String), Vec<String>>::new();
    for item in callable_items {
        let Some(import_path) = file_package_paths.get(&item.path) else {
            continue;
        };
        package_name_to_ids
            .entry((import_path.clone(), item.name.clone()))
            .or_default()
            .push(item.stable_id.clone());
    }

    let mut import_aliases = BTreeMap::<PathBuf, BTreeMap<String, String>>::new();
    for path in source_graphs.keys() {
        let Ok(text) = read_owned_file_text(root, path) else {
            continue;
        };
        import_aliases.insert(path.clone(), collect_go_import_aliases(&text));
    }

    let mut edges = BTreeMap::<String, BTreeSet<String>>::new();
    for item in callable_items {
        let Some(file_imports) = import_aliases.get(&item.path) else {
            continue;
        };
        for call in &item.calls {
            let Some(alias) = &call.qualifier else {
                continue;
            };
            let Some(import_path) = file_imports.get(alias) else {
                continue;
            };
            let Some(candidates) =
                package_name_to_ids.get(&(import_path.clone(), call.name.clone()))
            else {
                continue;
            };
            if candidates.len() != 1 {
                continue;
            }
            edges
                .entry(item.stable_id.clone())
                .or_default()
                .insert(candidates[0].clone());
        }
    }
    edges
}

pub(super) fn build_gopls_reference_edges(
    root: &Path,
    callable_items: &[SourceCallableItem],
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    if callable_items.is_empty() {
        return Ok(BTreeMap::new());
    }
    let started_at = Instant::now();
    let (mut client, index) = start_gopls_client_for_items(
        root,
        callable_items,
        started_at,
        "gopls reference collection",
        0,
    )?;
    let mut edges = BTreeMap::<String, BTreeSet<String>>::new();
    let mut query_count = 0usize;
    for item in callable_items {
        query_count += 1;
        for caller_id in client.reference_callers(item, &index)? {
            if caller_id == item.stable_id {
                continue;
            }
            edges.entry(caller_id).or_default().insert(item.stable_id.clone());
        }
    }
    emit_analysis_status(&format!(
        "gopls opened {} file(s) while collecting references",
        client.opened_file_count()
    ));
    client.shutdown()?;
    emit_analysis_status(&format!(
        "gopls reference collection queried {} item(s), discovered {} semantic edge(s)",
        query_count,
        edges.values().map(BTreeSet::len).sum::<usize>()
    ));
    Ok(edges)
}

pub(super) fn build_reverse_reachable_reference_edges(
    root: &Path,
    callable_items: &[SourceCallableItem],
    seed_ids: &BTreeSet<String>,
    parser_edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    if callable_items.is_empty() || seed_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let started_at = Instant::now();
    let (mut client, index) = start_gopls_client_for_items(
        root,
        callable_items,
        started_at,
        "gopls reverse caller walk",
        seed_ids.len(),
    )?;
    let item_by_id = callable_items
        .iter()
        .map(|item| (item.stable_id.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let mut reverse_parser_edges = BTreeMap::<String, BTreeSet<String>>::new();
    for (caller, callees) in parser_edges {
        for callee in callees {
            reverse_parser_edges
                .entry(callee.clone())
                .or_default()
                .insert(caller.clone());
        }
    }
    let mut edges = BTreeMap::<String, BTreeSet<String>>::new();
    let mut visited = BTreeSet::new();
    let mut query_count = 0usize;
    let mut pending = seed_ids.iter().cloned().collect::<Vec<String>>();
    while let Some(callee_id) = pending.pop() {
        if !visited.insert(callee_id.clone()) {
            continue;
        }
        let Some(callee) = item_by_id.get(&callee_id) else {
            continue;
        };
        let mut callers = reverse_parser_edges
            .get(&callee_id)
            .cloned()
            .unwrap_or_default();
        query_count += 1;
        for caller_id in client.reference_callers(callee, &index)? {
            if caller_id == callee.stable_id {
                continue;
            }
            edges
                .entry(caller_id.clone())
                .or_default()
                .insert(callee.stable_id.clone());
            callers.insert(caller_id);
        }
        for caller_id in callers {
            if item_by_id.contains_key(&caller_id) && !visited.contains(&caller_id) {
                pending.push(caller_id);
            }
        }
    }
    emit_analysis_status(&format!(
        "gopls opened {} file(s) while walking reverse callers",
        client.opened_file_count()
    ));
    client.shutdown()?;
    emit_analysis_status(&format!(
        "gopls reverse caller walk queried {} item(s), discovered {} semantic edge(s)",
        query_count,
        edges.values().map(BTreeSet::len).sum::<usize>()
    ));
    Ok(edges)
}

#[derive(Debug, Clone)]
pub(super) struct SourceCallableItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    path: PathBuf,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    calls: Vec<SourceCall>,
}

#[derive(Debug, Clone, Default)]
struct CallableIndexes {
    global_name_counts: BTreeMap<String, usize>,
    same_file_name_counts: BTreeMap<(PathBuf, String), usize>,
    global_qualified_name_counts: BTreeMap<String, usize>,
}

pub(super) fn collect_callable_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
) -> Vec<SourceCallableItem> {
    let mut items = Vec::new();
    for (path, graph) in source_graphs {
        items.extend(graph.items.iter().cloned().map(|item| SourceCallableItem {
            stable_id: item.stable_id,
            name: item.name,
            qualified_name: item.qualified_name,
            path: path.clone(),
            start_line: item.span.start_line,
            start_column: item.span.start_column,
            end_line: item.span.end_line,
            calls: item.calls,
        }));
    }
    items
}

#[derive(Debug)]
struct GoLspItemIndex {
    by_path: BTreeMap<PathBuf, Vec<IndexedItem>>,
}

#[derive(Debug, Clone)]
struct IndexedItem {
    stable_id: String,
    start_line: usize,
    end_line: usize,
}

impl GoLspItemIndex {
    fn new(root: &Path, items: &[SourceCallableItem]) -> Self {
        let mut by_path = BTreeMap::<PathBuf, Vec<IndexedItem>>::new();
        for item in items {
            let absolute = normalize_path(root.join(&item.path));
            by_path.entry(absolute).or_default().push(IndexedItem {
                stable_id: item.stable_id.clone(),
                start_line: item.start_line,
                end_line: item.end_line,
            });
        }
        Self { by_path }
    }

    fn resolve_containing(&self, path: &Path, line: usize) -> Option<String> {
        let items = self.by_path.get(path)?;
        let mut containing = items
            .iter()
            .filter(|item| item.start_line <= line && line <= item.end_line);
        if let Some(item) = containing.next()
            && containing.next().is_none()
        {
            return Some(item.stable_id.clone());
        }
        None
    }
}

struct GoLspClient {
    root: PathBuf,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
    opened_files: BTreeSet<PathBuf>,
    _cache_dir: TempDirGuard,
}

impl GoLspClient {
    fn start(root: &Path) -> Result<Self> {
        let root = normalize_path(root);
        let toolchain = ProjectToolchain::discover(&root)?
            .ok_or_else(|| anyhow!("project does not declare a supported toolchain contract"))?;
        let cache_dir = create_temp_dir("special-go-lsp-cache")
            .ok_or_else(|| anyhow!("failed to create temporary Go analysis cache"))?;
        let mut child = toolchain
            .command("gopls")
            .arg("serve")
            .env("GOCACHE", cache_dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("failed to launch gopls")?;
        let stdin = child.stdin.take().context("gopls stdin missing")?;
        let stdout = child.stdout.take().context("gopls stdout missing")?;
        let mut client = Self {
            root,
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            opened_files: BTreeSet::new(),
            _cache_dir: cache_dir,
        };
        client.initialize()?;
        Ok(client)
    }

    fn initialize(&mut self) -> Result<()> {
        let root_uri = Url::from_file_path(&self.root).map_err(|_| {
            anyhow!("failed to build gopls root uri for {}", self.root.display())
        })?;
        let params = json!({
            "processId": std::process::id(),
            "rootUri": root_uri.to_string(),
            "workspaceFolders": [{
                "uri": root_uri.to_string(),
                "name": self.root.file_name().and_then(|name| name.to_str()).unwrap_or("workspace"),
            }],
            "capabilities": {
                "textDocument": {
                    "references": {}
                }
            }
        });
        let _ = self.request("initialize", params)?;
        self.notify("initialized", json!({}))?;
        Ok(())
    }

    fn reference_callers(
        &mut self,
        item: &SourceCallableItem,
        index: &GoLspItemIndex,
    ) -> Result<BTreeSet<String>> {
        let absolute = normalize_path(self.root.join(&item.path));
        self.ensure_open_file(&absolute)?;
        let uri = Url::from_file_path(&absolute)
            .map_err(|_| anyhow!("failed to build gopls uri for {}", absolute.display()))?;
        let character = query_item_character(&absolute, item)?;
        let mut callers = BTreeSet::new();
        for attempt in 0..10 {
            let response = self.request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": uri.to_string() },
                    "position": {
                        "line": item.start_line.saturating_sub(1),
                        "character": character,
                    },
                    "context": {
                        "includeDeclaration": false,
                    }
                }),
            );
            let response = match response {
                Ok(response) => response,
                Err(error) if is_retryable_gopls_error(&error) && attempt < 9 => {
                    std::thread::sleep(Duration::from_millis(200));
                    continue;
                }
                Err(error) => return Err(error),
            };
            for reference in lsp_locations(&response) {
                let Ok(uri) = Url::parse(&reference.uri) else {
                    continue;
                };
                let Ok(path) = uri.to_file_path() else {
                    continue;
                };
                if let Some(caller_id) = index.resolve_containing(
                    &normalize_path(path),
                    reference.range.start.line as usize + 1,
                ) {
                    callers.insert(caller_id);
                }
            }
            break;
        }
        Ok(callers)
    }

    fn ensure_open_file(&mut self, path: &Path) -> Result<()> {
        let path = normalize_path(path);
        if !self.opened_files.insert(path.clone()) {
            return Ok(());
        }
        let uri = Url::from_file_path(&path)
            .map_err(|_| anyhow!("failed to build gopls uri for {}", path.display()))?;
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri.to_string(),
                    "languageId": "go",
                    "version": 0,
                    "text": text,
                }
            }),
        )?;
        Ok(())
    }

    fn opened_file_count(&self) -> usize {
        self.opened_files.len()
    }

    fn shutdown(&mut self) -> Result<()> {
        let _ = self.request("shutdown", json!({}));
        let _ = self.notify("exit", json!({}));
        let _ = self.child.wait();
        Ok(())
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }))?;

        loop {
            let message = self.read_message()?;
            if message.get("id").and_then(Value::as_i64) != Some(id) {
                continue;
            }
            if let Some(error) = message.get("error") {
                return Err(anyhow!("gopls request {method} failed: {error}"));
            }
            return Ok(message.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
    }

    fn write_message(&mut self, message: &Value) -> Result<()> {
        let body = serde_json::to_vec(message)?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())?;
        self.stdin.write_all(&body)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_message(&mut self) -> Result<Value> {
        let mut content_length = None;
        loop {
            let mut header = String::new();
            let bytes = self.stdout.read_line(&mut header)?;
            if bytes == 0 {
                return Err(anyhow!("gopls closed the LSP stream"));
            }
            if header == "\r\n" {
                break;
            }
            if let Some(value) = header.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse::<usize>()?);
            }
        }

        let length = content_length.context("missing gopls content length")?;
        let mut body = vec![0u8; length];
        self.stdout.read_exact(&mut body)?;
        Ok(serde_json::from_slice(&body)?)
    }
}

impl Drop for GoLspClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_gopls_client_for_items(
    root: &Path,
    items: &[SourceCallableItem],
    started_at: Instant,
    phase: &str,
    seed_count: usize,
) -> Result<(GoLspClient, GoLspItemIndex)> {
    emit_analysis_status(&format!(
        "{phase} for {} file(s), {} callable item(s), {} seed root(s)",
        items.iter().map(|item| &item.path).collect::<BTreeSet<_>>().len(),
        items.len(),
        seed_count
    ));
    let client = GoLspClient::start(root)?;
    emit_analysis_status(&format!(
        "gopls started in {:.1}s",
        started_at.elapsed().as_secs_f32()
    ));
    std::thread::sleep(Duration::from_millis(500));
    emit_analysis_status(&format!(
        "gopls initialized workspace for {} indexed file(s) in {:.1}s",
        items.iter().map(|item| &item.path).collect::<BTreeSet<_>>().len(),
        started_at.elapsed().as_secs_f32()
    ));
    Ok((client, GoLspItemIndex::new(root, items)))
}

#[derive(Debug, Deserialize)]
struct LspLocation {
    uri: String,
    range: LspRange,
}

#[derive(Debug, Deserialize)]
struct LspRange {
    start: LspPosition,
}

#[derive(Debug, Deserialize)]
struct LspPosition {
    line: u32,
}

fn lsp_locations(response: &Value) -> Vec<LspLocation> {
    if response.is_null() {
        return Vec::new();
    }
    if response.is_array() {
        return serde_json::from_value::<Vec<LspLocation>>(response.clone()).unwrap_or_default();
    }
    serde_json::from_value::<LspLocation>(response.clone())
        .map(|location| vec![location])
        .unwrap_or_default()
}

fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf())
}

fn query_item_character(path: &Path, item: &SourceCallableItem) -> Result<usize> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let line = text
        .lines()
        .nth(item.start_line.saturating_sub(1))
        .unwrap_or_default();
    let start = item.start_column.min(line.len());
    Ok(line
        .get(start..)
        .and_then(|tail| tail.find(&item.name).map(|offset| start + offset))
        .or_else(|| line.find(&item.name))
        .unwrap_or(start))
}

fn is_retryable_gopls_error(error: &anyhow::Error) -> bool {
    error.to_string().contains("content modified")
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

#[derive(Serialize, Deserialize)]
struct GoTraceabilityGraphFacts {
    source_graphs: BTreeMap<PathBuf, CachedParsedSourceGraph>,
    static_edges: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct GoTraceabilityScopeFacts {
    pub(super) source_graphs: BTreeMap<PathBuf, CachedParsedSourceGraph>,
    pub(super) file_adjacency: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    pub(super) static_edges: BTreeMap<String, BTreeSet<String>>,
    pub(super) tool_reference_edges: BTreeMap<String, BTreeSet<String>>,
    pub(super) root_supports: BTreeMap<String, CachedTraceabilityItemSupport>,
}

type GoGraphFactsDecoded = (
    BTreeMap<PathBuf, ParsedSourceGraph>,
    BTreeMap<String, BTreeSet<String>>,
);

pub(super) fn decode_traceability_graph_facts(
    facts: Option<&[u8]>,
) -> Result<Option<GoGraphFactsDecoded>> {
    let Some(facts) = facts else {
        return Ok(None);
    };
    if let Ok(facts) = serde_json::from_slice::<GoTraceabilityGraphFacts>(facts) {
        return Ok(Some((
            facts
                .source_graphs
                .into_iter()
                .map(|(path, graph)| (path, graph.into_parsed()))
                .collect(),
            facts.static_edges,
        )));
    }
    let facts = serde_json::from_slice::<GoTraceabilityScopeFacts>(facts)?;
    let mut edges = facts.static_edges;
    merge_trace_graph_edges(&mut edges, facts.tool_reference_edges);
    Ok(Some((
        facts
            .source_graphs
            .into_iter()
            .map(|(path, graph)| (path, graph.into_parsed()))
            .collect(),
        edges,
    )))
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct CachedParsedSourceGraph {
    items: Vec<CachedSourceItem>,
}

impl CachedParsedSourceGraph {
    pub(super) fn from_parsed(graph: &ParsedSourceGraph) -> Self {
        Self {
            items: graph.items.iter().map(CachedSourceItem::from_parsed).collect(),
        }
    }

    pub(super) fn into_parsed(self) -> ParsedSourceGraph {
        ParsedSourceGraph {
            language: crate::syntax::SourceLanguage::new("go"),
            items: self
                .items
                .into_iter()
                .map(CachedSourceItem::into_parsed)
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct CachedTraceabilityItemSupport {
    name: String,
    has_item_scoped_support: bool,
    has_file_scoped_support: bool,
    current_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
}

impl CachedTraceabilityItemSupport {
    pub(super) fn from_runtime(support: TraceabilityItemSupport) -> Self {
        Self {
            name: support.name,
            has_item_scoped_support: support.has_item_scoped_support,
            has_file_scoped_support: support.has_file_scoped_support,
            current_specs: support.current_specs,
            planned_specs: support.planned_specs,
            deprecated_specs: support.deprecated_specs,
        }
    }

    pub(super) fn into_runtime(self) -> TraceabilityItemSupport {
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
    invocations: Vec<CachedSourceInvocation>,
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
            invocations: item
                .invocations
                .iter()
                .map(CachedSourceInvocation::from_parsed)
                .collect(),
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
            invocations: self
                .invocations
                .into_iter()
                .map(CachedSourceInvocation::into_parsed)
                .collect(),
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
    fn from_parsed(call: &SourceCall) -> Self {
        Self {
            name: call.name.clone(),
            qualifier: call.qualifier.clone(),
            syntax: CachedCallSyntaxKind::from_parsed(call.syntax.clone()),
            span: CachedSourceSpan::from_parsed(call.span),
        }
    }

    fn into_parsed(self) -> SourceCall {
        SourceCall {
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
    fn from_parsed(kind: CallSyntaxKind) -> Self {
        match kind {
            CallSyntaxKind::Identifier => Self::Identifier,
            CallSyntaxKind::ScopedIdentifier => Self::ScopedIdentifier,
            CallSyntaxKind::Field => Self::Field,
        }
    }

    fn into_parsed(self) -> CallSyntaxKind {
        match self {
            Self::Identifier => CallSyntaxKind::Identifier,
            Self::ScopedIdentifier => CallSyntaxKind::ScopedIdentifier,
            Self::Field => CallSyntaxKind::Field,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct CachedSourceInvocation {
    span: CachedSourceSpan,
    kind: CachedSourceInvocationKind,
}

impl CachedSourceInvocation {
    fn from_parsed(invocation: &crate::syntax::SourceInvocation) -> Self {
        Self {
            span: CachedSourceSpan::from_parsed(invocation.span),
            kind: CachedSourceInvocationKind::from_parsed(&invocation.kind),
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceInvocation {
        crate::syntax::SourceInvocation {
            span: self.span.into_parsed(),
            kind: self.kind.into_parsed(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum CachedSourceInvocationKind {
    LocalCargoBinary { binary_name: String },
}

impl CachedSourceInvocationKind {
    fn from_parsed(kind: &crate::syntax::SourceInvocationKind) -> Self {
        match kind {
            crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name } => {
                Self::LocalCargoBinary {
                    binary_name: binary_name.clone(),
                }
            }
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceInvocationKind {
        match self {
            Self::LocalCargoBinary { binary_name } => {
                crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use crate::syntax::parse_source_graph;

    use super::{build_tool_call_edges, collect_callable_items, collect_go_import_aliases};
    use crate::language_packs::go::analyze::toolchain::go_list_packages;

    #[test]
    fn build_tool_call_edges_resolves_go_import_alias_targets() {
        let root =
            super::create_temp_dir("special-go-tool-edges").expect("root dir should be created");
        fs::create_dir_all(root.path().join("app")).expect("app dir should be created");
        fs::create_dir_all(root.path().join("left")).expect("left dir should be created");
        fs::create_dir_all(root.path().join("right")).expect("right dir should be created");
        fs::write(
            root.path().join("go.mod"),
            "module example.com/demo\n\ngo 1.23\n",
        )
        .expect("go.mod should be written");
        fs::write(
            root.path().join(".tool-versions"),
            "go 1.23.12\n",
        )
        .expect(".tool-versions should be written");
        fs::write(
            root.path().join("special.toml"),
            "root = \".\"\n\n[toolchain]\nmanager = \"mise\"\n",
        )
        .expect("special.toml should be written");
        fs::write(
            root.path().join("app/main.go"),
            "// @fileimplements DEMO\npackage app\n\nimport l \"example.com/demo/left\"\n\nfunc LiveImpl() int {\n    return helper() + l.SharedValue()\n}\n\nfunc helper() int {\n    return 1\n}\n",
        )
        .expect("main.go should be written");
        fs::write(
            root.path().join("left/shared.go"),
            "// @fileimplements LEFT\npackage left\n\nfunc SharedValue() int {\n    return 1\n}\n",
        )
        .expect("left/shared.go should be written");
        fs::write(
            root.path().join("right/shared.go"),
            "// @fileimplements RIGHT\npackage right\n\nfunc SharedValue() int {\n    return 2\n}\n",
        )
        .expect("right/shared.go should be written");
        fs::write(
            root.path().join("app/main_test.go"),
            "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
        )
        .expect("main_test.go should be written");

        let mut source_graphs = BTreeMap::new();
        for relative in [
            "app/main.go",
            "app/main_test.go",
            "left/shared.go",
            "right/shared.go",
        ] {
            let path = PathBuf::from(relative);
            let text = fs::read_to_string(root.path().join(&path))
                .expect("fixture file should be readable");
            let graph = parse_source_graph(&path, &text).expect("source graph should parse");
            source_graphs.insert(path, graph);
        }

        let main_graph = source_graphs
            .get(&PathBuf::from("app/main.go"))
            .expect("main graph should be present");
        let live_impl_item = main_graph
            .items
            .iter()
            .find(|item| item.name == "LiveImpl")
            .expect("LiveImpl should be present");
        assert!(
            live_impl_item.calls.iter().any(|call| {
                call.name == "SharedValue" && call.qualifier.as_deref() == Some("l")
            })
        );

        let aliases = collect_go_import_aliases(
            &fs::read_to_string(root.path().join("app/main.go"))
                .expect("main.go should be readable"),
        );
        assert_eq!(
            aliases.get("l").map(String::as_str),
            Some("example.com/demo/left")
        );

        let packages = go_list_packages(root.path()).expect("go list should find packages");
        assert!(
            packages
                .iter()
                .any(|package| package.import_path == "example.com/demo/left")
        );

        let callable_items = collect_callable_items(&source_graphs);
        let canonical_root = fs::canonicalize(root.path()).expect("root should canonicalize");
        let mut file_package_paths = BTreeMap::<PathBuf, String>::new();
        for package in &packages {
            for file_name in &package.go_files {
                let full_path = package.dir.join(file_name);
                let canonical_full_path =
                    fs::canonicalize(&full_path).expect("package file should canonicalize");
                if let Ok(relative) = canonical_full_path
                    .strip_prefix(&canonical_root)
                    .or_else(|_| full_path.strip_prefix(root.path()))
                {
                    file_package_paths.insert(relative.to_path_buf(), package.import_path.clone());
                }
            }
        }
        assert_eq!(
            file_package_paths
                .get(&PathBuf::from("left/shared.go"))
                .map(String::as_str),
            Some("example.com/demo/left")
        );

        let mut package_name_to_ids = BTreeMap::<(String, String), Vec<String>>::new();
        for item in &callable_items {
            let Some(import_path) = file_package_paths.get(&item.path) else {
                continue;
            };
            package_name_to_ids
                .entry((import_path.clone(), item.name.clone()))
                .or_default()
                .push(item.stable_id.clone());
        }
        assert_eq!(
            package_name_to_ids
                .get(&(
                    "example.com/demo/left".to_string(),
                    "SharedValue".to_string()
                ))
                .cloned(),
            Some(vec![
                "left/shared.go:left::shared::SharedValue:4".to_string()
            ])
        );

        let edges = build_tool_call_edges(root.path(), &source_graphs);
        let live_impl = "app/main.go:app::LiveImpl:6".to_string();
        let left_shared = "left/shared.go:left::shared::SharedValue:4".to_string();
        let right_shared = "right/shared.go:right::shared::SharedValue:4".to_string();
        let callees = edges
            .get(&live_impl)
            .expect("LiveImpl should have tool-derived edges");
        assert!(callees.contains(&left_shared));
        assert!(!callees.contains(&right_shared));
    }

    #[test]
    fn build_tool_call_edges_resolves_nested_go_constructor_calls() {
        let root = super::create_temp_dir("special-go-tool-edges-interface")
            .expect("root dir should be created");
        fs::create_dir_all(root.path().join("app")).expect("app dir should be created");
        fs::create_dir_all(root.path().join("live")).expect("live dir should be created");
        fs::create_dir_all(root.path().join("dead")).expect("dead dir should be created");
        fs::write(
            root.path().join("go.mod"),
            "module example.com/demo\n\ngo 1.23\n",
        )
        .expect("go.mod should be written");
        fs::write(root.path().join(".tool-versions"), "go 1.23.12\n")
            .expect(".tool-versions should be written");
        fs::write(
            root.path().join("special.toml"),
            "version = \"1\"\nroot = \".\"\n",
        )
        .expect("special.toml should be written");
        fs::write(
            root.path().join("app/main.go"),
            "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\ntype Runner interface {\n    Run() int\n}\n\nfunc LiveImpl() int {\n    return invoke(live.NewRunner())\n}\n\nfunc invoke(r Runner) int {\n    return r.Run()\n}\n",
        )
        .expect("main.go should be written");
        fs::write(
            root.path().join("live/live.go"),
            "// @fileimplements LIVE\npackage live\n\ntype LiveRunner struct{}\n\nfunc NewRunner() LiveRunner {\n    return LiveRunner{}\n}\n\nfunc (LiveRunner) Run() int {\n    return 1\n}\n",
        )
        .expect("live/live.go should be written");
        fs::write(
            root.path().join("dead/dead.go"),
            "// @fileimplements DEAD\npackage dead\n\ntype DeadRunner struct{}\n\nfunc NewRunner() DeadRunner {\n    return DeadRunner{}\n}\n\nfunc (DeadRunner) Run() int {\n    return 2\n}\n",
        )
        .expect("dead/dead.go should be written");

        let mut source_graphs = BTreeMap::new();
        for relative in ["app/main.go", "live/live.go", "dead/dead.go"] {
            let path = PathBuf::from(relative);
            let text = fs::read_to_string(root.path().join(&path))
                .expect("fixture file should be readable");
            let graph = parse_source_graph(&path, &text).expect("source graph should parse");
            source_graphs.insert(path, graph);
        }

        let main_graph = source_graphs
            .get(&PathBuf::from("app/main.go"))
            .expect("main graph should be present");
        let live_impl_item = main_graph
            .items
            .iter()
            .find(|item| item.name == "LiveImpl")
            .expect("LiveImpl should be present");
        assert!(
            live_impl_item.calls.iter().any(|call| {
                call.name == "NewRunner" && call.qualifier.as_deref() == Some("live")
            }),
            "expected nested constructor call to be parsed",
        );

        let aliases = collect_go_import_aliases(
            &fs::read_to_string(root.path().join("app/main.go"))
                .expect("main.go should be readable"),
        );
        assert_eq!(
            aliases.get("live").map(String::as_str),
            Some("example.com/demo/live")
        );

        let packages = go_list_packages(root.path()).expect("go list should find packages");
        assert!(
            packages
                .iter()
                .any(|package| package.import_path == "example.com/demo/live")
        );

        let edges = build_tool_call_edges(root.path(), &source_graphs);
        let live_impl = "app/main.go:app::LiveImpl:10".to_string();
        let live_constructor = "live/live.go:live::live::NewRunner:6".to_string();
        let dead_constructor = "dead/dead.go:dead::NewRunner:6".to_string();
        let callees = edges
            .get(&live_impl)
            .expect("LiveImpl should have tool-derived edges");
        assert!(
            callees.contains(&live_constructor),
            "expected nested constructor edge; actual={callees:?}"
        );
        assert!(!callees.contains(&dead_constructor));
    }
}
