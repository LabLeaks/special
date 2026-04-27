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

pub(super) use crate::annotation_syntax::normalize_markdown_annotation_line;
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
mod tests;
