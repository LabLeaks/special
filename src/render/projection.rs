/**
@module SPECIAL.RENDER.PROJECTION
Projects materialized specs and modules into the visible verbose or non-verbose shape shared by all render backends. This module does not format text or HTML.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION
use crate::model::{
    ArchitectureCoverageSummary, ModuleCoverageSummary, ModuleDocument, ModuleNode, SpecDocument,
    SpecNode,
};

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
            analysis: document
                .analysis
                .clone()
                .map(strip_module_document_analysis_paths),
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
    if let Some(analysis) = &mut node.analysis
        && let Some(coverage) = &mut analysis.coverage
    {
        strip_module_coverage_paths(coverage);
    }
    node.children = node
        .children
        .into_iter()
        .map(strip_module_implementation_bodies)
        .collect();
    node
}

fn strip_module_document_analysis_paths(
    mut analysis: crate::model::ArchitectureAnalysisSummary,
) -> crate::model::ArchitectureAnalysisSummary {
    if let Some(coverage) = &mut analysis.coverage {
        strip_architecture_coverage_paths(coverage);
    }
    analysis
}

fn strip_architecture_coverage_paths(coverage: &mut ArchitectureCoverageSummary) {
    coverage.uncovered_paths.clear();
    coverage.weak_paths.clear();
}

fn strip_module_coverage_paths(coverage: &mut ModuleCoverageSummary) {
    coverage.covered_paths.clear();
    coverage.weak_paths.clear();
}
