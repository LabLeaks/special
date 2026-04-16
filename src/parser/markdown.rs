/**
@module SPECIAL.PARSER.MARKDOWN
Parses declarative markdown annotations from ordinary markdown files under the project root, including `@group`, `@spec`, spec lifecycle markers, and `@fileattests`. This module does not attach item-scoped verifies or attests to code items.
*/
// @fileimplements SPECIAL.PARSER.MARKDOWN
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{Diagnostic, DiagnosticSeverity, ParsedRepo, SourceLocation};

use super::declarations::{SpecLifecycleMarkers, build_spec_decl};
use super::{ParseRules, starts_markdown_fence};

mod attests;
mod declarations;

use attests::{handle_markdown_file_attest, parse_markdown_file_attest};
use declarations::{
    collect_markdown_description_lines, maybe_consume_markdown_deprecated,
    maybe_consume_markdown_planned, parse_markdown_decl_header, parse_markdown_spec_decl,
    skip_blank_markdown_lines,
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
            let (mut planned, mut planned_release, next_cursor) =
                maybe_consume_markdown_planned(kind, &lines, cursor, parsed, &path, rules.planned);
            cursor = next_cursor;
            let (mut deprecated, mut deprecated_release, next_cursor) =
                maybe_consume_markdown_deprecated(
                    kind,
                    &lines,
                    cursor,
                    parsed,
                    &path,
                    rules.planned,
                );
            cursor = next_cursor;
            if header.planned && planned {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    line: line_number,
                    message: "@planned must appear only once per owning @spec".to_string(),
                });
                planned = false;
                planned_release = None;
            }
            if header.deprecated && deprecated {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    line: line_number,
                    message: "@deprecated must appear only once per owning @spec".to_string(),
                });
                deprecated = false;
                deprecated_release = None;
            }
            if header.planned && deprecated {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    line: line_number,
                    message: "@spec may not be both planned and deprecated".to_string(),
                });
                deprecated = false;
                deprecated_release = None;
            }
            if header.deprecated && planned {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    line: line_number,
                    message: "@spec may not be both planned and deprecated".to_string(),
                });
                planned = false;
                planned_release = None;
            }
            if (header.planned || planned) && (header.deprecated || deprecated) {
                parsed.diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    path: path.clone(),
                    line: line_number,
                    message: "@spec may not be both planned and deprecated".to_string(),
                });
                if deprecated {
                    deprecated = false;
                    deprecated_release = None;
                } else {
                    planned = false;
                    planned_release = None;
                }
            }
            let (description_lines, cursor) =
                collect_markdown_description_lines(&lines, cursor, starts_markdown_fence);

            match build_spec_decl(
                header,
                kind,
                description_lines.join(" "),
                SpecLifecycleMarkers {
                    planned,
                    planned_release,
                    deprecated,
                    deprecated_release,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::parser::ParseDialect;

    static MARKDOWN_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_markdown_root(name: &str) -> PathBuf {
        let suffix = MARKDOWN_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("special-markdown-{name}-{nanos}-{suffix}"));
        fs::create_dir_all(&root).expect("temp markdown root should be created");
        root
    }

    fn parse_markdown_fixture(markdown: &str) -> ParsedRepo {
        let root = temp_markdown_root("fixture");
        fs::write(root.join("specs.md"), markdown).expect("markdown fixture should be written");
        let mut parsed = ParsedRepo::default();
        parse_markdown_declarations(
            &root,
            &[],
            &mut parsed,
            ParseRules::for_dialect(ParseDialect::CurrentV1),
        )
        .expect("markdown declarations should parse");
        fs::remove_dir_all(&root).expect("temp markdown root should be removed");
        parsed
    }

    #[test]
    fn markdown_rejects_invalid_trailing_deprecated_suffix_with_neutral_message() {
        let parsed = parse_markdown_fixture("### `@spec APP.BAD @deprecatedx`\nBroken.\n");

        assert!(parsed.specs.is_empty());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("`@planned` or `@deprecated`")
        );
    }

    #[test]
    fn markdown_rejects_duplicate_inline_and_adjacent_planned_markers() {
        let parsed = parse_markdown_fixture(
            "### `@spec APP.BAD @planned 0.4.0`\n### `@planned 0.5.0`\nPlanned.\n",
        );

        assert_eq!(parsed.specs.len(), 1);
        assert!(parsed.specs[0].is_planned());
        assert_eq!(parsed.specs[0].planned_release(), Some("0.4.0"));
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@planned must appear only once")
        );
    }

    #[test]
    fn markdown_rejects_duplicate_inline_and_adjacent_deprecated_markers() {
        let parsed = parse_markdown_fixture(
            "### `@spec APP.BAD @deprecated 0.6.0`\n### `@deprecated 0.7.0`\nDeprecated.\n",
        );

        assert_eq!(parsed.specs.len(), 1);
        assert!(parsed.specs[0].is_deprecated());
        assert_eq!(parsed.specs[0].deprecated_release(), Some("0.6.0"));
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@deprecated must appear only once")
        );
    }

    #[test]
    fn markdown_rejects_adjacent_planned_and_deprecated_combination() {
        let parsed = parse_markdown_fixture(
            "### `@spec APP.BAD`\n### `@planned 0.4.0`\n### `@deprecated 0.6.0`\nConflicting.\n",
        );

        assert_eq!(parsed.specs.len(), 1);
        assert!(parsed.specs[0].is_planned());
        assert!(!parsed.specs[0].is_deprecated());
        assert_eq!(parsed.specs[0].planned_release(), Some("0.4.0"));
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@spec may not be both planned and deprecated")
        );
    }
}
