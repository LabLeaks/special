# Patterns

This file holds source-native pattern definitions for Special itself. A pattern should answer
"wait, why is it done like this?" for a recurring implementation problem, then give future humans
and agents a shortcut for recognizing the same problem shape again.

Patterns are not broad principles, style rules, specs, or one module's design notes. A useful
pattern names a reusable problem shape, explains the chosen solution shape, and gives enough
criteria to decide whether a new surface should follow it or deliberately do something else.

### `@pattern ADAPTER.FACTS_TO_MODEL`
@strictness high

Use this pattern when Special needs source-, language-, tool-, or ecosystem-specific facts but the
rest of the system should reason over one stable core model. The point is not to hide a call behind
a wrapper; it is to stop local fact-gathering weirdness from leaking into shared architecture,
health, traceability, rendering, or command orchestration.

Reach for this when a provider owns facts that are real but locally shaped: parser output,
language-server output, project-tool probes, framework conventions, generated sidecar facts, or
fixtures that only make sense inside that provider. The provider should normalize those facts at
the boundary and return shared Special model types. Downstream code should depend on the core model
and provider protocol, not on the provider's local tools, syntax, or fixture layout.

Expected shape:

- provider-local extraction, probing, and normalization
- explicit conversion into shared model or analysis structs
- shared orchestration dispatches through a stable provider boundary
- downstream code consumes the shared model without branching on provider internals

Do not use this when there is only one implementation and no real boundary, when the wrapper only
renames a helper, or when the shared model is being distorted to force unrelated providers through
a fake common shape.

### `@pattern ADAPTER.FACTS_TO_MODEL.TRACEABILITY_ITEMS`
@strictness high

Use this pattern when a provider projects parsed source items plus ownership facts into Special's
shared traceability item model. The reason for the shape is traceability portability: the shared
traceability core should reason over stable item ids, paths, module ownership, review-surface
status, context inclusion, and provider-specific mediation notes without knowing how each language
parser represents source code.

Reach for this when provider-local source graphs need to become `TraceabilityOwnedItem` values for
traceability analysis. The projection should preserve stable source identity, attach module
ownership from shared ownership maps, make its inclusion policy explicit, mark test/review-surface
state at the provider boundary, and sort output deterministically before handing it to shared graph
logic.

Expected shape:

- iterate provider-local parsed source graphs by path
- derive module ownership and test-file status at the path boundary
- map source items into shared traceability item records using the caller's inclusion policy
- attach provider-specific review-surface or mediation facts without leaking provider internals
- sort the shared items deterministically before returning them

Do not use this for parser-only helpers, owned-body item collection, or call graph indexing. Those
may share nearby facts, but this pattern is specifically the projection into the shared traceability
item model.

### `@pattern TRACEABILITY.SCOPED_PROJECTED_KERNEL`
@strictness low

Use this pattern when a scoped traceability implementation needs fast target-focused analysis without
changing the meaning of the full repo traceability graph. The reason for the shape is proof honesty:
the language pack may collect facts through a broader operational working set, but the semantic
boundary must reduce to a shared item-level projected contract and a Lean-derived reverse-closure
reference before it trims the output.

Reach for this when a language pack supports `special health --target` with scoped graph discovery,
semantic caller/reference discovery, or file-closure optimization. The pack should keep the broad
working contract separate from the exact projected contract, derive the exact reference through the
shared projected traceability kernel, and only then map that item-level result back to whatever
file-, package-, or tool-specific execution closure it needs.

Expected shape:

- identify projected item ids from the user's scoped files
- keep a broader provider working set only for collecting raw semantic facts
- derive the exact projected item contract/reference through the shared traceability kernel
- retain projected items plus the exact reverse closure needed for their support roots
- project the retained item set back to provider-specific files or facts for execution

Do not use this for full-repo traceability, parser-only graph construction, or a scoped filter that
simply drops output rows after full analysis. If a scoped route cannot preserve full-then-filtered
traceability semantics, it should be treated as a degraded or experimental route instead.

### `@pattern TEST_FIXTURE.REPRESENTATIVE_PROJECT`
@strictness low

Use this pattern when Special needs to prove behavior against a realistic throwaway project rather
than a narrow function input. The reason for the project fixture is product honesty: some behavior
depends on file discovery, source layout, project configuration, architecture declarations,
implementation ownership, language/tool files, and runnable or traceable entrypoints that do not
exist in an isolated source string.

Reach for this when the behavior under test crosses a real project boundary: root detection,
config loading, source-root selection, module ownership, spec verification, parser facts, framework
callbacks, test-file classification, dependency analysis, command execution, or cross-file
traceability. Keep the project compact, but preserve the files and relationships that make the
behavior real.

Expected shape:

- create a temp project directory tree with the relevant source, test, config, and tool files
- write `special.toml` plus architecture declarations when Special project behavior is involved
- add spec declarations and verifying tests when the behavior is contract-facing
- include source files that exercise the feature through real language syntax and file placement
- include runnable entrypoints, callbacks, fixtures, or nearby orphan/dead code only when they
  affect the behavior under test
- keep assertions focused on discovered facts and product behavior rather than incidental fixture
  prose

Do not use this for pure parser snippets, standalone helper tests, or repo-independent formatting
checks. If many fixtures share exactly the same boilerplate, extract a helper; the pattern is for
preserving realistic project shape, not duplicating identical setup.

### `@pattern COMMAND.PROJECTION_PIPELINE`
@strictness medium

Use this pattern when a Special command materializes a read-only product view from source
annotations, analysis, cache state, or lint state. The reason for the shape is command honesty:
users should get the same lifecycle signals, root/config warnings, cache behavior, diagnostics,
rendering choice, and exit-status rules even though each command owns a different projection.

Reach for this for commands that resolve the active project, build a typed document or report, and
render it as a terminal-facing view. The command boundary should stay thin: parse CLI flags, choose
the projection filter/options, run the model builder under status/cache instrumentation, render the
requested format, and decide success from the projection's diagnostics or summary state. Product
logic belongs behind the builder, not in the CLI handler.

Expected shape:

- create a command status plan and reset cache statistics
- resolve the project root and surface config/root warnings before building the view
- translate CLI flags into a typed filter or options object
- build the document/report under cache status instrumentation, with extra analysis status only
  when the command really performs deeper source analysis
- report cache activity, render diagnostics when the projection carries them, and render one output
  format from the typed document
- finish the status lifecycle and return success or failure from explicit diagnostics or summary
  state

Do not use this for mutating workflow commands, install commands, release scripts, or one-off helper
entrypoints. Those may share status helpers, but they do not promise the same read-only projection
contract.

### `@pattern SINGLE_FLIGHT.CACHE_FILL`
@strictness high

Use this pattern when Special fills a durable cache entry for expensive parse, analysis, or
tool-backed fact work and concurrent commands may ask for the same cache key. The reason for the
shape is honesty under concurrency: one caller should do the expensive fill, later callers should
reuse the result only when the fingerprint says the work is equivalent, and abandoned fills should
not leave the cache permanently blocked.

Expected shape:

- derive a stable fingerprint for the requested work
- read the cache before taking the fill lock
- acquire a per-entry fill lock
- reread the cache after winning the lock
- compute only on a real miss
- publish atomically and record hit/miss evidence where applicable

This pattern is appropriate when equivalent callers can safely share the first completed result. It
is not appropriate for ordinary exclusive mutation, work whose callers need distinct side effects,
or cases where a stale shared result would be less honest than recomputation.

### `@pattern REGISTRY.PROVIDER_DESCRIPTOR`
@strictness low

Use this pattern when Special has a known set of built-in providers and multiple shared subsystems
need to discover provider capabilities without growing their own provider-specific match arms. The
reason for the registry is controlled extension: adding a provider should update one descriptor
entry and provider-local implementation, while syntax, analysis, traceability, and command surfaces
continue selecting capabilities through the same descriptor boundary.

Expected shape:

- each provider publishes a descriptor containing identity, matching rules, and supported hooks
- shared subsystems select descriptors by path, language, capability, or discovered source set
- provider-specific work remains behind descriptor functions or provider context objects
- adding a provider should be mostly local to the provider and registry entry

This pattern is appropriate for built-in provider families where the project owns the set and wants
stable discovery semantics. It is not appropriate when call sites need one direct dependency, when
registration hides important ordering or ownership, or when the descriptor shape flattens providers
that do not actually share a capability model.
