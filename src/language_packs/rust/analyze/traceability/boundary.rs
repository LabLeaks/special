/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.BOUNDARY
Defines the exactness-critical Rust scoped traceability boundary, including projected items, retained context items, and repo-item classification.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.BOUNDARY
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::model::ModuleItemKind;
use crate::modules::analyze::{
    FileOwnership,
    traceability_core::{
        ProjectedTraceabilityContract, ProjectedTraceabilityReference, TraceGraph,
        TraceabilityOwnedItem, build_projected_traceability_contract,
        build_projected_traceability_reference_from_projected_items, owned_module_ids_for_path,
    },
};
#[cfg(test)]
use crate::modules::analyze::traceability_core::use_rust_reference_traceability_kernel_for_tests;
use crate::syntax::ParsedSourceGraph;

use super::RustMediatedReason;

/// Rust-scoped traceability separates projected output items from semantic context.
///
/// `projected_item_ids` are the actual items the scoped command should report.
/// `context_items` are the transitive module-context closure needed to preserve module-backed and
/// module-connected semantics for those projected items.
///
/// With this split, the scoped Rust summary is intended to match the
/// full-summary projection directly while the exact-contract/reference harness
/// reasons about a smaller reverse-closure target set derived from the full
/// graph.
pub(super) struct ScopedTraceabilityBoundary {
    /// Items in the requested display scope. These are the `target` set in the
    /// projected output layer of the public shared theorem family.
    pub(super) projected_item_ids: BTreeSet<String>,
    /// Retained working items for the scoped analysis context.
    pub(super) context_items: Vec<TraceabilityOwnedItem>,
    /// Semantic seeds used to ask rust-analyzer for reverse-reachable callers.
    pub(super) seed_ids: BTreeSet<String>,
}

/// Explicit exactness contract for the Rust scoped boundary.
///
/// This is the precise claim the scoped Rust adapter is trying to make:
/// if the full Rust traceability graph is analyzed and then projected to the
/// requested scope, the result should match analyzing only the exact reverse
/// closure induced by the support-backed projected items and then projecting.
///
/// That is intentionally narrower than the broad working reverse-walk seed set.
/// The working set is operational; this contract is the theorem claim.
pub(super) type ScopedTraceabilityContract = ProjectedTraceabilityContract;

/// Slow exact reverse-closure reference derived from the full graph for the
/// current scoped contract.
///
/// This is the implementation-shaped bridge to the stronger Lean theorem:
/// given a declared target set, what exact reverse closure does the full graph
/// actually induce for those targets?
#[cfg_attr(not(test), allow(dead_code))]
pub(super) type ScopedTraceabilityReference = ProjectedTraceabilityReference;

impl ScopedTraceabilityBoundary {
    /// Broad working seed set used to ask rust-analyzer for reverse callers.
    ///
    /// This is intentionally allowed to be larger than the exact preserved
    /// target set because it is an operational approximation for collecting the
    /// raw semantic graph.
    pub(super) fn working_contract(&self) -> ScopedTraceabilityContract {
        ScopedTraceabilityContract {
            projected_item_ids: self.projected_item_ids.clone(),
            preserved_reverse_closure_target_ids: self
                .context_items
                .iter()
                .map(|item| item.stable_id.clone())
                .collect(),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    /// Exact preserved-target contract induced by the full graph for this
    /// scoped boundary.
    ///
    /// The induced closure still retains every projected item plus every
    /// reverse-reachable semantic dependency needed by those support-backed
    /// projected items. The target set itself is therefore allowed to be
    /// smaller than the retained context item set.
    // @applies TRACEABILITY.SCOPED_PROJECTED_KERNEL
    pub(super) fn exact_contract(
        &self,
        graph: &TraceGraph,
    ) -> Result<ScopedTraceabilityContract, String> {
        #[cfg(test)]
        use_rust_reference_traceability_kernel_for_tests();

        build_projected_traceability_contract(self.projected_item_ids.clone(), graph)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn reference(&self, graph: &TraceGraph) -> Result<ScopedTraceabilityReference, String> {
        #[cfg(test)]
        use_rust_reference_traceability_kernel_for_tests();

        build_projected_traceability_reference_from_projected_items(
            self.projected_item_ids.clone(),
            graph,
        )
    }
}

pub(super) fn derive_scoped_traceability_boundary(
    repo_items: Vec<TraceabilityOwnedItem>,
    scoped_source_files: &[PathBuf],
) -> ScopedTraceabilityBoundary {
    let scoped_file_set = scoped_source_files.iter().cloned().collect::<BTreeSet<_>>();
    let projected_item_ids = repo_items
        .iter()
        .filter(|item| scoped_file_set.contains(&item.path))
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    let mut pending_module_ids = repo_items
        .iter()
        .filter(|item| scoped_file_set.contains(&item.path))
        .flat_map(|item| item.module_ids.iter().cloned())
        .collect::<Vec<_>>();
    let mut kept_item_ids = projected_item_ids.clone();
    let mut seen_module_ids = BTreeSet::new();

    while let Some(module_id) = pending_module_ids.pop() {
        if !seen_module_ids.insert(module_id.clone()) {
            continue;
        }
        for item in &repo_items {
            if !item.module_ids.contains(&module_id) {
                continue;
            }
            if kept_item_ids.insert(item.stable_id.clone()) {
                pending_module_ids.extend(item.module_ids.iter().cloned());
            }
        }
    }

    let context_items = repo_items
        .into_iter()
        .filter(|item| kept_item_ids.contains(&item.stable_id))
        .collect::<Vec<_>>();
    let seed_ids = context_items
        .iter()
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();

    ScopedTraceabilityBoundary {
        projected_item_ids,
        context_items,
        seed_ids,
    }
}

// @applies ADAPTER.FACTS_TO_MODEL.TRACEABILITY_ITEMS
pub(super) fn collect_repo_items(
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    mediated_reasons: &BTreeMap<String, RustMediatedReason>,
) -> Vec<TraceabilityOwnedItem> {
    let mut items = source_graphs
        .iter()
        .flat_map(|(path, graph)| {
            let module_ids = owned_module_ids_for_path(file_ownership, path);
            let test_file = is_test_file_path(path);
            graph
                .items
                .iter()
                .filter(|item| !item.is_test)
                .map(move |item| TraceabilityOwnedItem {
                    stable_id: item.stable_id.clone(),
                    name: item.name.clone(),
                    kind: source_item_kind(item.kind),
                    path: path.clone(),
                    public: item.public,
                    review_surface: is_review_surface(item, test_file),
                    test_file,
                    module_ids: module_ids.clone(),
                    mediated_reason: mediated_reasons
                        .get(&item.stable_id)
                        .map(|reason| reason.as_str()),
                })
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    items
}

pub(super) fn is_test_file_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
        || path.file_stem().and_then(|stem| stem.to_str()) == Some("tests")
}

pub(super) fn is_review_surface(item: &crate::syntax::SourceItem, test_file: bool) -> bool {
    !test_file && (item.public || is_process_entrypoint_name(&item.name, item.kind))
}

pub(super) fn source_item_kind(kind: crate::syntax::SourceItemKind) -> ModuleItemKind {
    match kind {
        crate::syntax::SourceItemKind::Function => ModuleItemKind::Function,
        crate::syntax::SourceItemKind::Method => ModuleItemKind::Method,
    }
}

fn is_process_entrypoint_name(name: &str, kind: crate::syntax::SourceItemKind) -> bool {
    kind == crate::syntax::SourceItemKind::Function && name == "main"
}
