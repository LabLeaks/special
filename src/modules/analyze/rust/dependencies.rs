/**
@module SPECIAL.MODULES.ANALYZE.RUST.DEPENDENCIES
Extracts Rust-specific `use`-path dependency evidence from owned Rust implementation without inferring architecture verdicts.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.DEPENDENCIES
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use syn::{Item, ItemUse, visit::Visit};

use super::super::coupling::ModuleCouplingInput;
use super::use_tree::flatten_use_tree;
use crate::model::ModuleDependencySummary;
use crate::modules::analyze::build_dependency_summary;

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
        build_dependency_summary(&self.targets)
    }

    pub(super) fn coupling_input(&self) -> ModuleCouplingInput {
        ModuleCouplingInput {
            internal_files: self.internal_files.clone(),
            external_targets: self.external_targets.clone(),
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
