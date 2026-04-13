/**
@group SPECIAL.QUALITY.RUST
special defines Rust-specific quality contracts for its implementation.

@group SPECIAL.QUALITY.RUST.CLIPPY
special enforces strict Rust quality guidelines with a dedicated clippy verification.

@group SPECIAL.QUALITY.RUST.RELEASE_REVIEW
special runs a warn-only Codex release review for Rust changes that are outside clippy's scope.
*/

/**
@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS
special verifies its clippy quality contract with a pinned command and explicit flags so the rule cannot drift with unrelated global lint settings.
*/

/**
@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.MISE_EXEC
the clippy verification script runs the command through `mise exec --`.
*/

/**
@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.CARGO_CLIPPY
the clippy verification script invokes `cargo clippy`.
*/

/**
@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_TARGETS
the clippy verification script passes `--all-targets`.
*/

/**
@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.ALL_FEATURES
the clippy verification script passes `--all-features`.
*/

/**
@spec SPECIAL.QUALITY.RUST.CLIPPY.PINNED_FLAGS.DENY_WARNINGS
the clippy verification script passes `-- -D warnings`.
*/

/**
@spec SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED
special keeps the quality-verification command owned by the related spec instead of relying on implicit repo-wide linter defaults.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SPEC_OWNED
special keeps the Codex-based release review command owned by the related quality spec instead of relying on ad hoc release instructions.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.DEFAULT_MODEL
the Rust release review script defaults to the pinned `gpt-5.3-codex` model when no model flag is provided.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.FAST_MODEL
the Rust release review script uses the pinned `gpt-5.3-codex-spark` model when invoked with `--fast`.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SMART_MODEL
the Rust release review script uses the pinned `gpt-5.4` model when invoked with `--smart`.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.STRUCTURED_OUTPUT
the Rust release review script constrains Codex output with a checked-in JSON schema.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.CODE_ONLY_SURFACE
the Rust release review script reviews implementation, tests, release scripts, workflows, and Cargo metadata as code, and does not pull in product specs or prose docs as related review context.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.READ_ONLY_SANDBOX
the Rust release review script invokes Codex in `read-only` sandbox mode.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.NO_WEB
the Rust release review script disables Codex web search so the release audit stays local to the provided code context.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.PROJECT_ROOT_READ_SCOPE
the Rust release review script grants explicit Codex read permissions for detected project roots instead of relying on broader ambient filesystem access.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.DIFF_SCOPED_BY_DEFAULT
by default, the Rust release review script scopes context to the release diff instead of rescanning the full codebase.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.JJ_LATEST_TAG_BASELINE
in a jj-backed repo, the Rust release review script defaults its baseline to the latest semver release tag.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SYNTAX_AWARE_CHANGED_CONTEXT
for diff-scoped review, the Rust release review script sends changed hunks plus local syntax-aware snippets around changed Rust items instead of full-file companion context.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.INPUT_BUDGET
the Rust release review script caps each Codex request at roughly 32k input tokens using a 4 chars-per-token estimate.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.CHUNKED_CONTEXT
when a release review pass would exceed the input budget, the Rust release review script splits it into smaller review chunks instead of sending one oversized prompt.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SKIPPED_CHUNK_WARNINGS
when a review chunk still cannot fit within the input budget, the Rust release review script reports a local runner warning instead of silently dropping or sending the oversized chunk.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.FULL_SCAN_MODE
the Rust release review script supports a full-scan mode for establishing the initial baseline or for occasional whole-repo passes.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.WARN_ONLY
the Rust release review script exits successfully when Codex returns style warnings, so review findings do not hard-block the release flow.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.LOCAL_ONLY
the Rust release review script is a local-only release check and does not invoke Codex from CI environments.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.NO_BYTECODE_ARTIFACTS
the local Python release-review and release-tagging scripts disable Python bytecode writes so running them does not create `__pycache__` artifacts in the repository.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.AUTO_RUNS_ONLY_FROM_RELEASE_TAG_FLOW
special only auto-runs the Rust release review from the local release-tagging flow and does not wire it into CI workflows.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW
special provides a dedicated local release-tagging script that runs the Rust release review before creating a release tag.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.PREVIOUS_TAG_DIFF
the release-tagging script runs the Rust release review with the default previous-tag diff scope instead of forcing a full scan.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.DRY_RUN
`special`'s release-tagging dry-run does not invoke a live Codex review or create a tag; it prints the planned review and tag actions plus a dry-run review preview.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.SKIP_REVIEW
the release-tagging script accepts `--skip-review` to bypass the Rust release review and create or preview the tag directly.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.MODE_FLAGS
the release-tagging script passes through the release review's `--fast` and `--smart` model-selection flags.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.SCOPE_OVERRIDES
the release-tagging script passes through explicit release-review scope overrides such as `--full` and `--base`, while defaulting to the previous-tag diff when neither is provided.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.SURFACES_REVIEW_FAILURES
when the release review exits unsuccessfully but still returns structured output, the release-tagging script prints that review payload before aborting instead of swallowing the failure details.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.PROMPTS_ON_WARNINGS
when the Rust release review returns warnings, the release-tagging script prompts before creating the tag.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.NON_INTERACTIVE_WARNINGS_ABORT
when the release review returns warnings and interactive confirmation is unavailable, the release-tagging script exits with an explicit error instead of crashing or creating the tag.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.MATCHES_MANIFEST_VERSION
the release-tagging script requires the requested tag version to exactly match the current `Cargo.toml` package version.
*/

/**
@spec SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW.VALIDATES_REVIEW_PAYLOAD
the release-tagging script validates structured review output before using it to decide whether tagging may proceed.
*/
