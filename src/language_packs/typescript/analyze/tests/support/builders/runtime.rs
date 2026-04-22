/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS.RUNTIME
Shared TypeScript scoped traceability runtime pair builders.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS.RUNTIME
use std::fs;
use std::path::{Path, PathBuf};

use crate::language_packs::typescript::analyze as analyze;
use crate::language_packs::typescript::analyze::boundary::derive_scoped_traceability_boundary;
use crate::model::ArchitectureTraceabilitySummary;

use super::super::helpers::{
    build_typescript_fixture_context, build_typescript_summary_from_closure,
    filter_summary_to_display_path, index_file_ownership_for_test_owned,
    is_typescript_tooling_unavailable, resolve_scoped_source_file,
};

pub(crate) fn build_direct_scoped_typescript_analysis_pair(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<(
    ArchitectureTraceabilitySummary,
    ArchitectureTraceabilitySummary,
    PathBuf,
)> {
    let (full, scoped, root) =
        build_direct_scoped_typescript_analysis_pair_raw(fixture_name, fixture_writer, scoped_path)?;
    Some((
        filter_summary_to_display_path(full, scoped_path),
        filter_summary_to_display_path(scoped, scoped_path),
        root,
    ))
}

pub(crate) fn build_live_scoped_typescript_analysis_pair(
    root: &Path,
    scoped_path: &str,
) -> Option<(ArchitectureTraceabilitySummary, ArchitectureTraceabilitySummary)> {
    analyze::typescript_runtime(root)?;

    let parsed_architecture =
        crate::modules::parse_architecture(root, &[]).expect("architecture should parse");
    let parsed_repo =
        crate::parser::parse_repo(root, &[], crate::parser::ParseDialect::CurrentV1)
            .expect("repo annotations should parse");
    let discovered = crate::discovery::discover_annotation_files(crate::discovery::DiscoveryConfig {
        root,
        ignore_patterns: &[],
    })
    .expect("fixture files should be discovered");
    let file_ownership = index_file_ownership_for_test_owned(&parsed_architecture);
    let full_summary = build_typescript_summary_from_closure(
        root,
        &discovered.source_files,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    )?;
    let facts = match analyze::build_traceability_scope_facts(
        root,
        &discovered.source_files,
        &parsed_repo,
    ) {
        Ok(facts) => facts,
        Err(error) if is_typescript_tooling_unavailable(&error) => return None,
        Err(error) => panic!("scope facts should build: {error}"),
    };
    let facts: analyze::TypeScriptTraceabilityScopeFacts =
        serde_json::from_slice(&facts).expect("scope facts should deserialize");
    let scoped_source_file = resolve_scoped_source_file(&discovered.source_files, root, scoped_path);
    let boundary = derive_scoped_traceability_boundary(
        &discovered.source_files,
        std::slice::from_ref(&scoped_source_file),
        &facts.adjacency,
    );
    let full_inputs = match analyze::build_traceability_inputs_for_typescript(
        root,
        &discovered.source_files,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    ) {
        Ok(inputs) => inputs,
        Err(error) if is_typescript_tooling_unavailable(&error) => return None,
        Err(error) => panic!("full typescript inputs should build: {error}"),
    };
    let exact_contract = boundary.exact_contract(&discovered.source_files, &full_inputs);
    let exact_closure = exact_contract.preserved_file_closure.clone();
    let scoped_summary = build_typescript_summary_from_closure(
        root,
        &exact_closure,
        Some(std::slice::from_ref(&scoped_source_file)),
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    )?;

    Some((
        filter_summary_to_display_path(full_summary, scoped_path),
        filter_summary_to_display_path(scoped_summary, scoped_path),
    ))
}

pub(crate) fn build_direct_scoped_typescript_analysis_pair_raw(
    fixture_name: &str,
    fixture_writer: fn(&Path),
    scoped_path: &str,
) -> Option<(
    ArchitectureTraceabilitySummary,
    ArchitectureTraceabilitySummary,
    PathBuf,
)> {
    let (root, parsed_repo, parsed_architecture, source_files, file_ownership) =
        build_typescript_fixture_context(fixture_name, fixture_writer)?;

    let full = match analyze::build_traceability_analysis_for_typescript(
        &root,
        &source_files,
        None,
        None,
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    ) {
        Ok(analysis) => analyze::summarize_shared_repo_traceability(&root, &analysis),
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("full typescript analysis should build: {error}"),
    };
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
    let scoped_boundary = derive_scoped_traceability_boundary(
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
    let exact_contract = scoped_boundary.exact_contract(&source_files, &full_inputs);
    let graph_facts =
        match analyze::build_traceability_graph_facts(&root, &exact_contract.preserved_file_closure) {
            Ok(facts) => facts,
            Err(error) if is_typescript_tooling_unavailable(&error) => {
                fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
                return None;
            }
            Err(error) => panic!("graph facts should build: {error}"),
        };
    let scoped = match analyze::build_traceability_analysis_for_typescript(
        &root,
        &exact_contract.preserved_file_closure,
        Some(std::slice::from_ref(&scoped_source_file)),
        Some(&graph_facts),
        &parsed_repo,
        &parsed_architecture,
        &file_ownership,
    ) {
        Ok(analysis) => analyze::summarize_shared_repo_traceability(&root, &analysis),
        Err(error) if is_typescript_tooling_unavailable(&error) => {
            fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
            return None;
        }
        Err(error) => panic!("scoped typescript analysis should build: {error}"),
    };

    Some((full, scoped, root))
}
