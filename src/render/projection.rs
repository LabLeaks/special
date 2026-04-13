/**
@module SPECIAL.RENDER.PROJECTION
Projects materialized specs and modules into the visible verbose or non-verbose shape shared by all render backends. This module does not format text or HTML.
*/
// @implements SPECIAL.RENDER.PROJECTION
use crate::model::{ModuleDocument, ModuleNode, SpecDocument, SpecNode};

pub(super) fn project_document(document: &SpecDocument, verbose: bool) -> SpecDocument {
    if verbose {
        document.clone()
    } else {
        SpecDocument {
            nodes: document
                .nodes
                .iter()
                .cloned()
                .map(strip_node_support_bodies)
                .collect(),
        }
    }
}

pub(super) fn project_module_document(document: &ModuleDocument, verbose: bool) -> ModuleDocument {
    if verbose {
        document.clone()
    } else {
        ModuleDocument {
            nodes: document
                .nodes
                .iter()
                .cloned()
                .map(strip_module_implementation_bodies)
                .collect(),
        }
    }
}

fn strip_node_support_bodies(mut node: SpecNode) -> SpecNode {
    for verify in &mut node.verifies {
        verify.body_location = None;
        verify.body = None;
    }
    for attest in &mut node.attests {
        attest.body = None;
    }
    node.children = node
        .children
        .into_iter()
        .map(strip_node_support_bodies)
        .collect();
    node
}

fn strip_module_implementation_bodies(mut node: ModuleNode) -> ModuleNode {
    for implementation in &mut node.implements {
        implementation.body_location = None;
        implementation.body = None;
    }
    node.children = node
        .children
        .into_iter()
        .map(strip_module_implementation_bodies)
        .collect();
    node
}
