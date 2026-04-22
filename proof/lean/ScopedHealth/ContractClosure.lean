import ScopedHealth.Closure
import ScopedHealth.ReverseClosure
import ScopedHealth.SupportWitness

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
An exact closure boundary stated in the same terms as the Rust scoped
traceability contract: a set of preserved targets and the kept subgraph that is
claimed to be exactly the reverse closure of those targets.

This is intentionally narrower and more product-shaped than `ScopeBoundary`.
-/
structure ContractClosureBoundary (R : α → α → Prop) where
  target : α → Prop
  keep : α → Prop
  keep_exact : ∀ x, keep x ↔ Reachable R target x

/--
Under an exact contract closure boundary, the whole per-target reverse closure
 is preserved for every declared target.
-/
theorem reachable_from_eq_of_contract_closure_boundary
    {R : α → α → Prop}
    (boundary : ContractClosureBoundary R)
    {target : α}
    (htarget : boundary.target target) :
    ReachableFrom (Induced boundary.keep R) target =
      ReachableFrom R target := by
  exact reachable_from_eq_under_exact_closure htarget boundary.keep_exact

/--
Under an exact contract closure boundary, support-root witnesses are preserved
for every declared target.
-/
theorem support_roots_for_eq_of_contract_closure_boundary
    {R : α → α → Prop}
    (boundary : ContractClosureBoundary R)
    (IsRoot : α → Prop)
    {target : α}
    (htarget : boundary.target target) :
    SupportRootsFor (Induced boundary.keep R) IsRoot target =
      SupportRootsFor R IsRoot target := by
  exact support_roots_for_eq_under_exact_closure IsRoot htarget boundary.keep_exact

end ScopedHealth
end SpecialProofs
