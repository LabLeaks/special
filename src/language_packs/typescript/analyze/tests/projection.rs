/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.PROJECTION
TypeScript reference, closure, and projected-kernel regression tests.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.PROJECTION
use std::path::Path;

use super::support::{
    assert_direct_scoped_typescript_context_reverse_closure_matches_full_analysis,
    assert_direct_scoped_typescript_context_reverse_subgraph_matches_full_analysis,
    assert_direct_scoped_typescript_context_support_roots_match_full_analysis,
    assert_direct_scoped_typescript_exact_file_closure_matches_reference,
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference,
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis,
    assert_direct_scoped_typescript_projected_reverse_subgraph_matches_full_analysis,
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis,
    assert_direct_scoped_typescript_reference_contract_matches_scoped_inputs,
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs,
    assert_direct_scoped_typescript_structure_matches_full_then_filtered,
};
use super::test_fixtures::{
    write_typescript_context_traceability_fixture, write_typescript_cycle_traceability_fixture,
    write_typescript_effect_traceability_fixture, write_typescript_event_traceability_fixture,
    write_typescript_forwarded_callback_traceability_fixture,
    write_typescript_hook_callback_traceability_fixture, write_typescript_next_traceability_fixture,
    write_typescript_react_traceability_fixture, write_typescript_reference_traceability_fixture,
    write_typescript_tool_traceability_fixture, write_typescript_traceability_fixture,
};

type TypeScriptProjectionFixture = (&'static str, fn(&Path), &'static str);

#[test]
fn scoped_typescript_direct_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-direct-reference",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-tool-reference",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-reference-reference",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-react-reference",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-next-reference",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-event-reference",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-forwarded-reference",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-hook-reference",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-effect-reference",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-context-reference",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_reference_reverse_closure_matches_scoped_inputs() {
    assert_direct_scoped_typescript_reference_reverse_closure_matches_scoped_inputs(
        "special-typescript-proof-boundary-cycle-reference",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_direct_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-direct-reference-file-closure",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-tool-reference-file-closure",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-reference-reference-file-closure",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-react-reference-file-closure",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-next-reference-file-closure",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-event-reference-file-closure",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-forwarded-reference-file-closure",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-hook-reference-file-closure",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-effect-reference-file-closure",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-context-reference-file-closure",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_exact_file_closure_matches_reference() {
    assert_direct_scoped_typescript_exact_file_closure_matches_reference(
        "special-typescript-proof-boundary-cycle-reference-file-closure",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_direct_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-direct-reference-item-kernel",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-tool-reference-item-kernel",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-reference-reference-item-kernel",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-react-reference-item-kernel",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-next-reference-item-kernel",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-event-reference-item-kernel",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-forwarded-reference-item-kernel",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-hook-reference-item-kernel",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-effect-reference-item-kernel",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-context-reference-item-kernel",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_exact_item_kernel_matches_reference() {
    assert_direct_scoped_typescript_exact_item_kernel_matches_reference(
        "special-typescript-proof-boundary-cycle-reference-item-kernel",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_direct_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-direct-projected-support-roots",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-tool-projected-support-roots",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-reference-projected-support-roots",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-react-projected-support-roots",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-next-projected-support-roots",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-event-projected-support-roots",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-forwarded-projected-support-roots",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-hook-projected-support-roots",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-effect-projected-support-roots",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-context-projected-support-roots",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_projected_support_roots_match_full_analysis() {
    assert_direct_scoped_typescript_projected_support_roots_match_full_analysis(
        "special-typescript-proof-boundary-cycle-projected-support-roots",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_direct_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-direct-projected-reverse-closure",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_tool_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-tool-projected-reverse-closure",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_reference_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-reference-projected-reverse-closure",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_react_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-react-projected-reverse-closure",
        write_typescript_react_traceability_fixture,
        "src/page.tsx",
    );
}

#[test]
fn scoped_typescript_next_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-next-projected-reverse-closure",
        write_typescript_next_traceability_fixture,
        "app/page.tsx",
    );
}

#[test]
fn scoped_typescript_event_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-event-projected-reverse-closure",
        write_typescript_event_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_forwarded_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-forwarded-projected-reverse-closure",
        write_typescript_forwarded_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_hook_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-hook-projected-reverse-closure",
        write_typescript_hook_callback_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_effect_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-effect-projected-reverse-closure",
        write_typescript_effect_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_context_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-context-projected-reverse-closure",
        write_typescript_context_traceability_fixture,
        "src/App.tsx",
    );
}

#[test]
fn scoped_typescript_cycle_projected_reverse_closure_matches_full_analysis() {
    assert_direct_scoped_typescript_projected_reverse_closure_matches_full_analysis(
        "special-typescript-proof-boundary-cycle-projected-reverse-closure",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_typescript_all_projected_reverse_subgraphs_match_full_analysis() {
    assert_for_all_typescript_projection_fixtures(
        "projected-reverse-subgraph",
        assert_direct_scoped_typescript_projected_reverse_subgraph_matches_full_analysis,
    );
}

#[test]
fn scoped_typescript_all_context_support_roots_match_full_analysis() {
    assert_for_all_typescript_projection_fixtures(
        "context-support-roots",
        assert_direct_scoped_typescript_context_support_roots_match_full_analysis,
    );
}

#[test]
fn scoped_typescript_all_context_reverse_closures_match_full_analysis() {
    assert_for_all_typescript_projection_fixtures(
        "context-reverse-closure",
        assert_direct_scoped_typescript_context_reverse_closure_matches_full_analysis,
    );
}

#[test]
fn scoped_typescript_all_context_reverse_subgraphs_match_full_analysis() {
    assert_for_all_typescript_projection_fixtures(
        "context-reverse-subgraph",
        assert_direct_scoped_typescript_context_reverse_subgraph_matches_full_analysis,
    );
}

#[test]
fn scoped_typescript_all_structures_match_full_then_filtered() {
    assert_for_all_typescript_projection_fixtures(
        "structure",
        assert_direct_scoped_typescript_structure_matches_full_then_filtered,
    );
}

#[test]
fn scoped_typescript_all_reference_contracts_match_scoped_inputs() {
    assert_for_all_typescript_projection_fixtures(
        "reference-contract",
        assert_direct_scoped_typescript_reference_contract_matches_scoped_inputs,
    );
}

fn assert_for_all_typescript_projection_fixtures(
    property: &str,
    assertion: fn(&str, fn(&Path), &str),
) {
    for (fixture_name, fixture_writer, scoped_path) in typescript_projection_fixtures() {
        let test_name = format!("special-typescript-proof-boundary-{fixture_name}-{property}");
        assertion(&test_name, fixture_writer, scoped_path);
    }
}

fn typescript_projection_fixtures() -> Vec<TypeScriptProjectionFixture> {
    vec![
        ("direct", write_typescript_traceability_fixture, "src/app.ts"),
        ("tool", write_typescript_tool_traceability_fixture, "src/app.ts"),
        (
            "reference",
            write_typescript_reference_traceability_fixture,
            "src/app.ts",
        ),
        ("react", write_typescript_react_traceability_fixture, "src/page.tsx"),
        ("next", write_typescript_next_traceability_fixture, "app/page.tsx"),
        ("event", write_typescript_event_traceability_fixture, "src/App.tsx"),
        (
            "forwarded",
            write_typescript_forwarded_callback_traceability_fixture,
            "src/App.tsx",
        ),
        (
            "hook",
            write_typescript_hook_callback_traceability_fixture,
            "src/App.tsx",
        ),
        (
            "effect",
            write_typescript_effect_traceability_fixture,
            "src/App.tsx",
        ),
        (
            "context",
            write_typescript_context_traceability_fixture,
            "src/App.tsx",
        ),
        ("cycle", write_typescript_cycle_traceability_fixture, "src/app.ts"),
    ]
}
