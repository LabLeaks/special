import ScopedHealth.SupportWitness

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
Abstract boundary for the weaker but product-relevant Rust-style contract.

Unlike `ScopeBoundary`, this does not require `keep` to be the exact reachable
closure of the seed set. Instead it directly states the property that matters
for projected scoped health: for projected targets, the kept subgraph preserves
the same support-root witnesses as the full graph.
-/
structure ProjectionWitnessBoundary
    (R : α → α → Prop)
    (IsRoot : α → Prop) where
  target : α → Prop
  keep : α → Prop
  preserves_support_roots :
    ∀ x, target x →
      SupportRootsFor (Induced keep R) IsRoot x =
        SupportRootsFor R IsRoot x

/--
If projected targets preserve the same support-root witnesses, then any local
summary derived solely from those witnesses is also preserved.

This is the weaker proof shape that lines up with the current Rust adapter more
honestly than `keep_exact`.
-/
theorem support_root_projection_eq_of_boundary
    {R : α → α → Prop}
    {IsRoot : α → Prop}
    (boundary : ProjectionWitnessBoundary R IsRoot)
    {x root : α}
    (htarget : boundary.target x) :
    SupportRootsFor (Induced boundary.keep R) IsRoot x root ↔
      SupportRootsFor R IsRoot x root := by
  simpa using congrFun (boundary.preserves_support_roots x htarget) root

end ScopedHealth
end SpecialProofs
