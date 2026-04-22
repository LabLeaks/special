import ScopedHealth.Closure

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
Traceability buckets modeled after `special health`'s repo/module traceability
summary categories.
-/
inductive TraceBucket where
  | currentSpec
  | plannedOnly
  | deprecatedOnly
  | fileScopedOnly
  | unverifiedTest
  | staticallyMediated
  | unexplained
  deriving DecidableEq, Repr

/--
A bucketed traceability summary is represented extensionally as a predicate over
bucket-item pairs.
-/
def BucketedSummary (β : Type u) :=
  TraceBucket → β → Prop

/--
Build the abstract traceability summary from:

- a graph relation
- a root seed set
- a local bucket classifier

This matches the current proof boundary: reachability is graph-derived, while
bucket membership is assumed local to the item once its reachable evidence is
fixed.
-/
def summarizeTraceability
    (R : α → α → Prop)
    (Seed : α → Prop)
    (Class : α → TraceBucket) :
    BucketedSummary α :=
  fun bucket x => Reachable R Seed x ∧ Class x = bucket

/--
Project a summary to the requested item scope.

This is the abstract shape of `special health PATH` filtering its output to the
requested scope after analysis.
-/
def projectScope
    (InScope : α → Prop)
    (summary : BucketedSummary α) :
    BucketedSummary α :=
  fun bucket x => InScope x ∧ summary bucket x

/--
If the analyzed closure is exact for the chosen seed set, then projecting the
full traceability summary to the requested scope is equivalent to projecting the
summary computed on the induced exact closure.

This is the first product-shaped corollary of `summary_eq_under_exact_closure`.
-/
theorem project_traceability_eq_under_exact_closure
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    (InScope : α → Prop)
    (Class : α → TraceBucket)
    (hkeep : ∀ y, Keep y ↔ Reachable R Seed y) :
    projectScope InScope (summarizeTraceability (Induced Keep R) Seed Class) =
      projectScope InScope (summarizeTraceability R Seed Class) := by
  funext bucket
  funext x
  apply propext
  constructor
  · intro hx
    exact ⟨hx.1, (reachable_iff_reachable_induced hkeep).1 hx.2.1, hx.2.2⟩
  · intro hx
    exact ⟨hx.1, (reachable_iff_reachable_induced hkeep).2 hx.2.1, hx.2.2⟩

end ScopedHealth
end SpecialProofs
