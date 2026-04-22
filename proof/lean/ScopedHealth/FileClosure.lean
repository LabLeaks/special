import ScopedHealth.Closure
import ScopedHealth.ReverseClosure

namespace SpecialProofs
namespace ScopedHealth

universe u v

variable {φ : Type u}
variable {β : Type v}

/--
An exact file-closure boundary stated in the same terms as the broad
TypeScript working closure:

- `projected` is the requested file set
- `keep` is the preserved file closure used for scoped analysis
- `keep_exact` says the preserved file set is exactly the reachable closure of
  the requested files in the pack-local file/module graph

This is the theorem-level shape for the broad TypeScript file/module closure,
which reasons about scope at the file graph rather than the item reverse-call
graph.
-/
structure FileClosureBoundary (Imports : φ → φ → Prop) where
  projected : φ → Prop
  keep : φ → Prop
  keep_exact : ∀ f, keep f ↔ Reachable Imports projected f

/--
Under an exact file-closure boundary, every local summary over reachable files
is preserved by analyzing only the induced file closure.
-/
theorem summary_eq_of_exact_file_closure_boundary
    {Imports : φ → φ → Prop}
    (boundary : FileClosureBoundary Imports)
    (Class : φ → β)
    (wanted : β) :
    Summary (Induced boundary.keep Imports) boundary.projected Class wanted =
      Summary Imports boundary.projected Class wanted := by
  exact summary_eq_under_exact_closure Class wanted boundary.keep_exact

/--
Under an exact file-closure boundary, file reachability itself is preserved for
every projected file.
-/
theorem reachable_from_eq_of_exact_file_closure_boundary
    {Imports : φ → φ → Prop}
    (boundary : FileClosureBoundary Imports)
    {file : φ}
    (hfile : boundary.projected file) :
    ReachableFrom (Induced boundary.keep Imports) file =
      ReachableFrom Imports file := by
  exact reachable_from_eq_under_exact_closure hfile boundary.keep_exact

end ScopedHealth
end SpecialProofs
