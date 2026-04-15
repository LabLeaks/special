# special

Pronounced "spec-ee-al".

`special` is a repo-native semantic spec tool.

It reads annotated source files, materializes the current live spec and architecture view, and lets you inspect the code and attestations attached to each claim or module.

## Source Of Truth

The canonical product truth for `special` lives in its own self-hosted spec declarations under [specs/special](specs/special).

If this README and the materialized spec ever disagree, the spec wins.

If published to crates.io, the package name is `special-cli` and the installed binary is `special`.

The repo root is explicitly anchored by [special.toml](special.toml).

## What This File Is For

This README is intentionally non-normative. It exists to explain the project at a glance:
- what `special` is
- where the real spec lives
- how to inspect the live and planned spec views
- how the annotation model works
- how root discovery works
- how to work on the repo

Behavioral requirements, planned work, and command semantics should be expressed in `special` format, not carried here as prose.

## What special Does

Today `special` is a Rust CLI that:
- parses annotation blocks from supported source files
- builds one spec tree across files and file types
- builds one architecture module tree across source-local declarations and `_project/ARCHITECTURE.md`
- materializes the live spec by default
- materializes live modules by default
- includes planned claims on request
- carries optional release metadata on planned claims
- reports annotation and reference errors
- shows the attached verification and attestation bodies in verbose mode
- shows attached implementation bodies for architecture review in verbose module views
- installs task-shaped project skills for working with product specs and architecture validation

This repo is self-hosting: `special`'s own behavior is described and verified in `special` format under [specs/special](specs/special).

## Command Surface

The core commands are `special specs` and `special modules` (`special spec` and `special module` also work as aliases).

```sh
special specs
```

That shows the current live spec only. Planned claims are hidden by default.

Useful variants:

```sh
special specs --all
special specs SPECIAL.CONFIG
special specs SPECIAL.CONFIG.SPECIAL_TOML --verbose
special specs --unsupported
special specs --json
special specs --html
special specs --html --verbose
special specs SPECIAL.SPEC_COMMAND --json --verbose
```

Architecture views work the same way:

```sh
special modules
special modules --all
special modules SPECIAL.PARSER --verbose
special modules --unsupported
special modules --json
special modules --html --verbose
```

`special lint` is the mechanical checker:

```sh
special lint
```

It reports malformed annotations, bad references, duplicate ids, missing intermediates, orphaned `@verifies`, invalid `@implements`, and related structural errors.

`special init` currently does one thing:

```sh
special init
```

It creates `special.toml` in the current directory with `root = "."`, and fails rather than overwriting an existing file.

`special skills` explains and prints bundled skills:

```sh
special skills
special skills ship-product-change
special skills install
special skills install ship-product-change
```

`special skills install` writes task-shaped skills into `.agents/skills/` or another selected destination for:
- shipping a product change without drifting the contract
- defining product specs
- validating whether a claim is honestly supported
- validating whether a concrete architecture module is honestly implemented
- inspecting the current live spec state
- finding planned work

The installed skill files are generated output and are typically ignored in the repo.

## Install

Published binaries are available from GitHub Releases for `LabLeaks/special`.

Homebrew is the primary install path:

```sh
brew install LabLeaks/homebrew-tap/special
```

Cargo is a secondary install path:

```sh
cargo install special-cli
```

That installs the `special` binary.

## Development

For local repo development, use the tool-managed commands:

```sh
mise exec -- cargo test
mise exec -- cargo run -- lint
mise exec -- cargo run -- spec --all
```

## Release Automation

This repo carries its own release automation contract in `special` format.

Create release tags through the local wrapper so the Rust release review runs first:

```sh
python3 scripts/tag-release.py 0.3.0
```

If you intentionally need to bypass the review step, use:

```sh
python3 scripts/tag-release.py 0.3.0 --skip-review
```

The current live distribution slice covers:
- crates.io package name and installed binary name
- GitHub repository metadata for release automation
- committed GitHub Actions release workflow
- published release archives and checksums for supported targets
- committed Homebrew formula in `LabLeaks/homebrew-tap`

Actual published GitHub Releases are a separate claim from release automation itself.

## Annotation Model

`special` currently uses these annotations:

- `@group ID`
  Structural container only. Groups organize subtrees and do not carry direct support.
- `@spec ID`
  Real claim node.
- `@planned`
  Marks a `@spec` as not part of the live spec yet, and may optionally carry a release string like `@planned 0.3.0`.
- `@verifies ID`
  Attaches one verification artifact to one claim.
- `@attests ID`
  Attaches a manual or external attestation to one claim.
- `@module ID`
  Concrete architecture module.
- `@area ID`
  Structural architecture node.
- `@implements ID`
  Attaches implementation ownership to a concrete architecture module.

Important constraints:

- `@group` and `@spec` are mutually exclusive for the same id.
- `@planned` is local to the owning `@spec`.
- one `@verifies` block may target only one spec id.
- child claims do not justify a parent `@spec`.
- `@verifies` only counts when it attaches to a supported owned item.
- live `@module` nodes require direct `@implements` unless they are planned.
- `@area` is structural only and does not accept `@planned` or `@implements`.

## Annotation Examples

```text
/**
@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.
*/

/**
@verifies EXPORT.CSV.HEADERS
*/
```

Planned claims use the same declaration form:

```text
/**
@spec EXPORT.METADATA
@planned
Exports include provenance metadata.
*/
```

Planned claims may also carry release metadata:

```text
/**
@spec EXPORT.METADATA @planned 0.3.0
Exports include provenance metadata.
*/
```

Structural organization uses `@group`:

```text
/**
@group EXPORT
Export-related claims.
*/
```

Architecture declarations follow the parallel model:

```text
/**
@area SPECIAL
Top-level product area.
*/

/**
@module SPECIAL.PARSER
Parses reserved annotations from extracted comment blocks.
*/

// @implements SPECIAL.PARSER
```

Verbose review works best when a `@verifies` block sits directly above the item it owns:

```ts
// @verifies EXPORT.CSV.HEADERS
test("csv export includes selected column headers", async () => {
  const csv = await exportOrdersCsv({
    columns: ["order_id", "status"],
  });

  expect(csv.split("\n")[0]).toBe("order_id,status");
});
```

## Root Discovery

`special` prefers explicit root selection.

The supported config file is `special.toml`:

```toml
version = "1"
root = "."
```

Current behavior:
- if `special.toml` is present, it anchors discovery
- `root` is resolved relative to the config file
- if no config exists, `special` prefers the nearest enclosing VCS root
- if no config or VCS marker exists, it falls back to the current directory
- implicit root selection emits a warning

`special init` exists to make that root explicit quickly.

## Supported File Types

Current self-hosted live support covers:
- Rust line comments
- generic block comments
- Go line comments
- TypeScript line comments
- TypeScript block comments
- shell `#` comments
- Python `#` comments

`special` also supports spec trees spread across multiple files and mixed supported file types.

## Self-Hosting

This repository uses `special` to describe and verify itself.

Useful inspection commands:

```sh
mise exec -- cargo run -- spec
mise exec -- cargo run -- spec --all
mise exec -- cargo run -- spec SPECIAL.CONFIG.SPECIAL_TOML --verbose
mise exec -- cargo run -- spec SPECIAL.SPEC_COMMAND --verbose
```

If you want the whole live contract in machine-readable form:

```sh
mise exec -- cargo run -- spec --json
```

If you want the broader picture including planned work:

```sh
mise exec -- cargo run -- spec --all --json
```

## Development

Run the local checks with:

```sh
mise exec -- cargo fmt
mise exec -- cargo test
mise exec -- cargo run -- lint
```

To inspect the repo's own current spec:

```sh
mise exec -- cargo run -- spec
```
