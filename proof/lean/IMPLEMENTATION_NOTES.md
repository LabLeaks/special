# Scoped Health Equivalence Notes

This note ties the sidecar Lean model to the current Rust implementation risk.

## Contract We Need

The product contract from [tests/cli_repo.rs](/Users/gk/work/lableaks/projects/special/tests/cli_repo.rs:30)
is:

- `special health PATH` scopes repo-wide quality and traceability reporting to
  analyzable items in matching files or directories
- without changing the underlying repo analysis model

That means the intended semantics are:

- build the full repo traceability model
- then filter the resulting projected traceability view to the requested scope

The Lean model in `ScopedHealth/Closure.lean` is the first abstraction of the
equivalence we need before reintroducing any early-pruning optimization.

## Rust Paths Currently Under Suspicion

The most important current semantic risk is in the scoped traceability path:

- [src/modules/analyze.rs](/Users/gk/work/lableaks/projects/special/src/modules/analyze.rs:145)
  narrows traceability inputs before repo traceability is summarized.
- [src/modules/analyze/repo_scope.rs](/Users/gk/work/lableaks/projects/special/src/modules/analyze/repo_scope.rs:83)
  reduces the candidate set to files from the same languages as the scoped
  files.
- [src/modules/analyze/registry.rs](/Users/gk/work/lableaks/projects/special/src/modules/analyze/registry.rs:51)
  resolves pack-local scoped facts and graph facts and now reports explicit
  unavailability when that preparation fails.

These are exactly the kinds of transformations that need a proof obligation.

## What The Current Lean Model Covers

The current model proves a generic statement:

- if `Keep` is exactly the reachable closure of the scoped seed set
- and the summary is a local classification over reachable items
- then analyzing on the induced closure yields the same summary as analyzing on
  the full graph

That is the mathematical core of the optimization we were trying to ship.

It now also includes a product-shaped corollary for bucketed traceability
summaries:

- build a traceability summary over the full graph
- project it to the requested scope

is equal to:

- build the same traceability summary over the exact induced closure
- project that summary to the requested scope

provided the closure is exact and the bucket classifier is local to the item.

One important implementation lesson from the Rust pack is that the optimized
scoped path may legitimately retain off-scope peers inside the kept closure.
So the correctness target is not equality of the raw intermediate summaries.

The right theorem shape is:

- `project(scoped_analysis(keep)) = project(full_analysis)`

not:

- `scoped_analysis(keep) = project(full_analysis)`

## What Still Needs To Be Modeled

Before this can certify the Rust implementation, we still need explicit models
for:

- how scope seeds are derived from files and symbols
- which edges count toward traceability and support
- which summary categories are truly local classifications
- which parts of repo health are graph-local versus global
- what assumptions are delegated to parser output and external tools

Recent Rust-side executable checks now cover a stronger structural layer beneath
the bucketed summary:

- projected retained item ids match between full and scoped Rust analysis
- projected supported item ids match between full and scoped Rust analysis
- projected per-item support evidence matches between full and scoped Rust
  analysis
- projected per-item module connectivity flags match between full and scoped
  Rust analysis
- projected support-root witness ids match between full and scoped Rust inputs

That still does not prove the reverse-walk itself is exact, but it narrows the
remaining hidden step to the support-reachability relation rather than the final
rendered summary buckets.

The Lean sidecar now mirrors that witness layer explicitly in
`ScopedHealth/SupportWitness.lean`:

- `SupportRootsFor R IsRoot target`
- `support_roots_for_eq_under_exact_closure`
- `support_roots_for_eq_of_exact_scope_boundary`

So the formal model now states the same kind of claim the Rust executable checks
exercise: for a scoped target item that belongs to the semantic seed set, exact
closure preserves the set of support-root witnesses.

The sidecar now also models the stronger structural layer exercised by the Rust
adapter tests in `ScopedHealth/ReverseClosure.lean`:

- `ReachableFrom R target`
- `reachable_from_eq_under_exact_closure`
- `reachable_from_eq_of_exact_scope_boundary`

That theorem is closer to the new Rust executable check that compares the exact
reverse-reachable caller closure for projected items and retained context items.
It does not yet certify the Rust implementation, but it means the formal model
now speaks about preserved backward closure directly, not just preserved
support-root witnesses.

To line the formal model up more explicitly with the production adapters, the
sidecar now treats `ScopedHealth/ProjectedContractClosure.lean` as the public
shared exact core:

- `ProjectedContractClosureBoundary`
- `reachable_from_eq_of_projected_contract_closure_boundary`
- `support_roots_for_eq_of_projected_contract_closure_boundary`

That boundary states the exact semantic shape Rust, TypeScript, and Go now
share through the normalized proof interface:

- projected output items
- a smaller exact target set
- the exact reverse-closure graph induced by those targets

So the remaining proof gap is now stated against that shared core, not against
a Rust-specific narrowing.

Rust now follows the same public projected-contract target story as Go and
TypeScript:

- `working_contract()` remains the broad operational reverse-walk seed set
- `exact_contract(&TraceGraph)` now uses the support-backed projected items as
  its public exact target set
- `reference(&TraceGraph)` derives the exact reverse-closure graph from that shared
  contract

The older Rust-only representative-compression target story is no longer part
of the public baseline. It survives only as private/research history.

## TypeScript Mapping

TypeScript is now explicit in the same important way as Rust, but the split is
different:

- the pack-owned scoped kernel lives in
  [src/language_packs/typescript/analyze/boundary.rs](/Users/gk/work/lableaks/projects/special/src/language_packs/typescript/analyze/boundary.rs)
- the production code now names both:
  - `working_contract()`
  - `exact_contract(...)`

Concretely:

- `projected_files` are the files that scoped `special health` should report
- `working_contract().preserved_file_closure` is the connected file closure
  induced by the TypeScript module graph scope facts
- `exact_contract(...).preserved_file_closure` is the smaller file set that
  keeps:
  - the projected scoped files
  - the support-root/test files needed to preserve projected traceability
    evidence

The current executable assurance for TypeScript now includes:

- direct pack-level `full + filter` vs `scoped` equality on:
  - direct call
  - tool-backed collision disambiguation
  - reference-backed callback routing
  - React render stack
  - Next client component stack
  - event / forwarded / hook / effect / context callback routing
- exact preserved-file identity checks on the main fixture families
- fixture-level ablation checks showing the smaller exact contract's kept file
  projection is minimal on the main fixture families
- an adversarial live-cycle fixture where:
  - the projected file closes over a reachable two-file import cycle
  - an unrelated dead two-file cycle exists in the same repo
  - the smaller exact contract still excludes the dead cycle

The first smaller TypeScript contract attempt was wrong: it collapsed to an
underpowered seed set once the tool-backed proof harness was made honest. The
current smaller contract is still item-derived, but is projected back to files
by preserving:

- the requested scoped files
- the files that own the support-root witnesses for those projected items

The contract now also exposes its exact target ids directly, and the current
fixture families show an important nuance:

- the target set is the set of supported projected items
- not just exported/top-level entrypoints
- so internal projected helpers can legitimately appear in the exact contract
  target set when they participate in the supported path

That is why the exact contract is now smaller than the broad file closure while
still preserving the structural facts we care about.

The formal side still names the broad operational file-closure theorem
explicitly in
`ScopedHealth/FileClosure.lean`:

- `FileClosureBoundary`
- `summary_eq_of_exact_file_closure_boundary`
- `reachable_from_eq_of_exact_file_closure_boundary`

That file-closure theorem now belongs to the broad operational TypeScript
closure, not to the final exact contract.

The current smaller exact TypeScript contract is instead backed by executable
checks at a lower structural layer:

- projected support-root witness ids match between full and scoped TypeScript
  inputs
- projected reverse-reachable closure matches between full and scoped
  TypeScript inputs
- summary equality still holds after projecting back to the requested scope
- the production-side `reference(...)` object derived from the full item graph
  matches the scoped graph directly on every current fixture family

The sidecar now also names stronger exact-contract theorems explicitly in:

- `ScopedHealth/ProjectedContractClosure.lean`
- `FileContractClosureBoundary`
- `ProjectedContractClosureBoundary`
- `support_roots_for_eq_of_projected_contract_closure_boundary`
- `reachable_from_eq_of_projected_contract_closure_boundary`
- `support_roots_for_eq_of_file_contract_closure_boundary`
- `reachable_from_eq_of_file_contract_closure_boundary`

The item-level boundary states that the kept semantic kernel is exactly:

- the projected scoped items
- plus the exact reverse closure of the supported projected target set

The file-level boundary then states that the kept execution file set is
exactly:

- the projected scoped files
- plus the files that own items in the reverse closure of the exact target set

This matches the current TypeScript production story much more closely than the
earlier witness-only fallback:

- the exact contract is item-derived
- it now has a first-class `reference(...)`
- the TypeScript adapter now exposes the normalized projected-contract fields
  plus pack-local preserved item/file sets
- the shared normalized proof core reconstructs the preserved graph/item set
  from the production reference object
- the live scoped runtime now filters `repo_items`, `context_items`, and the
  in-memory trace graph to that exact item kernel
- the pack-owned suite now checks that
  `exact_contract(...).preserved_file_closure`
  is exactly the file projection of `reference(...).exact_reverse_closure`

That gives TypeScript a stronger formal shape parallel to the current Rust
story:

- broad operational closure theorem in `FileClosure`
- smaller exact item-kernel theorem in `ProjectedContractClosure`
- matching execution projection theorem in `FileContractClosure`
- explicit production-side contract/reference split in
  `src/language_packs/typescript/analyze/boundary.rs`

Go now mirrors that same shared item-kernel shape directly.

The production Go adapter now also exposes:

- `working_contract()`
- `exact_contract(...)`
- `reference(...)`

and the pack-owned Go suite checks the same public semantic core against the
production reference object:

- `full + filter` vs `scoped`
- exact target-id identity
- projected support-root equality
- projected reverse-closure equality
- direct exact-contract equality against the production reference
- owned exact item-kernel equality reconstructed from the normalized reference
- scope-facts/runtime equality

Unlike TypeScript, Go does not currently have a separate public
file-contract theorem layer. It still loads files to execute its analyzers, and
that execution path widens from the preserved item kernel to a broader file set.

This also sharpens the shared cross-language picture:

- Rust, TypeScript, and Go now all have:
  - projected output items
  - exact reverse-closure targets
  - exact reverse-closure item context
- TypeScript still needs a file projection layer for execution, but the live
  semantic kernel is now item-level before that projection
- TypeScript executes over a file projection derived from that item kernel
- Go executes over a broader file set rebuilt from that item kernel
- Rust, TypeScript, and Go now all target the same projected-contract proof
  object before analysis is summarized, even when their execution-layer file
  loading differs

So the public shared exact theorem family is now `ProjectedContractClosure`.
The top-level proof harness currently checks projected
`analysis.traceability` equality on validated single-file fixtures, while the
stronger support-root, reverse-closure, and owned-item-kernel claims remain
pack-owned executable checks.

The implementation boundary is now tighter too:

- TypeScript scope facts now include enough cached graph material to rebuild
  the smaller exact contract during live source-file narrowing
- so the current runtime narrowing path now uses
  `exact_contract(...).preserved_file_closure`
- the broad `working_contract()` remains valuable as the operational closure
  theorem surface and as an audit/check against over-widening, but it no longer
  exclusively drives live narrowing

`ScopedHealth/FileProjectionWitness.lean` remains in the sidecar as the
historical fallback theorem surface we used before the stronger exact
file-contract closure was locked down.

On the Rust side, `ScopedTraceabilityBoundary` now names the same three moving
parts explicitly:

- `projected_item_ids`
- retained `context_items`
- semantic `seed_ids`

That does not prove the mapping, but it makes the implementation boundary line
up with the weaker theorem instead of leaving the target set implicit.

The Rust adapter now also separates output subjects from semantic context at the
shared-core boundary:

- `TraceabilityInputs.repo_items` are the items to summarize
- `TraceabilityInputs.context_items` are the items needed to compute support and
  module-context facts

For scoped Rust analysis, that means the raw scoped summary can now match the
full-summary projection directly instead of relying on a later projection step
to discard retained context peers.

Recent Rust-side executable checks now go one step further:

- projected support-root witness ids match between full and scoped inputs
- retained context-item support-root witness ids also match between full and
  scoped inputs

That is stronger than only checking projected outputs, because projected module
flags depend on those retained context peers.

## Next Engineering Step

Do not add new scoped-pruning logic until both of these are true:

1. the Rust path's preserved target set is shown to be the exact one required
   for scoped `special health`
2. any optimization still matches the exact-closure model proved here and
   checked against the Rust reference implementation
