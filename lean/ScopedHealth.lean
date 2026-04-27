/-
Lean traceability kernel
========================

This package contains the production scoped traceability kernel plus the small
abstract theorem family that audits its graph-narrowing contract. The executable
JSON kernel lives in `ScopedHealth.KernelCli`; this root imports the theorem
surface and executable kernel used by the released binary.

How to read this package
------------------------

Read the files in this order:

1. `Closure.lean`
   defines paths, reachability from a target set, and relation restriction.
2. `ReverseClosure.lean`
   proves that a kept graph preserves per-target reverse reachability when it
   keeps all nodes reachable from that target set.
3. `SupportWitness.lean`
   lifts the same fact to reachable support roots.
4. `ProjectedContractClosure.lean`
   packages the shared Rust/TypeScript/Go item-level production contract.
5. `FileContractClosure.lean`
   adds the TypeScript file-loading projection.
6. `ProjectedKernel.lean`
   defines the executable projected-kernel decisions and proves that those
   target-selection definitions inhabit the projected contract theorem surface.
7. `KernelCli.lean`
   adapts the projected kernel to stdin/stdout JSON.

Why there is a narrowed kernel
------------------------------

The product behavior we want for `special health PATH` is conceptually simple:
analyze the full repository, then show only the traceability results relevant to
`PATH`.

Doing that literally for every scoped request is often too expensive and, for
some language packs, not the way the underlying tools work. TypeScript and Go
tools naturally load files or packages; Rust analysis may need context items
that are outside the displayed scope. So the implementation uses a narrowed
kernel: keep the requested output items plus enough surrounding trace graph to
compute their support evidence.

The danger is that "enough surrounding graph" is easy to get subtly wrong. If
we keep too little, a scoped run can lose support roots or reverse callers that
full analysis would have found. If we keep unrelated material, results may be
slow or confusing, but the more serious correctness failure is losing evidence.

This Lean package states the exact condition under which narrowing is
traceability-preserving. It is not math for its own sake: it is the audit
contract that lets a faster scoped implementation stand in for "full analysis,
then filter" for the traceability evidence named below.

Objects
-------

Let `α` be the type of analyzed items. Let `R : α -> α -> Prop` be the
backward trace relation: `R callee caller` means that `caller` is direct
evidence for, or directly supports, `callee`. This is the inverse of the JSON
edge map accepted by `KernelCli.lean`, which stores direct `caller -> callee`
edges and then walks incoming callers.

`Path R a b` is the reflexive-transitive closure of `R`. Thus `Path R target x`
means that `x` is in the reverse support/caller closure of `target`.

`Reachable R Target x` means that `x` is reachable from at least one item in the
target predicate `Target`.

`Induced Keep R` is `R` restricted to edges whose endpoints both satisfy
`Keep`. This models running traceability over a narrowed kept graph.

The shared production contract
------------------------------

`ProjectedContractClosureBoundary R` has three predicates:

* `target`    -- the supported projected items chosen as semantic roots
* `projected` -- the output items the user asked to see
* `keep`      -- the item set retained by the narrowed kernel

Its only semantic assumption is:

    keep x <-> projected x \/ Reachable R target x

In words: the narrowed kernel keeps exactly the requested projected outputs plus
the full reverse closure of the supported projected targets.

What is proven
--------------

For every `target` item satisfying `boundary.target target`:

1. Reverse closure is unchanged:

       ReachableFrom (Induced boundary.keep R) target =
       ReachableFrom R target

   So every item that can support `target` in the full graph can still support
   it after narrowing, and narrowing introduces no new support path.

2. Support-root witnesses are unchanged for any root predicate `IsRoot`:

       SupportRootsFor (Induced boundary.keep R) IsRoot target =
       SupportRootsFor R IsRoot target

   So the kept graph preserves exactly the same reachable roots, such as tests
   or specs, for every supported projected target.

TypeScript has one extra execution-layer theorem,
`FileContractClosureBoundary`, because its analyzer loads files rather than
individual items. If the kept file set is exactly the projected files plus the
files that own the reverse closure of the target items, then the same two
item-level preservation results hold when `Keep x` is defined as "the owner file
of `x` is kept".

What is not proven here
-----------------------

This proof does not prove parser correctness, language-server correctness,
quality metrics, cache correctness, JSON parsing correctness, or that any
language pack computes predicates satisfying the boundary assumptions. Those
are checked by executable Rust/TypeScript/Go fixture tests and by Lean-vs-Rust
kernel equivalence tests around the production JSON kernel.

The proof is therefore conditional but exact: if the boundary predicate matches
the full reverse closure described above, then reverse-reachable traceability
evidence and support-root witnesses are preserved by the narrowed graph.
-/

import ScopedHealth.Closure
import ScopedHealth.ReverseClosure
import ScopedHealth.SupportWitness
import ScopedHealth.ProjectedContractClosure
import ScopedHealth.FileContractClosure
import ScopedHealth.ProjectedKernel
