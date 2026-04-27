/**
@module SPECIAL.RENDER.PROJECTION.MODULE
Projects architecture module documents and module analysis into backend-ready verbose or non-verbose shapes.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE
use crate::model::{ModuleCoverageSummary, ModuleDocument, ModuleNode};

pub(in crate::render) fn project_module_document(
    document: &ModuleDocument,
    verbose: bool,
) -> ModuleDocument {
    if verbose {
        document.clone()
    } else {
        let strip_tree_analysis = document.metrics.is_some() && !document.scoped;
        ModuleDocument {
            metrics: document.metrics.clone(),
            scoped: document.scoped,
            nodes: document
                .nodes
                .iter()
                .cloned()
                .map(|node| strip_module_non_verbose_detail(node, strip_tree_analysis))
                .collect(),
        }
    }
}

fn strip_module_implementation_bodies(mut node: ModuleNode) -> ModuleNode {
    for implementation in &mut node.implements {
        implementation.body_location = None;
        implementation.body = None;
    }
    for application in &mut node.pattern_applications {
        application.body_location = None;
        application.body = None;
    }
    node.children = node
        .children
        .into_iter()
        .map(strip_module_implementation_bodies)
        .collect();
    node
}

fn strip_module_non_verbose_detail(mut node: ModuleNode, strip_analysis: bool) -> ModuleNode {
    node = strip_module_implementation_bodies(node);
    if strip_analysis {
        node.analysis = None;
    } else if let Some(analysis) = &mut node.analysis {
        if let Some(coverage) = &mut analysis.coverage {
            strip_module_coverage_paths(coverage);
        }
        strip_module_analysis_detail(analysis);
    }
    node.children = node
        .children
        .into_iter()
        .map(|child| strip_module_non_verbose_detail(child, strip_analysis))
        .collect();
    node
}

fn strip_module_coverage_paths(_coverage: &mut ModuleCoverageSummary) {}

fn strip_module_analysis_detail(analysis: &mut crate::model::ModuleAnalysisSummary) {
    if let Some(item_signals) = &mut analysis.item_signals {
        item_signals.connected_items.clear();
        item_signals.outbound_heavy_items.clear();
        item_signals.isolated_items.clear();
        item_signals.unreached_items.clear();
        item_signals.highest_complexity_items.clear();
        item_signals.parameter_heavy_items.clear();
        item_signals.stringly_boundary_items.clear();
        item_signals.panic_heavy_items.clear();
    }
    if let Some(traceability) = &mut analysis.traceability {
        traceability.current_spec_items.clear();
        traceability.planned_only_items.clear();
        traceability.deprecated_only_items.clear();
        traceability.file_scoped_only_items.clear();
        traceability.unverified_test_items.clear();
        traceability.statically_mediated_items.clear();
        traceability.unexplained_items.clear();
    }
    if let Some(dependencies) = &mut analysis.dependencies {
        dependencies.targets.clear();
    }
}
