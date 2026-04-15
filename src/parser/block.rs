/**
@module SPECIAL.PARSER.BLOCK
Scans extracted source comment blocks, routes reserved annotation lines to their owning handlers, and coordinates block-local attachment semantics.
*/
// @fileimplements SPECIAL.PARSER.BLOCK
mod declarations;
mod verifies;

use crate::annotation_syntax::{
    ReservedSpecialAnnotation, is_any_tag_boundary, reserved_special_annotation,
    reserved_special_annotation_rest,
};
use crate::model::{AttestRef, CommentBlock, ParsedRepo, SourceLocation};
use crate::planned_syntax::PlannedSyntax;

use self::declarations::{BlockState, handle_decl_line, handle_standalone_planned_line};
use self::verifies::handle_verify_line;
use super::attestation::parse_attestation_metadata;
use super::{ParseDialect, push_diag};

#[derive(Debug, Clone, Copy)]
pub(super) struct ParseRules {
    pub(super) planned: PlannedSyntax,
}

impl ParseRules {
    pub(super) fn for_dialect(dialect: ParseDialect) -> Self {
        let planned = match dialect {
            ParseDialect::CompatibilityV0 => PlannedSyntax::LegacyBackward,
            ParseDialect::CurrentV1 => PlannedSyntax::AdjacentOwnedSpec,
        };
        Self { planned }
    }
}

pub(super) fn parse_block(block: &CommentBlock, parsed: &mut ParsedRepo, rules: ParseRules) {
    let mut index = 0;
    let mut state = BlockState::default();
    let mut seen_verifies = false;

    while index < block.lines.len() {
        let entry = &block.lines[index];
        let trimmed = entry.text.trim();

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some(next_index) =
            handle_decl_line(block, parsed, &mut state, index, entry.line, trimmed, rules)
        {
            index = next_index;
            continue;
        }

        if handle_standalone_planned_line(block, parsed, &state, entry.line, trimmed, rules) {
            index += 1;
            continue;
        }

        if handle_verify_line(block, parsed, &mut seen_verifies, entry.line, trimmed) {
            index += 1;
            continue;
        }

        if let Some(next_index) = handle_attest_line(block, parsed, index, entry.line, trimmed) {
            index = next_index;
            continue;
        }

        if is_reserved_arch_annotation(trimmed) {
            index = skip_reserved_arch_annotation(block, index + 1);
            continue;
        }

        index += 1;
    }
}

fn handle_attest_line(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    index: usize,
    line: usize,
    trimmed: &str,
) -> Option<usize> {
    let rest = reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Attests)?;
    let id = rest.trim();
    if id.is_empty() {
        push_diag(parsed, block, line, "missing spec id after @attests");
        return Some(index + 1);
    }

    let (attestation, cursor) = parse_attestation_metadata(parsed, block, line, index + 1);
    if let Some(attestation) = attestation {
        parsed.attests.push(AttestRef {
            spec_id: id.to_string(),
            artifact: attestation.artifact,
            owner: attestation.owner,
            last_reviewed: attestation.last_reviewed,
            review_interval_days: attestation.review_interval_days,
            location: SourceLocation {
                path: block.path.clone(),
                line,
            },
            body: Some(
                block
                    .lines
                    .iter()
                    .map(|line| line.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim()
                    .to_string(),
            ),
        });
    }

    Some(cursor)
}

pub(super) fn collect_description_lines(block: &CommentBlock, cursor: &mut usize) -> Vec<String> {
    let mut description_lines = Vec::new();
    while *cursor < block.lines.len() {
        let text = block.lines[*cursor].text.trim();
        if is_any_tag_boundary(text) {
            break;
        }
        if !text.is_empty() {
            description_lines.push(text.to_string());
        }
        *cursor += 1;
    }
    description_lines
}

fn skip_reserved_arch_annotation(block: &CommentBlock, mut index: usize) -> usize {
    while index < block.lines.len() {
        let text = block.lines[index].text.trim();
        if text.is_empty() {
            index += 1;
            continue;
        }
        if is_any_tag_boundary(text) {
            break;
        }
        index += 1;
    }
    index
}

fn is_reserved_arch_annotation(trimmed: &str) -> bool {
    matches!(
        reserved_special_annotation(trimmed),
        Some(ReservedSpecialAnnotation::Module)
            | Some(ReservedSpecialAnnotation::Area)
            | Some(ReservedSpecialAnnotation::Implements)
            | Some(ReservedSpecialAnnotation::FileImplements)
    )
}
