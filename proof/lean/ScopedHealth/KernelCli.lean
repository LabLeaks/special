import ScopedHealth.ProjectedKernel

/-
Thin executable adapter for the proof-facing projected traceability kernel.

All traceability decisions live in `ScopedHealth.ProjectedKernel`; this module
only handles process IO and exit behavior.
-/

namespace SpecialProofs
namespace ScopedHealth
namespace KernelCli

def main : IO Unit := do
  let stdin ← IO.getStdin
  let input ← stdin.readToEnd
  match ProjectedKernel.run input with
  | Except.ok output =>
      IO.println output
  | Except.error error =>
      IO.eprintln s!"special traceability kernel error: {error}"
      IO.Process.exit 1

end KernelCli
end ScopedHealth
end SpecialProofs

def main : IO Unit :=
  SpecialProofs.ScopedHealth.KernelCli.main
