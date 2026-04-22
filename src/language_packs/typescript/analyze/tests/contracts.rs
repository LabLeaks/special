/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.CONTRACTS
TypeScript exact and working contract regression tests.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.CONTRACTS
use super::support::{
    assert_direct_scoped_typescript_contract_is_minimal,
    assert_direct_scoped_typescript_exact_contract_paths,
    assert_direct_scoped_typescript_exact_contract_target_names,
    assert_direct_scoped_typescript_working_contract_contains_exact_contract,
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
fn scoped_typescript_direct_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-direct-exact-candidate",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-tool-exact-candidate",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-reference-exact-candidate",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-react-exact-candidate",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-next-exact-candidate",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-event-exact-candidate",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-forwarded-exact-candidate",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-hook-exact-candidate",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-effect-exact-candidate",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-context-exact-candidate",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_working_contract_contains_exact_contract() {
    assert_direct_scoped_typescript_working_contract_contains_exact_contract(
        "special-typescript-proof-boundary-cycle-exact-candidate",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_direct_exact_contract_targets_live_impl() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-direct-target-names",
        write_typescript_traceability_fixture,
        "src/app.ts",
        &["helper", "liveImpl"],
    );
}

#[test]
fn scoped_typescript_tool_exact_contract_targets_live_impl() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-tool-target-names",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
        &["helper", "liveImpl"],
    );
}

#[test]
fn scoped_typescript_reference_exact_contract_targets_run_live() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-reference-target-names",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
        &["invoke", "runLive"],
    );
}

#[test]
fn scoped_typescript_react_exact_contract_targets_home_page() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-react-target-names",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
        &["HomePage"],
    );
}

#[test]
fn scoped_typescript_next_exact_contract_targets_page() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-next-target-names",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
        &["Page"],
    );
}

#[test]
fn scoped_typescript_event_exact_contract_targets_app() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-event-target-names",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
        &["App"],
    );
}

#[test]
fn scoped_typescript_forwarded_exact_contract_targets_app() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-forwarded-target-names",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
        &["App"],
    );
}

#[test]
fn scoped_typescript_hook_exact_contract_targets_app() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-hook-target-names",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
        &["App"],
    );
}

#[test]
fn scoped_typescript_effect_exact_contract_targets_app() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-effect-target-names",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
        &["App"],
    );
}

#[test]
fn scoped_typescript_context_exact_contract_targets_app() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-context-target-names",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
        &["App", "CounterButton"],
    );
}

#[test]
fn scoped_typescript_cycle_exact_contract_targets_run_live() {
    assert_direct_scoped_typescript_exact_contract_target_names(
        "special-typescript-proof-boundary-cycle-target-names",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
        &["runLive"],
    );
}

#[test]
fn scoped_typescript_direct_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-direct-contract-paths",
        write_typescript_traceability_fixture,
        "src/app.ts",
        &["src/app.ts"],
    );
}

#[test]
fn scoped_typescript_tool_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-tool-contract-paths",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
        &["src/app.ts"],
    );
}

#[test]
fn scoped_typescript_reference_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-reference-contract-paths",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
        &["src/app.ts"],
    );
}

#[test]
fn scoped_typescript_react_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-react-contract-paths",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
        &["src/page.tsx"],
    );
}

#[test]
fn scoped_typescript_next_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-next-contract-paths",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
        &["app/page.tsx"],
    );
}

#[test]
fn scoped_typescript_event_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-event-contract-paths",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
        &["src/App.tsx"],
    );
}

#[test]
fn scoped_typescript_forwarded_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-forwarded-contract-paths",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
        &["src/App.tsx"],
    );
}

#[test]
fn scoped_typescript_hook_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-hook-contract-paths",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
        &["src/App.tsx"],
    );
}

#[test]
fn scoped_typescript_effect_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-effect-contract-paths",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
        &["src/App.tsx"],
    );
}

#[test]
fn scoped_typescript_context_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-context-contract-paths",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
        &["src/App.tsx"],
    );
}

#[test]
fn scoped_typescript_cycle_exact_contract_paths() {
    assert_direct_scoped_typescript_exact_contract_paths(
        "special-typescript-proof-boundary-cycle-contract-paths",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
        &["src/app.ts"],
    );
}

#[test]
fn scoped_typescript_direct_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-direct-minimality",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-tool-minimality",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-reference-minimality",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-react-minimality",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-next-minimality",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-event-minimality",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-forwarded-minimality",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-hook-minimality",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-effect-minimality",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-context-minimality",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_contract_is_minimal() {
    assert_direct_scoped_typescript_contract_is_minimal(
        "special-typescript-proof-boundary-cycle-minimality",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}
