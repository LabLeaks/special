/**
@module SPECIAL.PARSER.MARKDOWN
Parses declarative markdown annotations from ordinary markdown files under the project root, including `@group`, `@spec`, `@planned`, and `@fileattests`. This module does not attach item-scoped verifies or attests to code items.
*/
// @fileimplements SPECIAL.PARSER.MARKDOWN
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{Diagnostic, DiagnosticSeverity, ParsedRepo, SourceLocation};

use super::declarations::build_spec_decl;
use super::{ParseRules, starts_markdown_fence};

mod attests;
mod declarations;

use attests::{handle_markdown_file_attest, parse_markdown_file_attest};
use declarations::{
    collect_markdown_description_lines, maybe_consume_markdown_planned, parse_markdown_decl_header,
    parse_markdown_spec_decl, skip_blank_markdown_lines,
};

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

            if let Some(rest) = parse_markdown_file_attest(raw_line) {
                index = handle_markdown_file_attest(&content, &path, &lines, parsed, index, rest);
                continue;
            }

            let Some((kind, rest)) = parse_markdown_spec_decl(raw_line) else {
                index += 1;
                continue;
            };
            let line_number = index + 1;
            let (header, header_diag) = match parse_markdown_decl_header(kind, rest, rules.planned)
            {
                Ok(parsed) => parsed,
                Err(message) => {
                    parsed.diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        path: path.clone(),
                        line: line_number,
                        message,
                    });
                    index += 1;
                    continue;
                }
            };
            if let Some(message) = header_diag {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    line: line_number,
                    message,
                });
            }

            let mut cursor = skip_blank_markdown_lines(&lines, index + 1);
            let (planned, planned_release, next_cursor) =
                maybe_consume_markdown_planned(kind, &lines, cursor, parsed, &path, rules.planned);
            cursor = next_cursor;
            let (description_lines, cursor) =
                collect_markdown_description_lines(&lines, cursor, starts_markdown_fence);

            match build_spec_decl(
                header,
                kind,
                description_lines.join(" "),
                planned,
                planned_release,
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
