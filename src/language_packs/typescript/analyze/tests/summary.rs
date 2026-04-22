/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUMMARY
TypeScript scoped traceability summary and coarse contract regression tests.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUMMARY
use std::path::PathBuf;

use super::support::{
    assert_contract_contains_path, assert_direct_scoped_typescript_analysis_matches_full_then_filtered,
    build_typescript_exact_contract, relativize_contract_paths,
};
use super::test_fixtures::{
    write_typescript_context_traceability_fixture, write_typescript_cycle_traceability_fixture,
    write_typescript_effect_traceability_fixture, write_typescript_event_traceability_fixture,
    write_typescript_forwarded_callback_traceability_fixture,
    write_typescript_hook_callback_traceability_fixture, write_typescript_next_traceability_fixture,
    write_typescript_react_traceability_fixture, write_typescript_reference_traceability_fixture,
    write_typescript_tool_traceability_fixture, write_typescript_traceability_fixture,
};

#[test]
fn scoped_typescript_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-direct",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-tool",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-reference",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-react",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-next",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-event",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-forwarded",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-hook",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-effect",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-context",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_analysis_matches_full_then_filtered_traceability_summary() {
    assert_direct_scoped_typescript_analysis_matches_full_then_filtered(
        "special-typescript-proof-boundary-cycle",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_contract_excludes_unrelated_collision_file() {
    let contract = build_typescript_exact_contract(
        "special-typescript-proof-boundary-tool-contract",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    )
    .expect("TypeScript tooling should be available for tool contract fixture");

    assert_eq!(
        relativize_contract_paths(contract.projected_files.iter().cloned()),
        [PathBuf::from("src/app.ts")].into_iter().collect()
    );
    assert_contract_contains_path(&contract.preserved_file_closure, "src/app.test.ts");
    assert!(!super::support::contract_contains_path(
        &contract.preserved_file_closure,
        "src/left.ts"
    ));
    assert!(!super::support::contract_contains_path(
        &contract.preserved_file_closure,
        "src/right.ts"
    ));
}

#[test]
fn scoped_typescript_reference_contract_excludes_dead_callback_file() {
    let contract = build_typescript_exact_contract(
        "special-typescript-proof-boundary-reference-contract",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    )
    .expect("TypeScript tooling should be available for reference contract fixture");

    assert_eq!(
        relativize_contract_paths(contract.projected_files.iter().cloned()),
        [PathBuf::from("src/app.ts")].into_iter().collect()
    );
    assert_contract_contains_path(&contract.preserved_file_closure, "src/app.test.ts");
    assert!(!super::support::contract_contains_path(
        &contract.preserved_file_closure,
        "src/live.ts"
    ));
    assert!(!super::support::contract_contains_path(
        &contract.preserved_file_closure,
        "src/dead.ts"
    ));
}
