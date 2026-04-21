/**
@module SPECIAL.MODULES.ANALYZE.GO
Applies the built-in Go analysis provider to owned Go implementation while keeping Go parsing out of the language-agnostic analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.GO
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use serde::Deserialize;
use tree_sitter::{Node, Parser};

use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleDependencySummary, ModuleItemKind,
    ModuleMetricsSummary, ParsedArchitecture, ParsedRepo,
};
use crate::syntax::{ParsedSourceGraph, SourceCall, parse_source_graph};

use super::coupling::ModuleCouplingInput;
use super::source_item_signals::summarize_source_item_signals;
use super::traceability_core::{
    TraceGraph, TraceabilityAnalysis, TraceabilityInputs, TraceabilityLanguagePack,
    TraceabilityOwnedItem, build_root_supports, build_traceability_analysis,
    merge_trace_graph_edges, owned_module_ids_for_path, summarize_module_traceability,
    summarize_repo_traceability as summarize_shared_repo_traceability,
};
use super::{FileOwnership, ProviderModuleAnalysis, build_dependency_summary, visit_owned_texts};

pub(crate) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    context: &GoRepoAnalysisContext,
    include_traceability: bool,
) -> Result<ProviderModuleAnalysis> {
    let mut surface = GoSurfaceSummary::default();
    let mut owned_items = Vec::new();
    let mut dependencies = GoDependencySummary::default();
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
        if !is_go_path(path) {
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

pub(crate) struct GoRepoAnalysisContext {
    traceability_pack: GoTraceabilityPack,
    traceability: Option<TraceabilityAnalysis>,
    pub(crate) traceability_unavailable_reason: Option<String>,
}

pub(crate) fn build_repo_analysis_context(
    root: &Path,
    source_files: &[PathBuf],
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> GoRepoAnalysisContext {
    let traceability_pack = GoTraceabilityPack;
    let traceability_unavailable_reason = traceability_pack
        .backward_trace_availability()
        .unavailable_reason()
        .map(ToString::to_string);
    let traceability =
        (include_traceability && traceability_unavailable_reason.is_none()).then(|| {
            build_traceability_analysis(traceability_pack.build_inputs(
                root,
                source_files,
                parsed_repo,
                parsed_architecture,
                file_ownership,
            ))
        });
    GoRepoAnalysisContext {
        traceability_pack,
        traceability,
        traceability_unavailable_reason,
    }
}

pub(crate) fn summarize_repo_traceability(
    root: &Path,
    context: &GoRepoAnalysisContext,
) -> Option<ArchitectureTraceabilitySummary> {
    context
        .traceability
        .as_ref()
        .map(|traceability| summarize_shared_repo_traceability(root, traceability))
}

#[derive(Debug, Clone, Copy)]
struct GoTraceabilityPack;

impl TraceabilityLanguagePack for GoTraceabilityPack {
    fn backward_trace_availability(&self) -> super::traceability_core::BackwardTraceAvailability {
        let Some(go_binary) = discover_go_binary() else {
            return super::traceability_core::BackwardTraceAvailability::unavailable(
                "Go backward trace is unavailable because `go` is not installed through `mise`",
            );
        };
        let Some(go_bin_dir) = go_binary.parent() else {
            return super::traceability_core::BackwardTraceAvailability::unavailable(
                "Go backward trace is unavailable because the installed `go` binary has no parent directory",
            );
        };
        if !go_bin_dir.join("gopls").exists() {
            return super::traceability_core::BackwardTraceAvailability::unavailable(
                "Go backward trace is unavailable because `gopls` is not installed alongside `go`",
            );
        }
        super::traceability_core::BackwardTraceAvailability::default()
    }

    fn build_inputs(
        &self,
        root: &Path,
        source_files: &[PathBuf],
        parsed_repo: &ParsedRepo,
        _parsed_architecture: &ParsedArchitecture,
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> TraceabilityInputs {
        let source_graphs = parse_go_source_graphs(root, source_files);
        let repo_items = collect_repo_items(&source_graphs, file_ownership);
        let mut graph = TraceGraph {
            edges: build_parser_call_edges(&source_graphs),
            root_supports: BTreeMap::new(),
        };
        merge_trace_graph_edges(
            &mut graph.edges,
            build_tool_call_edges(root, &source_graphs),
        );
        graph.root_supports = build_root_supports(parsed_repo, &source_graphs, |path, body| {
            parse_source_graph(path, body)
                .and_then(|graph| graph.items.first().map(|item| item.span.start_line))
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

fn is_go_path(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("go"))
}

#[derive(Default)]
struct GoSurfaceSummary {
    public_items: usize,
    internal_items: usize,
}

impl GoSurfaceSummary {
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
struct GoDependencySummary {
    targets: BTreeMap<String, usize>,
    internal_files: BTreeSet<PathBuf>,
    external_targets: BTreeSet<String>,
}

impl GoDependencySummary {
    fn observe(&mut self, root: &Path, _source_path: &Path, text: &str) {
        let mut parser = Parser::new();
        if parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
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
            let internal_files = resolve_internal_imports(root, &target);
            if internal_files.is_empty() {
                self.external_targets.insert(target);
            } else {
                self.internal_files.extend(internal_files);
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
    if node.kind() == "import_spec"
        && let Some(import_source) = node.child_by_field_name("path")
        && let Ok(text) = import_source.utf8_text(source)
    {
        imports.push(text.trim_matches('"').trim_matches('`').to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_sources(child, source, imports);
    }
}

fn resolve_internal_imports(root: &Path, target: &str) -> BTreeSet<PathBuf> {
    let segments = target
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return BTreeSet::new();
    }

    let mut matches = BTreeSet::new();
    for start in 0..segments.len() {
        let suffix = PathBuf::from_iter(segments[start..].iter().copied());
        matches.extend(find_go_owned_paths(root, &suffix));
        if !matches.is_empty() {
            break;
        }
    }
    matches
}

fn find_go_owned_paths(root: &Path, suffix: &Path) -> BTreeSet<PathBuf> {
    let mut matches = BTreeSet::new();
    let direct_file = root.join(suffix).with_extension("go");
    if direct_file.exists() {
        matches.insert(direct_file);
    }

    let directory = root.join(suffix);
    if directory.is_dir() {
        matches.extend(read_go_files(&directory));
    }
    matches
}

fn read_go_files(directory: &Path) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("go") {
            files.insert(path);
        }
    }
    files
}

#[derive(Debug, Clone)]
struct SourceCallableItem {
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

fn parse_go_source_graphs(
    root: &Path,
    source_files: &[PathBuf],
) -> BTreeMap<PathBuf, ParsedSourceGraph> {
    source_files
        .iter()
        .filter(|path| is_go_path(path))
        .filter_map(|path| {
            let text = super::read_owned_file_text(root, path).ok()?;
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

    let text = super::read_owned_file_text(root, &implementation.location.path).ok()?;
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
) -> BTreeMap<String, BTreeSet<String>> {
    let callable_items = collect_callable_items(source_graphs);
    if callable_items.is_empty() {
        return BTreeMap::new();
    }
    let mut edges = BTreeMap::new();
    merge_trace_graph_edges(
        &mut edges,
        build_go_list_package_edges(root, source_graphs, &callable_items),
    );
    merge_trace_graph_edges(
        &mut edges,
        build_gopls_reference_edges(root, &callable_items),
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
        let Ok(text) = super::read_owned_file_text(root, path) else {
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
                .or_insert_with(BTreeSet::new)
                .insert(candidates[0].clone());
        }
    }
    edges
}

fn build_gopls_reference_edges(
    root: &Path,
    callable_items: &[SourceCallableItem],
) -> BTreeMap<String, BTreeSet<String>> {
    let Some(go_binary) = discover_go_binary() else {
        return BTreeMap::new();
    };
    let Some(go_bin_dir) = go_binary.parent() else {
        return BTreeMap::new();
    };
    let gopls_binary = go_bin_dir.join("gopls");
    if !gopls_binary.exists() {
        return BTreeMap::new();
    }
    let cache_dir =
        std::env::temp_dir().join(format!("special-go-build-cache-{}", std::process::id()));
    let _ = fs::create_dir_all(&cache_dir);
    let grouped_items = group_callable_items_by_path(root, callable_items);
    let path_env = format!(
        "{}:{}",
        go_bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut edges = BTreeMap::<String, BTreeSet<String>>::new();
    for item in callable_items {
        let Some((line, column)) = item_name_position(root, item) else {
            continue;
        };
        let output = Command::new(&gopls_binary)
            .arg("references")
            .arg("-d")
            .arg(format!("{}:{}:{}", item.path.display(), line, column))
            .current_dir(root)
            .env("PATH", &path_env)
            .env("GOCACHE", &cache_dir)
            .output();
        let Ok(output) = output else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for location in parse_gopls_reference_locations(&stdout) {
            let Some(caller) = find_containing_item(&grouped_items, &location.path, location.line)
            else {
                continue;
            };
            if caller.stable_id == item.stable_id {
                continue;
            }
            edges
                .entry(caller.stable_id.clone())
                .or_default()
                .insert(item.stable_id.clone());
        }
    }
    edges
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
            start_line: item.span.start_line,
            start_column: item.span.start_column,
            end_line: item.span.end_line,
            calls: item.calls,
        }));
    }
    items
}

fn group_callable_items_by_path<'a>(
    root: &Path,
    items: &'a [SourceCallableItem],
) -> BTreeMap<PathBuf, Vec<&'a SourceCallableItem>> {
    let mut grouped = BTreeMap::<PathBuf, Vec<&SourceCallableItem>>::new();
    for item in items {
        grouped.entry(item.path.clone()).or_default().push(item);
        let full_path = root.join(&item.path);
        grouped.entry(full_path.clone()).or_default().push(item);
        if let Ok(canonical) = fs::canonicalize(&full_path) {
            grouped.entry(canonical).or_default().push(item);
        }
    }
    grouped
}

fn item_name_position(root: &Path, item: &SourceCallableItem) -> Option<(usize, usize)> {
    let text = super::read_owned_file_text(root, &item.path).ok()?;
    let line = text.lines().nth(item.start_line.saturating_sub(1))?;
    let start = item.start_column.min(line.len());
    let offset = line[start..].find(&item.name)?;
    Some((item.start_line, start + offset + 1))
}

fn find_containing_item<'a>(
    grouped: &'a BTreeMap<PathBuf, Vec<&'a SourceCallableItem>>,
    path: &Path,
    line: usize,
) -> Option<&'a SourceCallableItem> {
    grouped.get(path).and_then(|bucket| {
        bucket
            .iter()
            .copied()
            .find(|item| item.start_line <= line && line <= item.end_line)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GoReferenceLocation {
    path: PathBuf,
    line: usize,
}

fn parse_gopls_reference_locations(stdout: &str) -> Vec<GoReferenceLocation> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, ':');
            let path = parts.next()?;
            let line_number = parts.next()?.parse::<usize>().ok()?;
            Some(GoReferenceLocation {
                path: PathBuf::from(path),
                line: line_number,
            })
        })
        .collect()
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

fn collect_go_import_aliases(text: &str) -> BTreeMap<String, String> {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .is_err()
    {
        return BTreeMap::new();
    }
    let Some(tree) = parser.parse(text, None) else {
        return BTreeMap::new();
    };
    let mut aliases = BTreeMap::new();
    collect_import_aliases(tree.root_node(), text.as_bytes(), &mut aliases);
    aliases
}

fn collect_import_aliases(node: Node<'_>, source: &[u8], aliases: &mut BTreeMap<String, String>) {
    if node.kind() == "import_spec"
        && let Some(import_source) = node.child_by_field_name("path")
        && let Ok(text) = import_source.utf8_text(source)
    {
        let import_path = text.trim_matches('"').trim_matches('`').to_string();
        let alias = explicit_import_alias(node, import_source, source).unwrap_or_else(|| {
            import_path
                .rsplit('/')
                .next()
                .unwrap_or(import_path.as_str())
                .to_string()
        });
        if alias != "_" && alias != "." {
            aliases.insert(alias, import_path);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_aliases(child, source, aliases);
    }
}

fn explicit_import_alias(
    import_spec: Node<'_>,
    import_source: Node<'_>,
    source: &[u8],
) -> Option<String> {
    import_spec
        .child_by_field_name("name")
        .and_then(|name| name.utf8_text(source).ok())
        .map(ToString::to_string)
        .or_else(|| {
            let mut cursor = import_spec.walk();
            import_spec
                .named_children(&mut cursor)
                .find(|child| *child != import_source)
                .and_then(|child| child.utf8_text(source).ok())
                .map(ToString::to_string)
        })
}

fn go_list_packages(root: &Path) -> Option<Vec<GoListPackage>> {
    let go_binary = discover_go_binary()?;
    let cache_dir =
        std::env::temp_dir().join(format!("special-go-build-cache-{}", std::process::id()));
    let _ = fs::create_dir_all(&cache_dir);
    let output = Command::new(go_binary)
        .args(["list", "-json", "./..."])
        .current_dir(root)
        .env("GOCACHE", &cache_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut packages = Vec::new();
    let stream = serde_json::Deserializer::from_slice(&output.stdout).into_iter::<GoListPackage>();
    for package in stream.flatten() {
        packages.push(package);
    }
    (!packages.is_empty()).then_some(packages)
}

fn discover_go_binary() -> Option<PathBuf> {
    let output = Command::new("mise")
        .args(["ls", "--json", "go"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let installs: Vec<MiseGoInstall> = serde_json::from_slice(&output.stdout).ok()?;
    let install = installs
        .into_iter()
        .filter(|install| install.installed)
        .max_by(|left, right| {
            left.active
                .cmp(&right.active)
                .then_with(|| compare_semver(&left.version, &right.version))
        })?;
    let go_binary = install.install_path.join("bin/go");
    go_binary.exists().then_some(go_binary)
}

pub(crate) fn analysis_environment_fingerprint() -> String {
    let Some(go_binary) = discover_go_binary() else {
        return "go=unavailable".to_string();
    };
    let gopls = go_binary
        .parent()
        .map(|dir| dir.join("gopls"))
        .filter(|path| path.exists());
    format!(
        "go={};gopls={}",
        go_binary.display(),
        gopls
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unavailable".to_string())
    )
}

fn compare_semver(left: &str, right: &str) -> std::cmp::Ordering {
    let parse = |value: &str| {
        value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(left).cmp(&parse(right))
}

#[derive(Deserialize)]
struct MiseGoInstall {
    version: String,
    install_path: PathBuf,
    installed: bool,
    #[serde(default)]
    active: bool,
}

#[derive(Deserialize)]
struct GoListPackage {
    #[serde(rename = "ImportPath")]
    import_path: String,
    #[serde(rename = "Dir")]
    dir: PathBuf,
    #[serde(rename = "GoFiles", default)]
    go_files: Vec<String>,
}

fn is_test_file_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with("_test.go"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parse_source_graph;

    #[test]
    fn collect_go_import_aliases_tracks_explicit_aliases() {
        let aliases =
            collect_go_import_aliases("package app\n\nimport l \"example.com/demo/left\"\n");
        assert_eq!(
            aliases.get("l").map(String::as_str),
            Some("example.com/demo/left")
        );
    }

    #[test]
    fn build_tool_call_edges_resolves_go_import_alias_targets() {
        let root =
            std::env::temp_dir().join(format!("special-go-tool-edges-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("app")).expect("app dir should be created");
        fs::create_dir_all(root.join("left")).expect("left dir should be created");
        fs::create_dir_all(root.join("right")).expect("right dir should be created");
        fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
            .expect("go.mod should be written");
        fs::write(
            root.join("app/main.go"),
            "// @fileimplements DEMO\npackage app\n\nimport l \"example.com/demo/left\"\n\nfunc LiveImpl() int {\n    return helper() + l.SharedValue()\n}\n\nfunc helper() int {\n    return 1\n}\n",
        )
        .expect("main.go should be written");
        fs::write(
            root.join("left/shared.go"),
            "// @fileimplements LEFT\npackage left\n\nfunc SharedValue() int {\n    return 1\n}\n",
        )
        .expect("left/shared.go should be written");
        fs::write(
            root.join("right/shared.go"),
            "// @fileimplements RIGHT\npackage right\n\nfunc SharedValue() int {\n    return 2\n}\n",
        )
        .expect("right/shared.go should be written");
        fs::write(
            root.join("app/main_test.go"),
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
            let text =
                fs::read_to_string(root.join(&path)).expect("fixture file should be readable");
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
            &fs::read_to_string(root.join("app/main.go")).expect("main.go should be readable"),
        );
        assert_eq!(
            aliases.get("l").map(String::as_str),
            Some("example.com/demo/left")
        );

        let packages = go_list_packages(&root).expect("go list should find packages");
        assert!(
            packages
                .iter()
                .any(|package| package.import_path == "example.com/demo/left")
        );

        let callable_items = collect_callable_items(&source_graphs);
        let canonical_root = fs::canonicalize(&root).expect("root should canonicalize");
        let mut file_package_paths = BTreeMap::<PathBuf, String>::new();
        for package in &packages {
            for file_name in &package.go_files {
                let full_path = package.dir.join(file_name);
                let canonical_full_path =
                    fs::canonicalize(&full_path).expect("package file should canonicalize");
                if let Ok(relative) = canonical_full_path
                    .strip_prefix(&canonical_root)
                    .or_else(|_| full_path.strip_prefix(&root))
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

        let edges = build_tool_call_edges(&root, &source_graphs);
        let live_impl = "app/main.go:app::LiveImpl:6".to_string();
        let left_shared = "left/shared.go:left::shared::SharedValue:4".to_string();
        assert_eq!(edges.get(&live_impl), Some(&BTreeSet::from([left_shared])));

        let _ = fs::remove_dir_all(root);
    }
}
