/**
@module SPECIAL.PARSER
Interprets reserved spec annotations from extracted comment blocks, applies dialect-specific ownership rules, and emits parsed specs, verifies, attests, and diagnostics. This module does not own filesystem comment extraction or final tree materialization.
*/
// @implements SPECIAL.PARSER
mod attestation;
mod planned;

use anyhow::Result;

use crate::annotation_syntax::{
    ReservedSpecialAnnotation, is_any_tag_boundary, is_foreign_tag_boundary,
    reserved_special_annotation, reserved_special_annotation_rest,
};
use crate::extractor::collect_comment_blocks;
use crate::model::{
    AttestRef, CommentBlock, Diagnostic, DiagnosticSeverity, NodeKind, ParsedRepo, PlanState,
    PlannedRelease, SourceLocation, SpecDecl, VerifyRef,
};
use crate::planned_syntax::{PlannedAnnotationError, PlannedSyntax};
use attestation::parse_attestation_metadata;
use planned::{
    AdjacentPlanned, DeclHeader, DeclHeaderError, consume_adjacent_planned,
    parse_standalone_planned,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDialect {
    CompatibilityV0,
    CurrentV1,
}

// @implements SPECIAL.PARSER
pub fn parse_repo(root: &std::path::Path, dialect: ParseDialect) -> Result<ParsedRepo> {
    let rules = ParseRules::for_dialect(dialect);
    let mut parsed = ParsedRepo::default();
    for block in collect_comment_blocks(root)? {
        parse_block(&block, &mut parsed, rules);
    }
    Ok(parsed)
}

#[derive(Debug, Clone, Copy)]
struct ParseRules {
    planned: PlannedSyntax,
}

impl ParseRules {
    fn for_dialect(dialect: ParseDialect) -> Self {
        let planned = match dialect {
            ParseDialect::CompatibilityV0 => PlannedSyntax::LegacyBackward,
            ParseDialect::CurrentV1 => PlannedSyntax::AdjacentOwnedSpec,
        };
        Self { planned }
    }

    fn parse_decl_header<'a>(
        &self,
        kind: NodeKind,
        rest: &'a str,
        parsed: &mut ParsedRepo,
        block: &CommentBlock,
        line: usize,
    ) -> Option<DeclHeader<'a>> {
        let rest = rest.trim();
        let mut header = match DeclHeader::parse(rest, self.planned) {
            Ok(header) => header,
            Err(DeclHeaderError::MissingId) => {
                let annotation = if kind == NodeKind::Spec {
                    "@spec"
                } else {
                    "@group"
                };
                push_diag(
                    parsed,
                    block,
                    line,
                    &format!("missing spec id after {annotation}"),
                );
                return None;
            }
            Err(DeclHeaderError::InvalidTrailingContent) => {
                let message = match (kind, self.planned) {
                    (NodeKind::Group, _) => {
                        "unexpected trailing content after group id; only the id belongs on the @group line"
                    }
                    (NodeKind::Spec, PlannedSyntax::LegacyBackward) => {
                        "unexpected trailing content after spec id; compatibility parsing does not allow inline `@planned`"
                    }
                    (NodeKind::Spec, PlannedSyntax::AdjacentOwnedSpec) => {
                        "unexpected trailing content after spec id; use an exact trailing `@planned` marker if needed"
                    }
                };
                push_diag(parsed, block, line, message);
                return None;
            }
            Err(DeclHeaderError::InvalidPlannedRelease) => {
                push_diag(
                    parsed,
                    block,
                    line,
                    "planned release metadata must not be empty",
                );
                return None;
            }
        };

        if header.planned && kind == NodeKind::Group {
            push_diag(
                parsed,
                block,
                line,
                "@planned may only apply to @spec, not @group",
            );
            header.planned = false;
        }

        Some(header)
    }

    fn apply_standalone_planned(
        &self,
        parsed: &mut ParsedRepo,
        block: &CommentBlock,
        state: &BlockState,
        line: usize,
        planned_release: Option<PlannedRelease>,
    ) {
        match self.planned {
            PlannedSyntax::LegacyBackward => match state.last_decl_idx {
                Some(spec_index) if parsed.specs[spec_index].kind() == NodeKind::Spec => {
                    if let Err(err) =
                        parsed.specs[spec_index].set_plan(PlanState::planned(planned_release))
                    {
                        push_diag(parsed, block, line, &err.to_string());
                    }
                }
                None => push_diag(
                    parsed,
                    block,
                    line,
                    "@planned must appear after a @spec in the same block",
                ),
                Some(_) => push_diag(
                    parsed,
                    block,
                    line,
                    "@planned may only apply to @spec, not @group",
                ),
            },
            PlannedSyntax::AdjacentOwnedSpec => {
                if state.last_decl_line == Some(line.saturating_sub(1))
                    && state.last_decl_kind == Some(NodeKind::Spec)
                    && let Some(spec_index) = state.last_decl_idx
                {
                    if let Err(err) =
                        parsed.specs[spec_index].set_plan(PlanState::planned(planned_release))
                    {
                        push_diag(parsed, block, line, &err.to_string());
                    }
                    return;
                }

                if state.last_decl_line == Some(line.saturating_sub(1))
                    && state.last_decl_kind == Some(NodeKind::Group)
                {
                    push_diag(
                        parsed,
                        block,
                        line,
                        "@planned may only apply to @spec, not @group",
                    );
                } else if state.last_decl_idx.is_some() {
                    push_diag(
                        parsed,
                        block,
                        line,
                        "@planned must be adjacent to exactly one owning @spec; the backward-looking form is not allowed in version 1",
                    );
                } else {
                    push_diag(
                        parsed,
                        block,
                        line,
                        "@planned must be adjacent to exactly one owning @spec",
                    );
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct BlockState {
    last_decl_idx: Option<usize>,
    last_decl_kind: Option<NodeKind>,
    last_decl_line: Option<usize>,
}

impl BlockState {
    fn remember_decl(&mut self, spec_index: usize, kind: NodeKind, line: usize) {
        self.last_decl_idx = Some(spec_index);
        self.last_decl_kind = Some(kind);
        self.last_decl_line = Some(line);
    }
}

fn parse_block(block: &CommentBlock, parsed: &mut ParsedRepo, rules: ParseRules) {
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

        if let Some((kind, rest)) =
            reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Spec)
                .map(|rest| (NodeKind::Spec, rest))
                .or_else(|| {
                    reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Group)
                        .map(|rest| (NodeKind::Group, rest))
                })
        {
            let Some(header) = rules.parse_decl_header(kind, rest, parsed, block, entry.line)
            else {
                index += 1;
                continue;
            };

            let (adjacent_planned, adjacent_planned_release, mut cursor) =
                match consume_adjacent_planned(block, kind, index, rules.planned) {
                    AdjacentPlanned::Absent(cursor) => (false, None, cursor),
                    AdjacentPlanned::Parsed(release, cursor) => (true, release, cursor),
                    AdjacentPlanned::Invalid(message, cursor) => {
                        push_diag(parsed, block, block.lines[index + 1].line, message);
                        (false, None, cursor)
                    }
                };

            if header.planned && adjacent_planned {
                push_diag(
                    parsed,
                    block,
                    block.lines[index + 1].line,
                    "@planned must appear only once per owning @spec",
                );
            }

            let mut description_lines = Vec::new();
            while cursor < block.lines.len() {
                let text = block.lines[cursor].text.trim();
                if is_any_tag_boundary(text) {
                    break;
                }
                if !text.is_empty() {
                    description_lines.push(text.to_string());
                }
                cursor += 1;
            }

            let planned_release = if header.planned {
                header.planned_release
            } else {
                adjacent_planned_release
            };
            let plan = if header.planned || adjacent_planned {
                PlanState::planned(planned_release)
            } else {
                PlanState::live()
            };
            let spec = match SpecDecl::new(
                header.id.to_string(),
                kind,
                description_lines.join(" "),
                plan,
                SourceLocation {
                    path: block.path.clone(),
                    line: entry.line,
                },
            ) {
                Ok(spec) => spec,
                Err(err) => {
                    push_diag(parsed, block, entry.line, &err.to_string());
                    index = cursor;
                    continue;
                }
            };
            parsed.specs.push(spec);
            state.remember_decl(parsed.specs.len() - 1, kind, entry.line);
            index = cursor;
            continue;
        }

        if let Some(planned_release) = parse_standalone_planned(trimmed) {
            match planned_release {
                Ok(planned_release) => {
                    rules.apply_standalone_planned(
                        parsed,
                        block,
                        &state,
                        entry.line,
                        planned_release,
                    );
                }
                Err(PlannedAnnotationError::InvalidRelease) => push_diag(
                    parsed,
                    block,
                    entry.line,
                    "planned release metadata must not be empty",
                ),
                Err(PlannedAnnotationError::InvalidSuffix) => push_diag(
                    parsed,
                    block,
                    entry.line,
                    "use an exact standalone `@planned` marker with no trailing suffix",
                ),
            }
            index += 1;
            continue;
        }

        if let Some(rest) =
            reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Verifies)
        {
            if seen_verifies {
                push_diag(
                    parsed,
                    block,
                    entry.line,
                    "annotation block may contain only one @verifies",
                );
                index += 1;
                continue;
            }

            let id = rest.trim();
            if id.is_empty() {
                push_diag(parsed, block, entry.line, "missing spec id after @verifies");
            } else if let Some(owned_item) = &block.owned_item {
                parsed.verifies.push(VerifyRef {
                    spec_id: id.to_string(),
                    location: SourceLocation {
                        path: block.path.clone(),
                        line: entry.line,
                    },
                    body_location: Some(owned_item.location.clone()),
                    body: Some(owned_item.body.clone()),
                });
                seen_verifies = true;
            } else {
                parsed.verifies.push(VerifyRef {
                    spec_id: id.to_string(),
                    location: SourceLocation {
                        path: block.path.clone(),
                        line: entry.line,
                    },
                    body_location: None,
                    body: None,
                });
                push_diag(
                    parsed,
                    block,
                    entry.line,
                    "@verifies must attach to the next supported item",
                );
                seen_verifies = true;
            }
            index += 1;
            continue;
        }

        if let Some(rest) =
            reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Attests)
        {
            let id = rest.trim();
            if id.is_empty() {
                push_diag(parsed, block, entry.line, "missing spec id after @attests");
                index += 1;
                continue;
            }

            let (attestation, cursor) =
                parse_attestation_metadata(parsed, block, entry.line, index + 1);
            if let Some(attestation) = attestation {
                parsed.attests.push(AttestRef {
                    spec_id: id.to_string(),
                    artifact: attestation.artifact,
                    owner: attestation.owner,
                    last_reviewed: attestation.last_reviewed,
                    review_interval_days: attestation.review_interval_days,
                    location: SourceLocation {
                        path: block.path.clone(),
                        line: entry.line,
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

            index = cursor;
            continue;
        }

        if is_reserved_arch_annotation(trimmed) {
            index += 1;
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
            continue;
        }

        if is_foreign_tag_boundary(trimmed) {
            index += 1;
            continue;
        }
        index += 1;
    }
}

fn is_reserved_arch_annotation(text: &str) -> bool {
    matches!(
        reserved_special_annotation(text),
        Some(ReservedSpecialAnnotation::Implements)
            | Some(ReservedSpecialAnnotation::Module)
            | Some(ReservedSpecialAnnotation::Area)
    )
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
    use std::path::PathBuf;

    use crate::config::SpecialVersion;
    use crate::model::{BlockLine, CommentBlock, NodeKind, OwnedItem, ParsedRepo, SourceLocation};

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
                .contains("compatibility parsing does not allow inline `@planned`")
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
            ],
            owned_item: None,
        };

        let parsed = parse_current(&block);

        assert_eq!(parsed.diagnostics.len(), 3);
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
