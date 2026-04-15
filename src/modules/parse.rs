/**
@module SPECIAL.MODULES.PARSE
Coordinates architecture parsing across markdown declarations, source-local declarations, and explicit implementation attachments.
*/
// @fileimplements SPECIAL.MODULES.PARSE
use std::path::Path;

use anyhow::Result;

use crate::annotation_syntax::is_any_tag_boundary;
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, ParsedArchitecture, PlannedRelease,
};
use crate::planned_syntax::{
    DeclHeaderError, PlannedAnnotationContext, PlannedAnnotationError, PlannedSyntax,
    parse_decl_header, parse_planned_annotation,
};

mod implements;
mod source;

use implements::parse_implements_refs;
use source::parse_source_module_decls;

use super::parse_markdown::parse_markdown_architecture_decls as parse_markdown_architecture_nodes;

pub(super) fn parse_architecture(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<ParsedArchitecture> {
    let mut parsed = ParsedArchitecture::default();
    parse_markdown_architecture_nodes(root, ignore_patterns, &mut parsed)?;
    parse_source_module_decls(root, ignore_patterns, &mut parsed)?;
    parse_implements_refs(root, ignore_patterns, &mut parsed)?;
    Ok(parsed)
}

pub(super) fn normalized_architecture_heading(line: &str) -> Option<(ArchitectureKind, &str)> {
    if !line.trim_start().starts_with('#') {
        return None;
    }
    let trimmed = normalize_markdown_annotation_line(line)?;
    let trimmed = trimmed.trim_start_matches('#').trim();
    let trimmed = trimmed.strip_prefix('`').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('`').unwrap_or(trimmed);
    if let Some(rest) = trimmed.strip_prefix("@module ") {
        Some((ArchitectureKind::Module, rest))
    } else {
        trimmed
            .strip_prefix("@area ")
            .map(|rest| (ArchitectureKind::Area, rest))
    }
}

fn normalized_annotation_line(line: Option<&str>) -> Option<&str> {
    line.and_then(normalize_markdown_annotation_line)
}

pub(super) fn skip_blank_doc_lines(lines: &[&str], mut index: usize) -> usize {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }
    index
}

pub(super) fn maybe_consume_doc_planned(
    kind: ArchitectureKind,
    lines: &[&str],
    cursor: usize,
    parsed: &mut ParsedArchitecture,
    path: &Path,
    planned: bool,
    planned_release: Option<PlannedRelease>,
) -> (bool, Option<PlannedRelease>, usize) {
    if planned {
        return (planned, planned_release, cursor);
    }
    if let Some(annotation) = normalized_annotation_line(lines.get(cursor).copied()) {
        match maybe_consume_standalone_planned(kind, annotation, parsed, path, cursor + 1) {
            StandalonePlanned::Absent => {}
            StandalonePlanned::Parsed(release) => {
                let next = skip_blank_doc_lines(lines, cursor + 1);
                return (true, release, next);
            }
            StandalonePlanned::Invalid => {
                let next = skip_blank_doc_lines(lines, cursor + 1);
                return (false, None, next);
            }
        }
    }
    (planned, planned_release, cursor)
}

pub(super) fn collect_doc_description_lines(
    lines: &[&str],
    mut cursor: usize,
) -> (Vec<String>, usize) {
    let mut description_lines = Vec::new();
    while cursor < lines.len() {
        if normalized_architecture_heading(lines[cursor]).is_some() {
            break;
        }

        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            if !description_lines.is_empty() {
                break;
            }
            cursor += 1;
            continue;
        }

        if trimmed.starts_with("##") || is_any_tag_boundary(trimmed) {
            break;
        }

        description_lines.push(strip_markdown_prefix(trimmed).to_string());
        cursor += 1;
    }
    (description_lines, cursor)
}

pub(super) fn parse_module_header(
    kind: ArchitectureKind,
    rest: &str,
    parsed: &mut ParsedArchitecture,
    path: &Path,
    line: usize,
) -> Option<(String, Option<PlannedRelease>, bool)> {
    let header = match parse_decl_header(rest.trim(), PlannedSyntax::AdjacentOwnedSpec) {
        Ok(header) => header,
        Err(DeclHeaderError::MissingId) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: format!("missing module id after {}", kind.as_annotation()),
            });
            return None;
        }
        Err(DeclHeaderError::InvalidTrailingContent) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: format!(
                    "unexpected trailing content after {} id; use an exact trailing `@planned` marker if needed",
                    kind.as_annotation()
                ),
            });
            return None;
        }
        Err(DeclHeaderError::InvalidPlannedRelease) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: "planned release metadata must not be empty".to_string(),
            });
            return None;
        }
    };

    if header.planned && kind != ArchitectureKind::Module {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: format!(
                "@planned may only apply to @module, not {}",
                kind.as_annotation()
            ),
        });
        return None;
    }

    Some((
        header.id.to_string(),
        header.planned_release,
        header.planned,
    ))
}

pub(super) enum StandalonePlanned {
    Absent,
    Parsed(Option<PlannedRelease>),
    Invalid,
}

pub(super) fn maybe_consume_standalone_planned(
    kind: ArchitectureKind,
    text: &str,
    parsed: &mut ParsedArchitecture,
    path: &Path,
    line: usize,
) -> StandalonePlanned {
    let Some(result) = parse_planned_annotation(text, PlannedAnnotationContext::Standalone) else {
        return StandalonePlanned::Absent;
    };

    match result {
        Ok(annotation) => {
            if kind != ArchitectureKind::Module {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.to_path_buf(),
                    line,
                    message: format!(
                        "@planned may only apply to @module, not {}",
                        kind.as_annotation()
                    ),
                });
                return StandalonePlanned::Invalid;
            }

            StandalonePlanned::Parsed(annotation.release)
        }
        Err(PlannedAnnotationError::InvalidRelease) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: "planned release metadata must not be empty".to_string(),
            });
            StandalonePlanned::Invalid
        }
        Err(PlannedAnnotationError::InvalidSuffix) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: "use an exact standalone `@planned` marker with no trailing suffix"
                    .to_string(),
            });
            StandalonePlanned::Invalid
        }
    }
}

fn strip_markdown_prefix(text: &str) -> &str {
    text.strip_prefix("- ")
        .or_else(|| text.strip_prefix("* "))
        .unwrap_or(text)
}

fn normalize_markdown_annotation_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let trimmed = trimmed
        .strip_prefix('>')
        .map(str::trim_start)
        .unwrap_or(trimmed);
    let trimmed = trimmed.trim_start_matches('#').trim_start();
    let trimmed = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .unwrap_or(trimmed);
    let trimmed = trimmed.strip_prefix('`').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('`').unwrap_or(trimmed);
    Some(trimmed.trim())
}
