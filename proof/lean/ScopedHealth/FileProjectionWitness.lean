import ScopedHealth.ReverseClosure
import ScopedHealth.SupportWitness

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
A product-shaped formal boundary for the smaller TypeScript exact contract.

Unlike `FileClosureBoundary`, this does not claim that the kept file set is the
exact reachable closure of the projected files in the broad file/module graph.
Instead it states the lower-level properties that the current smaller
TypeScript contract is observed to preserve:

- projected targets keep the same support-root witnesses
- projected targets keep the same reverse-reachable closure

This is the honest formal shape for the current TypeScript `exact_contract(...)`
until it is proved to inhabit a stronger exact-closure boundary.
-/
structure FileProjectionWitnessBoundary
    (R : α → α → Prop)
    (IsRoot : α → Prop) where
  projected : α → Prop
  keep : α → Prop
  preserves_support_roots :
    ∀ x, projected x →
      SupportRootsFor (Induced keep R) IsRoot x =
        SupportRootsFor R IsRoot x
  preserves_reverse_closure :
    ∀ x, projected x →
      ReachableFrom (Induced keep R) x =
        ReachableFrom R x

/--
Under a file-projection witness boundary, projected targets preserve the same
support-root witnesses in the kept subgraph.
-/
theorem support_roots_for_eq_of_file_projection_witness_boundary
    {R : α → α → Prop}
    {IsRoot : α → Prop}
    (boundary : FileProjectionWitnessBoundary R IsRoot)
    {target : α} :
    boundary.projected target →
      SupportRootsFor (Induced boundary.keep R) IsRoot target =
        SupportRootsFor R IsRoot target :=
  boundary.preserves_support_roots target

/--
Under a file-projection witness boundary, projected targets preserve the same
reverse-reachable closure in the kept subgraph.
-/
theorem reachable_from_eq_of_file_projection_witness_boundary
    {R : α → α → Prop}
    {IsRoot : α → Prop}
    (boundary : FileProjectionWitnessBoundary R IsRoot)
    {target : α} :
    boundary.projected target →
      ReachableFrom (Induced boundary.keep R) target =
        ReachableFrom R target :=
  boundary.preserves_reverse_closure target

end ScopedHealth
end SpecialProofs
