# Architecture

`special modules` is the canonical architecture view for this repo.

This file records project-specific architectural rationale that does not belong to one concrete
source file. It should explain why this repo is shaped the way it is, not restate the module tree
or generic `special` annotation conventions.

## Broad System Shape

This repo is intentionally split into a few broad responsibilities:

- extraction and parsing
  Comment extraction is separate from annotation interpretation.
- domain and materialization
  Parsed records become typed domain nodes before text, JSON, or HTML rendering.
- architecture analysis
  `special modules --metrics` should keep a shared analysis core and hang
  language-specific analyzers beneath it rather than letting one parser model
  define the whole feature.
- boundary commands
  CLI modules should stay thin and orchestrate work rather than own deep policy.
- release tooling
  Release review and tagging are local tooling concerns, kept separate from product runtime code.

## Why The Architecture Works This Way

- This repo is intentionally using `special` on itself, so keeping architecture intent close to
  code reduces drift during refactors.
- The split between specs and modules is deliberate because this project needs to audit two
  different questions honestly:
  - does the product behavior match its claimed contract?
  - does the code actually implement the architecture it claims to own?
- The parser, module materializer, and release-review tooling are all design-heavy enough that
  local architecture declarations are useful during ongoing redesign, not just as static docs.
- The repo still keeps a thin central architecture file because a few areas are genuinely
  cross-cutting and do not have one honest single-file owner.

## Shared Structural Areas

These areas remain here because they are genuinely cross-cutting and do not have one obvious
source-local owner.

### `@area SPECIAL.DISTRIBUTION`
Release and distribution verification surface across scripts, workflow files, and distribution
tests.

### `@area SPECIAL.TESTS`
Executable regression and contract-proof surface under `tests/`.

### `@area SPECIAL.TESTS.SUPPORT`
Shared test-support surface under `tests/support/`.
