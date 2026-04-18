# special

Pronounced "spec-ee-al".

A CLI for repos that have started to outrun trust.

When agent-driven development starts going wrong, the failure mode usually is
not “the model is dumb.” The repo itself has become hard to read:

- behavior is half shipped and half planned
- tests are tagged as proof, but do not really prove the claim
- module boundaries sound clean in docs, but the implementation sprawls
- reviewers have to grep through the tree to reconstruct what is actually true

`special` makes that visible again.

It turns lightweight annotations in normal source files and markdown into three
inspectable views:

- `special specs`
  What the repo claims is live, planned, deprecated, or unsupported.
- `special modules`
  How the repo says implementation is organized.
- `special repo`
  Cross-cutting quality signals that do not belong to one module.

## Why It Exists

Most agent tooling helps you run work: planning, orchestration, memory,
autonomy, handoffs.

`special` helps you answer a different question:

What does this repo actually claim, what evidence is attached to those claims,
and what code really implements the architecture it describes?

That matters once the codebase is large enough that:

- planned work starts getting mistaken for shipped behavior
- tagged tests stop being trusted automatically
- architecture docs stop matching the implementation
- quality hotspots exist across the repo, not just inside one module

`special` is meant to be the thing you run before another hour of grep,
guessing, or cleanup debt.

## Source Of Truth

The canonical product truth for `special` lives in its own self-hosted `special`
declarations, primarily colocated with the owning source and test boundaries.
Small central markdown residue remains only for structural and planned contract
scaffolding.

If this README and the materialized spec ever disagree, the spec wins.

If published to crates.io, the package name is `special-cli` and the installed
binary is `special`.

The repo root is explicitly anchored by [special.toml](special.toml).

## What It Gives You

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

This repo is self-hosting: `special` describes and verifies its own behavior
with `special` annotations across its source, tests, and a small amount of
markdown residue.

## Typical Use Cases

### 1. Catching spec drift before it ships

Suppose the repo still says this is live:

```text
/**
@spec APP.DELETE.REMOTE
Delete immediately removes the remote file from storage.
*/
```

But the only nearby test is really checking something weaker:

```text
/**
@verifies APP.DELETE.REMOTE
*/
#[test]
fn delete_returns_202() {
    assert_eq!(delete("/files/123").status(), 202);
}
```

That is the kind of drift that causes real damage: the claim says “immediately
removes,” the test only proves “request accepted,” and an agent may happily
refactor around the stronger sentence because it looks supported.

Run:

```sh
special specs --unsupported --verbose
special specs APP.DELETE.REMOTE --verbose
```

The first command catches live claims with no direct support at all. The second
shows the exact verify body attached to one claim so you can judge whether the
proof matches the sentence.

Example shape when support is missing:

```text
$ special specs --unsupported --verbose

APP.DELETE.REMOTE [unsupported]
  text: Delete immediately removes the remote file from storage.
  verifies: 0
  attests: 0
```

Example shape when support exists but is too weak:

```text
$ special specs APP.DELETE.REMOTE --verbose

APP.DELETE.REMOTE
  text: Delete immediately removes the remote file from storage.
  verify body:
    #[test]
    fn delete_returns_202() {
        assert_eq!(delete("/files/123").status(), 202);
    }
```

What you can do with that evidence:

- downgrade, narrow, or directly verify unsupported live claims
- replace weak “request accepted” or helper-only verifies with real
  command/API-boundary proof
- stop your agents and reviewers from treating “tagged somewhere” as the same
  thing as “proved”

### 2. Driving an architecture refactor from evidence instead of vibes

Suppose the architecture still claims this:

```text
/**
@module APP.PARSER
Parses user query text into a structured search request.
*/
// @fileimplements APP.PARSER
```

But the file has slowly accumulated parsing, validation, normalization,
projection, and a little bit of logging glue. Everyone knows it feels wrong,
but “the parser is too big” is still too vague to refactor cleanly.

Run:

```sh
special modules APP.PARSER --metrics --verbose
```

You get the declared boundary plus the evidence inside it: ownership
granularity, item counts, coupling, complexity, and unreached items.

Example shape:

```text
$ special modules APP.PARSER --metrics

APP.PARSER
  file-scoped implements: 1
  item-scoped implements: 0
  public items: 2
  internal items: 18
  module coupling: 6
  unreached items: 5
```

That is not “a parser module with one rough edge.” That is a broad file-owned
bucket with multiple concerns hiding inside it.

What you can do with that evidence:

- aim refactors at the actually overloaded boundary instead of the one people
  complain about abstractly
- tighten broad `@fileimplements` ownership into item-scoped ownership
- split a suspected “parser” bucket into smaller parse, syntax, validation, or
  projection layers based on visible evidence instead of instinct

### 3. Finding repo-wide cleanup that architecture views miss

Suppose the module tree looks fine, but the repo still has repeated code across
multiple integration points:

```text
fn normalize_customer_record(...) -> ...
fn normalize_customer_record(...) -> ...
```

Neither copy is “wrong” in its own module view, so the duplication never gets
prioritized.

Run:

```sh
special repo --verbose
```

You get repo-wide signals that are not naturally owned by one module.

Example shape:

```text
$ special repo --verbose

special repo
repo-wide signals
duplicate items: 2
duplicate item: APP:billing/stripe.rs:normalize_customer_record [function; duplicate peers 1]
duplicate item: APP:billing/paypal.rs:normalize_customer_record [function; duplicate peers 1]
unowned unreached items: 0
```

What you can do with that evidence:

- turn repeated logic into explicit cleanup candidates instead of vague smells
- spot unowned implementation that is hiding outside the architecture tree
- use experimental traceability to ask whether code is actually connected to a
  spec path, even when no module view would have surfaced that question

If you want to ask that last question directly:

```sh
special repo --experimental --verbose
```

Example shape:

```text
experimental traceability
live spec item: APP:delete_remote_file
unknown item: APP:legacy_cleanup_path
```

That does not prove “dead code” or “safe to delete.” It tells you which code
the current analyzers can and cannot connect to a spec path.

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

## Command Surface

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

Useful command shapes:

```sh
special specs
special specs --all
special specs APP.CONFIG --verbose
special specs --unsupported
special specs --json

special modules
special modules --all
special modules APP.PARSER --verbose
special modules --metrics
special modules --metrics --verbose
special modules --json --metrics

special repo
special repo --verbose
special repo --experimental
special repo --json
```

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
mise exec -- cargo run -- repo
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
