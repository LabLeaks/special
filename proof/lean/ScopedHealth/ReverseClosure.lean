import ScopedHealth.Closure

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
target is one of those seeds, then every original path from that target stays
inside the kept graph.
-/
theorem path_preserved_when_reachable_nodes_are_kept
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
theorem reverse_closure_preserved_when_reachable_nodes_are_kept
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
    exact path_preserved_when_reachable_nodes_are_kept htarget hkeep_reachable h

end ScopedHealth
end SpecialProofs
