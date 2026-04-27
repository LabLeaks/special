namespace SpecialProofs
namespace ScopedHealth

universe u

variable {α : Type u}

/--
Reflexive-transitive paths over a relation.

We keep this local instead of depending on a library closure type so the model
stays explicit and self-contained.
-/
inductive Path (R : α → α → Prop) : α → α → Prop where
  | refl (a : α) : Path R a a
  | tail {a b c : α} : Path R a b → R b c → Path R a c

/--
`Reachable R Seed x` means that `x` is reachable from some seed under the
relation `R`.

In scoped health, `R` stands for the backward trace relation over the full
repository graph.
-/
def Reachable (R : α → α → Prop) (Seed : α → Prop) : α → Prop :=
  fun x => ∃ s, Seed s ∧ Path R s x

/--
Restrict a relation to nodes kept by a closure predicate.

This is the abstract shape of "analyze only the exact scoped closure".
-/
def Induced (Keep : α → Prop) (R : α → α → Prop) : α → α → Prop :=
  fun a b => Keep a ∧ Keep b ∧ R a b

theorem reachable_step
    {R : α → α → Prop}
    {Seed : α → Prop}
    {a b : α}
    (ha : Reachable R Seed a)
    (hab : R a b) :
    Reachable R Seed b := by
  rcases ha with ⟨s, hs, path⟩
  exact ⟨s, hs, Path.tail path hab⟩

/--
Every path in the induced relation is also a path in the original relation.
-/
theorem strip_induced_path
    {R : α → α → Prop}
    {Keep : α → Prop}
    {a b : α}
    (path : Path (Induced Keep R) a b) :
    Path R a b := by
  induction path with
  | refl =>
      exact Path.refl _
  | tail path h ih =>
      exact Path.tail ih h.2.2

end ScopedHealth
end SpecialProofs
