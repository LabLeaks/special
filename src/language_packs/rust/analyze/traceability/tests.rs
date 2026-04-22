/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.TESTS
Rust traceability boundary, witness-preservation, and scoped-equality regressions.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.TESTS
use super::{ScopedTraceabilityBoundary, derive_scoped_traceability_boundary};
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::ModuleItemKind;
use crate::modules::analyze::FileOwnership;
use crate::modules::analyze::traceability_core::{
    TraceabilityAnalysis, TraceabilityInputs, TraceabilityItemSupport, TraceabilityOwnedItem,
    collect_reverse_reachable_ids, collect_support_root_ids,
    effective_context_item_ids_for_inputs, preserved_item_ids_for_reference,
    projected_item_ids_for_inputs, projected_support_root_ids_for_inputs,
};
use crate::modules::parse_architecture;
use crate::parser::{ParseDialect, parse_repo};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn owned_item(stable_id: &str, path: &str, module_ids: &[&str]) -> TraceabilityOwnedItem {
    TraceabilityOwnedItem {
        stable_id: stable_id.to_string(),
        kind: ModuleItemKind::Function,
        name: stable_id.to_string(),
        path: PathBuf::from(path),
        public: false,
        review_surface: false,
        test_file: false,
        module_ids: module_ids.iter().map(|id| (*id).to_string()).collect(),
        mediated_reason: None,
    }
}

fn boundary_ids(boundary: &ScopedTraceabilityBoundary) -> BTreeSet<&str> {
    boundary
        .context_items
        .iter()
        .map(|item| item.stable_id.as_str())
        .collect()
}

fn boundary_contract(
    boundary: &ScopedTraceabilityBoundary,
) -> super::boundary::ScopedTraceabilityContract {
    boundary.working_contract()
}

fn boundary_exact_contract(
    boundary: &ScopedTraceabilityBoundary,
    graph: &crate::modules::analyze::traceability_core::TraceGraph,
) -> super::boundary::ScopedTraceabilityContract {
    boundary.exact_contract(graph)
}

fn boundary_reference(
    boundary: &ScopedTraceabilityBoundary,
    graph: &crate::modules::analyze::traceability_core::TraceGraph,
) -> super::boundary::ScopedTraceabilityReference {
    boundary.reference(graph)
}

#[test]
fn scoped_boundary_keeps_scoped_items_and_same_module_peers() {
    let boundary = derive_scoped_traceability_boundary(
        vec![
            owned_item("scoped", "src/lib.rs", &["DEMO"]),
            owned_item("same_module_peer", "src/peer.rs", &["DEMO"]),
            owned_item("other_module", "src/other.rs", &["OTHER"]),
        ],
        &[PathBuf::from("src/lib.rs")],
    );

    assert_eq!(
        boundary_ids(&boundary),
        ["same_module_peer", "scoped"].into_iter().collect()
    );
    assert_eq!(
        boundary.seed_ids,
        ["same_module_peer".to_string(), "scoped".to_string()]
            .into_iter()
            .collect()
    );
    assert_eq!(
        boundary.projected_item_ids,
        ["scoped".to_string()].into_iter().collect()
    );
    let contract = boundary_contract(&boundary);
    assert_eq!(
        contract.projected_item_ids,
        ["scoped".to_string()].into_iter().collect()
    );
    assert_eq!(
        contract.preserved_reverse_closure_target_ids,
        boundary.seed_ids
    );
}

#[test]
fn scoped_boundary_does_not_pull_unrelated_items_without_shared_module_ownership() {
    let boundary = derive_scoped_traceability_boundary(
        vec![
            owned_item("scoped", "src/lib.rs", &[]),
            owned_item("same_file", "src/lib.rs", &[]),
            owned_item("unrelated", "src/other.rs", &[]),
        ],
        &[PathBuf::from("src/lib.rs")],
    );

    assert_eq!(
        boundary_ids(&boundary),
        ["same_file", "scoped"].into_iter().collect()
    );
    assert!(!boundary.seed_ids.contains("unrelated"));
    assert_eq!(
        boundary.projected_item_ids,
        ["same_file".to_string(), "scoped".to_string()]
            .into_iter()
            .collect()
    );
    let contract = boundary_contract(&boundary);
    assert_eq!(
        contract.preserved_reverse_closure_target_ids,
        boundary.seed_ids
    );
}

#[test]
fn scoped_boundary_keeps_transitively_reachable_module_peers() {
    let boundary = derive_scoped_traceability_boundary(
        vec![
            owned_item("scoped", "src/lib.rs", &["A"]),
            owned_item("bridge", "src/bridge.rs", &["A", "B"]),
            owned_item("transitive_peer", "src/peer.rs", &["B"]),
            owned_item("unrelated", "src/other.rs", &["C"]),
        ],
        &[PathBuf::from("src/lib.rs")],
    );

    assert_eq!(
        boundary_ids(&boundary),
        ["bridge", "scoped", "transitive_peer"]
            .into_iter()
            .collect()
    );
    assert_eq!(
        boundary.projected_item_ids,
        ["scoped".to_string()].into_iter().collect()
    );
    assert_eq!(
        boundary.seed_ids,
        ["bridge".to_string(), "scoped".to_string(), "transitive_peer".to_string()]
            .into_iter()
            .collect()
    );
    let contract = boundary_contract(&boundary);
    assert_eq!(
        contract.projected_item_ids,
        ["scoped".to_string()].into_iter().collect()
    );
    assert_eq!(
        contract.preserved_reverse_closure_target_ids,
        boundary.seed_ids
    );
}

#[test]
fn scoped_rust_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_rust_analysis_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-imported",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_raw_summary_matches_full_filtered_projection() {
    let (full_summary, scoped_summary) = build_direct_scoped_rust_analysis_pair(
        "special-rust-traceability-proof-boundary-raw-vs-projected",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );

    let projected_full = filter_summary_to_display_path(full_summary, "src/render.rs");
    assert_eq!(
        serde_json::to_value(scoped_summary).expect("summary should serialize"),
        serde_json::to_value(projected_full).expect("summary should serialize")
    );
}

#[test]
fn scoped_rust_projected_item_and_supported_sets_match_full_analysis() {
    assert_direct_scoped_rust_structure_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-structural",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_exact_item_kernel_matches_reference() {
    assert_direct_scoped_rust_exact_item_kernel_matches_reference(
        "special-rust-traceability-proof-boundary-imported-item-kernel",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_exact_item_kernel_matches_reference() {
    assert_direct_scoped_rust_exact_item_kernel_matches_reference(
        "special-rust-traceability-proof-boundary-module-item-kernel",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_exact_item_kernel_matches_reference() {
    assert_direct_scoped_rust_exact_item_kernel_matches_reference(
        "special-rust-traceability-proof-boundary-instance-item-kernel",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_module_context_structure_matches_full_analysis() {
    assert_direct_scoped_rust_structure_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-module-structure",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_structure_matches_full_analysis() {
    assert_direct_scoped_rust_structure_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-instance-structure",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SupportFingerprint {
    name: String,
    has_item_scoped_support: bool,
    has_file_scoped_support: bool,
    current_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ModuleConnectivityFingerprint {
    module_backed_by_current_specs: bool,
    module_connected_to_current_specs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReverseSubgraphFingerprint {
    node_ids: BTreeSet<String>,
    internal_edges: BTreeMap<String, BTreeSet<String>>,
    root_supports: BTreeMap<String, SupportFingerprint>,
}

#[test]
fn scoped_rust_projected_support_root_ids_match_full_inputs() {
    assert_direct_scoped_rust_support_roots_match_full_then_filtered(
        "special-rust-traceability-proof-boundary-support-roots",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_support_root_ids_match_full_inputs() {
    assert_direct_scoped_rust_support_roots_match_full_then_filtered(
        "special-rust-traceability-proof-boundary-module-support-roots",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_context_support_root_ids_match_full_inputs() {
    assert_direct_scoped_rust_context_support_roots_match_full(
        "special-rust-traceability-proof-boundary-context-support-roots",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_reference_contract_matches_scoped_inputs() {
    assert_direct_scoped_rust_reference_contract_matches_scoped_inputs(
        "special-rust-traceability-proof-boundary-contract-reference",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_reference_contract_matches_scoped_inputs() {
    assert_direct_scoped_rust_reference_contract_matches_scoped_inputs(
        "special-rust-traceability-proof-boundary-module-contract-reference",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_reference_contract_matches_scoped_inputs() {
    assert_direct_scoped_rust_reference_contract_matches_scoped_inputs(
        "special-rust-traceability-proof-boundary-instance-contract-reference",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_multi_root_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_rust_analysis_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-multi-root",
        write_rust_multiple_support_roots_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_multi_root_projected_reverse_subgraph_matches_full_inputs() {
    assert_direct_scoped_rust_projected_reverse_subgraph_matches_full(
        "special-rust-traceability-proof-boundary-multi-root-subgraph",
        write_rust_multiple_support_roots_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_multi_root_reference_contract_matches_scoped_inputs() {
    assert_direct_scoped_rust_reference_contract_matches_scoped_inputs(
        "special-rust-traceability-proof-boundary-multi-root-contract",
        write_rust_multiple_support_roots_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_imported_call_exact_contract_targets_supported_projected_items() {
    assert_direct_scoped_rust_exact_contract_targets_match_supported_projected_items(
        "special-rust-traceability-proof-boundary-imported-exact-targets",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_exact_contract_targets_supported_projected_items() {
    assert_direct_scoped_rust_exact_contract_targets_match_supported_projected_items(
        "special-rust-traceability-proof-boundary-module-exact-targets",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_exact_contract_targets_supported_projected_items() {
    assert_direct_scoped_rust_exact_contract_targets_match_supported_projected_items(
        "special-rust-traceability-proof-boundary-instance-exact-targets",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_multi_root_exact_contract_targets_supported_projected_items() {
    assert_direct_scoped_rust_exact_contract_targets_match_supported_projected_items(
        "special-rust-traceability-proof-boundary-multi-root-exact-targets",
        write_rust_multiple_support_roots_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_recursive_cycle_exact_contract_targets_supported_projected_items() {
    assert_direct_scoped_rust_exact_contract_targets_match_supported_projected_items(
        "special-rust-traceability-proof-boundary-cycle-exact-targets",
        write_rust_recursive_cycle_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_recursive_cycle_reference_contract_matches_scoped_inputs() {
    assert_direct_scoped_rust_reference_contract_matches_scoped_inputs(
        "special-rust-traceability-proof-boundary-cycle-contract",
        write_rust_recursive_cycle_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_projected_reverse_subgraph_matches_full_inputs() {
    assert_direct_scoped_rust_projected_reverse_subgraph_matches_full(
        "special-rust-traceability-proof-boundary-projected-reverse-subgraph",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_projected_reverse_subgraph_matches_full_inputs() {
    assert_direct_scoped_rust_projected_reverse_subgraph_matches_full(
        "special-rust-traceability-proof-boundary-module-projected-reverse-subgraph",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_projected_reverse_subgraph_matches_full_inputs() {
    assert_direct_scoped_rust_projected_reverse_subgraph_matches_full(
        "special-rust-traceability-proof-boundary-instance-projected-reverse-subgraph",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_context_reverse_subgraph_matches_full_inputs() {
    assert_direct_scoped_rust_context_reverse_subgraph_matches_full(
        "special-rust-traceability-proof-boundary-context-reverse-subgraph",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_context_reverse_subgraph_matches_full_inputs() {
    assert_direct_scoped_rust_context_reverse_subgraph_matches_full(
        "special-rust-traceability-proof-boundary-module-context-reverse-subgraph",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_context_reverse_subgraph_matches_full_inputs() {
    assert_direct_scoped_rust_context_reverse_subgraph_matches_full(
        "special-rust-traceability-proof-boundary-instance-context-reverse-subgraph",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_projected_reverse_closure_matches_full_inputs() {
    assert_direct_scoped_rust_projected_reverse_closure_matches_full(
        "special-rust-traceability-proof-boundary-projected-reverse-closure",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_projected_reverse_closure_matches_full_inputs() {
    assert_direct_scoped_rust_projected_reverse_closure_matches_full(
        "special-rust-traceability-proof-boundary-module-projected-reverse-closure",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_projected_reverse_closure_matches_full_inputs() {
    assert_direct_scoped_rust_projected_reverse_closure_matches_full(
        "special-rust-traceability-proof-boundary-instance-projected-reverse-closure",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_context_reverse_closure_matches_full_inputs() {
    assert_direct_scoped_rust_context_reverse_closure_matches_full(
        "special-rust-traceability-proof-boundary-context-reverse-closure",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_context_reverse_closure_matches_full_inputs() {
    assert_direct_scoped_rust_context_reverse_closure_matches_full(
        "special-rust-traceability-proof-boundary-module-context-reverse-closure",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_context_reverse_closure_matches_full_inputs() {
    assert_direct_scoped_rust_context_reverse_closure_matches_full(
        "special-rust-traceability-proof-boundary-instance-context-reverse-closure",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_module_context_context_support_root_ids_match_full_inputs() {
    assert_direct_scoped_rust_context_support_roots_match_full(
        "special-rust-traceability-proof-boundary-module-context-support-roots",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_context_support_root_ids_match_full_inputs() {
    assert_direct_scoped_rust_context_support_roots_match_full(
        "special-rust-traceability-proof-boundary-instance-context-support-roots",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_support_root_ids_match_full_inputs() {
    assert_direct_scoped_rust_support_roots_match_full_then_filtered(
        "special-rust-traceability-proof-boundary-instance-support-roots",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_projected_support_evidence_matches_full_analysis() {
    assert_direct_scoped_rust_structure_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-support-evidence",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_projected_module_connectivity_matches_full_analysis() {
    assert_direct_scoped_rust_structure_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-module-connectivity",
        write_rust_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_rust_module_context_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_rust_analysis_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-module-context",
        write_rust_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_rust_instance_method_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_rust_analysis_matches_full_then_filtered(
        "special-rust-traceability-proof-boundary-instance-method",
        write_rust_instance_method_fixture,
        "src/lib.rs",
    );
}

fn assert_direct_scoped_rust_analysis_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full_summary, scoped_summary) =
        build_direct_scoped_rust_analysis_pair(fixture_name, fixture_writer, scoped_path);
    let projected_full = filter_summary_to_display_path(full_summary, scoped_path);
    let projected_scoped = filter_summary_to_display_path(scoped_summary, scoped_path);

    assert_eq!(
        serde_json::to_value(projected_scoped).expect("summary should serialize"),
        serde_json::to_value(projected_full).expect("summary should serialize")
    );
}

fn assert_direct_scoped_rust_structure_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_analysis_pair_raw(fixture_name, fixture_writer, scoped_path);

    assert_eq!(
        projected_item_ids(&full, scoped_path),
        projected_item_ids(&scoped, scoped_path)
    );
    assert_eq!(
        projected_supported_item_ids(&full, scoped_path),
        projected_supported_item_ids(&scoped, scoped_path)
    );
    assert_eq!(
        projected_support_fingerprints(&full, scoped_path),
        projected_support_fingerprints(&scoped, scoped_path)
    );
    assert_eq!(
        projected_module_connectivity(&full, scoped_path),
        projected_module_connectivity(&scoped, scoped_path)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_support_roots_match_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    assert_eq!(
        projected_support_root_ids(&full, scoped_path),
        projected_support_root_ids(&scoped, scoped_path)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_context_support_roots_match_full(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    assert_eq!(
        effective_context_support_root_ids(&full),
        effective_context_support_root_ids_for_item_ids(&full, effective_context_item_ids(&scoped))
    );
    assert_eq!(
        effective_context_support_root_ids_for_item_ids(&full, effective_context_item_ids(&scoped)),
        effective_context_support_root_ids(&scoped)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_projected_reverse_closure_matches_full(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    assert_eq!(
        projected_reverse_reachable_ids(&full, scoped_path),
        projected_reverse_reachable_ids(&scoped, scoped_path)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_projected_reverse_subgraph_matches_full(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    assert_eq!(
        projected_reverse_subgraphs(&full, scoped_path),
        projected_reverse_subgraphs(&scoped, scoped_path)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_reference_contract_matches_scoped_inputs(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        full.repo_items.clone(),
        &[root.join(scoped_path)],
    );
    let full_reference = boundary_reference(&boundary, &full.graph);
    let scoped_reference = boundary_reference(&boundary, &scoped.graph);
    let scoped_context_ids = effective_context_item_ids(&scoped)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let exact_contract = boundary_exact_contract(&boundary, &full.graph);

    assert_eq!(
        exact_contract.projected_item_ids,
        projected_item_ids_for_inputs(&scoped)
    );
    assert_eq!(
        full_reference.contract.projected_item_ids,
        scoped_reference.contract.projected_item_ids
    );
    assert_eq!(
        full_reference.contract.preserved_reverse_closure_target_ids,
        scoped_reference.contract.preserved_reverse_closure_target_ids
    );
    assert_eq!(
        full_reference.exact_reverse_closure,
        scoped_reference.exact_reverse_closure
    );
    assert!(
        exact_contract
            .preserved_reverse_closure_target_ids
            .is_subset(&scoped_context_ids)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_exact_item_kernel_matches_reference(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        full.repo_items.clone(),
        &[root.join(scoped_path)],
    );
    let reference = boundary_reference(&boundary, &full.graph);

    assert_eq!(
        projected_item_ids_for_inputs(&scoped),
        reference.contract.projected_item_ids
    );
    let preserved_item_ids = preserved_item_ids_for_reference(
        &reference,
        full.repo_items
            .iter()
            .map(|item| item.stable_id.clone())
            .chain(full.context_items.iter().map(|item| item.stable_id.clone())),
    );
    assert_eq!(
        effective_context_item_ids_for_inputs(&scoped),
        preserved_item_ids
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_exact_contract_targets_match_supported_projected_items(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, _scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        full.repo_items.clone(),
        &[root.join(scoped_path)],
    );
    let contract = boundary_exact_contract(&boundary, &full.graph);
    assert_eq!(
        contract.preserved_reverse_closure_target_ids,
        projected_support_root_ids_for_inputs(&full, projected_item_ids_for_inputs(&_scoped))
            .into_iter()
            .filter_map(|(item_id, roots)| (!roots.is_empty()).then_some(item_id))
            .collect()
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_context_reverse_closure_matches_full(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    let scoped_context_ids = effective_context_item_ids(&scoped);
    assert_eq!(
        reverse_reachable_ids_for_item_ids(&full, scoped_context_ids.clone()),
        reverse_reachable_ids_for_item_ids(&scoped, scoped_context_ids)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_direct_scoped_rust_context_reverse_subgraph_matches_full(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_inputs_pair(fixture_name, fixture_writer, scoped_path);
    let scoped_context_ids = effective_context_item_ids(&scoped);
    assert_eq!(
        reverse_subgraphs_for_item_ids(&full, scoped_context_ids.clone()),
        reverse_subgraphs_for_item_ids(&scoped, scoped_context_ids)
    );
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn build_direct_scoped_rust_analysis_pair(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) -> (
    crate::model::ArchitectureTraceabilitySummary,
    crate::model::ArchitectureTraceabilitySummary,
) {
    let (full, scoped, root) =
        build_direct_scoped_rust_analysis_pair_raw(fixture_name, fixture_writer, scoped_path);
    let full_summary = super::summarize_repo_traceability(&root, &full);
    let scoped_summary = super::summarize_repo_traceability(&root, &scoped);
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
    (full_summary, scoped_summary)
}

fn build_direct_scoped_rust_analysis_pair_raw(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) -> (
    crate::modules::analyze::traceability_core::TraceabilityAnalysis,
    crate::modules::analyze::traceability_core::TraceabilityAnalysis,
    PathBuf,
) {
    let root = temp_repo_dir(fixture_name);
    fixture_writer(&root);

    let parsed_architecture = parse_architecture(&root, &[]).expect("architecture should parse");
    let parsed_repo = parse_repo(&root, &[], ParseDialect::CurrentV1)
        .expect("repo annotations should parse");
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let file_ownership = index_file_ownership_for_test(&parsed_architecture);
    let pack = super::RustTraceabilityPack::new(
        super::super::toolchain::probe_local_toolchain_project(&root),
    );

    let full_inputs = super::build_traceability_inputs_from_cached_or_live_graph_facts(
        &root,
        &discovered.source_files,
        None,
        &parsed_repo,
        &file_ownership,
        &pack,
    )
    .expect("full rust traceability inputs should build");
    let scoped_inputs = super::build_scoped_traceability_inputs_from_cached_or_live_graph_facts(
        &root,
        &discovered.source_files,
        &[root.join(scoped_path)],
        None,
        &parsed_repo,
        &file_ownership,
        &pack,
    )
    .expect("scoped rust traceability inputs should build");
    let full = crate::modules::analyze::traceability_core::build_traceability_analysis(full_inputs);
    let scoped =
        crate::modules::analyze::traceability_core::build_traceability_analysis(scoped_inputs);
    (full, scoped, root)
}

fn build_direct_scoped_rust_inputs_pair(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) -> (TraceabilityInputs, TraceabilityInputs, PathBuf) {
    let root = temp_repo_dir(fixture_name);
    fixture_writer(&root);

    let parsed_architecture = parse_architecture(&root, &[]).expect("architecture should parse");
    let parsed_repo =
        parse_repo(&root, &[], ParseDialect::CurrentV1).expect("repo annotations should parse");
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let file_ownership = index_file_ownership_for_test(&parsed_architecture);
    let pack = super::RustTraceabilityPack::new(
        super::super::toolchain::probe_local_toolchain_project(&root),
    );

    let full = super::build_traceability_inputs_from_cached_or_live_graph_facts(
        &root,
        &discovered.source_files,
        None,
        &parsed_repo,
        &file_ownership,
        &pack,
    )
    .expect("full rust traceability inputs should build");
    let scoped = super::build_scoped_traceability_inputs_from_cached_or_live_graph_facts(
        &root,
        &discovered.source_files,
        &[root.join(scoped_path)],
        None,
        &parsed_repo,
        &file_ownership,
        &pack,
    )
    .expect("scoped rust traceability inputs should build");
    (full, scoped, root)
}

fn projected_item_ids(
    analysis: &crate::modules::analyze::traceability_core::TraceabilityAnalysis,
    scoped_path: &str,
) -> BTreeSet<String> {
    let scoped_path = std::path::Path::new(scoped_path);
    analysis
        .repo_items
        .iter()
        .filter(|item| item.path == scoped_path)
        .map(|item| item.stable_id.clone())
        .collect()
}

fn projected_supported_item_ids(analysis: &TraceabilityAnalysis, scoped_path: &str) -> BTreeSet<String> {
    let scoped_path = std::path::Path::new(scoped_path);
    analysis
        .repo_items
        .iter()
        .filter(|item| item.path == scoped_path)
        .filter(|item| analysis.item_supports.contains_key(&item.stable_id))
        .map(|item| item.stable_id.clone())
        .collect()
}

fn projected_support_fingerprints(
    analysis: &TraceabilityAnalysis,
    scoped_path: &str,
) -> BTreeMap<String, Vec<SupportFingerprint>> {
    let scoped_path = std::path::Path::new(scoped_path);
    analysis
        .repo_items
        .iter()
        .filter(|item| item.path == scoped_path)
        .map(|item| {
            let supports = analysis
                .item_supports
                .get(&item.stable_id)
                .into_iter()
                .flatten()
                .map(support_fingerprint)
                .collect::<Vec<_>>();
            (item.stable_id.clone(), supports)
        })
        .collect()
}

fn projected_module_connectivity(
    analysis: &TraceabilityAnalysis,
    scoped_path: &str,
) -> BTreeMap<String, ModuleConnectivityFingerprint> {
    let scoped_path = std::path::Path::new(scoped_path);
    analysis
        .repo_items
        .iter()
        .filter(|item| item.path == scoped_path)
        .map(|item| {
            (
                item.stable_id.clone(),
                ModuleConnectivityFingerprint {
                    module_backed_by_current_specs: item
                        .module_ids
                        .iter()
                        .any(|module_id| analysis.current_spec_backed_module_ids.contains(module_id)),
                    module_connected_to_current_specs: analysis
                        .module_connected_item_ids
                        .contains(&item.stable_id),
                },
            )
        })
        .collect()
}

fn support_fingerprint(support: &TraceabilityItemSupport) -> SupportFingerprint {
    SupportFingerprint {
        name: support.name.clone(),
        has_item_scoped_support: support.has_item_scoped_support,
        has_file_scoped_support: support.has_file_scoped_support,
        current_specs: support.current_specs.clone(),
        planned_specs: support.planned_specs.clone(),
        deprecated_specs: support.deprecated_specs.clone(),
    }
}

fn projected_support_root_ids(
    inputs: &TraceabilityInputs,
    scoped_path: &str,
) -> BTreeMap<String, BTreeSet<String>> {
    let scoped_path = std::path::Path::new(scoped_path);
    let item_ids = inputs
        .repo_items
        .iter()
        .filter(|item| item.path == scoped_path)
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();
    collect_support_root_ids(item_ids.clone(), &inputs.graph)
        .into_iter()
        .filter(|(item_id, _)| item_ids.contains(item_id))
        .collect()
}

fn projected_reverse_reachable_ids(
    inputs: &TraceabilityInputs,
    scoped_path: &str,
) -> BTreeMap<String, BTreeSet<String>> {
    let scoped_path = std::path::Path::new(scoped_path);
    let item_ids = inputs
        .repo_items
        .iter()
        .filter(|item| item.path == scoped_path)
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();
    reverse_reachable_ids_for_item_ids(inputs, item_ids)
}

fn projected_reverse_subgraphs(
    inputs: &TraceabilityInputs,
    scoped_path: &str,
) -> BTreeMap<String, ReverseSubgraphFingerprint> {
    let scoped_path = std::path::Path::new(scoped_path);
    let item_ids = inputs
        .repo_items
        .iter()
        .filter(|item| item.path == scoped_path)
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();
    reverse_subgraphs_for_item_ids(inputs, item_ids)
}

fn effective_context_item_ids(inputs: &TraceabilityInputs) -> Vec<String> {
    let items = if inputs.context_items.is_empty() {
        &inputs.repo_items
    } else {
        &inputs.context_items
    };
    items.iter().map(|item| item.stable_id.clone()).collect()
}

fn effective_context_support_root_ids(
    inputs: &TraceabilityInputs,
) -> BTreeMap<String, BTreeSet<String>> {
    effective_context_support_root_ids_for_item_ids(inputs, effective_context_item_ids(inputs))
}

fn effective_context_support_root_ids_for_item_ids(
    inputs: &TraceabilityInputs,
    item_ids: Vec<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let wanted = item_ids.iter().cloned().collect::<BTreeSet<_>>();
    collect_support_root_ids(item_ids, &inputs.graph)
        .into_iter()
        .filter(|(item_id, _)| wanted.contains(item_id))
        .collect()
}

fn reverse_reachable_ids_for_item_ids(
    inputs: &TraceabilityInputs,
    item_ids: Vec<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    collect_reverse_reachable_ids(item_ids, &inputs.graph)
}

fn reverse_subgraphs_for_item_ids(
    inputs: &TraceabilityInputs,
    item_ids: Vec<String>,
) -> BTreeMap<String, ReverseSubgraphFingerprint> {
    let closures = collect_reverse_reachable_ids(item_ids.clone(), &inputs.graph);
    item_ids
        .into_iter()
        .map(|item_id| {
            let mut node_ids = closures.get(&item_id).cloned().unwrap_or_default();
            node_ids.insert(item_id.clone());
            let internal_edges = inputs
                .graph
                .edges
                .iter()
                .filter(|(caller, _)| node_ids.contains(*caller))
                .map(|(caller, callees)| {
                    (
                        caller.clone(),
                        callees
                            .iter()
                            .filter(|callee| node_ids.contains(*callee))
                            .cloned()
                            .collect(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let root_supports = inputs
                .graph
                .root_supports
                .iter()
                .filter(|(item_id, _)| node_ids.contains(*item_id))
                .map(|(item_id, support)| (item_id.clone(), support_fingerprint(support)))
                .collect::<BTreeMap<_, _>>();
            (
                item_id,
                ReverseSubgraphFingerprint {
                    node_ids,
                    internal_edges,
                    root_supports,
                },
            )
        })
        .collect()
}

fn filter_summary_to_display_path(
    mut summary: crate::model::ArchitectureTraceabilitySummary,
    path: &str,
) -> crate::model::ArchitectureTraceabilitySummary {
    let path = std::path::Path::new(path);
    for items in [
        &mut summary.current_spec_items,
        &mut summary.planned_only_items,
        &mut summary.deprecated_only_items,
        &mut summary.file_scoped_only_items,
        &mut summary.unverified_test_items,
        &mut summary.statically_mediated_items,
        &mut summary.unexplained_items,
    ] {
        items.retain(|item| item.path == path);
    }
    summary.analyzed_items = summary
        .current_spec_items
        .iter()
        .chain(summary.planned_only_items.iter())
        .chain(summary.deprecated_only_items.iter())
        .chain(summary.file_scoped_only_items.iter())
        .chain(summary.unverified_test_items.iter())
        .chain(summary.statically_mediated_items.iter())
        .chain(summary.unexplained_items.iter())
        .map(|item| (item.path.clone(), item.line, item.name.clone(), item.kind))
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    summary.sort_items();
    summary
}

fn index_file_ownership_for_test(
    parsed: &crate::model::ParsedArchitecture,
) -> BTreeMap<PathBuf, FileOwnership<'_>> {
    let mut files: BTreeMap<PathBuf, FileOwnership<'_>> = BTreeMap::new();
    for implementation in &parsed.implements {
        let entry = files.entry(implementation.location.path.clone()).or_default();
        if implementation.body_location.is_some() {
            entry.item_scoped.push(implementation);
        } else {
            entry.file_scoped.push(implementation);
        }
    }
    files
}

fn temp_repo_dir(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{timestamp}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp repo dir should be created");
    path
}

fn write_rust_imported_call_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("_project")).expect("project dir should exist");
    fs::create_dir_all(root.join("specs")).expect("specs dir should exist");
    fs::create_dir_all(root.join("src")).expect("src dir should exist");
    fs::create_dir_all(root.join("tests")).expect("tests dir should exist");

    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.RENDER`\nImported function calls participate in traceability.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("src/lib.rs"),
        "mod render;\n\nuse render::render_entry;\n\n// @fileimplements DEMO\npub fn run() {\n    render_entry();\n}\n",
    )
    .expect("lib fixture should be written");
    fs::write(
        root.join("src/render.rs"),
        "// @fileimplements DEMO\npub fn render_entry() {\n    live_impl();\n}\n\npub fn live_impl() {}\n\npub fn orphan_impl() {}\n",
    )
    .expect("render fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.RENDER\n#[test]\nfn verifies_render_path() {\n    crate::run();\n}\n",
    )
    .expect("verify fixture should be written");
}

fn write_rust_module_context_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("_project")).expect("project dir should exist");
    fs::create_dir_all(root.join("specs")).expect("specs dir should exist");
    fs::create_dir_all(root.join("src")).expect("src dir should exist");
    fs::create_dir_all(root.join("tests")).expect("tests dir should exist");

    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.MODULE_CONTEXT`\nRepo traceability exposes whether unexplained items sit in spec-backed modules and whether they connect inside those modules to traced code.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    )
    .expect("cargo fixture should be written");
    fs::write(
        root.join("src/lib.rs"),
        "// @fileimplements DEMO\npub fn exercise() {\n    traced_entry();\n}\n\nfn traced_entry() {\n    live_leaf();\n}\n\nfn live_leaf() {}\n\nfn connected_helper() {\n    live_leaf();\n}\n\nfn isolated_helper() {}\n",
    )
    .expect("lib fixture should be written");
    fs::write(
        root.join("tests/module_context.rs"),
        "use demo::exercise;\n\n// @verifies APP.MODULE_CONTEXT\n#[test]\nfn verifies_module_context_path() {\n    exercise();\n}\n",
    )
    .expect("test fixture should be written");
}

fn write_rust_instance_method_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("_project")).expect("project dir should exist");
    fs::create_dir_all(root.join("specs")).expect("specs dir should exist");
    fs::create_dir_all(root.join("src")).expect("src dir should exist");
    fs::create_dir_all(root.join("tests")).expect("tests dir should exist");

    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.INSTANCE_METHOD`\nInstance-method dispatch traceability behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    )
    .expect("cargo fixture should be written");
    fs::write(
        root.join("src/lib.rs"),
        "// @fileimplements DEMO\npub fn exercise() {\n    let worker = Worker;\n    worker.run();\n}\n\npub struct Worker;\n\nimpl Worker {\n    fn run(&self) {\n        helper();\n    }\n}\n\nfn helper() {}\n\nfn orphan_impl() {}\n",
    )
    .expect("lib fixture should be written");
    fs::write(
        root.join("tests/instance_method.rs"),
        "use demo::exercise;\n\n// @verifies APP.INSTANCE_METHOD\n#[test]\nfn verifies_instance_method_path() {\n    exercise();\n}\n",
    )
    .expect("test fixture should be written");
}

fn write_rust_multiple_support_roots_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("_project")).expect("project dir should exist");
    fs::create_dir_all(root.join("specs")).expect("specs dir should exist");
    fs::create_dir_all(root.join("src")).expect("src dir should exist");
    fs::create_dir_all(root.join("tests")).expect("tests dir should exist");

    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module OTHER`\nOther module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.MULTI_ROOT`\nMultiple verifying roots and off-scope module context preserve the same scoped traceability.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    )
    .expect("cargo fixture should be written");
    fs::write(
        root.join("src/lib.rs"),
        "// @fileimplements DEMO\npub fn primary() {\n    render_entry();\n}\n\npub fn secondary() {\n    render_entry();\n}\n\npub fn render_entry() {\n    live_impl();\n}\n\nfn live_impl() {}\n\nfn same_file_noise() {}\n",
    )
    .expect("lib fixture should be written");
    fs::write(
        root.join("src/offscope_same_module.rs"),
        "// @fileimplements DEMO\npub fn offscope_same_module_noise() {\n    offscope_leaf();\n}\n\nfn offscope_leaf() {}\n",
    )
    .expect("same-module noise fixture should be written");
    fs::write(
        root.join("src/unrelated.rs"),
        "// @fileimplements OTHER\npub fn unrelated() {}\n",
    )
    .expect("other-module noise fixture should be written");
    fs::write(
        root.join("tests/primary.rs"),
        "use demo::primary;\n\n// @verifies APP.MULTI_ROOT\n#[test]\nfn verifies_primary_path() {\n    primary();\n}\n",
    )
    .expect("primary test fixture should be written");
    fs::write(
        root.join("tests/secondary.rs"),
        "use demo::secondary;\n\n// @verifies APP.MULTI_ROOT\n#[test]\nfn verifies_secondary_path() {\n    secondary();\n}\n",
    )
    .expect("secondary test fixture should be written");
}

fn write_rust_recursive_cycle_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("_project")).expect("project dir should exist");
    fs::create_dir_all(root.join("specs")).expect("specs dir should exist");
    fs::create_dir_all(root.join("src")).expect("src dir should exist");
    fs::create_dir_all(root.join("tests")).expect("tests dir should exist");

    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.RECURSIVE_CYCLE`\nRecursive supported cycles preserve scoped traceability under a canonical exact contract target.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    )
    .expect("cargo fixture should be written");
    fs::write(
        root.join("src/lib.rs"),
        "// @fileimplements DEMO\npub fn exercise() {\n    a();\n}\n\nfn a() {\n    b();\n}\n\nfn b() {\n    a_inner();\n}\n\nfn a_inner() {\n    a();\n}\n\nfn orphan_impl() {}\n",
    )
    .expect("cycle fixture should be written");
    fs::write(
        root.join("tests/cycle.rs"),
        "use demo::exercise;\n\n// @verifies APP.RECURSIVE_CYCLE\n#[test]\nfn verifies_recursive_cycle_path() {\n    exercise();\n}\n",
    )
    .expect("cycle test fixture should be written");
}
