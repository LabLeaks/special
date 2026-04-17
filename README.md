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
- surfacing repo-wide quality and experimental traceability with `special repo`
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

Inspect repo-wide quality signals and experimental cross-cutting traceability:

```sh
special repo
special repo --experimental
```

Check structural problems:

```sh
special lint
```

Initialize a repo root:

```sh
special init
```

## How To Read The Commands

`special` has three main surfaces:

- `special specs`
  The product-contract view.
- `special modules`
  The annotated architecture view.
- `special repo`
  The cross-cutting repo-quality view.

Two flags then refine those surfaces:

- `--verbose`
  Show more of the same surface: attached bodies, implementation detail, or
  fuller drilldown.
- `--metrics`
  Add computed analysis where the default command is otherwise just a
  materialized tree. Today that mainly applies to `special modules`.

In practice:

- use `special specs --verbose` when you want to inspect whether a claim is
  honestly supported
- use `special modules --metrics --verbose` when you want to inspect whether an
  architecture boundary is honest in code
- use `special repo --verbose` when you want repo-wide cleanup or quality
  signals that do not belong to one module

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
attests. Deprecated claims remain visible in the live view and surface their
retirement metadata when present.

Verbose spec views surface the attached `@verifies` and `@attests` bodies so a
human or agent can inspect the support directly.

Example shape:

```text
$ special specs --unsupported --verbose

APP.EXPORT.CSV [unsupported]
  text: CSV exports include a header row with the selected column names.
  verifies: 0
  attests: 0
```

## Architecture Views

The core command is `special modules` (`special module` also works as an alias).

Useful variants:

```sh
special modules
special modules --all
special modules APP.PARSER --verbose
special modules --metrics
special modules --metrics --verbose
special repo
special repo --verbose
special repo --experimental
special modules --json
special modules --json --metrics
special modules --html --verbose
```

`special modules` materializes the declared architecture tree.

`special modules --metrics` adds architecture-as-implemented evidence, including:

- module ownership granularity
- per-module implementation summaries
- public and internal item counts when a built-in analyzer can extract them
- dependency and coupling evidence when a built-in analyzer can resolve them
- quality evidence summaries when a built-in analyzer can extract them honestly
- unreached-code indicators within owned implementation when a built-in analyzer
  can identify them honestly

`file-scoped implements` and `item-scoped implements` are ownership-granularity
signals, not file-coverage grades. A module can be honestly owned at file scope
while still being coarser than item-scoped ownership.

`special repo` surfaces repo-wide quality signals that are not tied to one
architecture module, including:

- repo-wide duplicate-logic signals across owned implementation
- unowned unreached-code indicators outside any declared module
- experimental implementation traceability when requested with `--experimental`

Use `special repo --verbose` when you want fuller repo-wide drilldown, including
unowned unreached item locations and complete duplicate-item clusters when the
built-in analyzers can identify them honestly.

Example shape:

```text
$ special modules APP.PARSER --metrics

APP.PARSER
  file-scoped implements: 1
  item-scoped implements: 3
  public items: 2
  internal items: 7
  module coupling: 2
  unreached items: 1
```

Today the built-in implementation analysis is strongest for owned Rust code,
and it also surfaces first-pass implementation evidence for owned TypeScript
and Go code.
For Rust modules, `--metrics` can surface:

- public and internal item counts
- function complexity summaries
- cognitive complexity summaries
- quality evidence such as public API parameter shape, stringly typed boundaries,
  and recoverability signals
- unreached-code indicators such as private items with no observed path from
  public or test roots inside owned implementation
- `use`-path dependency evidence
- module coupling evidence derived from owned dependency targets

For TypeScript modules, `--metrics` can surface:

- public and internal item counts
- import-path dependency evidence
- module coupling evidence derived from owned relative imports
- per-item connected, outbound-heavy, isolated, and unreached evidence

For Go modules, `--metrics` can surface:

- public and internal item counts
- import-path dependency evidence
- module coupling evidence derived from owned local imports
- per-item connected, outbound-heavy, isolated, and unreached evidence

`special repo --experimental` also surfaces early implementation traceability
indicators when a built-in analyzer can connect owned code through tests to
declared specs.

Example shape:

```text
$ special repo --verbose

special repo
repo-wide signals
duplicate items: 3
duplicate item: APP:parser/a.rs:collect_calls [function; duplicate peers 1]
duplicate item: APP:parser/b.rs:collect_calls [function; duplicate peers 1]
unowned unreached items: 0
```

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
mise exec -- cargo run -- specs --all
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
- `@deprecated`
  Marks a live `@spec` for retirement while it is still materialized, and may
  optionally carry a release string like `@deprecated X.Y.Z`.
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
- `@deprecated` is local to the owning `@spec`.
- a `@spec` may not be both `@planned` and `@deprecated`.
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

Deprecated claims use the same local marker shape:

```text
/**
@spec EXPORT.LEGACY_HEADERS
@deprecated 0.6.0
Legacy CSV header behavior is scheduled for removal.
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
- core validation (`cargo test`, `special lint`, `special specs --all`)

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
