import ScopedHealth.ReverseClosure
import ScopedHealth.SupportWitness

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
An exact projected contract closure boundary.

This is the item-level shape used when a scoped analysis keeps:

- the projected output items themselves
- plus the exact reverse closure of a smaller target set

The theorem assumptions name the kept item set itself, so the preservation
claims below follow from an auditable closure predicate rather than from an
opaque implementation.
-/
structure ProjectedContractClosureBoundary (R : α → α → Prop) where
  target : α → Prop
  projected : α → Prop
  keep : α → Prop
  keep_exact : ∀ x, keep x ↔ projected x ∨ Reachable R target x

/--
Under a projected contract-closure boundary, every node reachable from the
target seed set is kept.
-/
theorem projected_contract_keeps_reachable
    {R : α → α → Prop}
    (boundary : ProjectedContractClosureBoundary R)
    {x : α} :
    Reachable R boundary.target x → boundary.keep x := by
  intro hx
  exact (boundary.keep_exact _).2 (Or.inr hx)

/--
If the kept item set contains the full reverse closure of the target set, then
every declared target preserves the same reverse closure in the induced kept
subgraph.
-/
theorem reachable_from_eq_of_projected_contract_closure_boundary
    {R : α → α → Prop}
    (boundary : ProjectedContractClosureBoundary R)
    {target : α}
    (htarget : boundary.target target) :
    ReachableFrom (Induced boundary.keep R) target =
      ReachableFrom R target := by
  exact reverse_closure_preserved_when_reachable_nodes_are_kept
    htarget
    (fun x hx => projected_contract_keeps_reachable boundary hx)

/--
If the kept item set contains the full reverse closure of the target set, then
every declared target preserves the same support-root witnesses in the induced
kept subgraph.
-/
theorem support_roots_for_eq_of_projected_contract_closure_boundary
    {R : α → α → Prop}
    (boundary : ProjectedContractClosureBoundary R)
    (IsRoot : α → Prop)
    {target : α}
    (htarget : boundary.target target) :
    SupportRootsFor (Induced boundary.keep R) IsRoot target =
      SupportRootsFor R IsRoot target := by
  exact support_roots_preserved_when_reachable_nodes_are_kept
    IsRoot
    htarget
    (fun x hx => projected_contract_keeps_reachable boundary hx)

end ScopedHealth
end SpecialProofs
