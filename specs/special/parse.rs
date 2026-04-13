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

@spec SPECIAL.PARSE.PLANNED
special records @planned on the owning @spec declared in the same annotation block.

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

@spec SPECIAL.PARSE.ATTESTS.DATE_FORMAT
special requires last_reviewed to use YYYY-MM-DD format.

@spec SPECIAL.PARSE.ATTESTS.REVIEW_INTERVAL_DAYS
special requires review_interval_days to be a positive integer when present.

@spec SPECIAL.PARSE.PYTHON_LINE_COMMENTS
special parses annotation blocks from Python `#` comments in `.py` files instead of docstring ownership.
*/
