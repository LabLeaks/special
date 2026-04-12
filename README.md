# special

Pronounced "spec-ee-al".

`special` is a repo-native claim-and-support tool for specification-driven development.

It reads source annotations, materializes the current live spec, and can include planned claims on request.

## Source Of Truth

The canonical product truth for `special` lives in its own self-hosted spec declarations under [specs/special](specs/special).

If this README and the materialized spec ever disagree, the spec wins.

Useful commands:

```sh
mise exec -- cargo run -- spec
mise exec -- cargo run -- spec --all
mise exec -- cargo run -- lint
```

The repo root is explicitly anchored by [special.toml](special.toml).

## What This File Is For

This README is intentionally non-normative. It exists to explain the project at a glance:
- what `special` is
- where the real spec lives
- how to inspect the live and planned spec views
- how to work on the repo

Behavioral requirements, planned work, and command semantics should be expressed in `special` format, not carried here as prose.

## Current Shape

Today `special` is:
- a Rust CLI
- a parser/indexer for annotation blocks in source files
- a self-hosted spec demo, where the repo describes and checks its own behavior

The default live view excludes `@planned` claims. Use `special spec --all` to include them.

## Annotation Example

```text
/**
@spec EXPORT.DOESNTCRASH
When you export something, it does not crash.
*/

/**
@verifies EXPORT.DOESNTCRASH
*/
```

Planned claims stay in the same format:

```text
/**
@spec EXPORT.METADATA
Exports include provenance metadata.

@planned
*/
```

## Development

Run the local checks with:

```sh
mise exec -- cargo fmt
mise exec -- cargo test
```

To inspect the repo's own current spec:

```sh
mise exec -- cargo run -- spec
```
