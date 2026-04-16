/**
@module SPECIAL.PARSER.MARKDOWN.DECLARATIONS
Shared markdown declaration helpers for spec and group headings, adjacent planned markers, blank-line skipping, and description accumulation.
*/
// @fileimplements SPECIAL.PARSER.MARKDOWN.DECLARATIONS
use std::path::Path;

use crate::annotation_syntax::is_any_tag_boundary;
use crate::model::{Diagnostic, DiagnosticSeverity, NodeKind, ParsedRepo, PlannedRelease};
use crate::planned_syntax::PlannedSyntax;

use super::super::declarations::{
    AdjacentPlanned, parse_adjacent_spec_planned, parse_spec_decl_header,
};
use super::super::normalize_markdown_annotation_line;

pub(super) fn parse_markdown_spec_decl(line: &str) -> Option<(NodeKind, &str)> {
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

pub(super) fn skip_blank_markdown_lines(lines: &[&str], mut index: usize) -> usize {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }
    index
}

pub(super) fn maybe_consume_markdown_planned(
    kind: NodeKind,
    lines: &[&str],
    cursor: usize,
    parsed: &mut ParsedRepo,
    path: &Path,
    planned_syntax: PlannedSyntax,
) -> (bool, Option<PlannedRelease>, usize) {
    let Some(annotation) = lines
        .get(cursor)
        .and_then(|line| normalize_markdown_annotation_line(line))
    else {
        return (false, None, cursor);
    };

    let (state, release, message) = parse_adjacent_spec_planned(kind, annotation, planned_syntax);
    match state {
        AdjacentPlanned::Absent => (false, None, cursor),
        AdjacentPlanned::Parsed => (true, release, skip_blank_markdown_lines(lines, cursor + 1)),
        AdjacentPlanned::Invalid => {
            if let Some(message) = message {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.to_path_buf(),
                    line: cursor + 1,
                    message: message.to_string(),
                });
            }
            (false, None, skip_blank_markdown_lines(lines, cursor + 1))
        }
    }
}

pub(super) fn collect_markdown_description_lines(
    lines: &[&str],
    mut cursor: usize,
    starts_markdown_fence: fn(&str) -> bool,
) -> (Vec<String>, usize) {
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
            || normalize_markdown_annotation_line(raw).is_some_and(is_any_tag_boundary)
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

pub(super) fn parse_markdown_decl_header(
    kind: NodeKind,
    rest: &str,
    planned_syntax: PlannedSyntax,
) -> Result<(crate::planned_syntax::ParsedDeclHeader<'_>, Option<String>), String> {
    parse_spec_decl_header(kind, rest, planned_syntax)
}

fn normalize_markdown_text_line(line: &str) -> Option<&str> {
    let trimmed = normalize_markdown_annotation_line(line)?;
    if trimmed.starts_with('@') || trimmed.starts_with('\\') {
        return None;
    }
    Some(trimmed)
}
