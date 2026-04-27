/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS.CONTRACTS
Shared TypeScript exact and working contract test context builders.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS.CONTRACTS
use std::fs;
use std::path::Path;

use crate::language_packs::typescript::analyze as analyze;
use crate::language_packs::typescript::analyze::boundary::derive_scoped_traceability_boundary;

use super::{
    TypeScriptContractComparisonContext, TypeScriptContractTestContext, TypeScriptExactTargetContext,
};
use super::super::helpers::{
    build_typescript_fixture_context, build_typescript_summary_from_closure,
    is_typescript_tooling_unavailable, resolve_scoped_source_file,
};

pub(crate) fn build_typescript_exact_contract(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<analyze::boundary::ScopedTraceabilityContract> {
    let (root, parsed_repo, parsed_architecture, source_files, file_ownership) =
        build_typescript_fixture_context(fixture_name, fixture_writer)?;
    let facts = match analyze::build_traceability_scope_facts(&root, &source_files, &source_files, &parsed_repo, &file_ownership) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scope facts should build: {error}"),
    };
    let facts: analyze::TypeScriptTraceabilityScopeFacts =
        serde_json::from_slice(&facts).expect("scope facts should deserialize");
    let scoped_source_file = resolve_scoped_source_file(&source_files, &root, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        &source_files,
        std::slice::from_ref(&scoped_source_file),
        &facts.adjacency,
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
    let contract = boundary.exact_contract(&source_files, &full_inputs).expect("exact traceability contract should derive");
    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
    Some(contract)
}

pub(crate) fn build_typescript_working_and_exact_contract(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<TypeScriptContractComparisonContext> {
    let (root, parsed_repo, parsed_architecture, source_files, file_ownership) =
        build_typescript_fixture_context(fixture_name, fixture_writer)?;
    let facts = match analyze::build_traceability_scope_facts(&root, &source_files, &source_files, &parsed_repo, &file_ownership) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scope facts should build: {error}"),
    };
    let facts: analyze::TypeScriptTraceabilityScopeFacts =
        serde_json::from_slice(&facts).expect("scope facts should deserialize");
    let scoped_source_file = resolve_scoped_source_file(&source_files, &root, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        &source_files,
        std::slice::from_ref(&scoped_source_file),
        &facts.adjacency,
    );
    let working_contract = boundary.working_contract();
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
    let exact_contract = boundary.exact_contract(&source_files, &full_inputs).expect("exact traceability contract should derive");
    Some((working_contract, exact_contract, root))
}

pub(crate) fn build_typescript_exact_contract_target_context(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<TypeScriptExactTargetContext> {
    let (root, parsed_repo, parsed_architecture, source_files, file_ownership) =
        build_typescript_fixture_context(fixture_name, fixture_writer)?;
    let facts = match analyze::build_traceability_scope_facts(&root, &source_files, &source_files, &parsed_repo, &file_ownership) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scope facts should build: {error}"),
    };
    let facts: analyze::TypeScriptTraceabilityScopeFacts =
        serde_json::from_slice(&facts).expect("scope facts should deserialize");
    let scoped_source_file = resolve_scoped_source_file(&source_files, &root, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        &source_files,
        std::slice::from_ref(&scoped_source_file),
        &facts.adjacency,
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
    let contract = boundary.exact_contract(&source_files, &full_inputs).expect("exact traceability contract should derive");
    Some((contract, full_inputs, root))
}

pub(crate) fn build_typescript_contract_test_context(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<TypeScriptContractTestContext> {
    let (root, parsed_repo, parsed_architecture, source_files, file_ownership) =
        build_typescript_fixture_context(fixture_name, fixture_writer)?;
    let full_summary = build_typescript_summary_from_closure(
        &root,
        &source_files,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    )?;
    let facts = match analyze::build_traceability_scope_facts(&root, &source_files, &source_files, &parsed_repo, &file_ownership) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scope facts should build: {error}"),
    };
    let facts: analyze::TypeScriptTraceabilityScopeFacts =
        serde_json::from_slice(&facts).expect("scope facts should deserialize");
    let scoped_source_file = resolve_scoped_source_file(&source_files, &root, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        &source_files,
        std::slice::from_ref(&scoped_source_file),
        &facts.adjacency,
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
    let contract = boundary.exact_contract(&source_files, &full_inputs).expect("exact traceability contract should derive");
    Some((
        full_summary,
        contract,
        root,
        parsed_repo,
        parsed_architecture,
        file_ownership,
    ))
}
