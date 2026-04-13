/**
@spec SPECIAL.SPEC_COMMAND
special spec materializes the current live spec view from parsed annotations.

@spec SPECIAL.SPEC_COMMAND.LIVE_ONLY
special spec excludes planned items by default.

@spec SPECIAL.SPEC_COMMAND.ALL
special spec --all includes planned items.

@spec SPECIAL.SPEC_COMMAND.ID_SCOPE
special spec SPEC.ID scopes the materialized view to the matching spec or group node and its descendants.

@spec SPECIAL.SPEC_COMMAND.UNSUPPORTED
special spec --unsupported shows live items with zero verifies and zero attests.

@spec SPECIAL.SPEC_COMMAND.JSON
special spec --json emits the materialized spec as JSON.

@spec SPECIAL.SPEC_COMMAND.HTML
special spec --html emits the materialized spec as HTML with attached verifies and attests in collapsed detail blocks.

@spec SPECIAL.SPEC_COMMAND.HTML.CODE_HIGHLIGHTING
special spec --html renders attached code blocks with best-effort language-sensitive highlighting.

@spec SPECIAL.SPEC_COMMAND.VERBOSE
special spec --verbose shows the attached verifies and attests bodies for review.

@spec SPECIAL.SPEC_COMMAND.VERBOSE.JSON
special spec --json --verbose includes attached verifies and attests bodies in JSON output.

@spec SPECIAL.LINT_COMMAND
special lint reports annotation parsing and reference errors.

@spec SPECIAL.LINT_COMMAND.UNKNOWN_VERIFY_REFS
special lint reports @verifies references to unknown spec ids.

@spec SPECIAL.LINT_COMMAND.UNKNOWN_ATTEST_REFS
special lint reports @attests references to unknown spec ids.

@spec SPECIAL.LINT_COMMAND.INTERMEDIATE_SPECS
special lint reports missing intermediate spec ids in dot-path hierarchies.

@spec SPECIAL.LINT_COMMAND.DUPLICATE_IDS
special lint reports duplicate node ids.

@spec SPECIAL.LINT_COMMAND.PLANNED_SCOPE
special lint reports @planned outside a spec declaration block.

@spec SPECIAL.LINT_COMMAND.UNKNOWN_ANNOTATIONS
special lint reports unknown annotations in annotation blocks.

@spec SPECIAL.LINT_COMMAND.UNSUPPORTED_EXCLUDED
special lint does not report unsupported live specs.

@spec SPECIAL.LINT_COMMAND.ORPHAN_VERIFIES
special lint reports @verifies blocks that do not attach to a supported owned item.
*/
