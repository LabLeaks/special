/**
@module SPECIAL.MODULES.PARSE.SOURCE
Parses source-local `@module` and `@area` declarations from extracted comment blocks, including adjacent planned metadata and block-local description text.
*/
// @fileimplements SPECIAL.MODULES.PARSE.SOURCE
use std::path::Path;

use anyhow::Result;

use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::extractor::collect_comment_blocks;
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, ModuleDecl, ParsedArchitecture, PlanState,
    PlannedRelease, SourceLocation,
};

use super::declarations::{
    StandalonePlanned, maybe_consume_standalone_planned, parse_module_header,
};

pub(super) fn parse_source_module_decls(
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
                    PlanState::current()
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

fn skip_blank_block_lines(block: &crate::model::CommentBlock, mut index: usize) -> usize {
    while index < block.lines.len() && block.lines[index].text.trim().is_empty() {
        index += 1;
    }
    index
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

fn collect_block_description_lines(
    block: &crate::model::CommentBlock,
    mut cursor: usize,
) -> (Vec<String>, usize) {
    let mut description_lines = Vec::new();
    while cursor < block.lines.len() {
        let text = block.lines[cursor].text.trim();
        if crate::annotation_syntax::is_any_tag_boundary(text) {
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
