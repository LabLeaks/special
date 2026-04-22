import ScopedHealth.Closure
import ScopedHealth.ReverseClosure
import ScopedHealth.ScopeBoundary

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
`SupportRootsFor R IsRoot target root` means that `root` is a witness root for
`target` under the reverse support relation `R`.

This matches the Rust-side structural check more closely than the bucketed
summary layer: for a projected item, compare the reachable support-root witness
ids recovered by the full graph and the scoped kept graph.
-/
def SupportRootsFor
    (R : α → α → Prop)
    (IsRoot : α → Prop)
    (target : α) :
    α → Prop :=
  fun root => IsRoot root ∧ Path R target root

/--
If `Keep` is exact for the chosen seed set and `target` is one of those seeds,
then the per-target support-root witness set is preserved under the induced
closure.

This is the witness-layer analogue of the summary projection theorem.
-/
theorem support_roots_for_eq_under_exact_closure
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    (IsRoot : α → Prop)
    {target : α}
    (htarget : Seed target)
    (hkeep : ∀ x, Keep x ↔ Reachable R Seed x) :
    SupportRootsFor (Induced Keep R) IsRoot target =
      SupportRootsFor R IsRoot target := by
  funext root
  apply propext
  constructor
  · intro h
    exact ⟨h.1, strip_induced_path h.2⟩
  · intro h
    exact ⟨h.1, lift_path_to_induced hkeep htarget h.2⟩

/--
Scope-boundary corollary of the witness theorem.

When a target belongs to the semantic seed set used by the boundary, exactness
of the kept closure preserves the same support-root witnesses for that target.
-/
theorem support_roots_for_eq_of_exact_scope_boundary
    {R : α → α → Prop}
    {σ : Type u}
    (boundary : ScopeBoundary (σ := σ) R)
    (IsRoot : α → Prop)
    {target : α}
    (htarget : boundary.seed target) :
    SupportRootsFor (Induced boundary.keep R) IsRoot target =
      SupportRootsFor R IsRoot target := by
  exact support_roots_for_eq_under_exact_closure IsRoot htarget boundary.keep_exact

/--
If every node reachable from the chosen seed set is kept, then per-target
support-root witnesses are preserved for any kept seed in that set.
-/
theorem support_roots_for_eq_if_keeps_reachable
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    (IsRoot : α → Prop)
    {target : α}
    (htarget : Seed target)
    (hkeep_reachable : ∀ x, Reachable R Seed x → Keep x) :
    SupportRootsFor (Induced Keep R) IsRoot target =
      SupportRootsFor R IsRoot target := by
  funext root
  apply propext
  constructor
  · intro h
    exact ⟨h.1, strip_induced_path h.2⟩
  · intro h
    exact ⟨h.1, lift_path_to_induced_of_keeps_reachable htarget hkeep_reachable h.2⟩

end ScopedHealth
end SpecialProofs
