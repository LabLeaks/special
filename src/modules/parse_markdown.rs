/**
@module SPECIAL.MODULES.PARSE.MARKDOWN
Parses declarative `@area` and `@module` annotations from ordinary markdown files under the project root. This module does not parse `@implements` attachments from code comments.
*/
// @fileimplements SPECIAL.MODULES.PARSE.MARKDOWN
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    ArchitectureKind, Diagnostic, DiagnosticSeverity, ModuleDecl, ParsedArchitecture,
    PatternDefinition, PlanState, SourceLocation,
};

use super::parse::declarations::{
    collect_doc_description_lines, maybe_consume_doc_planned, maybe_consume_pattern_strictness,
    normalized_annotation_line, normalized_architecture_heading, normalized_pattern_heading,
    parse_module_header, parse_pattern_id, skip_blank_doc_lines,
};
use crate::parser::starts_markdown_fence;

pub(super) fn parse_markdown_architecture_decls(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &mut ParsedArchitecture,
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
            let raw = lines[index];
            if starts_markdown_fence(raw) {
                in_code_fence = !in_code_fence;
                index += 1;
                continue;
            }
            if in_code_fence {
                index += 1;
                continue;
            }

            if let Some(raw_pattern) = normalized_pattern_heading(raw) {
                let line_number = index + 1;
                let Some(pattern_id) =
                    parse_pattern_id(raw_pattern, "@pattern", parsed, &path, line_number)
                else {
                    index += 1;
                    continue;
                };
                let mut cursor = skip_blank_doc_lines(&lines, index + 1);
                let strictness = if let Some(strictness) = maybe_consume_pattern_strictness(
                    normalized_annotation_line(lines.get(cursor).copied()),
                    parsed,
                    &path,
                    cursor + 1,
                ) {
                    cursor = skip_blank_doc_lines(&lines, cursor + 1);
                    strictness
                } else {
                    Default::default()
                };
                let (description_lines, cursor) = collect_doc_description_lines(&lines, cursor);
                parsed.patterns.push(PatternDefinition {
                    pattern_id,
                    strictness,
                    text: description_lines.join(" "),
                    location: SourceLocation {
                        path: path.to_path_buf(),
                        line: line_number,
                    },
                });
                index = cursor;
                continue;
            }

            let Some((kind, raw_decl)) = normalized_architecture_heading(raw) else {
                if let Some(annotation) = normalized_annotation_line(Some(raw))
                    && (reserved_special_annotation_rest(
                        annotation,
                        ReservedSpecialAnnotation::Applies,
                    )
                    .is_some()
                        || reserved_special_annotation_rest(
                            annotation,
                            ReservedSpecialAnnotation::FileApplies,
                        )
                        .is_some())
                {
                    parsed.diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        path: path.to_path_buf(),
                        line: index + 1,
                        message: "@applies and @fileapplies must attach to source code, not markdown declarations".to_string(),
                    });
                }
                index += 1;
                continue;
            };

            let line_number = index + 1;
            let Some((id, inline_release, inline_planned)) =
                parse_module_header(kind, raw_decl, parsed, &path, line_number)
            else {
                index += 1;
                continue;
            };

            let mut cursor = skip_blank_doc_lines(&lines, index + 1);
            let (planned, planned_release, next_cursor) = maybe_consume_doc_planned(
                kind,
                &lines,
                cursor,
                parsed,
                &path,
                inline_planned,
                inline_release,
            );
            cursor = next_cursor;
            let (description_lines, cursor) = collect_doc_description_lines(&lines, cursor);
            let location = SourceLocation {
                path: path.to_path_buf(),
                line: line_number,
            };
            let plan = if planned {
                PlanState::planned(planned_release)
            } else {
                PlanState::current()
            };
            push_module_decl(
                parsed,
                kind,
                id,
                description_lines.join(" "),
                plan,
                location,
            );
            index = cursor;
        }
    }

    Ok(())
}

fn push_module_decl(
    parsed: &mut ParsedArchitecture,
    kind: ArchitectureKind,
    id: String,
    text: String,
    plan: PlanState,
    location: SourceLocation,
) {
    let module = match ModuleDecl::new(id, kind, text, plan, location.clone()) {
        Ok(module) => module,
        Err(err) => {
            parsed.diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                path: location.path,
                line: location.line,
                message: err.to_string(),
            });
            return;
        }
    };
    parsed.modules.push(module);
}
