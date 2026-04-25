# Changelog

## 0.7.1 - 2026-04-25

- TypeScript and Go now get the same `special arch --metrics` depth as Rust:
  complexity, quality, dependencies, coupling, and per-item hotspot detail all
  show up in text and JSON output.
- Per-item metrics are more trustworthy across all three shipped language packs:
  nested helpers no longer inflate the parent item, and dense or generated
  TypeScript/Go code is less likely to collapse distinct items into one signal.
- The module metrics contract is now explicit for Rust, TypeScript, and Go, so
  future analyzer work has a clearer parity target.
- Hardened release tooling so repeat publication attempts choose the intended
  Jujutsu release revision and release review keeps a useful baseline when the
  current release tag already exists locally.
- Added a 0.7.2 backlog for moving shared traceability reasoning into a
  production Lean kernel and upgrading language packs into canonical fact
  adapters.

## 0.7.0 - 2026-04-23

- Landed a shared projected-contract proof architecture across the shipped Rust,
  TypeScript, and Go language packs, including a normalized proof object in
  core, pack-local adapters, a top-level proof-boundary harness, and Lean
  sidecar theorem files that now speak to the same public semantic kernel.
- Hardened the shipped backward-trace surfaces around exact scoped closure,
  contract-only tool discovery, fail-closed cache/tool/runtime behavior, and
  cleaner repo-level `special arch --metrics` coupling summaries.
- Tightened internal structure around the new proof-heavy surfaces, including
  owned proof-boundary modules, refactored TypeScript test helpers, and a
  durable native review wrapper for full-system `codex exec review` passes.
- Removed the Python language pack from the shipped built-in registry for this
  release line while preserving the in-progress implementation in-tree as
  dormant work to reactivate once its bridge, toolchain contract, and honesty
  story are hardened enough to ship.

## 0.6.0 - 2026-04-20

- Reframed the product around four stable command surfaces: `special`, `special specs`, `special arch`, and `special health`, with a consistent `--metrics` / `--verbose` ladder and a compact root overview.
- Promoted implementation traceability into the default `special health` surface, including built-in Rust, TypeScript, Go, Python, and TSX/React-style analysis under the compile-time language-pack registry.
- Added shared parsed and analysis caching across overview, specs, architecture, and health, including lock recovery, contention hardening, and real-time cache-wait status so concurrent agent runs explain when they are reusing another run’s analysis.
- Tightened architecture ownership and self-hosting boundaries across the repo, including summary-first `special arch --metrics`, explicit ownership for previously unowned implementation files, and removal of legacy health heuristics that were dominated by traceability.
- Updated bundled skills, README guidance, release automation, and self-hosted contracts to match the shipped command model and current/planned terminology.

## 0.5.0 - 2026-04-17

- Added lightweight claim retirement metadata with `@deprecated <release>`, surfaced across `special specs` text, JSON, and HTML views without changing live-claim support semantics.
- Added a shared cross-language syntax and analysis substrate, with shipped built-in module metrics for owned Rust, TypeScript, and Go code on the same provider seam.
- Tightened `special modules --metrics` around annotated architecture: module ownership granularity, implementation summaries, dependencies, coupling, quality evidence, and conservative unreached-code indicators now stay on the module side, while repo-wide quality signals moved out.
- Added `special repo` as the repo-wide quality surface, including duplication and unowned unreached-code signals, with `--verbose` for fuller drilldown and `--experimental` for early implementation traceability.
- Landed an initial experimental impl-to-test-to-spec traceability indicator behind `special repo --experimental`, while keeping deeper cross-language traceability hardening planned for later releases.
- Hardened release and parser correctness around the new surfaces, including stricter Homebrew formula verification and consistent planned/deprecated lifecycle validation across block and markdown parsing.

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
- If you used fake source files as centralized contract containers, prefer moving current claims into the owning source or test files. Use markdown heading annotations only for real declarative residue that has no honest code home yet.
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
