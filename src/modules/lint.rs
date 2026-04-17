/**
@module SPECIAL.MODULES.LINT
Builds module lint diagnostics from parsed architecture declarations and implementation attachments. This module does not read source files or materialize the module tree.
*/
// @fileimplements SPECIAL.MODULES.LINT
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::immediate_parent_id;
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, LintReport, ParsedArchitecture,
};

pub(super) fn build_module_lint_report(parsed: &ParsedArchitecture) -> LintReport {
    let mut diagnostics = parsed.diagnostics.clone();
    let mut declared: BTreeMap<String, usize> = BTreeMap::new();

    for module in &parsed.modules {
        if let Some(previous_line) = declared.insert(module.id.clone(), module.location.line) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: module.location.path.clone(),
                line: module.location.line,
                message: format!(
                    "duplicate module id `{}`; first declared on line {}",
                    module.id, previous_line
                ),
            });
        }
    }

    let ids: BTreeSet<String> = parsed
        .modules
        .iter()
        .map(|module| module.id.clone())
        .collect();
    let kinds: BTreeMap<String, ArchitectureKind> = parsed
        .modules
        .iter()
        .map(|module| (module.id.clone(), module.kind()))
        .collect();

    for module in &parsed.modules {
        for missing in missing_intermediates(&module.id, &ids) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: module.location.path.clone(),
                line: module.location.line,
                message: format!("missing intermediate module `{missing}`"),
            });
        }
    }

    for implementation in &parsed.implements {
        match kinds.get(&implementation.module_id) {
            None => diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: implementation.location.path.clone(),
                line: implementation.location.line,
                message: format!(
                    "unknown module id `{}` referenced by @implements or @fileimplements",
                    implementation.module_id
                ),
            }),
            Some(ArchitectureKind::Area) => diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: implementation.location.path.clone(),
                line: implementation.location.line,
                message: format!(
                    "@implements and @fileimplements may only reference @module ids; `{}` is declared as @area",
                    implementation.module_id
                ),
            }),
            Some(ArchitectureKind::Module) => {}
        }
    }

    let mut file_scoped: BTreeMap<PathBuf, usize> = BTreeMap::new();
    let mut item_scoped: BTreeMap<(PathBuf, usize), usize> = BTreeMap::new();

    for implementation in &parsed.implements {
        if let Some(body_location) = &implementation.body_location {
            let key = (body_location.path.clone(), body_location.line);
            if let Some(previous_line) = item_scoped.insert(key, implementation.location.line) {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: implementation.location.path.clone(),
                    line: implementation.location.line,
                    message: format!(
                        "duplicate @implements for attached item starting on line {}; first declared on line {}",
                        body_location.line, previous_line
                    ),
                });
            }
        } else {
            let key = implementation.location.path.clone();
            if let Some(previous_line) = file_scoped.insert(key, implementation.location.line) {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: implementation.location.path.clone(),
                    line: implementation.location.line,
                    message: format!(
                        "duplicate @fileimplements; first declared on line {}",
                        previous_line
                    ),
                });
            }
        }
    }

    for module in &parsed.modules {
        if module.kind() == ArchitectureKind::Module
            && !module.is_planned()
            && !parsed
                .implements
                .iter()
                .any(|implementation| implementation.module_id == module.id)
        {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: module.location.path.clone(),
                line: module.location.line,
                message: format!(
                    "live module `{}` has no direct @implements or @fileimplements attachment",
                    module.id
                ),
            });
        }
    }

    diagnostics.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then(left.path.cmp(&right.path))
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });
    diagnostics.dedup_by(|left, right| {
        left.severity == right.severity
            && left.path == right.path
            && left.line == right.line
            && left.message == right.message
    });

    LintReport { diagnostics }
}

fn missing_intermediates<'a>(id: &'a str, ids: &BTreeSet<String>) -> Vec<&'a str> {
    let mut missing = Vec::new();
    let mut prefix = id;
    while let Some(parent) = immediate_parent_id(prefix) {
        if !ids.contains(parent) {
            missing.push(parent);
        }
        prefix = parent;
    }
    missing.reverse();
    missing
}
