/**
@module SPECIAL.MODULES.MATERIALIZE
Builds the visible architecture tree from parsed module declarations and `@implements` attachments. This module does not read source files or emit diagnostics.
*/
// @fileimplements SPECIAL.MODULES.MATERIALIZE
use std::collections::{BTreeMap, BTreeSet};

use crate::id_path::{immediate_parent_id, nearest_visible_parent_id};
use crate::model::{
    ImplementRef, ModuleAnalysisSummary, ModuleDecl, ModuleDocument, ModuleFilter, ModuleNode,
    ParsedArchitecture,
};

#[derive(Debug, Clone)]
struct FlatModuleNode {
    decl: ModuleDecl,
    implements: Vec<ImplementRef>,
}

pub(super) fn build_module_document(
    parsed: &ParsedArchitecture,
    filter: ModuleFilter,
    module_analysis: Option<&BTreeMap<String, ModuleAnalysisSummary>>,
) -> ModuleDocument {
    let mut flat_nodes: BTreeMap<String, FlatModuleNode> = BTreeMap::new();

    for module in &parsed.modules {
        flat_nodes
            .entry(module.id.clone())
            .or_insert_with(|| FlatModuleNode {
                decl: module.clone(),
                implements: Vec::new(),
            });
    }

    for implementation in &parsed.implements {
        if let Some(node) = flat_nodes.get_mut(&implementation.module_id) {
            node.implements.push(implementation.clone());
        }
    }

    let directly_visible_ids: BTreeSet<String> = flat_nodes
        .values()
        .filter(|node| {
            filter.matches(
                node.decl.kind(),
                node.decl.plan().is_planned(),
                node.implements.is_empty(),
            )
        })
        .map(|node| node.decl.id.clone())
        .collect();

    let mut visible_ids = directly_visible_ids.clone();
    for id in &directly_visible_ids {
        let mut parent = immediate_parent_id(id);
        while let Some(candidate) = parent {
            if flat_nodes.contains_key(candidate) {
                visible_ids.insert(candidate.to_string());
            }
            parent = immediate_parent_id(candidate);
        }
    }

    let mut children_map: BTreeMap<Option<String>, Vec<String>> = BTreeMap::new();
    for id in &visible_ids {
        let visible_parent = nearest_visible_parent_id(id, &visible_ids);
        children_map
            .entry(visible_parent)
            .or_default()
            .push(id.clone());
    }
    for children in children_map.values_mut() {
        children.sort();
    }

    let mut nodes = build_children(None, &children_map, &flat_nodes, module_analysis);
    if let Some(scope) = filter.scope.as_deref() {
        nodes = scoped_nodes(nodes, scope);
    }

    ModuleDocument {
        metrics: None,
        scoped: filter.scope.is_some(),
        nodes,
    }
}

impl ModuleFilter {
    fn matches(
        &self,
        kind: crate::model::ArchitectureKind,
        planned: bool,
        has_no_implements: bool,
    ) -> bool {
        match self.state {
            crate::model::DeclaredStateFilter::All => {}
            crate::model::DeclaredStateFilter::Current if planned => return false,
            crate::model::DeclaredStateFilter::Planned if !planned => return false,
            crate::model::DeclaredStateFilter::Current
            | crate::model::DeclaredStateFilter::Planned => {}
        }
        if self.unimplemented_only
            && (kind != crate::model::ArchitectureKind::Module || planned || !has_no_implements)
        {
            return false;
        }
        true
    }
}

fn build_children(
    parent: Option<String>,
    children_map: &BTreeMap<Option<String>, Vec<String>>,
    flat_nodes: &BTreeMap<String, FlatModuleNode>,
    module_analysis: Option<&BTreeMap<String, ModuleAnalysisSummary>>,
) -> Vec<ModuleNode> {
    let mut result = Vec::new();

    if let Some(children) = children_map.get(&parent) {
        for child_id in children {
            if let Some(node) = flat_nodes.get(child_id) {
                result.push(ModuleNode::new(
                    node.decl.clone(),
                    node.implements.clone(),
                    module_analysis
                        .and_then(|analysis| analysis.get(child_id))
                        .cloned(),
                    build_children(
                        Some(child_id.clone()),
                        children_map,
                        flat_nodes,
                        module_analysis,
                    ),
                ));
            }
        }
    }

    result
}

fn scoped_nodes(nodes: Vec<ModuleNode>, scope: &str) -> Vec<ModuleNode> {
    for node in &nodes {
        if node.id == scope {
            return vec![node.clone()];
        }
        let scoped_children = scoped_nodes(node.children.clone(), scope);
        if !scoped_children.is_empty() {
            return scoped_children;
        }
    }
    Vec::new()
}
