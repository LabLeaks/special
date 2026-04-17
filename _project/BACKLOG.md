# Backlog

## 0.4.1 priorities

- File attests: add `@fileattests` for explicit file-scoped attestation artifacts such as long Markdown review records without overloading item-scoped `@attests`.
- Parser declaration flow cleanup: reduce the remaining duplication between spec and module declaration/block traversal so reserved-tag and `@planned` semantics stay unified in one place.
- Config boundary split: further separate `special.toml` parsing, root discovery, and version/dialect selection into smaller units with clearer responsibilities.
- Review-runner factoring: keep shrinking the release-review control tower by separating invocation/runner concerns from planning and validation.
- Module definition quality: strengthen source-local `@module` text where it is still just ownership labeling instead of role, boundary, and invariant description.
- Distributed architecture migration: continue moving concrete architecture intent out of central scaffolding and into source-local ownership points where it honestly belongs.
- Test semantic quality: continue replacing incidental string/copy assertions with typed or semantic checks where wording is not the contract.

## 0.5.0 priorities

- Minimal claim deprecation tag: add `@deprecated <release>` as lightweight lifecycle metadata so teams can mark a live claim for retirement, clean up attached verifies and attests, and only then delete the claim without relying on orphan diagnostics as the workflow.
- Stable module metrics release shape: keep `special modules --metrics` centered on honest module/item evidence across Rust, TypeScript, and Go, with experimental traceability living under `special repo --experimental` instead of the architecture command.
- Repo-wide duplication signal: surface lightweight duplicate-logic signals across owned implementation as a parallel architectural-analysis signal without depending on embeddings or language-specific heavyweight tooling.
- Shared provider contract hardening: consolidate the language-agnostic analysis seam now that Rust, TypeScript, and Go all exist so new providers do not reintroduce Rust-specific assumptions into the core.
- Shared analysis graph substrate: keep separating repo-wide extraction, parse-tree creation, and graph construction from language-specific ownership, item extraction, and metrics so architecture analysis does not leak parser-specific details upward.
- Cross-language metrics follow-through: tighten the shared metrics surface and explanation text across Rust, TypeScript, and Go so `0.5.0` ships a coherent language-agnostic analysis story.
- Release-stack cleanup: split and polish the current `0.5.0` working stack into a reviewable, release-ready shape instead of carrying one large mixed checkpoint change.

## 0.6.0 candidates

- Spec-ended implementation traceability headline: surface whether owned implementation code reaches any live, planned, or deprecated spec through a verifying test, and flag implementation with no such association as unknown or underspecified.
- Rust-first traceability proof hardening: keep improving impl item -> verifying test -> spec-state resolution on Rust until the traceability surface is honest enough to stand on its own.
- TypeScript traceability follow-through: extend the shipped TypeScript metrics provider with impl-to-verifying-test traceability support on the shared analysis core without backsliding into Rust-specific assumptions.
- Go traceability follow-through: extend the shipped Go metrics provider with impl-to-verifying-test traceability support on the shared analysis core once the provider contract is stable enough to do it honestly.
- Parse-layer expansion research: evaluate whether a tree-sitter-backed shared parse layer is the right way to support cross-language traceability and owned-item extraction without hardwiring Rust-specific traversal into the core.
- Traceability IR convergence checkpoint: keep the broader lowered-unit/source-trace IR direction documented and aligned in spirit with the proofs spike, but do not block near-term releases on landing the full IR refactor before the experimental traceability surface is honest enough to graduate.

## Later / open design

- Repo-quality surface refinement: keep deciding which cross-cutting quality signals belong under `special repo` versus future deeper views so module metrics stay focused on annotated architecture instead of becoming a generic code-quality dump.
- Interactive dashboard surface: design a human-oriented `special` entrypoint that opens a live HTML and/or TUI dashboard combining specs, modules, metrics, and quality signals into one browsable analysis surface rather than optimizing only for agent-friendly CLI output.
- Spec diffing for changelog generation: investigate version/tag-aware spec diffs as an input to an LLM changelog generator, with explicit research on how to choose a meaningful diff baseline without baking in brittle release-history assumptions.
- Parser test/file navigability: keep impl-tied parser tests colocated where private access is useful, but revisit structure if `src/parser.rs` becomes too hard to navigate by concern.
- Markdown parsing placement: keep using Markdown only for real declarative residue that has no honest code or test home.
- Architecture ontology evolution: revisit whether `@area` is enough, and design configurable architecture grouping kinds only after real usage pressure justifies it.
- Orthogonal architecture classifications: design non-owning architecture views on top of the primary `@module`/`@area` tree, and avoid introducing multiple competing ownership trees.
- Report/UI hardening: continue tightening the rendered report and HTML UI without promoting generic implementation-quality cleanup into product specs.
- Richer filtering and metadata views: design broader filtering/projection capabilities once the desired query model is concrete enough to spec as real product behavior.
- Verification contract colocation: make verification contracts locally inspectable so reviewers can validate them without chasing multiple files.
- Verification review surfaces: prefer verification designs where the important contract can be understood from one local review surface instead of being split across spec prose, helper code, and tool config.
- Verification drift warnings: design warnings for verification artifacts that rely on external or ambient inputs that can silently drift.
- Verification undeclared external inputs: surface when a verification artifact depends on undeclared external tools, configuration, environment variables, or shell features.
- Claim retirement workflow: keep the first `@deprecated <release>` design narrow as lifecycle metadata and surfacing only, and defer any stronger semantics until real usage pressure justifies them.
