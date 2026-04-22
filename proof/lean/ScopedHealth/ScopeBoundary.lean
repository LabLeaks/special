import ScopedHealth.TraceSummary

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {σ : Type u}
variable {α : Type u}

/--
Abstract boundary for scoped analysis:

- `request` is the user-facing scope input
- `inScope` is the final projection predicate used for display
- `seed` is the semantic root set used to begin graph traversal
- `keep` is the closure candidate kept for optimized analysis

The key obligation is `keep_exact`: the kept closure must be exactly the
reachable closure of `seed` in the full graph.
-/
structure ScopeBoundary (R : α → α → Prop) where
  request : σ
  inScope : α → Prop
  seed : α → Prop
  keep : α → Prop
  keep_exact : ∀ x, keep x ↔ Reachable R seed x

/--
Two seed derivations are semantically equivalent when they induce the same
 reachable set, even if they were computed differently from the request.

This is the right notion for comparing "full + filter" against a derived scoped
seed pipeline.
-/
theorem reachable_eq_of_seed_equiv
    {R : α → α → Prop}
    {Seed₁ Seed₂ : α → Prop}
    (hseed : ∀ x, Seed₁ x ↔ Seed₂ x) :
    Reachable R Seed₁ = Reachable R Seed₂ := by
  funext x
  apply propext
  constructor
  · intro hx
    rcases hx with ⟨s, hs, path⟩
    exact ⟨s, (hseed s).1 hs, path⟩
  · intro hx
    rcases hx with ⟨s, hs, path⟩
    exact ⟨s, (hseed s).2 hs, path⟩

/--
If two seed derivations are equivalent, then they produce the same bucketed
traceability summary over the same full graph.
-/
theorem summarize_traceability_eq_of_seed_equiv
    {R : α → α → Prop}
    {Seed₁ Seed₂ : α → Prop}
    (Class : α → TraceBucket)
    (hseed : ∀ x, Seed₁ x ↔ Seed₂ x) :
    summarizeTraceability R Seed₁ Class =
      summarizeTraceability R Seed₂ Class := by
  funext bucket
  funext x
  apply propext
  constructor
  · intro hx
    exact ⟨by
      simpa [reachable_eq_of_seed_equiv hseed] using hx.1, hx.2⟩
  · intro hx
    exact ⟨by
      simpa [reachable_eq_of_seed_equiv hseed] using hx.1, hx.2⟩

/--
Main scoped-health corollary:

If a scope boundary satisfies exact closure, then projecting the full summary is
equivalent to projecting the summary computed only on the kept closure.
-/
theorem project_traceability_eq_of_exact_scope_boundary
    {R : α → α → Prop}
    (boundary : ScopeBoundary (σ := σ) R)
    (Class : α → TraceBucket) :
    projectScope boundary.inScope
        (summarizeTraceability (Induced boundary.keep R) boundary.seed Class) =
      projectScope boundary.inScope
        (summarizeTraceability R boundary.seed Class) := by
  exact project_traceability_eq_under_exact_closure
    boundary.inScope
    Class
    boundary.keep_exact

end ScopedHealth
end SpecialProofs
