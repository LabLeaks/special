Known issues intentionally left unfixed for release-review eval:

1. `src/render.rs`
   `render_spec_html()` ignores the caller's `verbose` argument and always renders verbose details.

2. `src/parser.rs`
   `review_interval_days = 0` is accepted even though the contract says the value must be positive.

3. `scripts/verify-homebrew-formula.sh`
   The check validates only assets that exist in the GitHub release payload and does not fail when a required Homebrew archive is missing entirely.

4. `tests/distribution.rs`
   Package selection relies on `cargo metadata["packages"][0]` ordering instead of selecting the package by stable identity.
