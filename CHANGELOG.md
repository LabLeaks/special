# Changelog

## 0.3.0 - 2026-04-13

- Added versioned `@planned` parsing rules with explicit `special.toml` `version` support, plus legacy fallback with a warning when the version key is missing.
- Made planned ownership local and explicit in version `1`, including support for `@spec ID @planned` and next-line adjacency.
- Added optional planned release metadata parsing with output surfacing for `@planned <version>` in text, JSON, and HTML spec views.
- Reworked `special skills` so `special skills` prints overview help, `special skills SKILL_ID` prints a specific skill to stdout, and `special skills install [SKILL_ID]` handles installation with project/global/custom destinations and overwrite prompts.
- Improved bundled skill guidance and examples to emphasize stable product invariants, durable tests, and general engineering principles instead of repo-specific rules.
- Added a local-only Rust release review flow using Codex Spark, scoped to the diff from the previous release tag by default, and wired it into `python3 scripts/tag-release.py <version>` before tagging.
- Tightened several release and contract checks, including exact distribution asset validation, directory-only config roots, HTML verbose parity, and positive attestation review intervals.
