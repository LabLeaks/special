# special

Pronounced "spec-ee-al".

AI dev starts fast.

Then it bogs down.

The agent writes code against behavior that is only planned. A tagged test does
not really prove the claim it points at. A module boundary sounds clean, but the
implementation is broad and tangled. You stop trusting what the agent thinks is
true, and now you are back to grepping through the repo to reconstruct reality.

`special` is for that moment.

It turns annotations in ordinary source files and markdown into an inspectable
contract and architecture view, so you can see:

- what is live
- what is planned
- what supports a claim
- which code implements a module

## Why It Exists

The pain is not "the model is dumb." The pain is that once the repo gets even a
little complicated, the agent stops feeling grounded in what is actually
implemented.

That is when AI-assisted development starts to slow down:

- the agent treats planned behavior like shipped behavior
- a test is tagged as support, but does not really prove the claim
- a module sounds clean in docs, but the implementation is broad, tangled, or
  leaking across boundaries
- reviewers have to grep through code to reconstruct what is true
- progress slows down because everyone is reloading context and second-guessing
  the contract

There are already good tools for agent memory, planning flows, and orchestration.
Those mostly help you run the work.

`special` solves a different problem: once the work is already happening, what
does the repo actually claim, what support is attached to those claims, and what
code really implements each module?

`special` exists to make that explicit before the repo turns into vibes,
reconstruction work, and cleanup debt.

It does that by:

- materializing live and planned specs from source-local annotations
- surfacing attached `@verifies` and `@attests`
- materializing architecture modules and areas
- attaching implementation ownership with `@implements`
- surfacing architecture analysis evidence with `special modules --metrics`
- installing repo-local skills that teach agents how to work with the contract

## Source Of Truth

The canonical product truth for `special` lives in its own self-hosted `special`
declarations, primarily colocated with the owning source and test boundaries.
Small central markdown residue remains only for structural and planned contract
scaffolding.

If this README and the materialized spec ever disagree, the spec wins.

If published to crates.io, the package name is `special-cli` and the installed
binary is `special`.

The repo root is explicitly anchored by [special.toml](special.toml).

## What `special` Does

Today `special` is a Rust CLI that:

- parses annotation blocks from supported source files and markdown headings
- builds one spec tree across files and file types
- builds one architecture module tree across source-local declarations and
  project architecture notes
- materializes the live spec by default
- materializes live modules by default
- includes planned claims and planned modules on request
- reports annotation and reference errors
- shows attached verification and attestation bodies in verbose views
- shows implementation ownership and architecture analysis evidence in module
  views
- installs task-shaped skills for product-spec and architecture workflows

This repo is self-hosting: `special`'s own behavior is described and verified in
`special` format across its source files, test files, and minimal central
markdown contract residue.

## Who It Is For

`special` is for people already using coding agents in real repos where the
early speed gains are starting to turn into breakage, review thrash, and
confusion:

- what is actually implemented?
- what is only planned?
- does this test really support this claim?
- does this code actually belong to the module that claims it?

This gets worse in brownfield codebases, but it is not limited to brownfield
work. If your main problem is "AI dev starts fast, then quickly bogs down
because you no longer fully trust what the agent is doing or what the repo
actually implements," `special` is aimed at that problem.

## Quick Start

Inspect the current live contract:

```sh
special specs
```

Inspect the architecture tree:

```sh
special modules
```

Inspect architecture ownership and implementation evidence:

```sh
special modules --metrics
```

Check structural problems:

```sh
special lint
```

Initialize a repo root:

```sh
special init
```

## Contract Views

The core command is `special specs` (`special spec` also works as an alias).

Useful variants:

```sh
special specs
special specs --all
special specs APP.CONFIG
special specs APP.CONFIG.FILE --verbose
special specs --unsupported
special specs --json
special specs --html
special specs --html --verbose
special specs APP.EXPORT --json --verbose
```

`special specs` gives you the current live contract by default. `--all` includes
planned items. `--unsupported` shows live items with zero verifies and zero
attests.

Verbose spec views surface the attached `@verifies` and `@attests` bodies so a
human or agent can inspect the support directly.

## Architecture Views

The core command is `special modules` (`special module` also works as an alias).

Useful variants:

```sh
special modules
special modules --all
special modules APP.PARSER --verbose
special modules --metrics
special modules --metrics --verbose
special modules --json
special modules --json --metrics
special modules --html --verbose
```

`special modules` materializes the declared architecture tree.

`special modules --metrics` adds architecture-as-implemented evidence, including:

- ownership coverage
- per-module implementation summaries
- public and internal item counts when a built-in analyzer can extract them
- dependency and coupling evidence when a built-in analyzer can resolve them
- quality evidence summaries when a built-in analyzer can extract them honestly

Today the built-in implementation analysis is strongest for owned Rust code.
For Rust modules, `--metrics` can surface:

- public and internal item counts
- function complexity summaries
- cognitive complexity summaries
- quality evidence such as public API parameter shape, stringly typed boundaries,
  and recoverability signals
- `use`-path dependency evidence
- module coupling evidence derived from owned dependency targets

Use `--verbose` when you want uncovered or weakly covered paths plus per-module
coverage detail.

## Skills

`special skills` explains and prints bundled skills:

```sh
special skills
special skills ship-product-change
special skills install
special skills install ship-product-change
```

`special skills install` writes task-shaped skills into `.agents/skills/` or
another selected destination for:

- shipping a product change without drifting the contract
- defining product specs
- validating whether a claim is honestly supported
- validating whether a concrete architecture module is honestly implemented
- inspecting the current live spec state
- finding planned work

The installed skill files are generated output and are typically ignored in the
repo.

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
mise exec -- cargo run -- modules --metrics
```

## Annotation Model

`special` currently uses these annotations:

- `@group ID`
  Structural container only. Groups organize subtrees and do not carry direct
  support.
- `@spec ID`
  Real claim node.
- `@planned`
  Marks a `@spec` as not part of the live spec yet, and may optionally carry a
  release string like `@planned X.Y.Z`.
- `@verifies ID`
  Attaches one verification artifact to one claim.
- `@attests ID`
  Attaches a manual or external attestation to one claim.
- `@fileattests ID`
  Attaches one file-scoped attestation artifact to one claim.
- `@module ID`
  Concrete architecture module.
- `@area ID`
  Structural architecture node.
- `@implements ID`
  Attaches implementation ownership for one owned item to a concrete
  architecture module.
- `@fileimplements ID`
  Attaches implementation ownership for the containing file to a concrete
  architecture module.
- `@fileverifies ID`
  Attaches one file-scoped verification artifact to one claim.

Important constraints:

- `@group` and `@spec` are mutually exclusive for the same id.
- `@planned` is local to the owning `@spec`.
- one `@verifies` block may target only one spec id.
- one `@fileverifies` block may target only one spec id.
- child claims do not justify a parent `@spec`.
- `@verifies` only counts when it attaches to a supported owned item.
- live `@module` nodes require direct `@implements` or `@fileimplements` unless
  they are planned.
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
@area APP
Top-level product area.
*/

/**
@module APP.PARSER
Parses reserved annotations from extracted comment blocks.
*/

// @fileimplements APP.PARSER
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
- markdown heading annotations

`special` supports spec and module trees spread across multiple files and mixed
supported file types.

## Release Automation

This repo carries its own release automation contract in `special` format.

Run the Rust code review separately when you want it:

```sh
python3 scripts/review-rust-release-style.py
```

Publish a release through the local wrapper so one process handles the release
checklist, main bookmark push, release tag push, GitHub release verification, and
Homebrew formula update:

```sh
python3 scripts/tag-release.py X.Y.Z
```

The wrapper will walk you through the easy-to-forget prerelease items before it
publishes:

- public docs like `README.md`
- `CHANGELOG.md`
- version bump and release references
- core validation (`cargo test`, `special lint`, `special spec --all`)

If you have already checked the prerelease list and want to bypass the
interactive prompts, use:

```sh
python3 scripts/tag-release.py X.Y.Z --skip-checklist
```

The current live distribution slice covers:

- crates.io package name and installed binary name
- GitHub repository metadata for release automation
- committed GitHub Actions release workflow
- published release archives and checksums for supported targets
- committed Homebrew formula in `LabLeaks/homebrew-tap`

Actual published GitHub Releases are a separate claim from release automation
itself.
