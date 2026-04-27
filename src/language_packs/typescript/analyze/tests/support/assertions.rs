/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.ASSERTIONS
Shared TypeScript scoped traceability assertion helpers.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.ASSERTIONS
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::modules::analyze::traceability_core::{
    TraceabilityAnalysis, TraceabilityInputs, TraceabilityItemSupport, build_traceability_analysis,
    collect_reverse_reachable_ids, collect_support_root_ids, effective_context_item_ids_for_inputs,
    projected_item_ids_for_inputs, projected_reverse_closure_for_inputs,
    projected_support_root_ids_for_inputs,
};

use super::builders::{
    build_direct_scoped_typescript_analysis_pair, build_typescript_contract_test_context,
    build_typescript_exact_contract, build_typescript_exact_contract_target_context,
    build_typescript_input_comparison_context, build_typescript_reference_comparison_context,
    build_typescript_working_and_exact_contract,
};
use super::helpers::{
    build_typescript_summary_from_closure, contract_contains_path, filter_summary_to_display_path,
    relativize_contract_paths, summary_identity,
};

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

pub(crate) fn assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((full, scoped, root)) =
        build_direct_scoped_typescript_analysis_pair(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    assert_eq!(
        summary_identity(&filter_summary_to_display_path(full, scoped_path)),
        summary_identity(&scoped),
        "scoped summary should match full summary filtered to {scoped_path}",
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_exact_contract_paths(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
    expected_paths: &[&str],
) {
    let Some(contract) = build_typescript_exact_contract(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    let actual = relativize_contract_paths(contract.projected_files.iter().cloned());
    let expected = expected_paths
        .iter()
        .map(PathBuf::from)
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

pub(crate) fn assert_direct_scoped_typescript_contract_is_minimal(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((full_summary, contract, root, parsed_repo, parsed_architecture, file_ownership)) =
        build_typescript_contract_test_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    let expected = filter_summary_to_display_path(full_summary, scoped_path);

    for index in 0..contract.preserved_file_closure.len() {
        let mut reduced = contract.preserved_file_closure.clone();
        let removed = reduced.remove(index);
        let Some(summary) = build_typescript_summary_from_closure(
            &root,
            &reduced,
            Some(&[root.join(scoped_path)]),
            &parsed_repo,
            &parsed_architecture,
            &file_ownership,
        ) else {
            continue;
        };
        let filtered = filter_summary_to_display_path(summary, scoped_path);
        assert_ne!(
            summary_identity(&filtered),
            summary_identity(&expected),
            "removing {} from preserved closure should change scoped summary",
            removed.display(),
        );
    }

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_working_contract_contains_exact_contract(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((working_contract, exact_contract, root)) =
        build_typescript_working_and_exact_contract(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    assert!(
        exact_contract
            .preserved_file_closure
            .iter()
            .all(|path| working_contract.preserved_file_closure.contains(path)),
        "working contract should contain exact contract file closure",
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_exact_contract_target_names(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
    expected_names: &[&str],
) {
    let Some((contract, full_inputs, root)) =
        build_typescript_exact_contract_target_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    let actual = contract
        .preserved_reverse_closure_target_ids
        .iter()
        .filter_map(|stable_id| {
            full_inputs
                .repo_items
                .iter()
                .find(|item| item.stable_id == *stable_id)
                .map(|item| item.name.clone())
        })
        .collect::<BTreeSet<_>>();
    let expected = expected_names
        .iter()
        .map(|name| (*name).to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((full_inputs, scoped_inputs, projected_files, root)) =
        build_typescript_input_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    let full_roots =
        projected_support_root_ids_for_inputs(&full_inputs, projected_files.iter().cloned());
    let scoped_roots =
        projected_support_root_ids_for_inputs(&scoped_inputs, projected_files.iter().cloned());
    assert_eq!(full_roots, scoped_roots);

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((full_inputs, scoped_inputs, projected_files, root)) =
        build_typescript_input_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    let full_closure =
        projected_reverse_closure_for_inputs(&full_inputs, projected_files.iter().cloned());
    let scoped_closure =
        projected_reverse_closure_for_inputs(&scoped_inputs, projected_files.iter().cloned());
    assert_eq!(full_closure, scoped_closure);

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_projected_reverse_subgraph_matches_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((_exact_contract, reference, full_inputs, scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    assert_eq!(
        reverse_subgraphs_for_item_ids(
            &scoped_inputs,
            reference.contract.projected_item_ids.iter().cloned().collect(),
        ),
        reverse_subgraphs_for_item_ids(
            &full_inputs,
            reference.contract.projected_item_ids.iter().cloned().collect(),
        ),
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_context_support_roots_match_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((_exact_contract, _reference, full_inputs, scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };
    let scoped_context_ids = effective_context_item_ids(&scoped_inputs);

    assert_eq!(
        support_root_ids_for_item_ids(&scoped_inputs, scoped_context_ids.clone()),
        support_root_ids_for_item_ids(&full_inputs, scoped_context_ids),
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_context_reverse_closure_matches_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((_exact_contract, _reference, full_inputs, scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };
    let scoped_context_ids = effective_context_item_ids(&scoped_inputs);

    assert_eq!(
        reverse_reachable_ids_for_item_ids(&scoped_inputs, scoped_context_ids.clone()),
        reverse_reachable_ids_for_item_ids(&full_inputs, scoped_context_ids),
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_context_reverse_subgraph_matches_full_analysis(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((_exact_contract, _reference, full_inputs, scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };
    let scoped_context_ids = effective_context_item_ids(&scoped_inputs);

    assert_eq!(
        reverse_subgraphs_for_item_ids(&scoped_inputs, scoped_context_ids.clone()),
        reverse_subgraphs_for_item_ids(&full_inputs, scoped_context_ids),
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_structure_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((_exact_contract, reference, full_inputs, scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };
    let full = build_traceability_analysis(full_inputs);
    let scoped = build_traceability_analysis(scoped_inputs);
    let projected_ids = reference.contract.projected_item_ids;

    assert_eq!(
        analysis_item_ids(&scoped, &projected_ids),
        projected_ids,
        "scoped TypeScript repo items should be exactly the projected item set",
    );
    assert_eq!(
        projected_supported_item_ids(&full, &projected_ids),
        projected_supported_item_ids(&scoped, &projected_ids),
    );
    assert_eq!(
        projected_support_fingerprints(&full, &projected_ids),
        projected_support_fingerprints(&scoped, &projected_ids),
    );
    assert_eq!(
        projected_module_connectivity(&full, &projected_ids),
        projected_module_connectivity(&scoped, &projected_ids),
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_reference_contract_matches_scoped_inputs(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((exact_contract, reference, _full_inputs, scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };
    let scoped_context_ids = effective_context_item_ids_for_inputs(&scoped_inputs);
    let scoped_exact_reverse_closure = projected_reverse_closure_for_inputs(
        &scoped_inputs,
        reference
            .contract
            .preserved_reverse_closure_target_ids
            .iter()
            .cloned(),
    );

    assert_eq!(
        exact_contract.projected_item_ids,
        projected_item_ids_for_inputs(&scoped_inputs),
    );
    assert_eq!(
        exact_contract.projected_item_ids,
        reference.contract.projected_item_ids,
    );
    assert_eq!(
        exact_contract.preserved_reverse_closure_target_ids,
        reference.contract.preserved_reverse_closure_target_ids,
    );
    assert_eq!(reference.exact_reverse_closure, scoped_exact_reverse_closure);
    assert!(
        exact_contract
            .preserved_reverse_closure_target_ids
            .is_subset(&scoped_context_ids),
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((_contract, reference, _full_inputs, scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    let reference_preserved_items = reference.contract.preserved_item_ids.clone();
    let scoped_preserved_items = effective_context_item_ids_for_inputs(&scoped_inputs);
    assert_eq!(reference_preserved_items, scoped_preserved_items);

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_exact_file_closure_matches_reference(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((exact_contract, reference, _full_inputs, _scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    assert_eq!(
        relativize_contract_paths(exact_contract.preserved_file_closure.iter().cloned()),
        relativize_contract_paths(reference.contract.preserved_file_closure.iter().cloned()),
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) {
    let Some((exact_contract, reference, _full_inputs, _scoped_inputs, root)) =
        build_typescript_reference_comparison_context(fixture_name, fixture_writer, scoped_path)
    else {
        return;
    };

    assert_eq!(
        exact_contract.preserved_item_ids,
        reference.contract.preserved_item_ids,
    );

    fs::remove_dir_all(root).expect("temp repo should be cleaned up");
}

pub(crate) fn assert_contract_contains_path(paths: &[PathBuf], expected: &str) {
    assert!(
        contract_contains_path(paths, expected),
        "expected contract to contain {expected}, got {:?}",
        relativize_contract_paths(paths.iter().cloned())
    );
}

fn analysis_item_ids(
    analysis: &TraceabilityAnalysis,
    projected_ids: &BTreeSet<String>,
) -> BTreeSet<String> {
    analysis
        .repo_items
        .iter()
        .filter(|item| projected_ids.contains(&item.stable_id))
        .map(|item| item.stable_id.clone())
        .collect()
}

fn projected_supported_item_ids(
    analysis: &TraceabilityAnalysis,
    projected_ids: &BTreeSet<String>,
) -> BTreeSet<String> {
    analysis
        .repo_items
        .iter()
        .filter(|item| projected_ids.contains(&item.stable_id))
        .filter(|item| analysis.item_supports.contains_key(&item.stable_id))
        .map(|item| item.stable_id.clone())
        .collect()
}

fn projected_support_fingerprints(
    analysis: &TraceabilityAnalysis,
    projected_ids: &BTreeSet<String>,
) -> BTreeMap<String, Vec<SupportFingerprint>> {
    analysis
        .repo_items
        .iter()
        .filter(|item| projected_ids.contains(&item.stable_id))
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
    projected_ids: &BTreeSet<String>,
) -> BTreeMap<String, ModuleConnectivityFingerprint> {
    analysis
        .repo_items
        .iter()
        .filter(|item| projected_ids.contains(&item.stable_id))
        .map(|item| {
            (
                item.stable_id.clone(),
                ModuleConnectivityFingerprint {
                    module_backed_by_current_specs: item
                        .module_ids
                        .iter()
                        .any(|module_id| {
                            analysis.current_spec_backed_module_ids.contains(module_id)
                        }),
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

fn effective_context_item_ids(inputs: &TraceabilityInputs) -> Vec<String> {
    let items = if inputs.context_items.is_empty() {
        &inputs.repo_items
    } else {
        &inputs.context_items
    };
    items.iter().map(|item| item.stable_id.clone()).collect()
}

fn support_root_ids_for_item_ids(
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
