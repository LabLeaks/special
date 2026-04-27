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

It turns lightweight annotations in normal source files and markdown into five
inspectable views:

- `special`
  A compact health overview that tells you where to look next.
- `special specs`
  What the repo claims across current, planned, deprecated, and unverified specs.
- `special arch`
  How the repo says implementation is organized.
- `special patterns`
  Adopted implementation patterns and the source surfaces that apply them.
- `special health`
  Cross-cutting code health and traceability that do not belong to one module.

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
- builds one adopted-pattern tree with source-attached applications
- materializes all declared specs by default
- materializes all declared modules by default
- materializes all declared patterns by default
- lets you filter to current or planned declarations on request
- reports annotation and reference errors
- shows attached verification and attestation bodies in verbose views
- shows implementation ownership and architecture analysis evidence in module
  views
- shows pattern definitions, applications, and advisory shape metrics
- installs task-shaped skills for product-spec and architecture workflows

This repo is self-hosting: `special` describes and verifies its own behavior
with `special` annotations across its source, tests, and a small amount of
markdown residue.

## Typical Use Cases

### 1. Catching spec mismatches before they ship

Suppose the repo still says this is current:

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

That is the kind of mismatch that causes real damage: the claim says “immediately
removes,” the test only proves “request accepted,” and an agent may happily
refactor around the stronger sentence because it looks supported.

Run:

```sh
special
special specs --unverified --verbose
special specs --metrics
special specs APP.DELETE.REMOTE --verbose
```

The second command catches current claims with no direct support at all. The fourth
shows the exact verify body attached to one claim so you can judge whether the
proof matches the sentence.

Example shape when support is missing:

```text
$ special specs --unverified --verbose

APP.DELETE.REMOTE [unverified]
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

- downgrade, narrow, or directly verify unverified current claims
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
special
special arch APP.PARSER --metrics --verbose
special health --metrics
```

You get the declared boundary plus the evidence inside it: ownership
granularity, item counts, coupling, complexity, and unreached items.

Example shape when Rust backward trace is available:

```text
$ special arch APP.PARSER --metrics

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

Neither copy is “wrong” in its own architecture view, so the duplication never gets
prioritized.

Run:

```sh
special health --metrics
special health --metrics --verbose
```

You get repo-wide signals that are not naturally owned by one module.

Example shape:

```text
$ special health --metrics --verbose

special health
repo-wide signals
duplicate items: 2
duplicate item: APP:billing/stripe.rs:normalize_customer_record [function; duplicate peers 1]
duplicate item: APP:billing/paypal.rs:normalize_customer_record [function; duplicate peers 1]
unowned items: 0
```

What you can do with that evidence:

- turn repeated logic into explicit cleanup candidates instead of vague smells
- spot unowned implementation that is hiding outside the architecture tree
- use traceability to ask whether code is actually connected to a
  spec path, even when no architecture-only view would have surfaced that question

If you want to ask that last question directly:

```sh
special health --verbose
```

Example shape:

```text
traceability
current spec item: src/delete.rs:delete_remote_file
unexplained item: src/cleanup.rs:legacy_cleanup_path
```

If a backward-trace route is unavailable, `special` says so plainly instead of
guessing from weaker analysis. Rust can fall back to parser-resolved call edges
when `rust-analyzer` is unavailable, and that degraded route is reported in the
health status.

### 4. Keeping repeated implementation approaches intentional

Some repeated code should become a helper or component. Some repeated code is
not a direct duplicate: it is the same adopted approach showing up in different
contexts.

`special patterns` is for the second case.

Suppose the repo has a cache-fill approach that intentionally looks similar
across several cache entries: check the cache, take a single-flight lock, rebuild
once, write the cache, and release waiters. That is not just “style.” It answers
the practical question a maintainer or agent will ask later:

Why is this done this way, and where else should I copy the approach instead of
inventing a new one?

Declare the pattern once:

```text
### `@pattern CACHE.SINGLE_FLIGHT_FILL`
@strictness high
Use single-flight cache fills when a shared cache entry may be requested by
multiple concurrent callers and rebuilding it more than once would waste work or
produce inconsistent progress reporting.
```

Then attach applications to the source item or file that actually uses the
approach:

```text
// @applies CACHE.SINGLE_FLIGHT_FILL
fn load_or_build_repo_analysis(...) -> ... {
    ...
}
```

Run:

```sh
special patterns
special patterns CACHE.SINGLE_FLIGHT_FILL --verbose
special patterns --metrics
special patterns --metrics --target src/cache.rs
```

The default view shows the known patterns and where they are applied. The verbose
view includes the definition and application bodies. The metrics view adds
advisory source-shape checks: possible missing applications, possible new pattern
clusters, and helper/component extraction candidates when implementations are so
similar that a named pattern may be the wrong abstraction.

What you can do with that evidence:

- preserve intentional approaches across future edits
- help agents reuse the project’s adopted implementation vocabulary
- notice when an application claims a pattern but does not resemble the other
  applications
- notice when repeated code should become a helper instead of a pattern

## Quick Start

Inspect the current contract:

```sh
special specs
```

Inspect the architecture tree:

```sh
special arch
```

Inspect architecture ownership and implementation evidence:

```sh
special arch --metrics
```

Inspect adopted implementation patterns:

```sh
special patterns
special patterns --metrics
```

Inspect repo-wide quality signals and cross-cutting traceability:

```sh
special health
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

`special` has five main surfaces:

- `special`
  The compact overview and “what should I inspect next?” surface.
- `special specs`
  The product-contract view.
- `special arch`
  The annotated architecture view.
- `special patterns`
  The adopted-pattern view.
- `special health`
  The cross-cutting code-health and traceability view.

Two flags then refine those surfaces:

- `--verbose`
  Show more detail from the current view: attached bodies, implementation
  detail, or fuller item-level evidence.
- `--metrics`
  Show deeper analysis for the current view.

In practice:

- use `special specs --verbose` when you want to inspect whether a claim is
  honestly supported
- use `special specs --metrics` when you want a deeper contract-health
  breakdown for the declared spec view
- use `special arch --metrics` when you want the architecture-wide grouped
  summary first
- use `special arch MODULE.ID --metrics` when you want deeper evidence for one
  boundary without dumping the whole repo
- use `special arch --metrics --verbose` when you really want the full
  architecture-wide drilldown
- use `special patterns --verbose` when you want pattern definitions and concrete
  source applications together
- use `special patterns --metrics` when you want advisory pattern similarity,
  missing-application, and candidate-cluster analysis
- use `special patterns --metrics --target PATH` when you want pattern advice for
  touched files
- use `special health --metrics` when you want deeper repo-wide cleanup,
  traceability, and grouped-count analysis
- use `special health --target PATH` when you want the same health view narrowed
  to touched files
- use `special health --within PATH` only when you intentionally want to limit the
  analysis corpus before building the health view
- use `special health --verbose` when you want more item-level detail within the
  current health view

## Command Surface

Today the built-in implementation analysis surfaces implementation evidence for
owned Rust, TypeScript, and Go code. For each of those modules, `--metrics` can
surface:

- public and internal item counts
- function complexity summaries
- cognitive complexity summaries
- quality evidence such as public API parameter shape, stringly typed boundaries,
  and recoverability signals
- unreached-code indicators such as private items with no observed path from
  public or test roots inside owned implementation
- language-native dependency evidence
- module coupling evidence derived from owned dependency targets
- per-item connected, outbound-heavy, isolated, unreached, high-complexity,
  parameter-heavy, stringly boundary, and panic-heavy evidence

`special health` also surfaces implementation traceability indicators when a
built-in analyzer can connect repo code through tests to declared specs,
including code outside any declared module.

Useful command shapes:

```sh
special specs
special specs --current
special specs --planned
special specs APP.CONFIG --verbose
special specs --unverified
special specs --metrics
special specs --json

special arch
special arch --current
special arch --planned
special arch APP.PARSER --verbose
special arch --metrics
special arch APP.PARSER --metrics
special arch --metrics --verbose
special arch --json --metrics

special patterns
special patterns APP.CACHE_FILL
special patterns APP.CACHE_FILL --verbose
special patterns --metrics
special patterns --metrics --target src/foo.rs
special patterns --metrics --target src/foo.rs --symbol parseConfig
special patterns --metrics --within crates/app
special patterns --json

special health
special health --target src/foo.rs
special health --target src/foo.rs --symbol parseConfig
special health --within crates/app
special health --metrics
special health --metrics --verbose
special health --verbose
special health --json
```

Example shape:

```text
$ special health --metrics --verbose

special health
special health metrics
duplicate items: 3
duplicate item: APP:parser/a.rs:collect_calls [function; duplicate peers 1]
duplicate item: APP:parser/b.rs:collect_calls [function; duplicate peers 1]
unowned items: 0
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

- shipping a product change without changing the contract by accident
- defining product specs
- validating whether a claim is honestly supported
- validating whether a concrete architecture module is honestly implemented
- inspecting the current spec state
- finding planned work
- following and reviewing adopted implementation patterns

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
mise exec -- cargo run -- specs
mise exec -- cargo run -- patterns
mise exec -- cargo run -- health
mise exec -- cargo run -- arch --metrics
```

The repo is a small polyglot monorepo: the Rust CLI crate lives at the root with
Cargo's conventional `src/`, `tests/`, and Askama `templates/` layout, while the
Lean scoped traceability kernel lives as a separate Lake project under `lean/`.
`build.rs` embeds the compiled Lean kernel for released host-native binaries.

## Annotation Model

`special` currently uses these annotations:

- `@group ID`
  Structural container only. Groups organize subtrees and do not carry direct
  support.
- `@spec ID`
  Real claim node.
- `@planned`
  Marks a `@spec` as not part of the current spec yet, and may optionally carry a
  release string like `@planned X.Y.Z`.
- `@deprecated`
  Marks a current `@spec` for retirement while it is still materialized, and may
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
- `@pattern ID`
  Declares one adopted implementation pattern. Pattern ids may nest with dot
  notation.
- `@strictness high|medium|low`
  Optional pattern metadata that tells advisory metrics how closely applications
  are expected to resemble each other. Omitted strictness defaults to `medium`.
- `@applies ID`
  Attaches one supported source item as a concrete application of a pattern.
- `@fileapplies ID`
  Attaches the containing source file as a concrete application of a pattern.

Important constraints:

- `@group` and `@spec` are mutually exclusive for the same id.
- `@planned` is local to the owning `@spec`.
- `@deprecated` is local to the owning `@spec`.
- a `@spec` may not be both `@planned` and `@deprecated`.
- one `@verifies` block may target only one spec id.
- one `@fileverifies` block may target only one spec id.
- child claims do not justify a parent `@spec`.
- `@verifies` only counts when it attaches to a supported owned item.
- current `@module` nodes require direct `@implements` or `@fileimplements` unless
  they are planned.
- `@area` is structural only and does not accept `@planned` or `@implements`.
- each `@pattern ID` may have only one definition.
- `@applies` and `@fileapplies` must attach to source code, not markdown
  declarations.
- pattern metrics are advisory and do not create lint failures.

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

Pattern declarations name an adopted approach:

```text
### `@pattern CACHE.SINGLE_FLIGHT_FILL`
@strictness high
Use this when concurrent callers may request the same expensive cache fill and
only one caller should rebuild while the rest wait for the shared result.
```

Pattern applications attach to source:

```text
// @applies CACHE.SINGLE_FLIGHT_FILL
fn load_or_build_repo_analysis(...) -> ... {
    ...
}
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

Current self-hosted current support covers:

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

Publish a release through the local wrapper as a three-step pipeline. The first
step makes you enter the exact release-visible changelog bullets and writes the
versioned `CHANGELOG.md` section:

```sh
python3 scripts/tag-release.py X.Y.Z --prepare
```

Then run the validation phase. It executes the release validation commands and
records ignored local evidence for the current release revision:

```sh
python3 scripts/tag-release.py X.Y.Z --validate
```

Publish only after the prepared changelog and validation evidence are attached
to the release revision:

```sh
python3 scripts/tag-release.py X.Y.Z --publish
```

The wrapper refuses missing or placeholder changelog notes, tracked private or
generated files, mismatched manifest versions, missing validation evidence, and
legacy checklist bypass flags.

The current distribution slice covers:

- crates.io package name and installed binary name
- GitHub repository metadata for release automation
- committed GitHub Actions release workflow
- published release archives and checksums for supported targets
- committed Homebrew formula in `LabLeaks/homebrew-tap`

Actual published GitHub Releases are a separate claim from release automation
itself.
