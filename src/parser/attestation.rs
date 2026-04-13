/**
@module SPECIAL.PARSER.ATTESTATION
Attestation metadata parsing in `src/parser/attestation.rs`.
*/
// @implements SPECIAL.PARSER.ATTESTATION
use chrono::NaiveDate;

use crate::model::{CommentBlock, ParsedRepo};

use super::push_diag;

#[derive(Debug, Clone)]
pub(super) struct AttestationMetadata {
    pub(super) artifact: String,
    pub(super) owner: String,
    pub(super) last_reviewed: String,
    pub(super) review_interval_days: Option<u32>,
}

#[derive(Debug, Clone)]
struct MetadataValue {
    line: usize,
    value: String,
}

#[derive(Debug, Default, Clone)]
struct AttestationMetadataDraft {
    artifact: Option<MetadataValue>,
    owner: Option<MetadataValue>,
    last_reviewed: Option<MetadataValue>,
    review_interval_days: Option<MetadataValue>,
    valid: bool,
}

impl AttestationMetadataDraft {
    fn new() -> Self {
        Self {
            valid: true,
            ..Self::default()
        }
    }

    fn collect_line(
        &mut self,
        parsed: &mut ParsedRepo,
        block: &CommentBlock,
        line: usize,
        text: &str,
    ) {
        let Some((key, value)) = text.split_once(':') else {
            push_diag(
                parsed,
                block,
                line,
                "attestation metadata must use key: value format",
            );
            self.valid = false;
            return;
        };

        let key = key.trim();
        let value = value.trim().to_string();
        let slot = match key {
            "artifact" => &mut self.artifact,
            "owner" => &mut self.owner,
            "last_reviewed" => &mut self.last_reviewed,
            "review_interval_days" => &mut self.review_interval_days,
            _ => {
                push_diag(
                    parsed,
                    block,
                    line,
                    &format!("unknown attestation metadata `{key}`"),
                );
                self.valid = false;
                return;
            }
        };

        if let Some(first) = slot.as_ref() {
            push_diag(
                parsed,
                block,
                line,
                &format!(
                    "duplicate attestation metadata `{key}`; first declared on line {}",
                    first.line
                ),
            );
            self.valid = false;
            return;
        }

        *slot = Some(MetadataValue { line, value });
    }

    fn finalize(
        self,
        parsed: &mut ParsedRepo,
        block: &CommentBlock,
        header_line: usize,
    ) -> Option<AttestationMetadata> {
        let mut valid = self.valid;
        let last_reviewed_line = self
            .last_reviewed
            .as_ref()
            .map(|value| value.line)
            .unwrap_or(header_line);
        let artifact = required_attestation_value(
            parsed,
            block,
            header_line,
            self.artifact,
            "artifact",
            &mut valid,
        );
        let owner =
            required_attestation_value(parsed, block, header_line, self.owner, "owner", &mut valid);
        let last_reviewed = required_attestation_value(
            parsed,
            block,
            header_line,
            self.last_reviewed,
            "last_reviewed",
            &mut valid,
        );
        let review_interval_days =
            optional_review_interval(parsed, block, self.review_interval_days, &mut valid);

        if let Some(date) = last_reviewed.as_deref()
            && NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err()
        {
            push_diag(
                parsed,
                block,
                last_reviewed_line,
                "last_reviewed must use YYYY-MM-DD format",
            );
            valid = false;
        }

        if valid {
            Some(AttestationMetadata {
                artifact: artifact?,
                owner: owner?,
                last_reviewed: last_reviewed?,
                review_interval_days,
            })
        } else {
            None
        }
    }
}

pub(super) fn parse_attestation_metadata(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    header_line: usize,
    start_index: usize,
) -> (Option<AttestationMetadata>, usize) {
    let mut metadata = AttestationMetadataDraft::new();
    let mut cursor = start_index;
    while cursor < block.lines.len() {
        let text = block.lines[cursor].text.trim();
        if text.starts_with('@') {
            break;
        }
        if !text.is_empty() {
            metadata.collect_line(parsed, block, block.lines[cursor].line, text);
        }
        cursor += 1;
    }

    (metadata.finalize(parsed, block, header_line), cursor)
}

fn required_attestation_value(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    header_line: usize,
    value: Option<MetadataValue>,
    key: &str,
    valid: &mut bool,
) -> Option<String> {
    match value {
        Some(MetadataValue { value, .. }) if !value.is_empty() => Some(value),
        Some(MetadataValue { line, .. }) => {
            push_diag(
                parsed,
                block,
                line,
                &format!("missing required attestation metadata `{key}`"),
            );
            *valid = false;
            None
        }
        None => {
            push_diag(
                parsed,
                block,
                header_line,
                &format!("missing required attestation metadata `{key}`"),
            );
            *valid = false;
            None
        }
    }
}

fn optional_review_interval(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    value: Option<MetadataValue>,
    valid: &mut bool,
) -> Option<u32> {
    let MetadataValue {
        line: diagnostic_line,
        value,
    } = value?;

    match value.parse::<u32>() {
        Ok(days) if days > 0 => Some(days),
        Ok(_) | Err(_) => {
            push_diag(
                parsed,
                block,
                diagnostic_line,
                "review_interval_days must be a positive integer",
            );
            *valid = false;
            None
        }
    }
}
