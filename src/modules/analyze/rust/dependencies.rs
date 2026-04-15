/**
@module SPECIAL.MODULES.ANALYZE.RUST.DEPENDENCIES
Extracts Rust-specific `use`-path dependency evidence from owned Rust implementation without inferring architecture verdicts.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.DEPENDENCIES
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::{Item, ItemUse, UseTree, visit::Visit};

use super::super::coupling::ModuleCouplingInput;
use crate::model::{ModuleDependencySummary, ModuleDependencyTargetSummary};

#[derive(Debug, Default)]
pub(super) struct RustDependencySummary {
    targets: BTreeMap<String, usize>,
    internal_files: BTreeSet<PathBuf>,
    external_targets: BTreeSet<String>,
}

impl RustDependencySummary {
    pub(super) fn observe(&mut self, root: &Path, source_path: &Path, text: &str) {
        if let Ok(file) = syn::parse_file(text) {
            let mut collector = UseCollector {
                root,
                source_path,
                targets: &mut self.targets,
                internal_files: &mut self.internal_files,
                external_targets: &mut self.external_targets,
            };
            collector.visit_file(&file);
            return;
        }

        if let Ok(item) = syn::parse_str::<Item>(text) {
            let mut collector = UseCollector {
                root,
                source_path,
                targets: &mut self.targets,
                internal_files: &mut self.internal_files,
                external_targets: &mut self.external_targets,
            };
            collector.visit_item(&item);
        }
    }

    pub(super) fn summary(&self) -> ModuleDependencySummary {
        let reference_count = self.targets.values().sum();
        let distinct_targets = self.targets.len();
        let targets = self
            .targets
            .iter()
            .map(|(path, count)| ModuleDependencyTargetSummary {
                path: path.clone(),
                count: *count,
            })
            .collect();
        ModuleDependencySummary {
            reference_count,
            distinct_targets,
            targets,
        }
    }

    pub(super) fn coupling_input(&self) -> ModuleCouplingInput {
        ModuleCouplingInput {
            internal_files: self.internal_files.clone(),
            external_target_count: self.external_targets.len(),
        }
    }
}

struct UseCollector<'a> {
    root: &'a Path,
    source_path: &'a Path,
    targets: &'a mut BTreeMap<String, usize>,
    internal_files: &'a mut BTreeSet<PathBuf>,
    external_targets: &'a mut BTreeSet<String>,
}

impl Visit<'_> for UseCollector<'_> {
    fn visit_item_use(&mut self, node: &ItemUse) {
        for path in flatten_use_tree(&node.tree) {
            *self.targets.entry(path.clone()).or_default() += 1;
            if let Some(file) = resolve_internal_file(self.root, self.source_path, &path) {
                self.internal_files.insert(file);
            } else if !is_internal_target(&path) {
                self.external_targets.insert(path);
            }
        }
        syn::visit::visit_item_use(self, node);
    }
}

fn flatten_use_tree(tree: &UseTree) -> Vec<String> {
    let mut paths = Vec::new();
    flatten_use_tree_with_prefix(tree, String::new(), &mut paths);
    paths
}

fn flatten_use_tree_with_prefix(tree: &UseTree, prefix: String, paths: &mut Vec<String>) {
    match tree {
        UseTree::Path(path) => {
            let next_prefix = append_segment(&prefix, &path.ident.to_string());
            flatten_use_tree_with_prefix(&path.tree, next_prefix, paths);
        }
        UseTree::Name(name) => paths.push(append_segment(&prefix, &name.ident.to_string())),
        UseTree::Rename(rename) => paths.push(append_segment(&prefix, &rename.ident.to_string())),
        UseTree::Glob(_) => paths.push(format!("{prefix}::*")),
        UseTree::Group(group) => {
            for item in &group.items {
                flatten_use_tree_with_prefix(item, prefix.clone(), paths);
            }
        }
    }
}

fn append_segment(prefix: &str, segment: &str) -> String {
    if prefix.is_empty() {
        segment.to_string()
    } else {
        format!("{prefix}::{segment}")
    }
}

fn is_internal_target(path: &str) -> bool {
    path.starts_with("crate::") || path.starts_with("self::") || path.starts_with("super::")
}

fn resolve_internal_file(root: &Path, source_path: &Path, path: &str) -> Option<PathBuf> {
    let mut segments = path.split("::");
    let anchor = segments.next()?;
    let remainder: Vec<&str> = segments.collect();
    if remainder.is_empty() {
        return None;
    }

    let source_dir = root
        .join(source_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.to_path_buf());
    let module_dir = match anchor {
        "crate" => root.to_path_buf(),
        "self" => source_dir,
        "super" => {
            let mut dir = source_dir;
            let mut remaining = remainder.as_slice();
            while matches!(remaining.first(), Some(segment) if *segment == "super") {
                dir = dir.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
                remaining = &remaining[1..];
            }
            return resolve_internal_segments(root, dir, remaining);
        }
        _ => return None,
    };

    resolve_internal_segments(root, module_dir, &remainder)
}

fn resolve_internal_segments(
    _root: &Path,
    module_dir: PathBuf,
    remainder: &[&str],
) -> Option<PathBuf> {
    for prefix_len in (1..=remainder.len()).rev() {
        let candidate = &remainder[..prefix_len];
        let file_candidate = module_dir.join(candidate.join("/")).with_extension("rs");
        if file_candidate.exists() {
            return Some(file_candidate);
        }

        let mod_candidate = module_dir.join(candidate.join("/")).join("mod.rs");
        if mod_candidate.exists() {
            return Some(mod_candidate);
        }
    }
    None
}
