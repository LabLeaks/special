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

- TypeScript analysis provider: extend `special modules --metrics` with a built-in TypeScript provider on top of the shared analysis core.
- Go analysis provider: extend `special modules --metrics` with a built-in Go provider on top of the shared analysis core.
- Shared provider contract: keep strengthening the language-agnostic analysis model so new providers do not reintroduce Rust-specific assumptions into the core.
- Architecture implementation analysis beyond Rust: continue broadening evidence-first metrics and owned-scope item signals once at least one non-Rust provider is live.

## Later / open design

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
- Spec deprecation lifecycle: design whether live claims need explicit deprecation metadata such as `@deprecated <release>` without allowing support to invert still-positive claim text.
