# Changelog

## 0.3.0 - 2026-04-14

- Added plural primary command surfaces: `special specs` and `special modules`, while keeping singular aliases for compatibility.
- Added architecture annotations and materialization: `@module`, `@area`, and `@implements`, plus `special modules` text/JSON/HTML/verbose views and matching lint support.
- Moved toward distributed authoring by supporting source-local module declarations and implementation ownership markers, with `ARCHITECTURE.md` reduced to project-specific rationale and cross-cutting structure.
- Added versioned `@planned` parsing rules with explicit `special.toml` `version` support, legacy compatibility fallback with warnings, and optional planned release metadata surfaced in spec and module views.
- Reworked `special skills` so `special skills` prints overview help, `special skills SKILL_ID` prints a specific skill to stdout, and `special skills install [SKILL_ID]` supports project/global/custom destinations, overwrite handling, and non-interactive destination flags.
- Split product-contract validation from architecture validation by shipping a dedicated `validate-architecture-implementation` skill alongside the existing product-spec workflow skills.
- Added a local-only Rust release review and tagging flow in `scripts/tag-release.py`, including diff-scoped review by default, `--fast`/`--smart` model selection, and an explicit `--skip-review` escape hatch for local release use.
- Tightened release, parser, and install robustness across the repo, including exact distribution asset validation, real TOML parsing for `special.toml`, stricter reserved-tag handling, directory-only config roots, and safer staged skill installation.
