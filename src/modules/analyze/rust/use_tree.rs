/**
@module SPECIAL.MODULES.ANALYZE.RUST.USE_TREE
Shared Rust `use`-tree flattening helpers for dependency and traceability analysis.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.USE_TREE
use std::collections::BTreeMap;

use syn::UseTree;

pub(super) fn flatten_use_tree(tree: &UseTree) -> Vec<String> {
    let mut paths = Vec::new();
    flatten_use_tree_with_prefix(tree, Vec::new(), &mut paths);
    paths
}

pub(super) fn collect_use_aliases(tree: &UseTree) -> BTreeMap<String, Vec<String>> {
    let mut aliases = BTreeMap::new();
    collect_use_aliases_with_prefix(tree, Vec::new(), &mut aliases);
    aliases
}

fn flatten_use_tree_with_prefix(tree: &UseTree, prefix: Vec<String>, paths: &mut Vec<String>) {
    match tree {
        UseTree::Path(path) => {
            let mut next_prefix = prefix;
            next_prefix.push(path.ident.to_string());
            flatten_use_tree_with_prefix(&path.tree, next_prefix, paths);
        }
        UseTree::Name(name) => {
            let mut full_path = prefix;
            full_path.push(name.ident.to_string());
            paths.push(full_path.join("::"));
        }
        UseTree::Rename(rename) => {
            let mut full_path = prefix;
            full_path.push(rename.ident.to_string());
            paths.push(full_path.join("::"));
        }
        UseTree::Glob(_) => {}
        UseTree::Group(group) => {
            for item in &group.items {
                flatten_use_tree_with_prefix(item, prefix.clone(), paths);
            }
        }
    }
}

fn collect_use_aliases_with_prefix(
    tree: &UseTree,
    prefix: Vec<String>,
    aliases: &mut BTreeMap<String, Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            let mut next_prefix = prefix;
            next_prefix.push(path.ident.to_string());
            collect_use_aliases_with_prefix(&path.tree, next_prefix, aliases);
        }
        UseTree::Name(name) => {
            let mut full_path = prefix;
            let alias = name.ident.to_string();
            full_path.push(alias.clone());
            aliases.entry(alias).or_default().push(full_path.join("::"));
        }
        UseTree::Rename(rename) => {
            let mut full_path = prefix;
            full_path.push(rename.ident.to_string());
            aliases
                .entry(rename.rename.to_string())
                .or_default()
                .push(full_path.join("::"));
        }
        UseTree::Glob(_) => {}
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_aliases_with_prefix(item, prefix.clone(), aliases);
            }
        }
    }
}
