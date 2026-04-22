/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.BOUNDARY
Defines the exactness-critical Go scoped traceability boundary in terms of projected scoped items, the broad working collection context, and the smaller exact contract/reference vocabulary that now drives the kept semantic kernel.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.BOUNDARY
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::modules::analyze::traceability_core::{
    ProjectedTraceabilityContract, ProjectedTraceabilityReference, TraceGraph,
    TraceabilityOwnedItem, build_projected_traceability_reference, collect_support_root_ids,
};

/// Current Go scoped traceability separates projected output items from the
/// broader working context still used to collect raw reverse-call information.
///
/// `projected_item_ids` are the items located directly in the requested scoped
/// files. `context_items` are the broader working set used for collection:
/// projected items plus same-module peers. `seed_ids` are the broad reverse-
/// walk seeds derived from that working context.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) struct ScopedTraceabilityBoundary {
    pub(super) projected_item_ids: BTreeSet<String>,
    pub(super) context_items: Vec<TraceabilityOwnedItem>,
    pub(super) seed_ids: BTreeSet<String>,
}

/// Explicit exactness claim for the Go scoped boundary.
///
/// This mirrors the shared projected-contract shape used publicly for Rust,
/// TypeScript, and Go: projected output items remain visible, while the exact
/// reverse-closure target set is the subset of projected items that are
/// actually support-backed in the full graph. Go still needs an execution-level
/// file projection later, but that projection is downstream of this item-level
/// contract.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) type ScopedTraceabilityContract = ProjectedTraceabilityContract;

/// Slow exact reverse-closure reference derived from the full Go item graph for
/// the current scoped exact contract.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) type ScopedTraceabilityReference = ProjectedTraceabilityReference;

impl ScopedTraceabilityBoundary {
    /// Broad operational seed set used by the current Go adapter during raw
    /// collection before the live path narrows back to the exact kept kernel.
    #[cfg_attr(not(test), allow(dead_code))]
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

    /// Smaller exact target set induced by the full graph for the current
    /// scoped boundary. This is the target set used by the shared projected-
    /// contract theorem family.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn exact_contract(&self, graph: &TraceGraph) -> ScopedTraceabilityContract {
        let preserved_reverse_closure_target_ids =
            collect_support_root_ids(self.projected_item_ids.iter().cloned(), graph)
                .into_iter()
                .filter_map(|(item_id, supports)| (!supports.is_empty()).then_some(item_id))
                .collect::<BTreeSet<_>>();

        ScopedTraceabilityContract {
            projected_item_ids: self.projected_item_ids.clone(),
            preserved_reverse_closure_target_ids,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn reference(&self, graph: &TraceGraph) -> ScopedTraceabilityReference {
        build_projected_traceability_reference(self.exact_contract(graph), graph)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
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
    let scoped_module_ids = repo_items
        .iter()
        .filter(|item| scoped_file_set.contains(&item.path))
        .flat_map(|item| item.module_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let context_items = repo_items
        .into_iter()
        .filter(|item| {
            scoped_file_set.contains(&item.path)
                || item
                    .module_ids
                    .iter()
                    .any(|module_id| scoped_module_ids.contains(module_id))
        })
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use crate::model::ModuleItemKind;
    use crate::modules::analyze::traceability_core::{
        TraceGraph, TraceabilityItemSupport, TraceabilityOwnedItem,
    };

    use super::derive_scoped_traceability_boundary;

    fn owned_item(path: &str, stable_id: &str, module_ids: &[&str]) -> TraceabilityOwnedItem {
        TraceabilityOwnedItem {
            stable_id: stable_id.to_string(),
            name: stable_id.to_string(),
            kind: ModuleItemKind::Function,
            path: PathBuf::from(path),
            public: true,
            review_surface: true,
            test_file: false,
            module_ids: module_ids.iter().map(|id| id.to_string()).collect(),
            mediated_reason: None,
        }
    }

    #[test]
    fn scoped_boundary_keeps_projected_items_and_same_module_peers() {
        let boundary = derive_scoped_traceability_boundary(
            vec![
                owned_item("app/main.go", "app::LiveImpl", &["DEMO"]),
                owned_item("app/main.go", "app::helper", &["DEMO"]),
                owned_item("shared/shared.go", "shared::SharedValue", &["SHARED"]),
            ],
            &[PathBuf::from("app/main.go")],
        );

        assert_eq!(
            boundary.projected_item_ids,
            ["app::LiveImpl".to_string(), "app::helper".to_string()]
                .into_iter()
                .collect()
        );
        assert_eq!(boundary.context_items.len(), 2);
        assert_eq!(
            boundary.seed_ids,
            ["app::LiveImpl".to_string(), "app::helper".to_string()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn scoped_boundary_working_contract_is_broader_than_exact_contract_when_orphans_exist() {
        let boundary = derive_scoped_traceability_boundary(
            vec![
                owned_item("app/main.go", "app::LiveImpl", &["DEMO"]),
                owned_item("app/main.go", "app::OrphanImpl", &["DEMO"]),
            ],
            &[PathBuf::from("app/main.go")],
        );
        let graph = TraceGraph {
            edges: [(
                "app::TestLiveImpl".to_string(),
                ["app::LiveImpl".to_string()].into_iter().collect(),
            )]
            .into_iter()
            .collect(),
            root_supports: [(
                "app::TestLiveImpl".to_string(),
                TraceabilityItemSupport {
                    name: "TestLiveImpl".to_string(),
                    has_item_scoped_support: true,
                    has_file_scoped_support: false,
                    current_specs: ["APP.LIVE".to_string()].into_iter().collect(),
                    planned_specs: BTreeSet::new(),
                    deprecated_specs: BTreeSet::new(),
                },
            )]
            .into_iter()
            .collect(),
        };

        let working = boundary.working_contract();
        let exact = boundary.exact_contract(&graph);

        assert_eq!(
            working.preserved_reverse_closure_target_ids,
            ["app::LiveImpl".to_string(), "app::OrphanImpl".to_string()]
                .into_iter()
                .collect()
        );
        assert_eq!(
            exact.preserved_reverse_closure_target_ids,
            ["app::LiveImpl".to_string()].into_iter().collect()
        );
    }
}
