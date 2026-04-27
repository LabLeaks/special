/**
@module SPECIAL.MODULES.PARSE.DECLARATIONS
Shared architecture declaration helpers for markdown and source-local module parsing, including heading normalization, planned handling, and description accumulation.
*/
// @fileimplements SPECIAL.MODULES.PARSE.DECLARATIONS
use std::path::Path;

use crate::annotation_syntax::{
    ReservedSpecialAnnotation, is_any_tag_boundary, normalize_markdown_annotation_line,
    reserved_special_annotation_rest,
};
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, ParsedArchitecture, PatternStrictness,
    PlannedRelease,
};
use crate::planned_syntax::{
    DeclHeaderError, PlannedAnnotationContext, PlannedAnnotationError, PlannedSyntax,
    parse_decl_header, parse_planned_annotation,
};
pub(crate) use crate::text_lines::skip_blank_lines as skip_blank_doc_lines;

pub(crate) fn normalized_architecture_heading(line: &str) -> Option<(ArchitectureKind, &str)> {
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

pub(crate) fn normalized_pattern_heading(line: &str) -> Option<&str> {
    if !line.trim_start().starts_with('#') {
        return None;
    }
    let trimmed = normalize_markdown_annotation_line(line)?;
    let trimmed = trimmed.trim_start_matches('#').trim();
    let trimmed = trimmed.strip_prefix('`').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('`').unwrap_or(trimmed);
    trimmed.strip_prefix("@pattern ")
}

pub(crate) fn normalized_annotation_line(line: Option<&str>) -> Option<&str> {
    line.and_then(normalize_markdown_annotation_line)
}

pub(crate) fn maybe_consume_doc_planned(
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

pub(crate) fn maybe_consume_pattern_strictness(
    annotation: Option<&str>,
    parsed: &mut ParsedArchitecture,
    path: &Path,
    line: usize,
) -> Option<PatternStrictness> {
    let annotation = annotation?;
    let rest = reserved_special_annotation_rest(annotation, ReservedSpecialAnnotation::Strictness)?;

    let mut parts = rest.split_whitespace();
    let Some(value) = parts.next() else {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: "@strictness must be one of high, medium, or low".to_string(),
        });
        return Some(PatternStrictness::Medium);
    };
    if parts.next().is_some() {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: "@strictness must be one of high, medium, or low".to_string(),
        });
        return Some(PatternStrictness::Medium);
    }
    match PatternStrictness::parse(value) {
        Some(strictness) => Some(strictness),
        None => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: "@strictness must be one of high, medium, or low".to_string(),
            });
            Some(PatternStrictness::Medium)
        }
    }
}

pub(crate) fn collect_doc_description_lines(
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

pub(crate) fn parse_module_header(
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
        Err(DeclHeaderError::InvalidDeprecatedRelease) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: "unexpected trailing content after module id; only `@planned` is supported on module declarations".to_string(),
            });
            return None;
        }
    };

    if header.deprecated {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: format!(
                "@deprecated may only apply to @spec, not {}",
                kind.as_annotation()
            ),
        });
        return None;
    }

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

pub(crate) enum StandalonePlanned {
    Absent,
    Parsed(Option<PlannedRelease>),
    Invalid,
}

pub(crate) fn maybe_consume_standalone_planned(
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

pub(crate) fn parse_pattern_id(
    rest: &str,
    annotation: &str,
    parsed: &mut ParsedArchitecture,
    path: &Path,
    line: usize,
) -> Option<String> {
    let mut parts = rest.split_whitespace();
    let Some(id) = parts.next() else {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: format!("missing pattern id after {annotation}"),
        });
        return None;
    };

    if parts.next().is_some() {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: format!("unexpected trailing content after {annotation} pattern id"),
        });
        return None;
    }

    Some(id.to_string())
}

fn strip_markdown_prefix(text: &str) -> &str {
    text.strip_prefix("- ")
        .or_else(|| text.strip_prefix("* "))
        .unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::normalized_annotation_line;

    #[test]
    fn normalized_annotation_line_preserves_inline_code_at_line_start() {
        assert_eq!(
            normalized_annotation_line(Some(
                "`paypal config` manages `paypal.env.yaml` against linked remote apps."
            )),
            Some("`paypal config` manages `paypal.env.yaml` against linked remote apps.")
        );
    }

    #[test]
    fn normalized_annotation_line_unwraps_whole_line_code_span() {
        assert_eq!(
            normalized_annotation_line(Some("`@module APP.CORE`")),
            Some("@module APP.CORE")
        );
    }
}
