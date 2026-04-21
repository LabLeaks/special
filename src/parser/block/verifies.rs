/**
@module SPECIAL.PARSER.BLOCK.VERIFIES
Handles `@verifies` and `@fileverifies` parsing within a source comment block.
*/
// @fileimplements SPECIAL.PARSER.BLOCK.VERIFIES
use std::fs;

use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::model::{CommentBlock, ParsedRepo, SourceLocation, VerifyRef};

use super::super::push_diag;

pub(super) fn handle_verify_line(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    seen_verifies: &mut bool,
    line: usize,
    trimmed: &str,
) -> bool {
    if let Some(rest) =
        reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Verifies)
    {
        handle_item_verify(block, parsed, seen_verifies, line, rest);
        return true;
    }

    if let Some(rest) =
        reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::FileVerifies)
    {
        handle_file_verify(block, parsed, seen_verifies, line, rest);
        return true;
    }

    false
}

fn handle_item_verify(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    seen_verifies: &mut bool,
    line: usize,
    rest: &str,
) {
    if *seen_verifies {
        push_diag(
            parsed,
            block,
            line,
            "annotation block may contain only one @verifies",
        );
        return;
    }

    let Some(id) = parse_spec_id(rest, parsed, block, line, "@verifies") else {
        return;
    };
    if let Some(owned_item) = &block.owned_item {
        parsed.verifies.push(VerifyRef {
            spec_id: id,
            location: SourceLocation {
                path: block.path.clone(),
                line,
            },
            body_location: Some(owned_item.location.clone()),
            body: Some(owned_item.body.clone()),
        });
        *seen_verifies = true;
    } else {
        push_diag(
            parsed,
            block,
            line,
            "@verifies must attach to the next supported item",
        );
        *seen_verifies = true;
    }
}

fn handle_file_verify(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    seen_verifies: &mut bool,
    line: usize,
    rest: &str,
) {
    if *seen_verifies {
        push_diag(
            parsed,
            block,
            line,
            "annotation block may contain only one @verifies or @fileverifies",
        );
        return;
    }

    let Some(id) = parse_spec_id(rest, parsed, block, line, "@fileverifies") else {
        return;
    };
    let body = fs::read_to_string(&block.path)
        .ok()
        .map(|body| body.trim_end().to_string());
    parsed.verifies.push(VerifyRef {
        spec_id: id,
        location: SourceLocation {
            path: block.path.clone(),
            line,
        },
        body_location: None,
        body,
    });
    *seen_verifies = true;
}

fn parse_spec_id(
    rest: &str,
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    line: usize,
    annotation: &str,
) -> Option<String> {
    let mut parts = rest.split_whitespace();
    let Some(id) = parts.next() else {
        push_diag(
            parsed,
            block,
            line,
            &format!("missing spec id after {annotation}"),
        );
        return None;
    };
    if parts.next().is_some() {
        push_diag(
            parsed,
            block,
            line,
            &format!("unexpected trailing content after {annotation} spec id"),
        );
        return None;
    }
    Some(id.to_string())
}
