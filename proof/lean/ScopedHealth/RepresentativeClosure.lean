import ScopedHealth.Closure

namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

theorem append_path
    {R : α → α → Prop}
    {a b c : α}
    (left : Path R a b)
    (right : Path R b c) :
    Path R a c := by
  induction right with
  | refl =>
      exact left
  | tail right h ih =>
      exact Path.tail ih h

/--
A smaller representative target set that preserves the same reachable closure
as the original target set.

The intended Rust reading is:
- `target` is the full supported target set
- `representative` is the smaller exact contract target set
- every original target is reachable from some representative under the chosen
  relation

When that holds, replacing `target` with `representative` does not change the
reachable closure.
-/
structure RepresentativeClosureBoundary (R : α → α → Prop) where
  target : α → Prop
  representative : α → Prop
  representative_subset : ∀ x, representative x → target x
  target_covered : ∀ x, target x → ∃ r, representative r ∧ Path R r x

/--
Replacing the full target set with a representative target set preserves the
same reachable closure when every original target is itself reachable from some
representative.
-/
theorem reachable_eq_of_representative_closure_boundary
    {R : α → α → Prop}
    (boundary : RepresentativeClosureBoundary R) :
    Reachable R boundary.representative = Reachable R boundary.target := by
  funext x
  apply propext
  constructor
  · intro hx
    rcases hx with ⟨s, hs, path⟩
    exact ⟨s, boundary.representative_subset _ hs, path⟩
  · intro hx
    rcases hx with ⟨s, hs, path⟩
    rcases boundary.target_covered _ hs with ⟨r, hr, coverPath⟩
    exact ⟨r, hr, append_path coverPath path⟩

end ScopedHealth
end SpecialProofs
