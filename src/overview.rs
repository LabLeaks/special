/**
@module SPECIAL.OVERVIEW
Builds the compact root `special` overview by combining lint, specs, architecture, and health into one small document. This module should stay a projection over existing subsystem evidence rather than introducing its own parsing rules.
*/
// @fileimplements SPECIAL.OVERVIEW
use std::path::Path;

use anyhow::Result;

use crate::cache::{load_or_parse_architecture, load_or_parse_repo};
use crate::config::SpecialVersion;
use crate::index::{build_lint_report_from_parsed, build_spec_document_from_parsed};
use crate::model::{
    DeclaredStateFilter, DiagnosticSeverity, ModuleFilter, NodeKind, OverviewArchSummary,
    OverviewDocument, OverviewHealthSummary, OverviewLintSummary, OverviewSpecsSummary,
    ParsedArchitecture, ParsedRepo, SpecFilter,
};
use crate::modules::{
    build_module_document_from_parsed, build_module_lint_report_from_parsed,
    build_repo_document_from_parsed,
};

pub fn build_overview_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<OverviewDocument> {
    let parsed_repo = load_or_parse_repo(root, ignore_patterns, version)?;
    let parsed_arch = load_or_parse_architecture(root, ignore_patterns)?;

    let spec_lint = build_lint_report_from_parsed(&parsed_repo);
    let module_lint = build_module_lint_report_from_parsed(&parsed_arch);
    let unverified_specs = build_spec_document_from_parsed(
        &parsed_repo,
        SpecFilter {
            state: DeclaredStateFilter::Current,
            unverified_only: true,
            scope: None,
        },
        false,
    );
    let unimplemented_modules = build_module_document_from_parsed(
        &parsed_arch,
        ModuleFilter {
            state: DeclaredStateFilter::Current,
            unimplemented_only: true,
            scope: None,
        },
    );
    let repo = build_repo_document_from_parsed(
        root,
        ignore_patterns,
        version,
        &parsed_arch,
        &parsed_repo,
        false,
        None,
        None,
    )?;
    let analysis = repo.analysis;

    Ok(OverviewDocument {
        lint: summarize_lint(&spec_lint.diagnostics, &module_lint.diagnostics),
        specs: summarize_specs(&parsed_repo, &unverified_specs.nodes),
        arch: summarize_arch(&parsed_arch, &unimplemented_modules.nodes),
        health: OverviewHealthSummary {
            duplicate_items: analysis
                .as_ref()
                .and_then(|summary| summary.repo_signals.as_ref())
                .map(|signals| signals.duplicate_items)
                .unwrap_or_default(),
            unowned_items: analysis
                .as_ref()
                .and_then(|summary| summary.repo_signals.as_ref())
                .map(|signals| signals.unowned_items)
                .unwrap_or_default(),
        },
        traceability: None,
    })
}

fn summarize_lint(
    spec_diagnostics: &[crate::model::Diagnostic],
    module_diagnostics: &[crate::model::Diagnostic],
) -> OverviewLintSummary {
    let mut errors = 0;
    let mut warnings = 0;

    for diagnostic in spec_diagnostics.iter().chain(module_diagnostics) {
        match diagnostic.severity {
            DiagnosticSeverity::Error => errors += 1,
            DiagnosticSeverity::Warning => warnings += 1,
        }
    }

    OverviewLintSummary { errors, warnings }
}

fn summarize_specs(
    parsed_repo: &ParsedRepo,
    unverified_nodes: &[crate::model::SpecNode],
) -> OverviewSpecsSummary {
    OverviewSpecsSummary {
        total_specs: parsed_repo
            .specs
            .iter()
            .filter(|decl| decl.kind() == NodeKind::Spec)
            .count(),
        planned_specs: parsed_repo
            .specs
            .iter()
            .filter(|decl| decl.is_planned())
            .count(),
        deprecated_specs: parsed_repo
            .specs
            .iter()
            .filter(|decl| decl.is_deprecated())
            .count(),
        unverified_specs: count_unverified_specs(unverified_nodes),
    }
}

fn summarize_arch(
    parsed_arch: &ParsedArchitecture,
    unimplemented_nodes: &[crate::model::ModuleNode],
) -> OverviewArchSummary {
    OverviewArchSummary {
        total_modules: parsed_arch
            .modules
            .iter()
            .filter(|decl| decl.kind() == crate::model::ArchitectureKind::Module)
            .count(),
        total_areas: parsed_arch
            .modules
            .iter()
            .filter(|decl| decl.kind() == crate::model::ArchitectureKind::Area)
            .count(),
        unimplemented_modules: count_unimplemented_modules(unimplemented_nodes),
    }
}

fn count_unverified_specs(nodes: &[crate::model::SpecNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            usize::from(node.kind() == NodeKind::Spec && node.is_unverified())
                + count_unverified_specs(&node.children)
        })
        .sum()
}

fn count_unimplemented_modules(nodes: &[crate::model::ModuleNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            usize::from(
                node.kind() == crate::model::ArchitectureKind::Module && node.is_unimplemented(),
            ) + count_unimplemented_modules(&node.children)
        })
        .sum()
}
