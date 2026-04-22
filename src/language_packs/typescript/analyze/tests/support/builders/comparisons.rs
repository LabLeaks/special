/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS.COMPARISONS
Shared TypeScript scoped traceability comparison-context builders.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS.COMPARISONS
use std::fs;
use std::path::Path;

use crate::language_packs::typescript::analyze as analyze;
use crate::language_packs::typescript::analyze::boundary::derive_scoped_traceability_boundary;

use super::{TypeScriptInputComparisonContext, TypeScriptReferenceComparisonContext};
use super::super::helpers::{
    build_typescript_fixture_context, is_typescript_tooling_unavailable, resolve_scoped_source_file,
};

pub(crate) fn build_typescript_input_comparison_context(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<TypeScriptInputComparisonContext> {
    let (root, parsed_repo, parsed_architecture, source_files, file_ownership) =
        build_typescript_fixture_context(fixture_name, fixture_writer)?;
    let scope_facts = match analyze::build_traceability_scope_facts(&root, &source_files, &parsed_repo)
    {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scope facts should build: {error}"),
    };
    let scope_facts: analyze::TypeScriptTraceabilityScopeFacts =
        serde_json::from_slice(&scope_facts).expect("scope facts should deserialize");
    let scoped_source_file = resolve_scoped_source_file(&source_files, &root, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        &source_files,
        std::slice::from_ref(&scoped_source_file),
        &scope_facts.adjacency,
    );
    let full_inputs = match analyze::build_traceability_inputs_for_typescript(
        &root,
        &source_files,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    ) {
        Ok(inputs) => inputs,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("full typescript inputs should build: {error}"),
    };
    let contract = boundary.exact_contract(&source_files, &full_inputs);
    let graph_facts = match analyze::build_traceability_graph_facts(
        &root,
        &contract.preserved_file_closure,
    ) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("graph facts should build: {error}"),
    };
    let scoped_inputs = match analyze::build_traceability_inputs_for_typescript(
        &root,
        &contract.preserved_file_closure,
        Some(&graph_facts),
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    ) {
        Ok(inputs) => inputs,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scoped typescript inputs should build: {error}"),
    };
    let scoped_inputs = analyze::narrow_scoped_traceability_inputs_for_typescript(
        &contract.preserved_file_closure,
        Some(std::slice::from_ref(&scoped_source_file)),
        scoped_inputs,
    );

    Some((full_inputs, scoped_inputs, contract.projected_item_ids, root))
}

pub(crate) fn build_typescript_reference_comparison_context(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<TypeScriptReferenceComparisonContext> {
    let (root, parsed_repo, parsed_architecture, source_files, file_ownership) =
        build_typescript_fixture_context(fixture_name, fixture_writer)?;
    let scope_facts = match analyze::build_traceability_scope_facts(&root, &source_files, &parsed_repo)
    {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scope facts should build: {error}"),
    };
    let scope_facts: analyze::TypeScriptTraceabilityScopeFacts =
        serde_json::from_slice(&scope_facts).expect("scope facts should deserialize");
    let scoped_source_file = resolve_scoped_source_file(&source_files, &root, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        &source_files,
        std::slice::from_ref(&scoped_source_file),
        &scope_facts.adjacency,
    );
    let full_inputs = match analyze::build_traceability_inputs_for_typescript(
        &root,
        &source_files,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    ) {
        Ok(inputs) => inputs,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("full typescript inputs should build: {error}"),
    };
    let exact_contract = boundary.exact_contract(&source_files, &full_inputs);
    let reference = boundary.reference(&source_files, &full_inputs);
    let graph_facts = match analyze::build_traceability_graph_facts(
        &root,
        &reference.contract.preserved_file_closure,
    ) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("graph facts should build: {error}"),
    };
    let scoped_inputs = match analyze::build_traceability_inputs_for_typescript(
        &root,
        &reference.contract.preserved_file_closure,
        Some(&graph_facts),
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    ) {
        Ok(inputs) => inputs,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scoped typescript inputs should build: {error}"),
    };
    let scoped_inputs = analyze::narrow_scoped_traceability_inputs_for_typescript(
        &reference.contract.preserved_file_closure,
        Some(std::slice::from_ref(&scoped_source_file)),
        scoped_inputs,
    );

    Some((exact_contract, reference, full_inputs, scoped_inputs, root))
}
