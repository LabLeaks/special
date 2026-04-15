/**
@module SPECIAL.PARSER.MARKDOWN
Parses declarative `@group` and `@spec` annotations from ordinary markdown files under the project root. This module does not attach verifies or attests to code items.
*/
// @fileimplements SPECIAL.PARSER.MARKDOWN
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    Diagnostic, DiagnosticSeverity, NodeKind, ParsedRepo, PlanState, PlannedRelease,
    SourceLocation, SpecDecl,
};
use crate::planned_syntax::PlannedSyntax;

use super::{
    DeclHeader, DeclHeaderError, ParseRules, normalize_markdown_annotation_line,
    parse_standalone_planned, starts_markdown_fence,
};
use crate::planned_syntax::PlannedAnnotationError;

pub(super) fn parse_markdown_declarations(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &mut ParsedRepo,
    rules: ParseRules,
) -> Result<()> {
    for path in discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .markdown_files
    {
        let content = fs::read_to_string(&path)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut index = 0;
        let mut in_code_fence = false;

        while index < lines.len() {
            let raw_line = lines[index];
            if starts_markdown_fence(raw_line) {
                in_code_fence = !in_code_fence;
                index += 1;
                continue;
            }
            if in_code_fence {
                index += 1;
                continue;
            }

            let Some((kind, rest)) = parse_markdown_spec_decl(raw_line) else {
                index += 1;
                continue;
            };
            let line_number = index + 1;
            let Some(header) =
                parse_markdown_decl_header(kind, rest, parsed, &path, line_number, rules.planned)
            else {
                index += 1;
                continue;
            };

            let mut cursor = skip_blank_markdown_lines(&lines, index + 1);
            let (planned, planned_release, next_cursor) =
                maybe_consume_markdown_planned(kind, &lines, cursor, parsed, &path);
            cursor = next_cursor;
            let (description_lines, cursor) = collect_markdown_description_lines(&lines, cursor);

            match SpecDecl::new(
                header.id.to_string(),
                kind,
                description_lines.join(" "),
                if header.planned || planned {
                    PlanState::planned(header.planned_release.or(planned_release))
                } else {
                    PlanState::live()
                },
                SourceLocation {
                    path: path.clone(),
                    line: line_number,
                },
            ) {
                Ok(spec) => parsed.specs.push(spec),
                Err(err) => parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    line: line_number,
                    message: err.to_string(),
                }),
            }

            index = cursor;
        }
    }

    Ok(())
}

fn parse_markdown_spec_decl(line: &str) -> Option<(NodeKind, &str)> {
    if !line.trim_start().starts_with('#') {
        return None;
    }
    let trimmed = normalize_markdown_annotation_line(line)?;
    if let Some(rest) = trimmed.strip_prefix("@spec ") {
        Some((NodeKind::Spec, rest))
    } else {
        trimmed
            .strip_prefix("@group ")
            .map(|rest| (NodeKind::Group, rest))
    }
}

fn parse_markdown_decl_header<'a>(
    kind: NodeKind,
    rest: &'a str,
    parsed: &mut ParsedRepo,
    path: &Path,
    line: usize,
    planned_syntax: PlannedSyntax,
) -> Option<DeclHeader<'a>> {
    let rest = rest.trim();
    let mut header = match DeclHeader::parse(rest, planned_syntax) {
        Ok(header) => header,
        Err(DeclHeaderError::MissingId) => {
            let annotation = if kind == NodeKind::Spec {
                "@spec"
            } else {
                "@group"
            };
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: format!("missing spec id after {annotation}"),
            });
            return None;
        }
        Err(DeclHeaderError::InvalidTrailingContent) => {
            let message = match (kind, planned_syntax) {
                (NodeKind::Group, _) => {
                    "unexpected trailing content after group id; only the id belongs on the @group line"
                }
                (NodeKind::Spec, PlannedSyntax::LegacyBackward) => {
                    "unexpected trailing content after spec id; compatibility parsing does not allow inline `@planned`"
                }
                (NodeKind::Spec, PlannedSyntax::AdjacentOwnedSpec) => {
                    "unexpected trailing content after spec id; use an exact trailing `@planned` marker if needed"
                }
            };
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line,
                message: message.to_string(),
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

    if header.planned && kind == NodeKind::Group {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: "@planned may only apply to @spec, not @group".to_string(),
        });
        header.planned = false;
    }

    Some(header)
}

fn maybe_consume_markdown_planned(
    kind: NodeKind,
    lines: &[&str],
    cursor: usize,
    parsed: &mut ParsedRepo,
    path: &Path,
) -> (bool, Option<PlannedRelease>, usize) {
    let Some(annotation) = lines
        .get(cursor)
        .and_then(|line| normalize_markdown_annotation_line(line))
    else {
        return (false, None, cursor);
    };
    let Some(result) = parse_standalone_planned(annotation) else {
        return (false, None, cursor);
    };

    match result {
        Ok(planned_release) => {
            if kind == NodeKind::Group {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.to_path_buf(),
                    line: cursor + 1,
                    message: "@planned may only apply to @spec, not @group".to_string(),
                });
                (false, None, skip_blank_markdown_lines(lines, cursor + 1))
            } else {
                (
                    true,
                    planned_release,
                    skip_blank_markdown_lines(lines, cursor + 1),
                )
            }
        }
        Err(PlannedAnnotationError::InvalidRelease) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line: cursor + 1,
                message: "planned release metadata must not be empty".to_string(),
            });
            (false, None, skip_blank_markdown_lines(lines, cursor + 1))
        }
        Err(PlannedAnnotationError::InvalidSuffix) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: path.to_path_buf(),
                line: cursor + 1,
                message: "use an exact standalone `@planned` marker with no trailing suffix"
                    .to_string(),
            });
            (false, None, skip_blank_markdown_lines(lines, cursor + 1))
        }
    }
}

fn skip_blank_markdown_lines(lines: &[&str], mut index: usize) -> usize {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }
    index
}

fn collect_markdown_description_lines(lines: &[&str], mut cursor: usize) -> (Vec<String>, usize) {
    let mut description_lines = Vec::new();
    let mut in_code_fence = false;
    while cursor < lines.len() {
        let raw = lines[cursor];
        if starts_markdown_fence(raw) {
            in_code_fence = !in_code_fence;
            cursor += 1;
            continue;
        }
        if in_code_fence {
            cursor += 1;
            continue;
        }

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            if !description_lines.is_empty() {
                break;
            }
            cursor += 1;
            continue;
        }

        if parse_markdown_spec_decl(raw).is_some()
            || normalize_markdown_annotation_line(raw)
                .is_some_and(crate::annotation_syntax::is_any_tag_boundary)
        {
            break;
        }

        if let Some(text) = normalize_markdown_text_line(raw) {
            description_lines.push(text.to_string());
        }
        cursor += 1;
    }
    (description_lines, cursor)
}

fn normalize_markdown_text_line(line: &str) -> Option<&str> {
    let trimmed = normalize_markdown_annotation_line(line)?;
    if trimmed.starts_with('@') || trimmed.starts_with('\\') {
        return None;
    }
    Some(trimmed)
}
