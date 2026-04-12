/**
@group SPECIAL.QUALITY.RUST
special defines Rust-specific quality contracts for its implementation.

@group SPECIAL.QUALITY.RUST.CLIPPY
special enforces strict Rust quality guidelines with a dedicated clippy verification.
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
