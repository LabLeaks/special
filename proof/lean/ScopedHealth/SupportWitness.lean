import ScopedHealth.Closure
import ScopedHealth.ReverseClosure

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
`SupportRootsFor R IsRoot target root` means that `root` is a witness root for
`target` under the reverse support relation `R`.

This is the structural evidence check for a projected item: compare the
reachable support-root witness ids recovered by the full graph and the narrowed
kept graph.
-/
def SupportRootsFor
    (R : α → α → Prop)
    (IsRoot : α → Prop)
    (target : α) :
    α → Prop :=
  fun root => IsRoot root ∧ Path R target root

/--
If every node reachable from the chosen seed set is kept, then per-target
support-root witnesses are preserved for any kept seed in that set.
-/
theorem support_roots_preserved_when_reachable_nodes_are_kept
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
    exact ⟨h.1,
      path_preserved_when_reachable_nodes_are_kept htarget hkeep_reachable h.2⟩

end ScopedHealth
end SpecialProofs
