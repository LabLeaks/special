# Changelog

## 0.4.1 - 2026-04-16

- Added `@fileattests` as the file-scoped attestation companion to `@attests`, so long review artifacts can attach predictably without item-scope ambiguity.
- Tightened the self-hosted architecture around the metrics POC by splitting the extractor, markdown declaration parsing, architecture declaration helpers, and skill-install transaction flow into clearer module boundaries.
- Refreshed the Homebrew install support record for the current release line and relaxed help-surface verifies so they prove semantic command/help contracts instead of incidental prose.
- Hardened the reusable spec and architecture audit skills so delegated fan-out reviews explicitly use `special spec ... --verbose` and `special modules ... --verbose/--metrics --verbose` as their primary evidence source.

## 0.4.0 - 2026-04-15

- Added `special modules --metrics` as a Rust-first architecture-as-implemented view, including ownership coverage, complexity, dependency and coupling evidence, quality signals, and item-level outlier surfacing inside claimed module boundaries.
- Added explicit file-scoped architecture and verification annotations with `@fileimplements` and `@fileverifies`, which makes ownership and support attachment more predictable across languages and removes brittle header-position inference.
- Generalized annotation discovery with shared ignore handling, default `.gitignore` / `.jjignore` respect, markdown heading annotations as a first-class declaration surface, and no reliance on a privileged architecture or spec directory.
- Pushed self-hosted contracts much closer to their owning boundaries, leaving only minimal central structural/planned residue in `specs/root.md`.
- Refactored major internal hotspots uncovered by the new metrics, including parser block handling and module-analysis rendering, around clearer module boundaries and a shared projection/viewmodel layer for text and HTML output.
- Added top-level `special help`, `special -h`, `special -v`, and `special --version`.
- Upgraded local release publication so `scripts/tag-release.py` runs an explicit prerelease checklist before pushing `main`, tagging, verifying the GitHub release, and updating Homebrew.

### Migration Notes

- If a file-level ownership marker previously used `@implements`, change it to `@fileimplements`. Plain `@implements` is now item-scoped only.
- If a file-level verification marker previously used `@verifies`, change it to `@fileverifies`. Plain `@verifies` is now item-scoped only.
- If you used fake source files as centralized contract containers, prefer moving live claims into the owning source or test files. Use markdown heading annotations only for real declarative residue that has no honest code home yet.
- Do not rely on a privileged architecture/spec directory. Discovery is now shared across the project root and respects `special.toml` ignores plus VCS ignore files.

## 0.3.0 - 2026-04-14

- Added plural primary command surfaces: `special specs` and `special modules`, while keeping singular aliases for compatibility.
- Added architecture annotations and materialization: `@module`, `@area`, and `@implements`, plus `special modules` text/JSON/HTML/verbose views and matching lint support.
- Moved toward distributed authoring by supporting source-local module declarations and implementation ownership markers, with `ARCHITECTURE.md` reduced to project-specific rationale and cross-cutting structure.
- Added versioned `@planned` parsing rules with explicit `special.toml` `version` support, legacy compatibility fallback with warnings, and optional planned release metadata surfaced in spec and module views.
- Reworked `special skills` so `special skills` prints overview help, `special skills SKILL_ID` prints a specific skill to stdout, and `special skills install [SKILL_ID]` supports project/global/custom destinations, overwrite handling, and non-interactive destination flags.
- Split product-contract validation from architecture validation by shipping a dedicated `validate-architecture-implementation` skill alongside the existing product-spec workflow skills.
- Added a local-only Rust release review and tagging flow in `scripts/tag-release.py`, including diff-scoped review by default, `--fast`/`--smart` model selection, and an explicit `--skip-review` escape hatch for local release use.
- Tightened release, parser, and install robustness across the repo, including exact distribution asset validation, real TOML parsing for `special.toml`, stricter reserved-tag handling, directory-only config roots, and safer staged skill installation.
