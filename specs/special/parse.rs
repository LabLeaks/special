/**
@spec SPECIAL.PARSE
special parses annotated comment blocks into structured spec records.

@spec SPECIAL.PARSE.LINE_COMMENTS
special parses annotation blocks from contiguous line comments.

@spec SPECIAL.PARSE.BLOCK_COMMENTS
special parses annotation blocks from block comments.

@spec SPECIAL.PARSE.PLANNED
special records @planned on the owning @spec declared in the same annotation block.

@spec SPECIAL.PARSE.VERIFIES
special parses @verifies references from annotation blocks.

@spec SPECIAL.PARSE.VERIFIES.ONE_PER_BLOCK
special allows at most one @verifies per annotation block.

@spec SPECIAL.PARSE.VERIFIES.ATTACHES_TO_NEXT_ITEM
special will attach a @verifies annotation block to the next supported item in comment-based languages.

@spec SPECIAL.PARSE.VERIFIES.ONLY_ATTACHED_SUPPORT_COUNTS
special counts a @verifies reference as support only when it successfully attaches to an owned item.

@spec SPECIAL.PARSE.ATTESTS
special parses @attests records from annotation blocks.

@spec SPECIAL.PARSE.ATTESTS.REQUIRED_FIELDS
special requires the mandatory metadata fields for @attests.

@spec SPECIAL.PARSE.ATTESTS.DATE_FORMAT
special requires last_reviewed to use YYYY-MM-DD format.

@spec SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
special requires review_interval_days to be a positive integer when present.

@spec SPECIAL.PARSE.PYTHON_LINE_COMMENTS
special will support Python through comment-based annotation blocks instead of docstring ownership.

@planned
*/
