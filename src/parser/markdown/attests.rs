/**
@module SPECIAL.PARSER.MARKDOWN.ATTESTS
Parses markdown `@fileattests` metadata and records whole-document attestation bodies for declarative markdown sources.
*/
// @fileimplements SPECIAL.PARSER.MARKDOWN.ATTESTS
use std::path::Path;

use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::model::{
    AttestRef, AttestScope, Diagnostic, DiagnosticSeverity, ParsedRepo, SourceLocation,
};

use super::super::attestation::parse_markdown_attestation_metadata;
use super::super::{normalize_markdown_annotation_line, starts_markdown_fence};
use super::declarations::skip_blank_markdown_lines;

pub(super) fn parse_markdown_file_attest(line: &str) -> Option<&str> {
    if !line.trim_start().starts_with('#') {
        return None;
    }
    let trimmed = normalize_markdown_annotation_line(line)?;
    reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::FileAttests)
}

pub(super) fn handle_markdown_file_attest(
    content: &str,
    path: &Path,
    lines: &[&str],
    parsed: &mut ParsedRepo,
    index: usize,
    rest: &str,
) -> usize {
    let line_number = index + 1;
    let id = rest.trim();
    if id.is_empty() {
        parsed.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            path: path.to_path_buf(),
            line: line_number,
            message: "missing spec id after @fileattests".to_string(),
        });
        return index + 1;
    }

    let metadata_start = skip_blank_markdown_lines(lines, index + 1);
    let (attestation, cursor) = parse_markdown_attestation_metadata(
        parsed,
        path,
        line_number,
        lines,
        metadata_start,
        normalize_markdown_annotation_line,
        starts_markdown_fence,
    );
    if let Some(attestation) = attestation {
        parsed.attests.push(AttestRef {
            spec_id: id.to_string(),
            artifact: attestation.artifact,
            owner: attestation.owner,
            last_reviewed: attestation.last_reviewed,
            review_interval_days: attestation.review_interval_days,
            scope: AttestScope::File,
            location: SourceLocation {
                path: path.to_path_buf(),
                line: line_number,
            },
            body: Some(content.trim_end().to_string()),
        });
    }

    cursor
}
