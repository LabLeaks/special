/**
@module SPECIAL.INDEX.MATERIALIZE
Builds the visible spec tree from parsed annotations and support attachments. This module does not parse source files or emit lint diagnostics.
*/
// @fileimplements SPECIAL.INDEX.MATERIALIZE
use std::collections::{BTreeMap, BTreeSet};

use crate::model::{NodeKind, ParsedRepo, SpecDecl, SpecDocument, SpecFilter, SpecNode, VerifyRef};

#[derive(Debug, Clone)]
struct FlatSpecNode {
    decl: SpecDecl,
    verifies: Vec<VerifyRef>,
    attests: Vec<crate::model::AttestRef>,
}

pub(super) fn materialize_spec(parsed: &ParsedRepo, filter: SpecFilter) -> SpecDocument {
    let mut flat_nodes: BTreeMap<String, FlatSpecNode> = BTreeMap::new();

    for spec in &parsed.specs {
        flat_nodes
            .entry(spec.id.clone())
            .or_insert_with(|| FlatSpecNode {
                decl: spec.clone(),
                verifies: Vec::new(),
                attests: Vec::new(),
            });
    }

    for verify in &parsed.verifies {
        if verify.body.is_some()
            && let Some(node) = flat_nodes.get_mut(&verify.spec_id)
        {
            node.verifies.push(verify.clone());
        }
    }

    for attest in &parsed.attests {
        if let Some(node) = flat_nodes.get_mut(&attest.spec_id) {
            node.attests.push(attest.clone());
        }
    }

    let directly_visible_ids: BTreeSet<String> = flat_nodes
        .values()
        .filter(|node| {
            node.decl.kind() == NodeKind::Spec
                && filter.matches(
                    node.decl.kind(),
                    node.decl.plan().is_planned(),
                    node.verifies.is_empty(),
                    node.attests.is_empty(),
                )
        })
        .map(|node| node.decl.id.clone())
        .collect();

    let mut visible_ids = directly_visible_ids.clone();
    for id in &directly_visible_ids {
        let mut parent = immediate_parent(id);
        while let Some(candidate) = parent {
            if flat_nodes.contains_key(candidate) {
                visible_ids.insert(candidate.to_string());
            }
            parent = immediate_parent(candidate);
        }
    }

    let mut children_map: BTreeMap<Option<String>, Vec<String>> = BTreeMap::new();
    for id in &visible_ids {
        let visible_parent = nearest_visible_parent(id, &visible_ids);
        children_map
            .entry(visible_parent)
            .or_default()
            .push(id.clone());
    }

    for children in children_map.values_mut() {
        children.sort();
    }

    let mut nodes = build_children(None, &children_map, &flat_nodes);

    if let Some(scope) = filter.scope.as_deref() {
        nodes = scoped_nodes(nodes, scope);
    }

    SpecDocument { nodes }
}

impl SpecFilter {
    fn matches(
        &self,
        kind: NodeKind,
        planned: bool,
        has_no_verifies: bool,
        has_no_attests: bool,
    ) -> bool {
        if !self.include_planned && planned {
            return false;
        }
        if self.unsupported_only
            && (kind != NodeKind::Spec || planned || !has_no_verifies || !has_no_attests)
        {
            return false;
        }
        true
    }
}

fn nearest_visible_parent(id: &str, visible_ids: &BTreeSet<String>) -> Option<String> {
    let mut parent = immediate_parent(id);
    while let Some(candidate) = parent {
        if visible_ids.contains(candidate) {
            return Some(candidate.to_string());
        }
        parent = immediate_parent(candidate);
    }
    None
}

fn build_children(
    parent: Option<String>,
    children_map: &BTreeMap<Option<String>, Vec<String>>,
    flat_nodes: &BTreeMap<String, FlatSpecNode>,
) -> Vec<SpecNode> {
    let Some(ids) = children_map.get(&parent) else {
        return Vec::new();
    };

    ids.iter()
        .filter_map(|id| flat_nodes.get(id))
        .map(|node| {
            SpecNode::new(
                node.decl.clone(),
                node.verifies.clone(),
                node.attests.clone(),
                build_children(Some(node.decl.id.clone()), children_map, flat_nodes),
            )
        })
        .collect()
}

fn immediate_parent(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(parent, _)| parent)
}

fn scoped_nodes(nodes: Vec<SpecNode>, scope: &str) -> Vec<SpecNode> {
    nodes
        .into_iter()
        .find_map(|node| find_scoped_node(node, scope))
        .into_iter()
        .collect()
}

fn find_scoped_node(node: SpecNode, scope: &str) -> Option<SpecNode> {
    if node.id == scope {
        return Some(node);
    }

    node.children
        .into_iter()
        .find_map(|child| find_scoped_node(child, scope))
}
