/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.BOUNDARY
Defines the exactness-critical TypeScript scoped traceability boundary in terms of projected scope files, the broad working file-closure, and the smaller exact item-kernel contract/reference that now drives live scoped analysis.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.BOUNDARY
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::modules::analyze::traceability_core::{
    ProjectedProofProtocol, ProjectedTraceabilityContract,
    ProjectedTraceabilityReference, ReverseClosureReference, TraceabilityInputs,
    build_projected_traceability_reference_from_projected_items, normalize_path_for_known_sources,
};
#[cfg(test)]
use crate::modules::analyze::traceability_core::use_rust_reference_traceability_kernel_for_tests;

/// TypeScript scoped traceability currently uses a broad file-level working
/// closure.
///
/// `projected_files` are the files the scoped command should report after the
/// full analysis is projected to the user-requested scope. `closure_files` are
/// the files retained for semantic computation under the current pack-local
/// file/module-graph model.
pub(super) struct ScopedTraceabilityBoundary {
    pub(super) projected_files: BTreeSet<PathBuf>,
    pub(super) closure_files: Vec<PathBuf>,
}

/// Explicit TypeScript scoped exactness claim.
///
/// The broad working closure is still file-shaped, but the smaller exact
/// contract is now item-shaped:
///
/// - `projected_item_ids` are the scoped items that must still appear in the
///   final projected summary
/// - `preserved_reverse_closure_target_ids` are the supported projected items
///   whose reverse closure must be preserved exactly
/// - `preserved_item_ids` are the live scoped analysis kernel:
///   projected items plus that exact reverse closure
/// - `preserved_file_closure` is the execution projection needed to load the
///   files owning those preserved items
///
/// This mirrors the Rust split much more closely: execution still happens over
/// files, but the semantic theorem surface is now the item-level kernel.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) struct ScopedTraceabilityContract {
    pub(super) projected_files: BTreeSet<PathBuf>,
    pub(super) projected_item_ids: BTreeSet<String>,
    pub(super) preserved_reverse_closure_target_ids: BTreeSet<String>,
    pub(super) preserved_item_ids: BTreeSet<String>,
    pub(super) preserved_file_closure: Vec<PathBuf>,
}

/// Slow exact reference derived from the full TypeScript item graph for the
/// current scoped exact contract.
///
/// This gives TypeScript the same production-side contract/reference split as
/// Rust. The exact reverse closure is first derived in item space, and only
/// then projected back to the kept file set needed for execution.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) struct ScopedTraceabilityReference {
    pub(super) contract: ScopedTraceabilityContract,
    #[cfg_attr(test, allow(dead_code))]
    pub(super) exact_reverse_closure: ReverseClosureReference,
}

impl ProjectedProofProtocol for ScopedTraceabilityReference {
    fn projected_contract(&self) -> ProjectedTraceabilityContract {
        ProjectedTraceabilityContract {
            projected_item_ids: self.contract.projected_item_ids.clone(),
            preserved_reverse_closure_target_ids: self
                .contract
                .preserved_reverse_closure_target_ids
                .clone(),
        }
    }

    fn projected_reference(&self) -> ProjectedTraceabilityReference {
        ProjectedTraceabilityReference {
            contract: self.projected_contract(),
            exact_reverse_closure: self.exact_reverse_closure.clone(),
        }
    }
}

impl ScopedTraceabilityBoundary {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn working_contract(&self) -> ScopedTraceabilityContract {
        ScopedTraceabilityContract {
            projected_files: self.projected_files.clone(),
            projected_item_ids: BTreeSet::new(),
            preserved_reverse_closure_target_ids: BTreeSet::new(),
            preserved_item_ids: BTreeSet::new(),
            preserved_file_closure: self.closure_files.clone(),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    // @applies TRACEABILITY.SCOPED_PROJECTED_KERNEL
    pub(super) fn exact_contract(
        &self,
        candidate_files: &[PathBuf],
        full_inputs: &TraceabilityInputs,
    ) -> Result<ScopedTraceabilityContract, String> {
        Ok(self.reference(candidate_files, full_inputs)?.contract)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn reference(
        &self,
        candidate_files: &[PathBuf],
        full_inputs: &TraceabilityInputs,
    ) -> Result<ScopedTraceabilityReference, String> {
        #[cfg(test)]
        use_rust_reference_traceability_kernel_for_tests();

        let projected_item_ids = projected_item_ids(self, candidate_files, full_inputs);
        let projected_item_ids = projected_item_ids.into_iter().collect::<BTreeSet<_>>();
        let projected_reference = build_projected_traceability_reference_from_projected_items(
            projected_item_ids.clone(),
            &full_inputs.graph,
        )?;
        let contract = build_exact_contract(
            self,
            candidate_files,
            full_inputs,
            &projected_item_ids,
            &projected_reference.exact_reverse_closure,
        );

        Ok(ScopedTraceabilityReference {
            contract,
            exact_reverse_closure: projected_reference.exact_reverse_closure,
        })
    }
}

fn projected_item_ids(
    boundary: &ScopedTraceabilityBoundary,
    candidate_files: &[PathBuf],
    full_inputs: &TraceabilityInputs,
) -> Vec<String> {
    full_inputs
        .repo_items
        .iter()
        .filter_map(|item| {
            resolve_candidate_file(candidate_files, &item.path)
                .filter(|path| boundary.projected_files.contains(path))
                .map(|_| item.stable_id.clone())
        })
        .collect::<Vec<_>>()
}

// @applies TRACEABILITY.SCOPED_PROJECTED_KERNEL
fn build_exact_contract(
    boundary: &ScopedTraceabilityBoundary,
    candidate_files: &[PathBuf],
    full_inputs: &TraceabilityInputs,
    projected_item_ids: &BTreeSet<String>,
    exact_reverse_closure: &ReverseClosureReference,
) -> ScopedTraceabilityContract {
    let preserved_item_ids = projected_item_ids
        .iter()
        .cloned()
        .chain(exact_reverse_closure.node_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let lookup_items = if full_inputs.context_items.is_empty() {
        &full_inputs.repo_items
    } else {
        &full_inputs.context_items
    };
    let kept_files = full_inputs
        .repo_items
        .iter()
        .filter_map(|item| {
            resolve_candidate_file(candidate_files, &item.path).filter(|path| {
                boundary.projected_files.contains(path)
                    || preserved_item_ids.contains(&item.stable_id)
            })
        })
        .chain(lookup_items.iter().filter_map(|item| {
            resolve_candidate_file(candidate_files, &item.path)
                .filter(|_| preserved_item_ids.contains(&item.stable_id))
        }))
        .chain(boundary.projected_files.iter().cloned())
        .collect::<BTreeSet<_>>();

    ScopedTraceabilityContract {
        projected_files: boundary.projected_files.clone(),
        projected_item_ids: projected_item_ids.clone(),
        preserved_reverse_closure_target_ids: exact_reverse_closure.target_ids.clone(),
        preserved_item_ids,
        preserved_file_closure: candidate_files
            .iter()
            .filter(|path| kept_files.contains(*path))
            .cloned()
            .collect(),
    }
}

fn resolve_candidate_file(
    candidate_files: &[PathBuf],
    repo_item_path: &std::path::Path,
) -> Option<PathBuf> {
    let normalized = normalize_path_for_known_sources(repo_item_path, candidate_files);
    candidate_files
        .iter()
        .find(|candidate| **candidate == normalized)
        .cloned()
}

pub(super) fn derive_scoped_traceability_boundary(
    candidate_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
    adjacency: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
) -> ScopedTraceabilityBoundary {
    let projected_files = scoped_source_files.iter().cloned().collect::<BTreeSet<_>>();
    let closure_files = expand_file_closure(candidate_files, scoped_source_files, adjacency);
    ScopedTraceabilityBoundary {
        projected_files,
        closure_files,
    }
}

pub(super) fn derive_projected_traceability_boundary(
    candidate_files: &[PathBuf],
    scoped_source_files: &[PathBuf],
) -> ScopedTraceabilityBoundary {
    ScopedTraceabilityBoundary {
        projected_files: scoped_source_files.iter().cloned().collect(),
        closure_files: candidate_files.to_vec(),
    }
}

fn expand_file_closure(
    candidate_files: &[PathBuf],
    seed_files: &[PathBuf],
    adjacency: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
) -> Vec<PathBuf> {
    let candidate_set = candidate_files.iter().cloned().collect::<BTreeSet<_>>();
    let mut closure = seed_files.iter().cloned().collect::<BTreeSet<_>>();
    let mut frontier = seed_files.to_vec();

    while let Some(file) = frontier.pop() {
        let Some(neighbors) = adjacency.get(&file) else {
            continue;
        };
        for neighbor in neighbors {
            if !candidate_set.contains(neighbor) || !closure.insert(neighbor.clone()) {
                continue;
            }
            frontier.push(neighbor.clone());
        }
    }

    candidate_files
        .iter()
        .filter(|path| closure.contains(*path))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::resolve_candidate_file;

    #[test]
    fn resolve_candidate_file_prefers_unique_normalized_match() {
        let candidate_files = vec![
            PathBuf::from("src/app.ts"),
            PathBuf::from("packages/foo/src/app.ts"),
        ];

        assert_eq!(
            resolve_candidate_file(&candidate_files, Path::new("packages/foo/src/app.ts")),
            Some(PathBuf::from("packages/foo/src/app.ts"))
        );
        assert_eq!(
            resolve_candidate_file(&candidate_files, Path::new("src/app.ts")),
            Some(PathBuf::from("src/app.ts"))
        );
    }

    #[test]
    fn resolve_candidate_file_rejects_ambiguous_suffix_match() {
        let candidate_files = vec![
            PathBuf::from("packages/foo/src/app.ts"),
            PathBuf::from("packages/bar/src/app.ts"),
        ];

        assert_eq!(
            resolve_candidate_file(&candidate_files, Path::new("src/app.ts")),
            None
        );
    }
}
