/**
@module SPECIAL.ID_PATH
Shared helpers for dotted identifier parent traversal used by both spec-tree and module-tree materialization and linting.
*/
// @fileimplements SPECIAL.ID_PATH
use std::collections::BTreeSet;

pub(crate) fn immediate_parent_id(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(parent, _)| parent)
}

pub(crate) fn nearest_visible_parent_id(
    id: &str,
    visible_ids: &BTreeSet<String>,
) -> Option<String> {
    let mut parent = immediate_parent_id(id);
    while let Some(candidate) = parent {
        if visible_ids.contains(candidate) {
            return Some(candidate.to_string());
        }
        parent = immediate_parent_id(candidate);
    }
    None
}
