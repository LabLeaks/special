import ScopedHealth.Closure
import ScopedHealth.ScopeBoundary

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
`ReachableFrom R target x` means that `x` lies in the per-target backward
closure rooted at `target`.

This matches the Rust-side structural check that compares reverse-reachable
caller ids between full and scoped analysis for projected items and retained
context items.
-/
def ReachableFrom (R : α → α → Prop) (target : α) : α → Prop :=
  fun x => Path R target x

/--
If every node reachable from the chosen seed set is kept, and the current
target is itself kept, then every original per-target path can be lifted into
the induced kept relation.
-/
theorem lift_path_to_induced_of_keeps_reachable
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    {s x : α}
    (hs : Seed s)
    (hkeep_reachable : ∀ y, Reachable R Seed y → Keep y)
    (path : Path R s x) :
    Path (Induced Keep R) s x := by
  induction path with
  | refl =>
      exact Path.refl _
  | tail path hbc ih =>
      have hb_reachable : Reachable R Seed _ := ⟨s, hs, path⟩
      have hc_reachable : Reachable R Seed _ := reachable_step hb_reachable hbc
      exact Path.tail ih
        ⟨hkeep_reachable _ hb_reachable, hkeep_reachable _ hc_reachable, hbc⟩

/--
If every node reachable from the chosen seed set is kept, then the per-target
backward closure is preserved for any kept seed in that set.
-/
theorem reachable_from_eq_if_keeps_reachable
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    {target : α}
    (htarget : Seed target)
    (hkeep_reachable : ∀ y, Reachable R Seed y → Keep y) :
    ReachableFrom (Induced Keep R) target =
      ReachableFrom R target := by
  funext x
  apply propext
  constructor
  · intro h
    exact strip_induced_path h
  · intro h
    exact lift_path_to_induced_of_keeps_reachable htarget hkeep_reachable h

/--
If `Keep` is exact for the chosen seed set and `target` belongs to that seed
set, then the per-target backward closure is preserved under the induced kept
subgraph.
-/
theorem reachable_from_eq_under_exact_closure
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    {target : α}
    (htarget : Seed target)
    (hkeep : ∀ y, Keep y ↔ Reachable R Seed y) :
    ReachableFrom (Induced Keep R) target =
      ReachableFrom R target := by
  funext x
  apply propext
  constructor
  · intro h
    exact strip_induced_path h
  · intro h
    exact lift_path_to_induced hkeep htarget h

/--
Scope-boundary corollary of the per-target backward-closure theorem.

For any semantic seed retained by an exact scope boundary, the induced closure
preserves exactly the same backward-reachable items as the full graph.
-/
theorem reachable_from_eq_of_exact_scope_boundary
    {R : α → α → Prop}
    {σ : Type u}
    (boundary : ScopeBoundary (σ := σ) R)
    {target : α}
    (htarget : boundary.seed target) :
    ReachableFrom (Induced boundary.keep R) target =
      ReachableFrom R target := by
  exact reachable_from_eq_under_exact_closure htarget boundary.keep_exact

end ScopedHealth
end SpecialProofs
