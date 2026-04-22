/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TESTS
Go scoped traceability regressions that compare direct full-vs-scoped pack behavior and make the current working vs exact contract split explicit on real fixture families.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.TESTS
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary};
use crate::modules::analyze::FileOwnership;
use crate::modules::analyze::traceability_core::{
    effective_context_item_ids_for_inputs, preserved_item_ids_for_reference,
    projected_item_ids_for_inputs, projected_reverse_closure_for_inputs,
    projected_support_root_ids_for_inputs,
};
use crate::modules::parse_architecture;
use crate::parser::{ParseDialect, parse_repo};
use crate::syntax::parse_source_graph;
use super::boundary::derive_scoped_traceability_boundary;

#[allow(dead_code)]
#[path = "../test_fixtures.rs"]
mod test_fixtures;

use test_fixtures::{
    write_go_embedded_interface_traceability_fixture,
    write_go_embedding_method_value_traceability_fixture,
    write_go_embedding_traceability_fixture,
    write_go_interface_traceability_fixture,
    write_go_method_expression_traceability_fixture,
    write_go_method_value_traceability_fixture,
    write_go_receiver_collision_traceability_fixture,
    write_go_reference_traceability_fixture, write_go_tool_traceability_fixture,
    write_go_traceability_fixture,
};

#[test]
fn scoped_go_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-direct",
        write_go_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_tool_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-tool",
        write_go_tool_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_reference_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-reference",
        write_go_reference_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_interface_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-interface",
        write_go_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedding_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-embedding",
        write_go_embedding_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_method_value_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-method-value",
        write_go_method_value_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedding_method_value_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-embedding-method-value",
        write_go_embedding_method_value_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_method_expression_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-method-expression",
        write_go_method_expression_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_receiver_collision_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-receiver-collision",
        write_go_receiver_collision_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedded_interface_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_go_analysis_matches_full_then_filtered(
        "special-go-proof-boundary-embedded-interface",
        write_go_embedded_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_direct_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_go_projected_support_roots_match_full_analysis(
        "special-go-proof-boundary-direct-support-roots",
        write_go_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_tool_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_go_projected_support_roots_match_full_analysis(
        "special-go-proof-boundary-tool-support-roots",
        write_go_tool_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_reference_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_go_projected_support_roots_match_full_analysis(
        "special-go-proof-boundary-reference-support-roots",
        write_go_reference_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_interface_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_go_projected_support_roots_match_full_analysis(
        "special-go-proof-boundary-interface-support-roots",
        write_go_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_receiver_collision_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_go_projected_support_roots_match_full_analysis(
        "special-go-proof-boundary-receiver-collision-support-roots",
        write_go_receiver_collision_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedded_interface_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_go_projected_support_roots_match_full_analysis(
        "special-go-proof-boundary-embedded-interface-support-roots",
        write_go_embedded_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_direct_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_go_projected_reverse_closure_matches_full_analysis(
        "special-go-proof-boundary-direct-reverse-closure",
        write_go_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_tool_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_go_projected_reverse_closure_matches_full_analysis(
        "special-go-proof-boundary-tool-reverse-closure",
        write_go_tool_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_reference_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_go_projected_reverse_closure_matches_full_analysis(
        "special-go-proof-boundary-reference-reverse-closure",
        write_go_reference_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_interface_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_go_projected_reverse_closure_matches_full_analysis(
        "special-go-proof-boundary-interface-reverse-closure",
        write_go_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_receiver_collision_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_go_projected_reverse_closure_matches_full_analysis(
        "special-go-proof-boundary-receiver-collision-reverse-closure",
        write_go_receiver_collision_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedded_interface_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_go_projected_reverse_closure_matches_full_analysis(
        "special-go-proof-boundary-embedded-interface-reverse-closure",
        write_go_embedded_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_working_contract_is_broader_than_exact_contract() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-direct-contract",
        write_go_traceability_fixture,
        "app/main.go",
    );
    let working = context.boundary.working_contract();
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);

    assert_eq!(
        working.projected_item_ids,
        exact.projected_item_ids,
        "working and exact contracts should agree on projected items",
    );
    assert!(
        working
            .preserved_reverse_closure_target_ids
            .is_superset(&exact.preserved_reverse_closure_target_ids),
        "working contract should contain the exact reverse-closure targets",
    );
    assert!(
        working
            .preserved_reverse_closure_target_ids
            .iter()
            .any(|item_id| item_id.ends_with("::OrphanImpl:10")),
        "broad working contract should still seed unexplained sibling items",
    );
    assert!(
        !exact
            .preserved_reverse_closure_target_ids
            .iter()
            .any(|item_id| item_id.ends_with("::OrphanImpl:10")),
        "exact contract should exclude unsupported projected siblings",
    );
}

#[test]
fn scoped_go_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-reference-contract",
        write_go_reference_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:6".to_string(),
            "::OrphanImpl:10".to_string(),
            "::invoke:14".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:6".to_string(), "::invoke:14".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
    assert_eq!(
        stable_id_suffixes(&reference.exact_reverse_closure.target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
    assert!(
        reference
            .exact_reverse_closure
            .node_ids
            .is_superset(&reference.exact_reverse_closure.target_ids),
        "reverse-closure nodes should contain the exact target ids",
    );
}

#[test]
fn scoped_go_interface_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-interface-contract",
        write_go_interface_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:10".to_string(),
            "::OrphanImpl:14".to_string(),
            "::invoke:18".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:10".to_string(), "::invoke:18".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
}

#[test]
fn scoped_go_embedding_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-embedding-contract",
        write_go_embedding_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:10".to_string(),
            "::OrphanImpl:14".to_string(),
            "::invoke:18".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:10".to_string(), "::invoke:18".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
}

#[test]
fn scoped_go_method_value_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-method-value-contract",
        write_go_method_value_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:6".to_string(),
            "::OrphanImpl:11".to_string(),
            "::invoke:15".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:6".to_string(), "::invoke:15".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
}

#[test]
fn scoped_go_embedding_method_value_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-embedding-method-value-contract",
        write_go_embedding_method_value_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:6".to_string(),
            "::OrphanImpl:11".to_string(),
            "::invoke:15".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:6".to_string(), "::invoke:15".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
}

#[test]
fn scoped_go_method_expression_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-method-expression-contract",
        write_go_method_expression_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:6".to_string(),
            "::OrphanImpl:11".to_string(),
            "::invoke:15".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:6".to_string(), "::invoke:15".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
}

#[test]
fn scoped_go_receiver_collision_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-receiver-collision-contract",
        write_go_receiver_collision_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:6".to_string(),
            "::OrphanImpl:11".to_string(),
            "::invoke:15".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:6".to_string(), "::invoke:15".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
}

#[test]
fn scoped_go_embedded_interface_exact_contract_targets_supported_projected_items() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-embedded-interface-contract",
        write_go_embedded_interface_traceability_fixture,
        "app/main.go",
    );
    let exact = context.boundary.exact_contract(&context.full_inputs.graph);
    let reference = context.boundary.reference(&context.full_inputs.graph);

    assert_eq!(
        stable_id_suffixes(&exact.projected_item_ids),
        [
            "::LiveImpl:10".to_string(),
            "::OrphanImpl:15".to_string(),
            "::invoke:19".to_string(),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
        ["::LiveImpl:10".to_string(), "::invoke:19".to_string()]
            .into_iter()
            .collect(),
    );
    assert_eq!(
        stable_id_suffixes(&reference.contract.preserved_reverse_closure_target_ids),
        stable_id_suffixes(&exact.preserved_reverse_closure_target_ids),
    );
}

#[test]
fn scoped_go_interface_full_graph_contains_live_constructor_edge() {
    let context = build_go_boundary_context(
        "special-go-proof-boundary-interface-edges",
        write_go_interface_traceability_fixture,
        "app/main.go",
    );
    let live_impl_id = context
        .full_inputs
        .repo_items
        .iter()
        .find(|item| item.name == "LiveImpl")
        .map(|item| item.stable_id.clone())
        .expect("LiveImpl should be present");
    let live_impl_callees = context
        .full_inputs
        .graph
        .edges
        .get(&live_impl_id)
        .cloned()
        .unwrap_or_default();

    assert!(
        stable_id_suffixes(&live_impl_callees).contains("::NewRunner:6"),
        "expected LiveImpl to call live.NewRunner; actual={:?}",
        stable_id_suffixes(&live_impl_callees),
    );
}

#[test]
fn scoped_go_interface_parser_collects_nested_constructor_call() {
    let root = temp_repo_dir("special-go-proof-boundary-interface-parser");
    write_go_interface_traceability_fixture(&root);
    let text = fs::read_to_string(root.join("app/main.go")).expect("main.go should be readable");
    let graph =
        parse_source_graph(Path::new("app/main.go"), &text).expect("source graph should parse");
    let live_impl = graph
        .items
        .iter()
        .find(|item| item.name == "LiveImpl")
        .expect("LiveImpl should be present");

    assert!(
        live_impl
            .calls
            .iter()
            .any(|call| call.name == "NewRunner" && call.qualifier.as_deref() == Some("live")),
        "expected parser to collect nested live.NewRunner call; actual={:?}",
        live_impl
            .calls
            .iter()
            .map(|call| format!("{}::{:?}", call.name, call.qualifier))
            .collect::<Vec<_>>(),
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_interface_discovery_includes_live_and_dead_files() {
    let root = temp_repo_dir("special-go-proof-boundary-interface-discovery");
    write_go_interface_traceability_fixture(&root);
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let discovered_suffixes = discovered
        .source_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        discovered_suffixes.contains(&PathBuf::from("live/live.go")),
        "expected discovery to include live/live.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("dead/dead.go")),
        "expected discovery to include dead/dead.go; actual={discovered_suffixes:?}",
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_embedding_discovery_includes_live_inner_and_dead_files() {
    let root = temp_repo_dir("special-go-proof-boundary-embedding-discovery");
    write_go_embedding_traceability_fixture(&root);
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let discovered_suffixes = discovered
        .source_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        discovered_suffixes.contains(&PathBuf::from("live/live.go")),
        "expected discovery to include live/live.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("inner/inner.go")),
        "expected discovery to include inner/inner.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("dead/dead.go")),
        "expected discovery to include dead/dead.go; actual={discovered_suffixes:?}",
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_method_value_discovery_includes_live_and_dead_files() {
    let root = temp_repo_dir("special-go-proof-boundary-method-value-discovery");
    write_go_method_value_traceability_fixture(&root);
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let discovered_suffixes = discovered
        .source_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        discovered_suffixes.contains(&PathBuf::from("live/live.go")),
        "expected discovery to include live/live.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("dead/dead.go")),
        "expected discovery to include dead/dead.go; actual={discovered_suffixes:?}",
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_embedding_method_value_discovery_includes_live_inner_and_dead_files() {
    let root = temp_repo_dir("special-go-proof-boundary-embedding-method-value-discovery");
    write_go_embedding_method_value_traceability_fixture(&root);
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let discovered_suffixes = discovered
        .source_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        discovered_suffixes.contains(&PathBuf::from("live/live.go")),
        "expected discovery to include live/live.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("inner/inner.go")),
        "expected discovery to include inner/inner.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("dead/dead.go")),
        "expected discovery to include dead/dead.go; actual={discovered_suffixes:?}",
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_method_expression_discovery_includes_live_and_dead_files() {
    let root = temp_repo_dir("special-go-proof-boundary-method-expression-discovery");
    write_go_method_expression_traceability_fixture(&root);
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let discovered_suffixes = discovered
        .source_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        discovered_suffixes.contains(&PathBuf::from("live/live.go")),
        "expected discovery to include live/live.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("dead/dead.go")),
        "expected discovery to include dead/dead.go; actual={discovered_suffixes:?}",
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_receiver_collision_discovery_includes_live_and_dead_files() {
    let root = temp_repo_dir("special-go-proof-boundary-receiver-collision-discovery");
    write_go_receiver_collision_traceability_fixture(&root);
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let discovered_suffixes = discovered
        .source_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        discovered_suffixes.contains(&PathBuf::from("live/live.go")),
        "expected discovery to include live/live.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("dead/dead.go")),
        "expected discovery to include dead/dead.go; actual={discovered_suffixes:?}",
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_embedded_interface_discovery_includes_live_inner_and_dead_files() {
    let root = temp_repo_dir("special-go-proof-boundary-embedded-interface-discovery");
    write_go_embedded_interface_traceability_fixture(&root);
    let discovered = discover_annotation_files(DiscoveryConfig {
        root: &root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let discovered_suffixes = discovered
        .source_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        discovered_suffixes.contains(&PathBuf::from("live/live.go")),
        "expected discovery to include live/live.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("inner/inner.go")),
        "expected discovery to include inner/inner.go; actual={discovered_suffixes:?}",
    );
    assert!(
        discovered_suffixes.contains(&PathBuf::from("dead/dead.go")),
        "expected discovery to include dead/dead.go; actual={discovered_suffixes:?}",
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
fn scoped_go_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-direct-kernel",
        write_go_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_tool_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-tool-kernel",
        write_go_tool_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_reference_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-reference-kernel",
        write_go_reference_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_interface_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-interface-kernel",
        write_go_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedding_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-embedding-kernel",
        write_go_embedding_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_method_value_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-method-value-kernel",
        write_go_method_value_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedding_method_value_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-embedding-method-value-kernel",
        write_go_embedding_method_value_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_method_expression_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-method-expression-kernel",
        write_go_method_expression_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_receiver_collision_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-receiver-collision-kernel",
        write_go_receiver_collision_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedded_interface_exact_item_kernel_matches_reference() {
    assert_direct_scoped_go_exact_item_kernel_matches_reference(
        "special-go-proof-boundary-embedded-interface-kernel",
        write_go_embedded_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_direct_scope_facts_expand_to_exact_closure_files() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-direct-scope-facts",
        write_go_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "shared/shared.go"],
        &[],
    );
}

#[test]
fn scoped_go_tool_scope_facts_exclude_unrelated_package_files() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-tool-scope-facts",
        write_go_tool_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "left/shared.go"],
        &["right/shared.go"],
    );
}

#[test]
fn scoped_go_reference_scope_facts_exclude_callback_package_files() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-reference-scope-facts",
        write_go_reference_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_interface_scope_facts_include_live_impl_and_exclude_dead_impl() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-interface-scope-facts",
        write_go_interface_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_embedding_scope_facts_include_live_and_inner_impls_and_exclude_dead_impl() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-embedding-scope-facts",
        write_go_embedding_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go", "inner/inner.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_method_value_scope_facts_include_live_impl_and_exclude_dead_impl() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-method-value-scope-facts",
        write_go_method_value_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_embedding_method_value_scope_facts_include_live_and_inner_impls_and_exclude_dead_impl()
{
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-embedding-method-value-scope-facts",
        write_go_embedding_method_value_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go", "inner/inner.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_method_expression_scope_facts_include_live_impl_and_exclude_dead_impl() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-method-expression-scope-facts",
        write_go_method_expression_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_receiver_collision_scope_facts_include_live_impl_and_exclude_dead_impl() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-receiver-collision-scope-facts",
        write_go_receiver_collision_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_embedded_interface_scope_facts_include_live_and_inner_impls_and_exclude_dead_impl() {
    assert_go_scope_facts_expand_to_exact_closure(
        "special-go-proof-boundary-embedded-interface-scope-facts",
        write_go_embedded_interface_traceability_fixture,
        "app/main.go",
        &["app/main.go", "app/main_test.go", "live/live.go", "inner/inner.go"],
        &["dead/dead.go"],
    );
}

#[test]
fn scoped_go_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-direct-scope-runtime",
        write_go_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_tool_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-tool-scope-runtime",
        write_go_tool_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_reference_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-reference-scope-runtime",
        write_go_reference_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_interface_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-interface-scope-runtime",
        write_go_interface_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedding_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-embedding-scope-runtime",
        write_go_embedding_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_method_value_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-method-value-scope-runtime",
        write_go_method_value_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedding_method_value_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-embedding-method-value-scope-runtime",
        write_go_embedding_method_value_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_method_expression_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-method-expression-scope-runtime",
        write_go_method_expression_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_receiver_collision_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-receiver-collision-scope-runtime",
        write_go_receiver_collision_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_go_embedded_interface_scope_facts_runtime_matches_full_then_filtered_summary() {
    assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
        "special-go-proof-boundary-embedded-interface-scope-runtime",
        write_go_embedded_interface_traceability_fixture,
        "app/main.go",
    );
}

struct GoBoundaryContext {
    root: PathBuf,
    full_inputs: crate::modules::analyze::traceability_core::TraceabilityInputs,
    boundary: super::boundary::ScopedTraceabilityBoundary,
}

struct GoReferenceComparisonContext {
    root: PathBuf,
    reference: super::boundary::ScopedTraceabilityReference,
    full_inputs: crate::modules::analyze::traceability_core::TraceabilityInputs,
    full_owned_item_ids: BTreeSet<String>,
    scoped_inputs: crate::modules::analyze::traceability_core::TraceabilityInputs,
}

impl Drop for GoReferenceComparisonContext {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Drop for GoBoundaryContext {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_direct_scoped_go_analysis_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
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
    let file_ownership = index_file_ownership_for_test_owned(&parsed_architecture);
    let scoped_source_file = resolve_scoped_source_file(&discovered.source_files, &root, scoped_path);

    let full_analysis = super::traceability::build_traceability_analysis_from_cached_or_live_graph_facts(
        &root,
        &discovered.source_files,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
        &super::traceability::GoTraceabilityPack,
    )
    .expect("full go traceability analysis should build");
    let scoped_analysis =
        super::scope::build_scoped_traceability_analysis_from_cached_or_live_graph_facts(
            &root,
            &discovered.source_files,
            std::slice::from_ref(&scoped_source_file),
            None,
            &parsed_repo,
            &file_ownership,
        )
        .expect("scoped go traceability analysis should build");

    let full_summary = filter_summary_to_display_path(
        super::traceability::summarize_repo_traceability(&root, &full_analysis),
        scoped_path,
    );
    let scoped_summary =
        super::traceability::summarize_repo_traceability(&root, &scoped_analysis);

    assert_traceability_summaries_match(&full_summary, &scoped_summary);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn build_go_boundary_context(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> GoBoundaryContext {
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
    let file_ownership = index_file_ownership_for_test_owned(&parsed_architecture);
    let full_inputs =
        super::traceability::build_traceability_inputs_from_cached_or_live_graph_facts(
            &root,
            &discovered.source_files,
            None,
            &parsed_repo,
            &file_ownership,
        )
        .expect("full go traceability inputs should build");
    let scoped_source_file = resolve_scoped_source_file(&discovered.source_files, &root, scoped_path);
    let scoped_source_file = scoped_source_file
        .strip_prefix(&root)
        .unwrap_or(&scoped_source_file)
        .to_path_buf();
    let boundary = derive_scoped_traceability_boundary(
        full_inputs.repo_items.clone(),
        std::slice::from_ref(&scoped_source_file),
    );

    GoBoundaryContext {
        root,
        full_inputs,
        boundary,
    }
}

fn assert_direct_scoped_go_exact_item_kernel_matches_reference(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let context = build_go_reference_comparison_context(fixture_name, fixture_writer, scoped_path);

    assert_eq!(
        projected_item_ids_for_inputs(&context.scoped_inputs),
        context.reference.contract.projected_item_ids,
        "scoped Go repo items should equal projected items in the exact contract",
    );

    let preserved_item_ids = preserved_item_ids_for_reference(
        &context.reference,
        context.full_owned_item_ids.iter().cloned(),
    );
    assert_eq!(
        effective_context_item_ids_for_inputs(&context.scoped_inputs),
        preserved_item_ids,
        "scoped Go context items should equal projected items plus exact reverse-closure context",
    );
}

fn assert_direct_scoped_go_projected_support_roots_match_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let context = build_go_reference_comparison_context(fixture_name, fixture_writer, scoped_path);
    assert_eq!(
        projected_support_root_ids_for_inputs(
            &context.scoped_inputs,
            context.reference.contract.projected_item_ids.iter().cloned(),
        ),
        projected_support_root_ids_for_inputs(
            &context.full_inputs,
            context.reference.contract.projected_item_ids.iter().cloned(),
        ),
    );
}

fn assert_direct_scoped_go_projected_reverse_closure_matches_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let context = build_go_reference_comparison_context(fixture_name, fixture_writer, scoped_path);
    assert_eq!(
        projected_reverse_closure_for_inputs(
            &context.scoped_inputs,
            context
                .reference
                .contract
                .preserved_reverse_closure_target_ids
                .iter()
                .cloned(),
        ),
        projected_reverse_closure_for_inputs(
            &context.full_inputs,
            context
                .reference
                .contract
                .preserved_reverse_closure_target_ids
                .iter()
                .cloned(),
        ),
    );
}

fn build_go_reference_comparison_context(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> GoReferenceComparisonContext {
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
    let file_ownership = index_file_ownership_for_test_owned(&parsed_architecture);
    let full_inputs =
        super::traceability::build_traceability_inputs_from_cached_or_live_graph_facts(
            &root,
            &discovered.source_files,
            None,
            &parsed_repo,
            &file_ownership,
        )
        .expect("full go traceability inputs should build");
    let scoped_source_file = resolve_scoped_source_file(&discovered.source_files, &root, scoped_path);
    let scoped_source_file = scoped_source_file
        .strip_prefix(&root)
        .unwrap_or(&scoped_source_file)
        .to_path_buf();
    let boundary = derive_scoped_traceability_boundary(
        full_inputs.repo_items.clone(),
        std::slice::from_ref(&scoped_source_file),
    );
    let reference = boundary.reference(&full_inputs.graph);
    let full_owned_item_ids = full_inputs
        .repo_items
        .iter()
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    let scoped_inputs =
        super::scope::build_scoped_traceability_inputs_from_cached_or_live_graph_facts(
            &root,
            &discovered.source_files,
            std::slice::from_ref(&scoped_source_file),
            None,
            &parsed_repo,
            &file_ownership,
        )
        .expect("scoped go traceability inputs should build");

    GoReferenceComparisonContext {
        root,
        reference,
        full_inputs,
        full_owned_item_ids,
        scoped_inputs,
    }
}

fn assert_go_scope_facts_expand_to_exact_closure(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
    expected_present: &[&str],
    expected_absent: &[&str],
) {
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
    let file_ownership = index_file_ownership_for_test_owned(&parsed_architecture);
    let facts = super::build_traceability_scope_facts(&root, &discovered.source_files, &parsed_repo)
        .expect("go scope facts should build");
    let scoped_source_file = resolve_scoped_source_file(&discovered.source_files, &root, scoped_path);
    let closure_files = super::expand_traceability_closure_from_facts(
        &discovered.source_files,
        std::slice::from_ref(&scoped_source_file),
        &file_ownership,
        &facts,
    )
    .expect("go scope facts should expand");
    let relative_paths = closure_files
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_path_buf()
        })
        .collect::<BTreeSet<_>>();

    for expected in expected_present {
        assert!(
            relative_paths.contains(&PathBuf::from(expected)),
            "expected exact closure to contain {expected}; actual={relative_paths:?}",
        );
    }
    for expected in expected_absent {
        assert!(
            !relative_paths.contains(&PathBuf::from(expected)),
            "expected exact closure to exclude {expected}; actual={relative_paths:?}",
        );
    }

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_go_scope_facts_runtime_matches_full_then_filtered_summary(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
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
    let file_ownership = index_file_ownership_for_test_owned(&parsed_architecture);
    let facts = super::build_traceability_scope_facts(&root, &discovered.source_files, &parsed_repo)
        .expect("go scope facts should build");
    let scoped_source_file = resolve_scoped_source_file(&discovered.source_files, &root, scoped_path);
    let closure_files = super::expand_traceability_closure_from_facts(
        &discovered.source_files,
        std::slice::from_ref(&scoped_source_file),
        &file_ownership,
        &facts,
    )
    .expect("go scope facts should expand");

    let full_analysis = super::traceability::build_traceability_analysis_from_cached_or_live_graph_facts(
        &root,
        &discovered.source_files,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
        &super::traceability::GoTraceabilityPack,
    )
    .expect("full go traceability analysis should build");
    let scoped_analysis =
        super::scope::build_scoped_traceability_analysis_from_cached_or_live_graph_facts(
            &root,
            &closure_files,
            std::slice::from_ref(&scoped_source_file),
            None,
            &parsed_repo,
            &file_ownership,
        )
        .expect("scoped go traceability analysis should build");
    let full_summary = filter_summary_to_display_path(
        super::traceability::summarize_repo_traceability(&root, &full_analysis),
        scoped_path,
    );
    let scoped_summary =
        super::traceability::summarize_repo_traceability(&root, &scoped_analysis);

    assert_traceability_summaries_match(&full_summary, &scoped_summary);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn filter_summary_to_display_path(
    mut summary: ArchitectureTraceabilitySummary,
    scoped_path: &str,
) -> ArchitectureTraceabilitySummary {
    let scoped_path = Path::new(scoped_path);
    for items in [
        &mut summary.current_spec_items,
        &mut summary.planned_only_items,
        &mut summary.deprecated_only_items,
        &mut summary.file_scoped_only_items,
        &mut summary.unverified_test_items,
        &mut summary.statically_mediated_items,
        &mut summary.unexplained_items,
    ] {
        items.retain(|item| item.path == scoped_path);
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
        .map(identity_key)
        .collect::<BTreeSet<_>>()
        .len();
    summary.sort_items();
    summary
}

fn assert_traceability_summaries_match(
    expected: &ArchitectureTraceabilitySummary,
    actual: &ArchitectureTraceabilitySummary,
) {
    assert_eq!(expected.analyzed_items, actual.analyzed_items);
    assert_eq!(expected.current_spec_items.len(), actual.current_spec_items.len());
    assert_eq!(expected.planned_only_items.len(), actual.planned_only_items.len());
    assert_eq!(
        expected.deprecated_only_items.len(),
        actual.deprecated_only_items.len()
    );
    assert_eq!(
        expected.file_scoped_only_items.len(),
        actual.file_scoped_only_items.len()
    );
    assert_eq!(
        expected.unverified_test_items.len(),
        actual.unverified_test_items.len()
    );
    assert_eq!(
        expected.statically_mediated_items.len(),
        actual.statically_mediated_items.len()
    );
    assert_eq!(expected.unexplained_items.len(), actual.unexplained_items.len());
    assert_eq!(
        expected
            .current_spec_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>(),
        actual
            .current_spec_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        expected
            .planned_only_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>(),
        actual
            .planned_only_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        expected
            .deprecated_only_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>(),
        actual
            .deprecated_only_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        expected
            .file_scoped_only_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>(),
        actual
            .file_scoped_only_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        expected
            .unverified_test_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>(),
        actual
            .unverified_test_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        expected
            .statically_mediated_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>(),
        actual
            .statically_mediated_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        expected
            .unexplained_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>(),
        actual
            .unexplained_items
            .iter()
            .map(identity_key)
            .collect::<BTreeSet<_>>()
    );
}

fn identity_key(item: &ArchitectureTraceabilityItem) -> String {
    serde_json::to_string(item).expect("traceability item identity should serialize")
}

fn stable_id_suffixes(item_ids: &BTreeSet<String>) -> BTreeSet<String> {
    item_ids
        .iter()
        .filter_map(|item_id| {
            item_id
                .rsplit_once("::")
                .map(|(_, suffix)| format!("::{suffix}"))
        })
        .collect()
}

fn index_file_ownership_for_test_owned(
    parsed: &crate::model::ParsedArchitecture,
) -> BTreeMap<PathBuf, FileOwnership<'static>> {
    let mut files: BTreeMap<PathBuf, FileOwnership<'static>> = BTreeMap::new();
    for implementation in &parsed.implements {
        let leaked = Box::leak(Box::new(implementation.clone()));
        let entry = files.entry(leaked.location.path.clone()).or_default();
        if leaked.body_location.is_some() {
            entry.item_scoped.push(leaked);
        } else {
            entry.file_scoped.push(leaked);
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

fn resolve_scoped_source_file(
    candidate_files: &[PathBuf],
    root: &Path,
    scoped_path: &str,
) -> PathBuf {
    let scoped_path = Path::new(scoped_path);
    candidate_files
        .iter()
        .find(|candidate| candidate.ends_with(scoped_path))
        .cloned()
        .unwrap_or_else(|| root.join(scoped_path))
}
