# Backlog

- Parser test/file navigability: keep impl-tied parser tests colocated where private access is useful, but revisit structure if `src/parser.rs` becomes too hard to navigate by concern.
- Markdown parsing for specs/modules: support parsing `@spec` and related annotations from Markdown so backlog and roadmap material can live in normal `.md` files before code exists.
- Parser declaration flow cleanup: reduce the remaining duplication between spec and module declaration/block traversal so reserved-tag and `@planned` semantics stay unified in one place.
- Config boundary split: further separate `special.toml` parsing, root discovery, and version/dialect selection into smaller units with clearer responsibilities.
- Review-runner factoring: keep shrinking the release-review control tower by separating invocation/runner concerns from planning and validation.
- Module definition quality: strengthen source-local `@module` text where it is still just ownership labeling instead of role, boundary, and invariant description.
- Distributed architecture migration: continue moving concrete architecture intent out of central scaffolding and into source-local ownership points where it honestly belongs.
- Test semantic quality: continue replacing incidental string/copy assertions with typed or semantic checks where wording is not the contract.
- Architecture ontology evolution: revisit whether `@area` is enough, and design configurable architecture grouping kinds only after real usage pressure justifies it.
