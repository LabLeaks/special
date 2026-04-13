# special

Pronounced "spec-ee-al".

`special` is a repo-native semantic spec tool.

It reads annotated source files, materializes the current live spec, and lets you inspect the code and attestations attached to each claim.

## Source Of Truth

The canonical product truth for `special` lives in its own self-hosted spec declarations under [specs/special](specs/special).

If this README and the materialized spec ever disagree, the spec wins.

If published to crates.io, the package name is `special-cli` and the installed binary is `special`.

Useful commands:

```sh
mise exec -- cargo run -- spec
mise exec -- cargo run -- spec --all
mise exec -- cargo run -- spec SPECIAL.SPEC_COMMAND --verbose
mise exec -- cargo run -- skills
mise exec -- cargo run -- lint
```

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
- materializes the live spec by default
- includes planned claims on request
- carries optional release metadata on planned claims
- reports annotation and reference errors
- shows the attached verification and attestation bodies in verbose mode
- installs task-shaped project skills for working with product specs

This repo is self-hosting: `special`'s own behavior is described and verified in `special` format under [specs/special](specs/special).

## Command Surface

The core command is `special specs` (`special spec` also works as an alias).

```sh
mise exec -- cargo run -- spec
```

That shows the current live spec only. Planned claims are hidden by default.

Useful variants:

```sh
mise exec -- cargo run -- spec --all
mise exec -- cargo run -- spec SPECIAL.CONFIG
mise exec -- cargo run -- spec SPECIAL.CONFIG.SPECIAL_TOML --verbose
mise exec -- cargo run -- spec --unsupported
mise exec -- cargo run -- spec --json
mise exec -- cargo run -- spec --html
mise exec -- cargo run -- spec --html --verbose
mise exec -- cargo run -- spec SPECIAL.SPEC_COMMAND --json --verbose
```

`special lint` is the mechanical checker:

```sh
mise exec -- cargo run -- lint
```

It reports malformed annotations, bad references, duplicate ids, missing intermediates, orphaned `@verifies`, and related structural errors.

`special init` currently does one thing:

```sh
mise exec -- cargo run -- init
```

It creates `special.toml` in the current directory with `root = "."`, and fails rather than overwriting an existing file.

`special skills` explains and prints bundled skills:

```sh
mise exec -- cargo run -- skills
mise exec -- cargo run -- skills ship-product-change
mise exec -- cargo run -- skills install
mise exec -- cargo run -- skills install ship-product-change
```

`special skills install` writes task-shaped skills into `.agents/skills/` or another selected destination for:
- shipping a product change without drifting the contract
- defining product specs
- validating whether a claim is honestly supported
- inspecting the current live spec state
- finding planned work

The installed skill files are generated output and are typically ignored in the repo.

## Install

Local development uses `cargo run`.

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

## Release Automation

This repo carries its own release automation contract in `special` format.

Create release tags through the local wrapper so the Rust release review runs first:

```sh
python3 scripts/tag-release.py 0.3.0
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

Important constraints:

- `@group` and `@spec` are mutually exclusive for the same id.
- `@planned` is local to the owning `@spec`.
- one `@verifies` block may target only one spec id.
- child claims do not justify a parent `@spec`.
- `@verifies` only counts when it attaches to a supported owned item.

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
