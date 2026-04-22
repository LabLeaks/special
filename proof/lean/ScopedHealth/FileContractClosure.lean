import ScopedHealth.ReverseClosure
import ScopedHealth.SupportWitness

namespace SpecialProofs
namespace ScopedHealth

universe u v

variable {α : Type u}
variable {φ : Type v}

/--
An exact TypeScript-style file contract closure boundary.

The kept file set is exactly:

- the projected scoped files
- plus every file that owns an item in the reverse closure of the target set

This is stronger than the earlier witness-only boundary because it states the
kept file set itself, not just downstream consequences.
-/
structure FileContractClosureBoundary
    (R : α → α → Prop)
    (owner : α → φ) where
  target : α → Prop
  projected : φ → Prop
  keep : φ → Prop
  target_projected : ∀ x, target x → projected (owner x)
  keep_exact :
    ∀ f, keep f ↔ projected f ∨ ∃ x, Reachable R target x ∧ owner x = f

/--
Under a file contract-closure boundary, every node reachable from the target
seed set lives in a kept file.
-/
theorem file_contract_keeps_reachable
    {R : α → α → Prop}
    {owner : α → φ}
    (boundary : FileContractClosureBoundary R owner)
    {x : α} :
    Reachable R boundary.target x → boundary.keep (owner x) := by
  intro hx
  exact (boundary.keep_exact _).2 (Or.inr ⟨x, hx, rfl⟩)

/--
Under a file contract-closure boundary, every declared target item itself lies
in a kept file.
-/
theorem file_contract_keeps_target
    {R : α → α → Prop}
    {owner : α → φ}
    (boundary : FileContractClosureBoundary R owner)
    {target : α} :
    boundary.target target → boundary.keep (owner target) := by
  intro htarget
  exact (boundary.keep_exact _).2 (Or.inl (boundary.target_projected _ htarget))

/--
If the kept file set is exactly the file projection of the reverse closure,
then every declared target preserves the same reverse-reachable closure in the
induced kept-item subgraph.
-/
theorem reachable_from_eq_of_file_contract_closure_boundary
    {R : α → α → Prop}
    {owner : α → φ}
    (boundary : FileContractClosureBoundary R owner)
    {target : α}
    (htarget : boundary.target target) :
    ReachableFrom (Induced (fun x => boundary.keep (owner x)) R) target =
      ReachableFrom R target := by
  exact reachable_from_eq_if_keeps_reachable
    htarget
    (fun x hx => file_contract_keeps_reachable boundary hx)

/--
If the kept file set is exactly the file projection of the reverse closure,
then every declared target preserves the same support-root witnesses in the
induced kept-item subgraph.
-/
theorem support_roots_for_eq_of_file_contract_closure_boundary
    {R : α → α → Prop}
    {owner : α → φ}
    (boundary : FileContractClosureBoundary R owner)
    (IsRoot : α → Prop)
    {target : α}
    (htarget : boundary.target target) :
    SupportRootsFor (Induced (fun x => boundary.keep (owner x)) R) IsRoot target =
      SupportRootsFor R IsRoot target := by
  exact support_roots_for_eq_if_keeps_reachable
    IsRoot
    htarget
    (fun x hx => file_contract_keeps_reachable boundary hx)

end ScopedHealth
end SpecialProofs
