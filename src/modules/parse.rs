/**
@module SPECIAL.MODULES.PARSE
Parses architecture declarations from ordinary markdown files and source-local comments, including `@implements`, `@fileimplements`, and planned module metadata.
*/
// @fileimplements SPECIAL.MODULES.PARSE
use std::path::Path;

use anyhow::Result;

use crate::annotation_syntax::{
    ReservedSpecialAnnotation, is_any_tag_boundary, reserved_special_annotation_rest,
};
use crate::extractor::collect_comment_blocks;
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, ImplementRef, ModuleDecl, ParsedArchitecture,
    PlanState, PlannedRelease, SourceLocation,
};
use crate::planned_syntax::{
    DeclHeaderError, PlannedAnnotationContext, PlannedAnnotationError, PlannedSyntax,
    parse_decl_header, parse_planned_annotation,
};

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

fn parse_source_module_decls(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &mut ParsedArchitecture,
) -> Result<()> {
    for block in collect_comment_blocks(root, ignore_patterns)? {
        let mut index = 0;

        while index < block.lines.len() {
            let entry = &block.lines[index];
            let trimmed = entry.text.trim();

            let Some((kind, rest)) = parse_source_architecture_decl(trimmed) else {
                index += 1;
                continue;
            };

            let Some((id, inline_release, inline_planned)) =
                parse_module_header(kind, rest, parsed, &block.path, entry.line)
            else {
                index += 1;
                continue;
            };

            let mut cursor = skip_blank_block_lines(&block, index + 1);
            let (planned, planned_release, next_cursor) = maybe_consume_block_planned(
                kind,
                &block,
                cursor,
                parsed,
                inline_planned,
                inline_release,
            );
            cursor = next_cursor;
            let (description_lines, cursor) = collect_block_description_lines(&block, cursor);

            let module = match ModuleDecl::new(
                id,
                kind,
                description_lines.join(" "),
                if planned {
                    PlanState::planned(planned_release)
                } else {
                    PlanState::live()
                },
                SourceLocation {
                    path: block.path.clone(),
                    line: entry.line,
                },
            ) {
                Ok(module) => module,
                Err(err) => {
                    parsed.diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        path: block.path.clone(),
                        line: entry.line,
                        message: err.to_string(),
                    });
                    index = cursor;
                    continue;
                }
            };
            parsed.modules.push(module);

            index = cursor;
        }
    }

    Ok(())
}

fn parse_implements_refs(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &mut ParsedArchitecture,
) -> Result<()> {
    for block in collect_comment_blocks(root, ignore_patterns)? {
        for entry in &block.lines {
            let trimmed = entry.text.trim();
            let (rest, file_scoped, annotation) = if let Some(rest) =
                reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Implements)
            {
                (rest, false, "@implements")
            } else if let Some(rest) =
                reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::FileImplements)
            {
                (rest, true, "@fileimplements")
            } else {
                continue;
            };
            let Some(module_id) =
                parse_implements_module_id(rest, annotation, parsed, &block.path, entry.line)
            else {
                continue;
            };

            let (body_location, body) = if file_scoped {
                (None, None)
            } else if let Some(owned_item) = &block.owned_item {
                (
                    Some(owned_item.location.clone()),
                    Some(owned_item.body.clone()),
                )
            } else {
                (None, None)
            };

            parsed.implements.push(ImplementRef {
                module_id,
                location: SourceLocation {
                    path: block.path.clone(),
                    line: entry.line,
                },
                body_location,
                body,
            });
        }
    }

    Ok(())
}

fn parse_implements_module_id(
    rest: &str,
    annotation: &str,
    parsed: &mut ParsedArchitecture,
    path: &Path,
    line: usize,
) -> Option<String> {
    let mut parts = rest.split_whitespace();
    let Some(module_id) = parts.next() else {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: format!("missing module id after {annotation}"),
        });
        return None;
    };

    if parts.next().is_some() {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line,
            message: format!("unexpected trailing content after {annotation} module id"),
        });
        return None;
    }

    Some(module_id.to_string())
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

fn skip_blank_block_lines(block: &crate::model::CommentBlock, mut index: usize) -> usize {
    while index < block.lines.len() && block.lines[index].text.trim().is_empty() {
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

fn maybe_consume_block_planned(
    kind: ArchitectureKind,
    block: &crate::model::CommentBlock,
    cursor: usize,
    parsed: &mut ParsedArchitecture,
    planned: bool,
    planned_release: Option<PlannedRelease>,
) -> (bool, Option<PlannedRelease>, usize) {
    if planned {
        return (planned, planned_release, cursor);
    }
    if let Some(annotation) = block.lines.get(cursor).map(|line| line.text.trim()) {
        match maybe_consume_standalone_planned(
            kind,
            annotation,
            parsed,
            &block.path,
            block.lines[cursor].line,
        ) {
            StandalonePlanned::Absent => {}
            StandalonePlanned::Parsed(release) => {
                let next = skip_blank_block_lines(block, cursor + 1);
                return (true, release, next);
            }
            StandalonePlanned::Invalid => {
                let next = skip_blank_block_lines(block, cursor + 1);
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

fn collect_block_description_lines(
    block: &crate::model::CommentBlock,
    mut cursor: usize,
) -> (Vec<String>, usize) {
    let mut description_lines = Vec::new();
    while cursor < block.lines.len() {
        let text = block.lines[cursor].text.trim();
        if is_any_tag_boundary(text) {
            break;
        }
        if !text.is_empty() {
            description_lines.push(text.to_string());
        }
        cursor += 1;
    }
    (description_lines, cursor)
}

fn parse_source_architecture_decl(trimmed: &str) -> Option<(ArchitectureKind, &str)> {
    if let Some(rest) = reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Module)
    {
        Some((ArchitectureKind::Module, rest))
    } else {
        reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Area)
            .map(|rest| (ArchitectureKind::Area, rest))
    }
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

enum StandalonePlanned {
    Absent,
    Parsed(Option<PlannedRelease>),
    Invalid,
}

fn maybe_consume_standalone_planned(
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
