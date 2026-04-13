/**
@spec SPECIAL.PARSE
special parses annotated comment blocks into structured spec records.

@spec SPECIAL.PARSE.LINE_COMMENTS
special parses annotation blocks from contiguous line comments.

@spec SPECIAL.PARSE.BLOCK_COMMENTS
special parses annotation blocks from block comments.

@spec SPECIAL.PARSE.GO_LINE_COMMENTS
special parses annotation blocks from Go line comments in `.go` files.

@spec SPECIAL.PARSE.TYPESCRIPT_LINE_COMMENTS
special parses annotation blocks from TypeScript line comments in `.ts` files.

@spec SPECIAL.PARSE.TYPESCRIPT_BLOCK_COMMENTS
special parses annotation blocks from TypeScript block comments in `.ts` files.

@spec SPECIAL.PARSE.MIXED_PURPOSE_COMMENTS
special parses reserved annotations from ordinary mixed-purpose comment blocks without requiring the whole block to be special-only.

@spec SPECIAL.PARSE.LINE_START_RESERVED_TAGS
special interprets reserved annotations only when they begin the normalized comment line after comment markers and leading whitespace are stripped.

@group SPECIAL.PARSE.RESERVED_TAGS
reserved special annotation shape and validation.

@spec SPECIAL.PARSE.RESERVED_TAGS.REQUIRE_DIRECTIVE_SHAPE
special reports malformed reserved annotations when a reserved tag appears at line start but omits the required directive shape, instead of silently treating it as foreign syntax.

@spec SPECIAL.PARSE.FOREIGN_TAG_BOUNDARIES
special treats foreign line-start `@...` and `\\...` tags as block boundaries for attached annotation text without treating them as special annotations.

@spec SPECIAL.PARSE.FOREIGN_TAGS_NOT_ERRORS
special does not report foreign line-start `@...` and `\\...` tags as lint errors inside mixed-purpose comment blocks.

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

@spec SPECIAL.PARSE.VERIFIES
special parses @verifies references from annotation blocks.

@spec SPECIAL.PARSE.VERIFIES.ONE_PER_BLOCK
special allows at most one @verifies per annotation block.

@spec SPECIAL.PARSE.VERIFIES.ATTACHES_TO_NEXT_ITEM
special attaches a @verifies annotation block to the next supported item in comment-based languages.

@spec SPECIAL.PARSE.VERIFIES.ONLY_ATTACHED_SUPPORT_COUNTS
special counts a @verifies reference as support only when it successfully attaches to an owned item.

@spec SPECIAL.PARSE.ATTESTS
special parses @attests records from annotation blocks.

@spec SPECIAL.PARSE.ATTESTS.REQUIRED_FIELDS
special requires the mandatory metadata fields for @attests.

@spec SPECIAL.PARSE.ATTESTS.ALLOWED_FIELDS
special rejects unknown metadata keys on @attests records.

@spec SPECIAL.PARSE.ATTESTS.DATE_FORMAT
special requires last_reviewed to use YYYY-MM-DD format.

@spec SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
special requires review_interval_days to be a positive integer when present.

@spec SPECIAL.PARSE.ARCH_ANNOTATIONS_RESERVED
special reserves `@module`, `@area`, and `@implements` for architecture metadata and does not report them as unknown spec annotations.

@spec SPECIAL.PARSE.PYTHON_LINE_COMMENTS
special parses annotation blocks from Python `#` comments in `.py` files instead of docstring ownership.
*/
