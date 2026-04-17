# Traceability IR Direction

## Goal

Define a shared, language-agnostic traceability layer for `special` that:

- starts from source-native ownership, tests, and spec attachments
- lowers parser output into a normalized semantic graph
- keeps exact source backreferences for every lowered unit and edge
- supports Rust now without baking Rust assumptions into the product boundary
- can later plug into the proofs spike architecture instead of diverging from it

This is private architecture planning, not public product contract.

## Why this exists

The current `0.5.0` traceability work has proven a few things:

- item-level impl -> test -> spec tracing is useful
- raw file-local parse graphs are not enough
- cross-file/module resolution, subprocess boundaries, and framework-driven edges
  all matter
- the current Rust resolver is already doing IR-like work, but without an
  explicit shared internal model

The proofs spike is converging on a similar shape:

- source-native authoring
- lowered internal semantic units
- exact source trace for each lowered unit
- language-specific frontends feeding a shared core

So `special` should make that shared layer explicit now instead of letting the
Rust resolver become the de facto long-term architecture.

## Alignment target with the proofs spike

This design should align in shape, not necessarily in exact type names.

Desired correspondence:

- `special` `SourceUnit`
  ~= proofs spike source unit
- `special` `LoweredUnit`
  ~= proofs spike lowered local semantic unit
- `special` source trace map
  ~= proofs spike lowered-unit source attribution
- `special` evidence/disposition kinds
  ~= proofs spike coverage/disposition accounting

The main difference:

- `special` uses the IR for implementation traceability and architecture metrics
- the proofs spike uses a sibling IR for proof bundle resolution and
  formalization

If both sides preserve the same broad shape, they can later share concepts,
serialization, or adapter contracts even if they do not converge byte-for-byte.

## Design principles

### 1. Source remains the user-facing truth

The product still reports:

- owned implementation items
- tests and verification artifacts
- spec ids and lifecycle state
- exact source paths and spans

The IR is an internal lowering layer, not a replacement for source-level review.

### 2. Tree-sitter is a source parser, not the product boundary

`tree-sitter` stays useful for:

- file-local parsing
- spans
- item/test discovery
- raw callsite extraction

But raw syntax nodes are not the long-term boundary. Language modules must lower
them into normalized units and edges.

### 3. The core graph is language-agnostic

The core should understand:

- units
- source traces
- edges
- evidence kinds
- reporting dispositions

It should not understand:

- Rust `super::`
- TypeScript `import type`
- Go package selectors
- framework-specific conventions

Those belong in adapters.

### 4. Framework behavior is modular

Framework-driven edges should be added by framework adapters, not hidden inside
core resolution or language parsers.

Examples:

- Askama template helper reachability
- web route registration
- test harness registration
- serializer callback entrypoints

### 5. Unknown must stay honest

`unknown` means:

- no honest path was found in the current lowered graph

It must not mean:

- definitely orphaned
- definitely underspecified

That distinction depends on evidence kinds and the limits of the active
adapters.

## Core entities

## 1. SourceUnit

Canonical source-authored unit before semantic lowering.

Suggested shape:

```rust
struct SourceUnitId(String);

enum SourceUnitKind {
    OwnedItem,
    TestItem,
    VerificationAttachment,
    FileAttachment,
    MarkdownBlock,
}

struct SourceUnit {
    id: SourceUnitId,
    language: SourceLanguage,
    kind: SourceUnitKind,
    path: PathBuf,
    span: SourceSpan,
    display_name: String,
    stable_symbol: Option<String>,
}
```

Notes:

- `SourceUnit` is the thing humans can inspect directly.
- The current `SourceItem` in `src/syntax.rs` is a partial Rust-specific
  precursor to this.

## 2. LoweredUnit

Normalized semantic unit used for graph construction and traceability.

Suggested shape:

```rust
struct LoweredUnitId(String);

enum LoweredUnitKind {
    ImplBody,
    TestBody,
    VerificationArtifact,
    EntrySurface,
    Helper,
    FrameworkHook,
}

struct LoweredUnit {
    id: LoweredUnitId,
    kind: LoweredUnitKind,
    language: SourceLanguage,
    stable_name: String,
    source_units: Vec<SourceUnitId>,
}
```

Notes:

- A lowered unit may map to one or more source units.
- This is the level where language and framework adapters can introduce
  synthetic-but-honest semantic nodes such as entry surfaces or framework hooks.

## 3. TraceEdge

Typed relation between lowered units.

Suggested shape:

```rust
struct TraceEdgeId(String);

enum TraceEdgeKind {
    DirectCall,
    QualifiedCall,
    ImportResolvedCall,
    RelativeModuleCall,
    SpawnLocalProgram,
    EntryDispatch,
    VerificationAttach,
    FileVerificationAttach,
    FrameworkGenerated,
}

enum EvidenceKind {
    ExplicitAnnotation,
    SyntaxDerived,
    ResolverDerived,
    FrameworkAdapter,
    BoundaryBridge,
    RuntimeObserved,
}

struct TraceEdge {
    id: TraceEdgeId,
    from: LoweredUnitId,
    to: LoweredUnitId,
    kind: TraceEdgeKind,
    evidence: EvidenceKind,
    source_units: Vec<SourceUnitId>,
}
```

Notes:

- `EvidenceKind` matters as much as the edge itself.
- It lets reports distinguish strong explicit attachment from weaker
  framework/runtime bridges without turning everything into `unknown`.

## 4. TraceDisposition

User-facing classification computed from the graph.

Suggested shape:

```rust
enum TraceDisposition {
    LiveSpec,
    PlannedOnly,
    DeprecatedOnly,
    FileScopedOnly,
    TestOnly,
    Unknown,
}
```

Notes:

- This replaces any temptation to classify directly during language-specific
  traversal.
- Language adapters emit graph facts; the core computes disposition.

## Adapter model

## 1. Language adapter

Each language module should implement a narrow lowering contract.

Responsibilities:

- parse supported source files
- emit `SourceUnit`s
- emit initial `LoweredUnit`s
- emit direct/obvious semantic edges
- resolve language-local module/import/call relationships conservatively

Non-responsibilities:

- spec lifecycle logic
- module metrics rendering
- framework-specific synthetic edges
- final disposition classification

Suggested conceptual interface:

```rust
trait TraceLanguageAdapter {
    fn language(&self) -> SourceLanguage;

    fn collect_source_units(&self, path: &Path, text: &str) -> Vec<SourceUnit>;

    fn lower_units(
        &self,
        path: &Path,
        text: &str,
        source_units: &[SourceUnit],
    ) -> LoweringResult;
}
```

Where `LoweringResult` contains:

- lowered units
- edges
- unresolved references diagnostics

## 2. Framework adapter

Framework adapters run after language lowering.

Responsibilities:

- inspect lowered units plus source artifacts
- add synthetic semantic edges where the framework establishes real reachability
- mark those edges with explicit `FrameworkAdapter` evidence

Examples:

- Askama template file references template helper methods
- test harness registration conventions
- route macros mapping handlers to entry surfaces

Suggested conceptual interface:

```rust
trait TraceFrameworkAdapter {
    fn name(&self) -> &'static str;

    fn augment(&self, graph: &mut TraceGraph, repo: &ParsedRepo);
}
```

## 3. Projection/reporting core

The core reporting layer should consume only the normalized graph plus repo
ownership/spec metadata.

Responsibilities:

- impl -> test -> spec joins
- file-scoped vs item-scoped verify classification
- lifecycle-aware buckets
- JSON/text/HTML metrics output

It should not know how Rust `super::foo()` works or how Askama names template
helpers.

## Proposed pipeline

### Phase A. Source discovery

Input:

- owned source files
- verification attachments from parsed repo
- architecture ownership

Output:

- candidate source texts grouped by language

### Phase B. Source unit collection

Language adapters emit source-native units:

- owned impl items
- test items
- attachable verification blocks

### Phase C. Lowering

Language adapters lower source units into:

- impl/test/helper/entry lowered units
- direct and conservative resolved edges

### Phase D. Boundary augmentation

Framework and boundary adapters add edges for:

- spawned local programs
- entrypoint dispatch
- template-generated helper reachability
- other explicit runtime/framework conventions

### Phase E. Traceability closure

The shared core computes:

- reachability
- edge provenance
- evidence mix
- final impl-item dispositions

### Phase F. Product projections

Project the graph into:

- `repo --experimental`
- later `specs --metrics` or other spec-ended traceability views
- later dedicated traceability views
- future proof/export bridges

## Migration from the current code

## Current reality

Today the code is split roughly like this:

- `src/syntax.rs`
  - Rust-only source item and call extraction
- `src/modules/analyze/rust/traceability.rs`
  - item ownership collection
  - Rust resolution
  - subprocess bridge
  - disposition computation

This means the Rust resolver is still mixing:

- source parsing
- semantic lowering
- boundary bridging
- final product classification

## Desired refactor path

### Step 1. Introduce explicit `TraceGraph` types

Add a small internal module, likely something like:

- `src/trace_ir.rs`

It should hold:

- `SourceUnit`
- `LoweredUnit`
- `TraceEdge`
- `TraceGraph`
- `EvidenceKind`
- `TraceDisposition`

No product behavior change required yet.

### Step 2. Move Rust lowering behind the graph

Refactor the current Rust traceability pass to:

- emit source units and lowered units
- emit edges
- stop computing report buckets inline during resolution

The current Rust logic becomes a first `TraceLanguageAdapter`.

### Step 3. Extract boundary bridges

Split subprocess/local-binary logic into an adapter layer instead of embedding it
in the Rust resolver.

That gives us a reusable pattern:

- source-native call edges
- boundary invocation edges
- entry dispatch edges

### Step 4. Add framework adapters

Start only where repo metrics stay visibly distorted.

Probable first case:

- Askama in `SPECIAL.RENDER.*`

### Step 5. Add a stable graph-to-analysis projection

Once the graph is stable enough, the repo-level experimental traceability
surface should consume only the projection layer, not Rust-specific traversal
results.

### Step 6. Add TS and Go frontends

The provider contract becomes:

- collect source units
- lower semantic units
- resolve conservative edges
- report unresolved diagnostics

Not:

- "reimplement the entire module metrics product in each language"

## How this plugs into the proofs spike later

The most important compatibility move is to preserve source attribution at the
same abstraction level the spike wants.

Potential integration path:

- `special` trace graph provides source units and lowered local semantic units
- proofs spike bundle resolution can consume or reuse those lowered units for
  one claim closure
- proofs lowering can attach additional proof-specific structure on top

This does not require one shared crate or exact shared schema immediately.

It does require:

- compatible concepts
- exact source trace on lowered outputs
- explicit boundaries instead of hidden resolution magic

## What not to do

### 1. Do not make raw tree-sitter nodes the stable core contract

They are a parser detail, not a reusable analysis IR.

### 2. Do not embed full compiler frontends as the default architecture

Compiler-backed lowering may still be useful later, especially for proofs, but
the cross-language trace core should not require `rust-analyzer`-class
dependencies to exist at all.

### 3. Do not hide framework magic in the core resolver

If a framework edge exists, it should show up as a framework-origin edge with an
explicit evidence kind.

### 4. Do not let `unknown` overclaim

`unknown` must remain "no honest lowered path found by active adapters," not
"definitely unrelated to any live spec."

## Immediate next implementation moves

1. Add `src/trace_ir.rs` with the core normalized entities.
2. Refactor the current Rust resolver to emit a `TraceGraph`.
3. Move local-binary subprocess bridging into a boundary adapter.
4. Keep metrics output behavior stable while swapping the internal substrate.
5. Add the first framework adapter only after the graph substrate is real.

## Success condition

This direction is successful if:

- Rust traceability no longer defines the product boundary
- TS/Go can target the same IR without inheriting Rust internals
- framework support is additive and modular
- source trace is preserved well enough to keep metrics explainable
- the proofs spike can later recognize the same broad source-unit -> lowered-unit
  -> source-trace architecture instead of facing a parallel incompatible model
