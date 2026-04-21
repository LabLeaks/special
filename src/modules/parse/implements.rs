/**
@module SPECIAL.MODULES.PARSE.IMPLEMENTS
Parses explicit `@implements` and `@fileimplements` attachments from source-local comments and records their body ownership semantics.
*/
// @fileimplements SPECIAL.MODULES.PARSE.IMPLEMENTS
use std::path::Path;

use anyhow::Result;

use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::extractor::collect_comment_blocks;
use crate::model::{
    Diagnostic, DiagnosticSeverity, ImplementRef, ParsedArchitecture, SourceLocation,
};

pub(super) fn parse_implements_refs(
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
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: block.path.clone(),
                    line: entry.line,
                    message: "@implements must attach to the next supported item".to_string(),
                });
                continue;
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
