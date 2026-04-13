/**
@module SPECIAL.INDEX.LINT
Builds spec lint diagnostics from parsed annotations. This module does not choose parser dialects or materialize spec trees.
*/
// @implements SPECIAL.INDEX.LINT
use std::collections::{BTreeMap, BTreeSet};

use crate::model::{Diagnostic, DiagnosticSeverity, LintReport, NodeKind, ParsedRepo};

pub(super) fn lint_from_parsed(parsed: &ParsedRepo) -> LintReport {
    let mut diagnostics = parsed.diagnostics.clone();
    let mut declared: BTreeMap<String, (&std::path::Path, usize, NodeKind)> = BTreeMap::new();

    for spec in &parsed.specs {
        if let Some((previous_path, previous_line, previous_kind)) = declared.insert(
            spec.id.clone(),
            (&spec.location.path, spec.location.line, spec.kind()),
        ) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: spec.location.path.clone(),
                line: spec.location.line,
                message: format!(
                    "duplicate node id `{}`; first declared as {} at {}:{}",
                    spec.id,
                    kind_label(previous_kind),
                    previous_path.display(),
                    previous_line
                ),
            });
        }
    }

    let ids: BTreeSet<String> = parsed.specs.iter().map(|spec| spec.id.clone()).collect();
    let kinds: BTreeMap<&str, NodeKind> = parsed
        .specs
        .iter()
        .map(|spec| (spec.id.as_str(), spec.kind()))
        .collect();

    for spec in &parsed.specs {
        for missing in missing_intermediates(&spec.id, &ids) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: spec.location.path.clone(),
                line: spec.location.line,
                message: format!("missing intermediate spec `{missing}`"),
            });
        }
    }

    for verify in &parsed.verifies {
        if !ids.contains(&verify.spec_id) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: verify.location.path.clone(),
                line: verify.location.line,
                message: format!(
                    "unknown spec id `{}` referenced by @verifies",
                    verify.spec_id
                ),
            });
        } else if kinds.get(verify.spec_id.as_str()) == Some(&NodeKind::Group) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: verify.location.path.clone(),
                line: verify.location.line,
                message: format!(
                    "cannot reference @group `{}` from @verifies",
                    verify.spec_id
                ),
            });
        }
    }

    for attest in &parsed.attests {
        if !ids.contains(&attest.spec_id) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: attest.location.path.clone(),
                line: attest.location.line,
                message: format!(
                    "unknown spec id `{}` referenced by @attests",
                    attest.spec_id
                ),
            });
        } else if kinds.get(attest.spec_id.as_str()) == Some(&NodeKind::Group) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: attest.location.path.clone(),
                line: attest.location.line,
                message: format!("cannot reference @group `{}` from @attests", attest.spec_id),
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

fn missing_intermediates(id: &str, declared: &BTreeSet<String>) -> Vec<String> {
    let mut missing = Vec::new();
    let mut cursor = id;
    while let Some(parent) = immediate_parent(cursor) {
        if !declared.contains(parent) {
            missing.push(parent.to_string());
        }
        cursor = parent;
    }
    missing
}

fn immediate_parent(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(parent, _)| parent)
}

fn kind_label(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Spec => "@spec",
        NodeKind::Group => "@group",
    }
}
