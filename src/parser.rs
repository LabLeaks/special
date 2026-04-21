/**
@spec SPECIAL.PARSE
special parses annotated comment blocks into structured spec records.

@group SPECIAL.PARSE.RESERVED_TAGS
reserved special annotation shape and validation.

@spec SPECIAL.PARSE.RESERVED_TAGS.REQUIRE_DIRECTIVE_SHAPE
special reports malformed reserved annotations when a reserved tag appears at line start but omits the required directive shape, instead of silently treating it as foreign syntax.

@spec SPECIAL.PARSE.FOREIGN_TAG_BOUNDARIES
special treats foreign line-start `@...` and `\\...` tags as block boundaries for attached annotation text without treating them as special annotations.

@spec SPECIAL.PARSE.PLANNED
special records @planned on the owning @spec according to the configured `special.toml` version.

@spec SPECIAL.PARSE.PLANNED.LEGACY_V0
without `version = "1"` in `special.toml`, special preserves the legacy backward-looking `@planned` association within an annotation block.

@spec SPECIAL.PARSE.PLANNED.ADJACENT_V1
with `version = "1"` in `special.toml`, special requires `@planned` to be adjacent to its owning `@spec`.

@spec SPECIAL.PARSE.PLANNED.ADJACENT_V1.INLINE
with `version = "1"` in `special.toml`, special accepts `@spec ID @planned` on one line.

@spec SPECIAL.PARSE.PLANNED.ADJACENT_V1.NEXT_LINE
with `version = "1"` in `special.toml`, special accepts `@planned` on the line immediately after `@spec` and before the claim text.

@spec SPECIAL.PARSE.PLANNED.ADJACENT_V1.EXACT_INLINE_MARKER
with `version = "1"` in `special.toml`, special only accepts an exact trailing `@planned` marker in `@spec` headers.

@spec SPECIAL.PARSE.PLANNED.ADJACENT_V1.EXACT_STANDALONE_MARKER
with `version = "1"` in `special.toml`, special only accepts an exact standalone `@planned` marker on the adjacent next line.

@spec SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_DUPLICATE_MARKERS
with `version = "1"` in `special.toml`, special rejects duplicate inline and adjacent `@planned` markers on the same `@spec`.

@spec SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_BACKWARD_FORM
with `version = "1"` in `special.toml`, special rejects non-adjacent backward-looking `@planned` markers later in the annotation block.

@spec SPECIAL.PARSE.PLANNED.RELEASE_TARGET
special parses an optional release string after `@planned` and records it on the owning spec as planned release metadata.

@spec SPECIAL.PARSE.DEPRECATED
special records @deprecated on the owning @spec according to the configured `special.toml` version.

@spec SPECIAL.PARSE.DEPRECATED.RELEASE_TARGET
special parses an optional release string after `@deprecated` and records it on the owning spec as deprecated release metadata.

@spec SPECIAL.PARSE.VERIFIES
special parses @verifies references from annotation blocks.

@spec SPECIAL.PARSE.VERIFIES.ONE_PER_BLOCK
special allows at most one @verifies or @fileverifies per annotation block.

@spec SPECIAL.PARSE.VERIFIES.FILE_SCOPE
special parses @fileverifies references as file-scoped verification attachments over the containing file.

@spec SPECIAL.PARSE.VERIFIES.ONLY_ATTACHED_SUPPORT_COUNTS
special counts a @verifies reference as support only when it successfully attaches to an owned item.

@spec SPECIAL.PARSE.ATTESTS
special parses @attests records from annotation blocks.

@spec SPECIAL.PARSE.ATTESTS.FILE_SCOPE
special parses @fileattests records as file-scoped attestation attachments over the containing file in source comments and markdown annotation files.

@spec SPECIAL.PARSE.ATTESTS.REQUIRED_FIELDS
special requires the mandatory metadata fields for @attests.

@spec SPECIAL.PARSE.ATTESTS.ALLOWED_FIELDS
special rejects unknown metadata keys on @attests records.

@spec SPECIAL.PARSE.ATTESTS.DATE_FORMAT
special requires last_reviewed to use YYYY-MM-DD format.

@spec SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
special requires review_interval_days to be a positive integer when present.

@spec SPECIAL.PARSE.ARCH_ANNOTATIONS_RESERVED
special reserves `@module`, `@area`, `@implements`, and `@fileimplements` for architecture metadata and does not report them as unknown spec annotations.

@module SPECIAL.PARSER
Interprets reserved spec annotations from extracted comment blocks, applies dialect-specific ownership rules, and emits parsed specs, verifies, attests, and diagnostics. This module does not own filesystem comment extraction or final tree materialization.
*/
// @fileimplements SPECIAL.PARSER
mod attestation;
mod block;
mod declarations;
mod markdown;
mod planned;

use std::path::Path;

use anyhow::Result;

use crate::extractor::collect_comment_blocks;
use crate::model::{CommentBlock, Diagnostic, DiagnosticSeverity, ParsedRepo};
use block::{ParseRules, parse_block};
use markdown::parse_markdown_declarations;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDialect {
    CompatibilityV0,
    CurrentV1,
}

// @implements SPECIAL.PARSER
pub fn parse_repo(
    root: &Path,
    ignore_patterns: &[String],
    dialect: ParseDialect,
) -> Result<ParsedRepo> {
    let rules = ParseRules::for_dialect(dialect);
    let mut parsed = ParsedRepo::default();
    for block in collect_comment_blocks(root, ignore_patterns)? {
        parse_block(&block, &mut parsed, rules);
    }
    parse_markdown_declarations(root, ignore_patterns, &mut parsed, rules)?;
    Ok(parsed)
}

pub(super) fn normalize_markdown_annotation_line(line: &str) -> Option<&str> {
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
    let trimmed = trimmed.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

pub(super) fn starts_markdown_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

fn push_diag(parsed: &mut ParsedRepo, block: &CommentBlock, line: usize, message: &str) {
    parsed.diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Error,
        path: block.path.clone(),
        line,
        message: message.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::config::SpecialVersion;
    use crate::model::{
        AttestScope, BlockLine, CommentBlock, NodeKind, OwnedItem, ParsedRepo, SourceLocation,
    };

    use super::{ParseDialect, ParseRules, parse_block};

    fn parse_with_version(block: &CommentBlock, version: SpecialVersion) -> ParsedRepo {
        let mut parsed = ParsedRepo::default();
        let dialect = match version {
            SpecialVersion::V0 => ParseDialect::CompatibilityV0,
            SpecialVersion::V1 => ParseDialect::CurrentV1,
        };
        parse_block(block, &mut parsed, ParseRules::for_dialect(dialect));
        parsed
    }

    fn parse_current(block: &CommentBlock) -> ParsedRepo {
        parse_with_version(block, SpecialVersion::V1)
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED
    fn records_planned_on_the_owning_spec_for_each_supported_version() {
        let legacy = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.LEGACY".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Legacy planned behavior.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@planned".to_string(),
                },
            ],
            owned_item: None,
        };
        let current = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 10,
                    text: "@spec EXPORT.CURRENT".to_string(),
                },
                BlockLine {
                    line: 11,
                    text: "@planned".to_string(),
                },
                BlockLine {
                    line: 12,
                    text: "Current planned behavior.".to_string(),
                },
            ],
            owned_item: None,
        };

        let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
        let current_parsed = parse_current(&current);

        assert!(legacy_parsed.specs[0].is_planned());
        assert!(current_parsed.specs[0].is_planned());
        assert_eq!(legacy_parsed.specs[0].planned_release(), None);
        assert_eq!(current_parsed.specs[0].planned_release(), None);
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1
    fn version_1_requires_local_planned_ownership() {
        let accepted = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.CURRENT".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@planned".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "Current planned behavior.".to_string(),
                },
            ],
            owned_item: None,
        };
        let rejected = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 10,
                    text: "@spec EXPORT.OLD".to_string(),
                },
                BlockLine {
                    line: 11,
                    text: "Old planned behavior.".to_string(),
                },
                BlockLine {
                    line: 12,
                    text: "@planned".to_string(),
                },
            ],
            owned_item: None,
        };

        let accepted_parsed = parse_current(&accepted);
        let rejected_parsed = parse_current(&rejected);

        assert!(accepted_parsed.specs[0].is_planned());
        assert!(accepted_parsed.diagnostics.is_empty());
        assert!(!rejected_parsed.specs[0].is_planned());
        assert_eq!(rejected_parsed.diagnostics.len(), 1);
    }

    #[test]
    // @verifies SPECIAL.PARSE
    fn parses_mixed_annotation_kinds() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@planned".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "Exports data.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].id, "EXPORT");
        assert_eq!(parsed.specs[0].kind(), NodeKind::Spec);
        assert!(parsed.specs[0].is_planned());
        assert_eq!(parsed.specs[0].text, "Exports data.");
        assert!(parsed.verifies.is_empty());
        assert!(parsed.attests.is_empty());
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.MIXED_PURPOSE_COMMENTS
    fn parses_reserved_annotations_from_mixed_purpose_comment_blocks() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "Human overview for maintainers.".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@spec EXPORT.CSV".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "CSV exports include a header row.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].id, "EXPORT.CSV");
        assert_eq!(parsed.specs[0].text, "CSV exports include a header row.");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.INLINE
    fn parses_inline_planned_in_version_1() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.METADATA @planned".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Exports include provenance metadata.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].kind(), NodeKind::Spec);
        assert!(parsed.specs[0].is_planned());
        assert_eq!(parsed.specs[0].planned_release(), None);
        assert_eq!(parsed.specs[0].text, "Exports include provenance metadata.");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.EXACT_INLINE_MARKER
    fn rejects_fuzzy_inline_planned_markers_in_version_1() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "@spec EXPORT.METADATA @plannedness".to_string(),
            }],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert!(parsed.specs.is_empty());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].path, PathBuf::from("src/example.rs"));
        assert_eq!(parsed.diagnostics[0].line, 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("unexpected trailing content after spec id")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.EXACT_STANDALONE_MARKER
    fn rejects_fuzzy_standalone_planned_markers_in_version_1() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.METADATA".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@plannedness".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "Exports include provenance metadata.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert!(!parsed.specs[0].is_planned());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("exact standalone `@planned` marker")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.NEXT_LINE
    fn parses_adjacent_next_line_planned_in_version_1() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.METADATA".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@planned".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "Exports include provenance metadata.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert!(parsed.specs[0].is_planned());
        assert_eq!(parsed.specs[0].planned_release(), None);
        assert_eq!(parsed.specs[0].text, "Exports include provenance metadata.");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_DUPLICATE_MARKERS
    fn rejects_duplicate_inline_and_adjacent_planned_markers_in_version_1() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.METADATA @planned 0.4.0".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@planned 0.5.0".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "Exports include provenance metadata.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

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
    // @verifies SPECIAL.PARSE.PLANNED.LEGACY_V0
    fn preserves_legacy_planned_association_without_version() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.METADATA".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Exports include provenance metadata.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@planned".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_with_version(&block, SpecialVersion::V0);

        assert_eq!(parsed.specs.len(), 1);
        assert!(parsed.specs[0].is_planned());
        assert_eq!(parsed.specs[0].planned_release(), None);
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn rejects_inline_planned_syntax_in_compatibility_mode() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "@spec EXPORT.METADATA @planned".to_string(),
            }],
            owned_item: None,
        };

        let parsed = parse_with_version(&block, SpecialVersion::V0);

        assert!(parsed.specs.is_empty());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("compatibility parsing does not allow inline lifecycle markers")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.RELEASE_TARGET
    fn parses_planned_release_metadata_in_supported_forms() {
        let legacy = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.LEGACY".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Legacy planned behavior.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@planned 0.3.0".to_string(),
                },
            ],
            owned_item: None,
        };
        let inline_v1 = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 10,
                    text: "@spec EXPORT.INLINE @planned 0.4.0".to_string(),
                },
                BlockLine {
                    line: 11,
                    text: "Inline planned behavior.".to_string(),
                },
            ],
            owned_item: None,
        };
        let next_line_v1 = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 20,
                    text: "@spec EXPORT.NEXT".to_string(),
                },
                BlockLine {
                    line: 21,
                    text: "@planned 0.5.0".to_string(),
                },
                BlockLine {
                    line: 22,
                    text: "Next-line planned behavior.".to_string(),
                },
            ],
            owned_item: None,
        };

        let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
        let inline_parsed = parse_current(&inline_v1);
        let next_line_parsed = parse_current(&next_line_v1);

        assert_eq!(legacy_parsed.specs[0].planned_release(), Some("0.3.0"));
        assert_eq!(inline_parsed.specs[0].planned_release(), Some("0.4.0"));
        assert_eq!(next_line_parsed.specs[0].planned_release(), Some("0.5.0"));
    }

    #[test]
    // @verifies SPECIAL.PARSE.DEPRECATED
    fn records_deprecated_on_the_owning_spec_for_each_supported_version() {
        let legacy = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.LEGACY".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Legacy deprecated behavior.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@deprecated".to_string(),
                },
            ],
            owned_item: None,
        };
        let current = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 10,
                    text: "@spec EXPORT.CURRENT".to_string(),
                },
                BlockLine {
                    line: 11,
                    text: "@deprecated".to_string(),
                },
                BlockLine {
                    line: 12,
                    text: "Current deprecated behavior.".to_string(),
                },
            ],
            owned_item: None,
        };

        let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
        let current_parsed = parse_current(&current);

        assert!(legacy_parsed.specs[0].is_deprecated());
        assert!(current_parsed.specs[0].is_deprecated());
        assert_eq!(legacy_parsed.specs[0].deprecated_release(), None);
        assert_eq!(current_parsed.specs[0].deprecated_release(), None);
    }

    #[test]
    // @verifies SPECIAL.PARSE.DEPRECATED.RELEASE_TARGET
    fn parses_deprecated_release_metadata_in_supported_forms() {
        let legacy = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.LEGACY".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Legacy deprecated behavior.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@deprecated 0.6.0".to_string(),
                },
            ],
            owned_item: None,
        };
        let inline_v1 = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 10,
                    text: "@spec EXPORT.INLINE @deprecated 0.7.0".to_string(),
                },
                BlockLine {
                    line: 11,
                    text: "Inline deprecated behavior.".to_string(),
                },
            ],
            owned_item: None,
        };
        let next_line_v1 = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 20,
                    text: "@spec EXPORT.NEXT".to_string(),
                },
                BlockLine {
                    line: 21,
                    text: "@deprecated 0.8.0".to_string(),
                },
                BlockLine {
                    line: 22,
                    text: "Next-line deprecated behavior.".to_string(),
                },
            ],
            owned_item: None,
        };

        let legacy_parsed = parse_with_version(&legacy, SpecialVersion::V0);
        let inline_parsed = parse_current(&inline_v1);
        let next_line_parsed = parse_current(&next_line_v1);

        assert_eq!(legacy_parsed.specs[0].deprecated_release(), Some("0.6.0"));
        assert_eq!(inline_parsed.specs[0].deprecated_release(), Some("0.7.0"));
        assert_eq!(
            next_line_parsed.specs[0].deprecated_release(),
            Some("0.8.0")
        );
    }

    #[test]
    fn keeps_the_spec_when_inline_planned_conflicts_with_adjacent_deprecated() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.CONFLICT @planned 0.4.0".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@deprecated 0.6.0".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "Conflicting lifecycle behavior.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert!(parsed.specs[0].is_planned());
        assert_eq!(parsed.specs[0].planned_release(), Some("0.4.0"));
        assert!(!parsed.specs[0].is_deprecated());
        assert_eq!(parsed.specs[0].deprecated_release(), None);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@spec may not be both planned and deprecated")
        );
    }

    #[test]
    // @verifies SPECIAL.GROUPS
    fn parses_group_declarations() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@group EXPORT".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Export-related claims.".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].kind(), NodeKind::Group);
        assert!(!parsed.specs[0].is_planned());
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.VERIFIES
    fn parses_single_verify_reference() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "@verifies EXPORT.DOESNTCRASH".to_string(),
            }],
            owned_item: Some(OwnedItem {
                location: SourceLocation {
                    path: PathBuf::from("src/example.rs"),
                    line: 2,
                },
                body: "fn verifies_export_doesnt_crash() {}".to_string(),
            }),
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.verifies.len(), 1);
        assert_eq!(parsed.verifies[0].spec_id, "EXPORT.DOESNTCRASH");
        assert_eq!(
            parsed.verifies[0].body.as_deref(),
            Some("fn verifies_export_doesnt_crash() {}")
        );
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.VERIFIES.FILE_SCOPE
    fn parses_file_verify_reference() {
        let root = std::env::temp_dir().join(format!("special-fileverify-{}", std::process::id()));
        fs::create_dir_all(&root).expect("temp dir should be created");
        let path = root.join("example.rs");
        let content = "// @fileverifies EXPORT.DOESNTCRASH\nfn verifies_export_doesnt_crash() {}\n";
        fs::write(&path, content).expect("fixture should be written");

        let block = CommentBlock {
            path: path.clone(),
            lines: vec![BlockLine {
                line: 1,
                text: "@fileverifies EXPORT.DOESNTCRASH".to_string(),
            }],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.verifies.len(), 1);
        assert_eq!(parsed.verifies[0].spec_id, "EXPORT.DOESNTCRASH");
        assert!(parsed.verifies[0].body_location.is_none());
        assert_eq!(parsed.verifies[0].body.as_deref(), Some(content.trim_end()));
        assert!(parsed.diagnostics.is_empty());

        fs::remove_file(path).expect("fixture should be removed");
        let _ = fs::remove_dir(root);
    }

    #[test]
    // @verifies SPECIAL.PARSE.ATTESTS
    fn parses_attestation_blocks() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@attests AUTH".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact: docs/auth.md".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "owner: security".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "last_reviewed: 2026-04-12".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.attests.len(), 1);
        assert_eq!(parsed.attests[0].scope, AttestScope::Block);
        assert!(
            parsed.attests[0]
                .body
                .as_deref()
                .expect("attest body should be present")
                .contains("@attests AUTH")
        );
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.ATTESTS.REQUIRED_FIELDS
    fn requires_attestation_metadata() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@attests AUTH".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact:".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.attests.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 3);
        let messages: Vec<&str> = parsed
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        let lines: Vec<usize> = parsed
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.line)
            .collect();
        assert!(messages
            .iter()
            .any(|message| message.contains("missing required attestation metadata `artifact`")));
        assert!(messages.iter().any(|message| {
            message.contains("missing required attestation metadata `last_reviewed`")
        }));
        assert!(
            messages
                .iter()
                .any(|message| message.contains("missing required attestation metadata `owner`"))
        );
        assert_eq!(lines, vec![2, 1, 1]);
    }

    #[test]
    // @verifies SPECIAL.PARSE.ATTESTS.FILE_SCOPE
    fn parses_file_scoped_attestation_blocks() {
        let path = std::env::temp_dir().join(format!(
            "special-parser-file-attests-{}-{}.rs",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        fs::write(
            &path,
            "// @fileattests AUTH\n// artifact: docs/auth-review.md\n// owner: security\n// last_reviewed: 2026-04-16\nfn helper() {}\n",
        )
        .expect("fixture should be written");
        let block = CommentBlock {
            path: path.clone(),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@fileattests AUTH".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact: docs/auth-review.md".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "owner: security".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "last_reviewed: 2026-04-16".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.attests.len(), 1);
        assert_eq!(parsed.attests[0].scope, AttestScope::File);
        assert!(
            parsed.attests[0]
                .body
                .as_deref()
                .expect("attest body should be present")
                .contains("fn helper() {}")
        );
        assert!(parsed.diagnostics.is_empty());

        fs::remove_file(path).expect("fixture should be removed");
    }

    #[test]
    fn rejects_duplicate_attestation_metadata_keys() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@attests EXPORT.CSV_HEADER".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact: cargo test".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "artifact: cargo test --release".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "owner: qa@example.com".to_string(),
                },
                BlockLine {
                    line: 5,
                    text: "last_reviewed: 2026-04-13".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert!(parsed.attests.is_empty());
        assert!(parsed.diagnostics.iter().any(|diag| {
            diag.message
                .contains("duplicate attestation metadata `artifact`")
        }));
    }

    #[test]
    // @verifies SPECIAL.PARSE.ATTESTS.ALLOWED_FIELDS
    fn rejects_unknown_attestation_metadata_keys() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@attests AUTH".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact: docs/auth.md".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "owner: security".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "last_reviewed: 2026-04-12".to_string(),
                },
                BlockLine {
                    line: 5,
                    text: "reviewed_by: qa".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert!(parsed.attests.is_empty());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].line, 5);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("unknown attestation metadata `reviewed_by`")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.ATTESTS.DATE_FORMAT
    fn requires_attestation_dates_in_iso_format() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@attests AUTH".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact: docs/auth.md".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "owner: security".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "last_reviewed: 04-12-2026".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.attests.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].line, 4);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("last_reviewed must use YYYY-MM-DD format")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
    fn requires_numeric_attestation_review_interval() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@attests AUTH".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact: docs/auth.md".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "owner: security".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "last_reviewed: 2026-04-12".to_string(),
                },
                BlockLine {
                    line: 5,
                    text: "review_interval_days: thirty".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.attests.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].line, 5);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("review_interval_days must be a positive integer")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
    fn requires_positive_attestation_review_interval() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@attests AUTH".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "artifact: docs/auth.md".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "owner: security".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "last_reviewed: 2026-04-12".to_string(),
                },
                BlockLine {
                    line: 5,
                    text: "review_interval_days: 0".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.attests.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(parsed.diagnostics[0].line, 5);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("review_interval_days must be a positive integer")
        );
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.PLANNED_SCOPE
    fn rejects_inline_planned_on_group_nodes() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "@group AUTH @planned".to_string(),
            }],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert!(!parsed.specs[0].is_planned());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@planned may only apply to @spec, not @group")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.VERIFIES.ONE_PER_BLOCK
    fn rejects_multiple_verifies_in_one_block() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@verifies AUTH.ONE".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@verifies AUTH.TWO".to_string(),
                },
            ],
            owned_item: Some(OwnedItem {
                location: SourceLocation {
                    path: PathBuf::from("src/example.rs"),
                    line: 3,
                },
                body: "fn verifies_auth() {}".to_string(),
            }),
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.verifies.len(), 1);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.diagnostics[0].message.contains("only one @verifies"));
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.PLANNED_SCOPE
    fn rejects_planned_outside_spec_declaration() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "@planned".to_string(),
            }],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@planned must be adjacent to exactly one owning @spec")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED.ADJACENT_V1.REJECTS_BACKWARD_FORM
    fn rejects_non_adjacent_planned_in_version_1() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.METADATA".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "Exports include provenance metadata.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@planned".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert!(!parsed.specs[0].is_planned());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("backward-looking form is not allowed in version 1")
        );
    }

    #[test]
    // @verifies SPECIAL.GROUPS.STRUCTURAL_ONLY
    fn rejects_planned_on_group_nodes() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@group EXPORT".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@planned".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@planned may only apply to @spec")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.FOREIGN_TAGS_NOT_ERRORS
    fn ignores_foreign_line_start_tags() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@param file output path".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "\\returns csv text".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert!(parsed.specs.is_empty());
        assert!(parsed.verifies.is_empty());
        assert!(parsed.attests.is_empty());
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.LINE_START_RESERVED_TAGS
    fn ignores_reserved_tag_text_when_it_does_not_begin_the_line() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "Human note mentioning @spec EXPORT.CSV inline.".to_string(),
            }],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert!(parsed.specs.is_empty());
        assert!(parsed.verifies.is_empty());
        assert!(parsed.attests.is_empty());
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.RESERVED_TAGS.REQUIRE_DIRECTIVE_SHAPE
    fn reports_malformed_reserved_tags_instead_of_treating_them_as_foreign() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@verifies".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@attests".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "@fileverifies".to_string(),
                },
                BlockLine {
                    line: 5,
                    text: "@fileattests".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.diagnostics.len(), 5);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("missing spec id after @spec")
        );
        assert!(
            parsed.diagnostics[1]
                .message
                .contains("missing spec id after @verifies")
        );
        assert!(
            parsed.diagnostics[2]
                .message
                .contains("missing spec id after @attests")
        );
        assert!(
            parsed.diagnostics[3]
                .message
                .contains("missing spec id after @fileverifies")
        );
        assert!(
            parsed.diagnostics[4]
                .message
                .contains("missing spec id after @fileattests")
        );
    }

    #[test]
    fn reserved_support_annotations_reject_trailing_tokens() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@verifies APP.ROOT trailing".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@attests APP.ROOT trailing".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@fileverifies APP.ROOT trailing".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "@fileattests APP.ROOT trailing".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.diagnostics.len(), 4);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("unexpected trailing content after @verifies spec id")
        );
        assert!(
            parsed.diagnostics[1]
                .message
                .contains("unexpected trailing content after @attests spec id")
        );
        assert!(
            parsed.diagnostics[2]
                .message
                .contains("unexpected trailing content after @fileverifies spec id")
        );
        assert!(
            parsed.diagnostics[3]
                .message
                .contains("unexpected trailing content after @fileattests spec id")
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.FOREIGN_TAG_BOUNDARIES
    fn foreign_tag_lines_stop_attached_spec_text() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@spec EXPORT.CSV".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "CSV exports include a header row.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@param file output path".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].text, "CSV exports include a header row.");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.ARCH_ANNOTATIONS_RESERVED
    fn ignores_reserved_architecture_annotations() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![
                BlockLine {
                    line: 1,
                    text: "@implements SPECIAL.PARSER".to_string(),
                },
                BlockLine {
                    line: 2,
                    text: "@module SPECIAL.PARSER.PLANNED".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@area SPECIAL.PARSER".to_string(),
                },
                BlockLine {
                    line: 4,
                    text: "@fileimplements SPECIAL.PARSER".to_string(),
                },
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert!(parsed.diagnostics.is_empty());
        assert!(parsed.specs.is_empty());
        assert!(parsed.verifies.is_empty());
        assert!(parsed.attests.is_empty());
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.ORPHAN_VERIFIES
    fn rejects_orphan_verifies_blocks() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "@verifies EXPORT.ORPHAN".to_string(),
            }],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.verifies.len(), 1);
        assert!(parsed.verifies[0].body.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@verifies must attach to the next supported item")
        );
    }
}
