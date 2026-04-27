/**
@module SPECIAL.RENDER.PROJECTION.PATTERN
Projects pattern documents into verbose or non-verbose shapes by hiding source application bodies unless explicitly requested.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.PATTERN
use crate::model::{PatternDocument, PatternNode};

pub(in crate::render) fn project_pattern_document(
    document: &PatternDocument,
    verbose: bool,
) -> PatternDocument {
    if verbose {
        return document.clone();
    }
    PatternDocument {
        metrics: document.metrics.clone(),
        scoped: document.scoped,
        patterns: document
            .patterns
            .iter()
            .cloned()
            .map(strip_pattern_verbose_detail)
            .collect(),
    }
}

fn strip_pattern_verbose_detail(mut node: PatternNode) -> PatternNode {
    if let Some(definition) = &mut node.definition {
        definition.text.clear();
    }
    for application in &mut node.applications {
        application.body_location = None;
        application.body = None;
    }
    node.children = node
        .children
        .into_iter()
        .map(strip_pattern_verbose_detail)
        .collect();
    node
}
