/**
@module SPECIAL.MODULES.ANALYZE.GO
Applies the built-in Go analysis provider to owned Go implementation while keeping Go parsing out of the language-agnostic analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.GO
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tree_sitter::{Node, Parser};

use crate::model::{
    ImplementRef, ModuleDependencySummary, ModuleDependencyTargetSummary, ModuleItemSignalsSummary,
    ModuleMetricsSummary,
};
use crate::syntax::parse_source_graph;

use super::coupling::ModuleCouplingInput;
use super::source_item_signals::summarize_source_item_signals;
use super::{FileOwnership, ProviderModuleAnalysis, visit_owned_texts};

pub(super) fn summarize_go_item_signals(path: &Path, text: &str) -> ModuleItemSignalsSummary {
    parse_source_graph(path, text)
        .map(|graph| summarize_source_item_signals(&graph.items))
        .unwrap_or_default()
}

pub(super) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<ProviderModuleAnalysis> {
    let mut surface = GoSurfaceSummary::default();
    let mut owned_items = Vec::new();
    let mut dependencies = GoDependencySummary::default();

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_go_path(path) {
            return;
        }
        if let Some(graph) = parse_source_graph(path, text) {
            surface.observe(&graph.items);
            owned_items.extend(graph.items);
        }
        dependencies.observe(root, path, text);
    })?;

    Ok(ProviderModuleAnalysis {
        metrics: ModuleMetricsSummary {
            public_items: surface.public_items,
            internal_items: surface.internal_items,
            ..ModuleMetricsSummary::default()
        },
        item_signals: Some(summarize_source_item_signals(&owned_items)),
        coupling: Some(dependencies.coupling_input()),
        dependencies: Some(dependencies.summary()),
        ..ProviderModuleAnalysis::default()
    })
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
        let reference_count = self.targets.values().sum();
        let distinct_targets = self.targets.len();
        ModuleDependencySummary {
            reference_count,
            distinct_targets,
            targets: self
                .targets
                .iter()
                .map(|(path, count)| ModuleDependencyTargetSummary {
                    path: path.clone(),
                    count: *count,
                })
                .collect(),
        }
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
