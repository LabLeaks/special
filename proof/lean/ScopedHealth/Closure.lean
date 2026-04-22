namespace SpecialProofs
namespace ScopedHealth

universe u v

variable {α : Type u}
variable {β : Type v}

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

/--
If `Keep` is exact for the chosen seed set, every original path starting at a
seed can be lifted to the induced relation.
-/
theorem lift_path_to_induced
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    {s x : α}
    (hkeep : ∀ y, Keep y ↔ Reachable R Seed y)
    (hs : Seed s)
    (path : Path R s x) :
    Path (Induced Keep R) s x := by
  induction path with
  | refl =>
      exact Path.refl _
  | tail path hbc ih =>
      have hb_reachable := (show Reachable R Seed _ from ⟨s, hs, path⟩)
      have hc_reachable := reachable_step hb_reachable hbc
      exact Path.tail ih ⟨(hkeep _).2 hb_reachable, (hkeep _).2 hc_reachable, hbc⟩

/--
If `Keep` is exactly the reachable closure of `Seed`, then reachability in the
induced relation is equivalent to reachability in the original relation.
-/
theorem reachable_iff_reachable_induced
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    {x : α}
    (hkeep : ∀ y, Keep y ↔ Reachable R Seed y) :
    Reachable (Induced Keep R) Seed x ↔ Reachable R Seed x := by
  constructor
  · intro hx
    rcases hx with ⟨s, hs, path⟩
    exact ⟨s, hs, strip_induced_path path⟩
  · intro hx
    rcases hx with ⟨s, hs, path⟩
    exact ⟨s, hs, lift_path_to_induced hkeep hs path⟩

/--
A summary bucket is any local classification projected over the reachable set.

This is the abstract shape of health subcategories such as current-spec,
planned-only, unexplained, or statically-mediated items, as long as the bucket
membership is local to the item and the analysis question is "which reachable
items fall into this bucket?"
-/
def Summary
    (R : α → α → Prop)
    (Seed : α → Prop)
    (Class : α → β)
    (wanted : β) :
    α → Prop :=
  fun x => Reachable R Seed x ∧ Class x = wanted

/--
Analyzing the exact closure and then projecting a local summary is equivalent to
analyzing the full graph and projecting the same local summary.
-/
theorem summary_eq_under_exact_closure
    {R : α → α → Prop}
    {Seed : α → Prop}
    {Keep : α → Prop}
    (Class : α → β)
    (wanted : β)
    (hkeep : ∀ y, Keep y ↔ Reachable R Seed y) :
    Summary (Induced Keep R) Seed Class wanted =
      Summary R Seed Class wanted := by
  funext x
  apply propext
  constructor
  · intro hx
    exact ⟨(reachable_iff_reachable_induced hkeep).1 hx.1, hx.2⟩
  · intro hx
    exact ⟨(reachable_iff_reachable_induced hkeep).2 hx.1, hx.2⟩

end ScopedHealth
end SpecialProofs
