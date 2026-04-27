import Lean
import ScopedHealth.ProjectedContractClosure

/-
Proof-facing executable projected traceability kernel.

The JSON CLI imports this module and delegates all traceability decisions to
these definitions. The theorem surface below uses the same target-selection
function as the executable path, so the production kernel and the preservation
statement stay attached to one Lean implementation boundary.
-/

namespace SpecialProofs
namespace ScopedHealth
namespace ProjectedKernel

open Lean

abbrev EdgeMap := List (String × List String)

structure KernelInput where
  projectedItemIds : List String
  explicitTargetIds : Option (List String)
  edges : EdgeMap
  supportRootIds : List String

def schemaVersion : Nat := 1

def contains (items : List String) (item : String) : Bool :=
  items.any (fun candidate => candidate == item)

def insertUnique (item : String) (items : List String) : List String :=
  if contains items item then items else item :: items

def unionUnique (left right : List String) : List String :=
  right.foldl (fun acc item => insertUnique item acc) left

def stringArrayJson (items : List String) : Json :=
  Json.arr <| items.reverse.map Json.str |>.toArray

def stringArrayFromJson (json : Json) : Except String (List String) := do
  let items ← json.getArr?
  items.toList.mapM fun item => item.getStr?

def optionalStringArrayFromJson (json : Json) : Except String (Option (List String)) := do
  match json with
  | Json.null => pure none
  | _ => return some (← stringArrayFromJson json)

def edgeMapFromJson (json : Json) : Except String EdgeMap := do
  let object ← json.getObj?
  object.toList.mapM fun (caller, callees) => do
    return (caller, ← stringArrayFromJson callees)

def parseInput (json : Json) : Except String KernelInput := do
  let schemaVersion ← (← json.getObjVal? "schema_version").getNat?
  if schemaVersion != ProjectedKernel.schemaVersion then
    throw s!"unsupported traceability kernel schema version {schemaVersion}"
  return {
    projectedItemIds := ← stringArrayFromJson (← json.getObjVal? "projected_item_ids")
    explicitTargetIds := ← optionalStringArrayFromJson
      (← json.getObjVal? "preserved_reverse_closure_target_ids")
    edges := ← edgeMapFromJson (← json.getObjVal? "edges")
    supportRootIds := ← stringArrayFromJson (← json.getObjVal? "support_root_ids")
  }

def reverseNeighbors (edges : EdgeMap) (target : String) : List String :=
  edges.foldl
    (fun callers edge =>
      let caller := edge.fst
      let callees := edge.snd
      if contains callees target then insertUnique caller callers else callers)
    []

def edgeNodeIds (edges : EdgeMap) : List String :=
  edges.foldl
    (fun nodes edge =>
      let nodes := insertUnique edge.fst nodes
      unionUnique nodes edge.snd)
    []

def insertPending (visited : List String) (item : String) (pending : List String) : List String :=
  if contains visited item then pending else insertUnique item pending

def unionPending (pending visited items : List String) : List String :=
  items.foldl (fun acc item => insertPending visited item acc) pending

def reverseReachableLoop
    (edges : EdgeMap)
    (fuel : Nat)
    (pending : List String)
    (visited : List String) : Except String (List String) :=
  match fuel with
  | 0 =>
      match pending with
      | [] => pure visited
      | _ =>
          throw s!"traceability kernel reverse reachability fuel exhausted with {pending.length} pending item(s)"
  | fuel + 1 =>
  match pending with
  | [] => pure visited
  | current :: rest =>
      if contains visited current then
        reverseReachableLoop edges fuel rest visited
      else
        let visited := insertUnique current visited
        let rest := unionPending rest visited (reverseNeighbors edges current)
        reverseReachableLoop edges fuel rest visited

def reverseReachable (edges : EdgeMap) (target : String) : Except String (List String) :=
  let pending := reverseNeighbors edges target
  let fuel := (edgeNodeIds edges).length + pending.length
  reverseReachableLoop edges fuel pending []

def supportRootsFor (input : KernelInput) (target : String) : Except String (List String) := do
  let initial :=
    if contains input.supportRootIds target then [target] else []
  let reachable ← reverseReachable input.edges target
  return reachable.foldl
    (fun roots caller =>
      if contains input.supportRootIds caller then insertUnique caller roots else roots)
    initial

def supportedProjectedTargetsLoop
    (input : KernelInput)
    (items targets : List String) : Except String (List String) := do
  match items with
  | [] => pure targets
  | item :: rest =>
      let roots ← supportRootsFor input item
      let targets := if roots.isEmpty then targets else insertUnique item targets
      supportedProjectedTargetsLoop input rest targets

def supportedProjectedTargets (input : KernelInput) : Except String (List String) :=
  supportedProjectedTargetsLoop input input.projectedItemIds []

def targetIds (input : KernelInput) : Except String (List String) :=
  match input.explicitTargetIds with
  | some targets => pure targets
  | none => supportedProjectedTargets input

def reverseClosureNodesLoop
    (edges : EdgeMap)
    (targets nodes : List String) : Except String (List String) := do
  match targets with
  | [] => pure nodes
  | target :: rest =>
      let reachable ← reverseReachable edges target
      let nodes := unionUnique (insertUnique target nodes) reachable
      reverseClosureNodesLoop edges rest nodes

def reverseClosureNodes (edges : EdgeMap) (targets : List String) : Except String (List String) :=
  reverseClosureNodesLoop edges targets []

def internalEdgesJson (edges : EdgeMap) (nodeIds : List String) : Json :=
  let edgeJson :=
    edges.foldl
      (fun entries edge =>
        let caller := edge.fst
        let callees := edge.snd
        if contains nodeIds caller then
          let keptCallees := callees.filter (fun callee => contains nodeIds callee)
          (caller, stringArrayJson keptCallees) :: entries
        else
          entries)
      []
  Json.mkObj edgeJson.reverse

def outputJson (input : KernelInput) : Except String Json := do
  let targetIds ← targetIds input
  let nodeIds ← reverseClosureNodes input.edges targetIds
  return Json.mkObj [
    ("schema_version", toJson schemaVersion),
    ("reference", Json.mkObj [
      ("contract", Json.mkObj [
        ("projected_item_ids", stringArrayJson input.projectedItemIds),
        ("preserved_reverse_closure_target_ids", stringArrayJson targetIds)
      ]),
      ("exact_reverse_closure", Json.mkObj [
        ("target_ids", stringArrayJson targetIds),
        ("node_ids", stringArrayJson nodeIds),
        ("internal_edges", internalEdgesJson input.edges nodeIds)
      ])
    ])
  ]

def run (stdin : String) : Except String String := do
  let input ← parseInput (← Json.parse stdin)
  return (← outputJson input).compress

def calleeIdsFor (edges : EdgeMap) (caller : String) : List String :=
  edges.foldl
    (fun callees edge =>
      if edge.fst == caller then unionUnique callees edge.snd else callees)
    []

def reverseRelation (edges : EdgeMap) : String → String → Prop :=
  fun callee caller => contains (calleeIdsFor edges caller) callee = true

def projectedPredicate (input : KernelInput) : String → Prop :=
  fun item => contains input.projectedItemIds item = true

def targetPredicate (input : KernelInput) : String → Prop :=
  fun item =>
    match targetIds input with
    | Except.ok targets => contains targets item = true
    | Except.error _ => False

def exactKeepPredicate (input : KernelInput) : String → Prop :=
  fun item =>
    projectedPredicate input item ∨
      Reachable (reverseRelation input.edges) (targetPredicate input) item

def projectedKernelBoundary
    (input : KernelInput) :
    ProjectedContractClosureBoundary (reverseRelation input.edges) where
  target := targetPredicate input
  projected := projectedPredicate input
  keep := exactKeepPredicate input
  keep_exact := by
    intro item
    rfl

theorem executable_target_reverse_closure_preserved
    (input : KernelInput)
    {target : String}
    (htarget : targetPredicate input target) :
    ReachableFrom
        (Induced (exactKeepPredicate input) (reverseRelation input.edges))
        target =
      ReachableFrom (reverseRelation input.edges) target := by
  exact reachable_from_eq_of_projected_contract_closure_boundary
    (projectedKernelBoundary input)
    htarget

theorem executable_target_support_roots_preserved
    (input : KernelInput)
    (isRoot : String → Prop)
    {target : String}
    (htarget : targetPredicate input target) :
    SupportRootsFor
        (Induced (exactKeepPredicate input) (reverseRelation input.edges))
        isRoot
        target =
      SupportRootsFor (reverseRelation input.edges) isRoot target := by
  exact support_roots_for_eq_of_projected_contract_closure_boundary
    (projectedKernelBoundary input)
    isRoot
    htarget

end ProjectedKernel
end ScopedHealth
end SpecialProofs
