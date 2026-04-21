/**
@module SPECIAL.RENDER.PROJECTION
Projects materialized specs and modules into the visible verbose or non-verbose shape shared by all render backends. This module does not format text or HTML.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION
mod module_analysis;

use crate::model::{
    ArchitectureRepoSignalsSummary, ModuleCoverageSummary, ModuleDocument, ModuleNode,
    RepoDocument, SpecDocument, SpecNode,
};

pub(super) use self::module_analysis::{
    ProjectedArchitectureTraceability, ProjectedCount, ProjectedExplanation, ProjectedMetaLine,
    ProjectedModuleAnalysis, ProjectedRepoSignals, project_module_analysis_view,
    project_repo_signals_view, project_repo_traceability_view,
};

pub(super) fn project_document(document: &SpecDocument, verbose: bool) -> SpecDocument {
    if verbose {
        document.clone()
    } else {
        SpecDocument {
            metrics: document.metrics.clone(),
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

pub(super) fn project_repo_document(document: &RepoDocument, verbose: bool) -> RepoDocument {
    if verbose {
        document.clone()
    } else {
        RepoDocument {
            metrics: document.metrics.clone(),
            analysis: document
                .analysis
                .clone()
                .map(strip_repo_document_analysis_paths),
        }
    }
}

pub(super) fn project_repo_document_json(document: &RepoDocument, verbose: bool) -> RepoDocument {
    if verbose {
        document.clone()
    } else {
        RepoDocument {
            metrics: document.metrics.clone(),
            analysis: document
                .analysis
                .clone()
                .map(strip_repo_json_analysis_paths),
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

fn strip_repo_document_analysis_paths(
    mut analysis: crate::model::ArchitectureAnalysisSummary,
) -> crate::model::ArchitectureAnalysisSummary {
    if let Some(repo_signals) = &mut analysis.repo_signals {
        strip_repo_signal_paths(repo_signals);
    }
    if let Some(traceability) = &mut analysis.traceability {
        strip_repo_traceability_detail(traceability);
    }
    analysis
}

fn strip_repo_json_analysis_paths(
    mut analysis: crate::model::ArchitectureAnalysisSummary,
) -> crate::model::ArchitectureAnalysisSummary {
    if let Some(repo_signals) = &mut analysis.repo_signals {
        strip_repo_signal_paths(repo_signals);
    }
    if let Some(traceability) = &mut analysis.traceability {
        strip_repo_traceability_detail(traceability);
    }
    analysis
}

fn strip_repo_signal_paths(repo_signals: &mut ArchitectureRepoSignalsSummary) {
    repo_signals.unowned_item_details.clear();
}

fn strip_repo_traceability_detail(
    traceability: &mut crate::model::ArchitectureTraceabilitySummary,
) {
    traceability.current_spec_items.clear();
    traceability.planned_only_items.clear();
    traceability.deprecated_only_items.clear();
    traceability.file_scoped_only_items.clear();
    traceability.unverified_test_items.clear();
    traceability.statically_mediated_items.clear();
    traceability.unexplained_items.clear();
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
