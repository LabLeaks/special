/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.ASSERTIONS
Shared TypeScript scoped traceability assertion helpers.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.ASSERTIONS
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::modules::analyze::traceability_core::{
    effective_context_item_ids_for_inputs, projected_reverse_closure_for_inputs,
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
