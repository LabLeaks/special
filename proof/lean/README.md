# Scoped Health Lean Model

This is a sidecar proof workspace for the scoped-health equivalence problem in
`special`.

It is intentionally not wired into the Rust build, CLI, or release pipeline
yet. The purpose of this directory is to make the semantic contract explicit
before we optimize the implementation further.

## Why This Exists

`special health PATH` is supposed to preserve the projected traceability
meaning:

- build the full repo traceability model
- then filter the traceability result to the requested scope

The recent scoped-health work introduced earlier pruning in core and in some
language packs. That created a semantic risk:

- `filter(scope, full_health(repo))`

is not obviously equivalent to:

- `analyze(projected_or_closed_repo(scope))`

This workspace is where we lock down the conditions under which those two
approaches are actually equal.

## Current Formal Target

The first theorem target is intentionally narrower than the full product:

1. model backward reachability over a generic relation
2. define an exact closure as "all and only nodes reachable from the scoped
   seed set"
3. prove that analyzing on the induced exact closure preserves the reachable
   summary produced by the full graph

In other words, the first question is:

- if a scoped closure is truly exact, does `closure + analyze` produce the same
  reachable summary as `full + filter`?

That is the core mathematical shape behind the current bug cluster.

## What This Does Not Model Yet

Not yet in scope:

- Rust / TypeScript / Go syntax extraction
- parser correctness
- `rust-analyzer`, `gopls`, or `pyright` trust boundaries
- repo signals outside projected traceability
- cache correctness

Those belong in later layers.

This directory is only trying to prove the abstract semantic invariant that our
optimization strategy must satisfy before we trust it.

## Current Shared Mapping

The public proof baseline is now a single shared theorem family across packs:

- projected output items
- a smaller exact target set
- the exact reverse-closure item context induced by that target set

In Lean, that shared boundary is:

- `ScopedHealth/ProjectedContractClosure.lean`
- `ProjectedContractClosureBoundary`
- `reachable_from_eq_of_projected_contract_closure_boundary`
- `support_roots_for_eq_of_projected_contract_closure_boundary`

Rust, TypeScript, and Go now expose the same production-side proof interface:

- `working_contract()`
- `exact_contract(...)`
- `reference(...)`

and the pack-owned suites check the same semantic core against the production
reference object:

- `full + filter` vs `scoped`
- support-root equality
- reverse-closure equality
- owned exact item-kernel equality reconstructed from the normalized reference

So the public story is no longer “Rust’s narrower theorem first, then maybe
something weaker for other languages.” The proof baseline is the shared
projected-contract exact kernel that all three packs target through a common
proof interface, with stronger structural checks living in the pack-owned
executable suites.

One important boundary line:

- the Lean sidecar currently proves the abstract projected-contract theorem
  family
- the top-level proof harness currently checks projected `analysis.traceability`
  equality on validated single-file fixtures
- neither currently certifies the entire `special health PATH` surface

## Current Language Projections

Below that shared item-kernel theorem, languages can still have different
execution projections.

TypeScript keeps an additional file-layer theorem because its tools load files
rather than individual items. The broad operational TS file closure is still
modeled in `ScopedHealth/FileClosure.lean`:

- `FileClosureBoundary`
- `summary_eq_of_exact_file_closure_boundary`
- `reachable_from_eq_of_exact_file_closure_boundary`

That theorem now belongs only to the **working** TypeScript closure.

For the smaller current TypeScript `exact_contract(...)`, the sidecar also
includes a file-level execution projection theorem in
`ScopedHealth/FileContractClosure.lean`:

- `FileContractClosureBoundary`
- `support_roots_for_eq_of_file_contract_closure_boundary`
- `reachable_from_eq_of_file_contract_closure_boundary`

That file boundary states that the kept execution file set is exactly:

- the projected scoped files
- plus the files owning items in the reverse closure of the exact target set

The production TypeScript adapter mirrors that split explicitly and the
pack-owned suite checks it directly against the production reference across the
full TS fixture family. Its pack-local contract still carries extra file-level
execution data beyond the normalized shared proof object.

The current executable assurance for the TypeScript exact contract includes:

- direct pack-level `full + filter` vs `scoped` equality
- exact preserved-file identity checks
- fixture-level ablation checks showing the kept execution file projection is
  minimal on the current fixture families
- projected support-root equality between full and scoped inputs
- projected reverse-closure equality between full and scoped inputs
- direct exact-contract equality against the production reference
- owned exact item-kernel equality:
  projected repo items plus exact reverse-closure context match the production
  reference object on the current fixture family
- exact file-closure equality:
  the kept file set equals the file projection of the full reverse-closure
  reference on the current fixture family

The implementation boundary is tighter too:

- live source-file narrowing happens from enriched serialized scope facts that
  carry enough cached graph material to rebuild the smaller exact contract
- the TypeScript runtime now filters `repo_items`, `context_items`, and the
  in-memory graph to that exact item kernel
- execution then uses the kept file projection needed to load those items

One important detail from the TypeScript contract-target checks:

- the smaller exact target set is the set of supported projected items
- not just exported entrypoints
- so internal projected helpers like `helper`, `invoke`, or `CounterButton`
  can legitimately belong to the exact target set when they participate in the
  supported path

Go now inhabits that same projected-contract item kernel semantically.

The production Go adapter mirrors the same split explicitly:

- `working_contract()`
- `exact_contract(...)`
- `reference(...)`

And the current Go executable assurance now includes:

- direct pack-level `full + filter` vs `scoped` equality
- exact target-id identity checks across direct, reference, interface,
  embedding, method-value, method-expression, receiver-collision, and embedded-
  interface fixture families
- projected support-root equality between full and scoped inputs
- projected reverse-closure equality between full and scoped inputs
- direct exact-contract equality against the production reference
- owned exact item-kernel equality:
  projected repo items plus exact reverse-closure context match the production
  reference object on the current fixture family
- scope-facts/runtime equality:
  the pre-narrowed file closure from cached Go scope facts produces the same
  kept item kernel as the live scoped runtime

Unlike TypeScript, Go does not currently have a separate public file-contract
theorem layer. Its semantic narrowing is item-level first, but runtime still
rebuilds a broader execution file set from that preserved item kernel.

The honest current cross-language statement is therefore:

- Rust, TypeScript, and Go all inhabit `ProjectedContractClosure`
- that projected-contract theorem is the public shared exact baseline
- any narrower language-specific strengthening is secondary to that shared core

Rust no longer exposes its older representative-compression target story as the
public baseline. That narrower move is retained only as private/research
history.

`ScopedHealth/FileProjectionWitness.lean` remains in the sidecar only as the
earlier fallback theorem surface we used before the stronger exact contract was
locked down.

## File Layout

- `ScopedHealth/Closure.lean`
  Generic reachability, induced-closure, and summary-equivalence definitions
  and proofs.
- `ScopedHealth/TraceSummary.lean`
  Bucketed traceability-summary and scoped-projection equivalence definitions
  and proofs.
- `ScopedHealth/ScopeBoundary.lean`
  Explicit scope-request / seed / keep-set boundary model for reasoning about
  whether an optimized scoped pipeline is semantically valid.
- `ScopedHealth/ProjectedContractClosure.lean`
  Shared exact projected-item-plus-reverse-closure theorem for the public
  cross-language core.
- `ScopedHealth/FileClosure.lean`
  Exact file-import-closure theorem for the broad TypeScript working closure.
- `ScopedHealth/FileContractClosure.lean`
  Stronger exact file-contract-closure theorem for the smaller current
  TypeScript exact contract.
- `ScopedHealth/FileProjectionWitness.lean`
  Earlier witness-only fallback theorem for the smaller current TypeScript
  exact contract.
- `ScopedHealth/RepresentativeClosure.lean`
  Historical/private Rust-only strengthening, no longer the public baseline.
- `ScopedHealth.lean`
  Root import for the sidecar library.

## Intended Next Steps

1. Extend the model from generic reachability to the specific traceability
   summary shape that `special health` computes.
2. Make the trust boundary explicit:
   - proven in Lean
   - assumed from extracted graph facts
   - unmodeled in tool/runtime integrations
3. Add a slow Rust reference implementation and differential tests that compare:
   - full analysis then filter
   - optimized scoped analysis
4. Only reintroduce scoped pruning where the implementation can be traced back
   to the proved model.

## Build

This repo uses `elan` through `mise`.

To typecheck the sidecar workspace:

```sh
mise exec -- elan run leanprover/lean4:v4.29.1 lake build
```
