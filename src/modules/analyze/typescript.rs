/**
@module SPECIAL.MODULES.ANALYZE.TYPESCRIPT
Applies the built-in TypeScript analysis provider to owned TypeScript implementation while keeping TypeScript parsing out of the language-agnostic analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.TYPESCRIPT
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

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

pub(super) fn summarize_typescript_item_signals(
    path: &Path,
    text: &str,
) -> ModuleItemSignalsSummary {
    parse_source_graph(path, text)
        .map(|graph| summarize_source_item_signals(&graph.items))
        .unwrap_or_default()
}

pub(super) fn analyze_module(
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<ProviderModuleAnalysis> {
    let mut surface = TypeScriptSurfaceSummary::default();
    let mut owned_items = Vec::new();
    let mut dependencies = TypeScriptDependencySummary::default();

    visit_owned_texts(root, implementations, file_ownership, |path, text| {
        if !is_typescript_path(path) {
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
