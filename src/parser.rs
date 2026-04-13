use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDate;

use crate::extractor::collect_comment_blocks;
use crate::model::{
    AttestRef, CommentBlock, Diagnostic, NodeKind, ParsedRepo, SourceLocation, SpecDecl, VerifyRef,
};

pub fn parse_repo(root: &std::path::Path) -> Result<ParsedRepo> {
    let mut parsed = ParsedRepo::default();
    for block in collect_comment_blocks(root)? {
        parse_block(&block, &mut parsed);
    }
    Ok(parsed)
}

fn parse_block(block: &CommentBlock, parsed: &mut ParsedRepo) {
    let mut index = 0;
    let mut last_decl_idx: Option<usize> = None;
    let mut seen_verifies = false;

    while index < block.lines.len() {
        let entry = &block.lines[index];
        let trimmed = entry.text.trim();

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some((kind, rest)) = trimmed
            .strip_prefix("@spec ")
            .map(|rest| (NodeKind::Spec, rest))
            .or_else(|| {
                trimmed
                    .strip_prefix("@group ")
                    .map(|rest| (NodeKind::Group, rest))
            })
        {
            let id = rest.trim();
            if id.is_empty() {
                let annotation = if kind == NodeKind::Spec {
                    "@spec"
                } else {
                    "@group"
                };
                push_diag(
                    parsed,
                    block,
                    entry.line,
                    &format!("missing spec id after {annotation}"),
                );
                index += 1;
                continue;
            }

            let mut description_lines = Vec::new();
            let mut cursor = index + 1;
            while cursor < block.lines.len() {
                let text = block.lines[cursor].text.trim();
                if text.starts_with('@') {
                    break;
                }
                if !text.is_empty() {
                    description_lines.push(text.to_string());
                }
                cursor += 1;
            }

            let spec = SpecDecl {
                id: id.to_string(),
                kind,
                text: description_lines.join(" "),
                planned: false,
                location: SourceLocation {
                    path: block.path.clone(),
                    line: entry.line,
                },
            };
            parsed.specs.push(spec);
            last_decl_idx = Some(parsed.specs.len() - 1);
            index = cursor;
            continue;
        }

        if trimmed == "@planned" {
            match last_decl_idx {
                Some(spec_index) if parsed.specs[spec_index].kind == NodeKind::Spec => {
                    parsed.specs[spec_index].planned = true
                }
                None => push_diag(
                    parsed,
                    block,
                    entry.line,
                    "@planned must appear after a @spec in the same block",
                ),
                Some(_) => push_diag(
                    parsed,
                    block,
                    entry.line,
                    "@planned may only apply to @spec, not @group",
                ),
            }
            index += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@verifies ") {
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
            }
            index += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("@attests ") {
            let id = rest.trim();
            if id.is_empty() {
                push_diag(parsed, block, entry.line, "missing spec id after @attests");
                index += 1;
                continue;
            }

            let mut metadata = HashMap::new();
            let mut cursor = index + 1;
            while cursor < block.lines.len() {
                let text = block.lines[cursor].text.trim();
                if text.starts_with('@') {
                    break;
                }
                if !text.is_empty() {
                    if let Some((key, value)) = text.split_once(':') {
                        metadata.insert(key.trim().to_string(), value.trim().to_string());
                    } else {
                        push_diag(
                            parsed,
                            block,
                            block.lines[cursor].line,
                            "attestation metadata must use key: value format",
                        );
                    }
                }
                cursor += 1;
            }

            let artifact = required_metadata(parsed, block, entry.line, &metadata, "artifact");
            let owner = required_metadata(parsed, block, entry.line, &metadata, "owner");
            let last_reviewed =
                required_metadata(parsed, block, entry.line, &metadata, "last_reviewed");
            let mut valid = true;
            let review_interval_days =
                optional_review_interval(parsed, block, entry.line, &metadata, &mut valid);

            if let Some(date) = &last_reviewed
                && NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err()
            {
                push_diag(
                    parsed,
                    block,
                    entry.line,
                    "last_reviewed must use YYYY-MM-DD format",
                );
                valid = false;
            }

            if let (Some(artifact), Some(owner), Some(last_reviewed)) =
                (artifact, owner, last_reviewed)
                && valid
            {
                parsed.attests.push(AttestRef {
                    spec_id: id.to_string(),
                    artifact,
                    owner,
                    last_reviewed,
                    review_interval_days,
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

        if trimmed.starts_with('@') {
            push_diag(
                parsed,
                block,
                entry.line,
                &format!("unknown annotation `{trimmed}`"),
            );
        } else {
            push_diag(
                parsed,
                block,
                entry.line,
                "unexpected text in annotation block",
            );
        }
        index += 1;
    }
}

fn required_metadata(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    line: usize,
    metadata: &HashMap<String, String>,
    key: &str,
) -> Option<String> {
    match metadata.get(key) {
        Some(value) if !value.is_empty() => Some(value.clone()),
        _ => {
            push_diag(
                parsed,
                block,
                line,
                &format!("missing required attestation metadata `{key}`"),
            );
            None
        }
    }
}

fn optional_review_interval(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    line: usize,
    metadata: &HashMap<String, String>,
    valid: &mut bool,
) -> Option<u32> {
    let value = metadata.get("review_interval_days")?;

    match value.parse::<u32>() {
        Ok(days) => Some(days),
        Err(_) => {
            push_diag(
                parsed,
                block,
                line,
                "review_interval_days must be a positive integer",
            );
            *valid = false;
            None
        }
    }
}

fn push_diag(parsed: &mut ParsedRepo, block: &CommentBlock, line: usize, message: &str) {
    parsed.diagnostics.push(Diagnostic {
        path: block.path.clone(),
        line,
        message: message.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::model::{BlockLine, CommentBlock, NodeKind, OwnedItem, ParsedRepo, SourceLocation};

    use super::parse_block;

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
                    text: "Exports data.".to_string(),
                },
                BlockLine {
                    line: 3,
                    text: "@planned".to_string(),
                },
            ],
            owned_item: None,
        };

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].id, "EXPORT");
        assert_eq!(parsed.specs[0].kind, NodeKind::Spec);
        assert!(parsed.specs[0].planned);
        assert!(parsed.verifies.is_empty());
        assert!(parsed.attests.is_empty());
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.PLANNED
    fn parses_spec_and_planned() {
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].kind, NodeKind::Spec);
        assert!(parsed.specs[0].planned);
        assert!(parsed.diagnostics.is_empty());
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.specs[0].kind, NodeKind::Group);
        assert!(!parsed.specs[0].planned);
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.attests.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 3);
        let messages: Vec<&str> = parsed
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.attests.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 1);
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.attests.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("review_interval_days must be a positive integer")
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.specs.len(), 0);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@planned must appear after a @spec")
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.specs.len(), 1);
        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(
            parsed.diagnostics[0]
                .message
                .contains("@planned may only apply to @spec")
        );
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.UNKNOWN_ANNOTATIONS
    fn rejects_unknown_annotations() {
        let block = CommentBlock {
            path: PathBuf::from("src/example.rs"),
            lines: vec![BlockLine {
                line: 1,
                text: "@unknown THING".to_string(),
            }],
            owned_item: None,
        };

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

        assert_eq!(parsed.diagnostics.len(), 1);
        assert!(parsed.diagnostics[0].message.contains("unknown annotation"));
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

        let mut parsed = ParsedRepo::default();
        parse_block(&block, &mut parsed);

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
