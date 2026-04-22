/**
@module SPECIAL.MODULES.ANALYZE.TRACEABILITY_CORE
Defines the shared item-evidence traceability IR used by language packs to contribute one combined test-rooted trace graph without hardcoding parser or toolchain details into repo or module projections. This core owns the portable item/test/evidence shape, graph propagation, availability contract, and classification rules, while language-specific adapters decide whether backward trace can run at all and only populate the graph when their required local tool is available.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.TRACEABILITY_CORE
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::model::{
    ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary, ImplementRef, ModuleItemKind,
    ModuleTraceabilityItem, ModuleTraceabilitySummary, ParsedRepo,
};
use crate::syntax::ParsedSourceGraph;

use super::{FileOwnership, display_path};

#[derive(Debug, Clone, Default)]
pub(crate) struct TraceabilityInputs {
    pub(crate) repo_items: Vec<TraceabilityOwnedItem>,
    pub(crate) context_items: Vec<TraceabilityOwnedItem>,
    pub(crate) graph: TraceGraph,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TraceabilityAnalysis {
    pub(crate) repo_items: Vec<TraceabilityOwnedItem>,
    pub(crate) item_supports: BTreeMap<String, Vec<TraceabilityItemSupport>>,
    pub(crate) current_spec_backed_module_ids: BTreeSet<String>,
    pub(crate) module_connected_item_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TraceGraph {
    pub(crate) edges: BTreeMap<String, BTreeSet<String>>,
    pub(crate) root_supports: BTreeMap<String, TraceabilityItemSupport>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ReverseClosureReference {
    pub(crate) target_ids: BTreeSet<String>,
    pub(crate) node_ids: BTreeSet<String>,
    pub(crate) internal_edges: BTreeMap<String, BTreeSet<String>>,
}

/// Shared exact projected-contract proof object used by every language pack.
///
/// The core theorem surface is intentionally language-agnostic:
///
/// - `projected_item_ids` are the output items that must remain visible after
///   scoped analysis is projected back to the user-requested scope
/// - `preserved_reverse_closure_target_ids` are the smaller support-backed seed
///   set whose reverse closure must be kept exactly
///
/// Pack-specific execution projections, such as file closures, stay downstream
/// of this item-kernel contract.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ProjectedTraceabilityContract {
    pub(crate) projected_item_ids: BTreeSet<String>,
    pub(crate) preserved_reverse_closure_target_ids: BTreeSet<String>,
}

/// Shared exact reference object derived from a projected-contract proof
/// object plus the full graph.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ProjectedTraceabilityReference {
    pub(crate) contract: ProjectedTraceabilityContract,
    pub(crate) exact_reverse_closure: ReverseClosureReference,
}

/// Shared proof-protocol adapter for language-pack exact contracts.
///
/// The core does not know language semantics; it only requires that packs can
/// expose their exact scoped proof object in this normalized shape.
#[allow(dead_code)]
pub(crate) trait ProjectedProofProtocol {
    fn projected_contract(&self) -> ProjectedTraceabilityContract;
    fn projected_reference(&self) -> ProjectedTraceabilityReference;
}

impl ProjectedProofProtocol for ProjectedTraceabilityReference {
    fn projected_contract(&self) -> ProjectedTraceabilityContract {
        self.contract.clone()
    }

    fn projected_reference(&self) -> ProjectedTraceabilityReference {
        self.clone()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TraceabilityItemSupport {
    pub(crate) name: String,
    pub(crate) has_item_scoped_support: bool,
    pub(crate) has_file_scoped_support: bool,
    pub(crate) current_specs: BTreeSet<String>,
    pub(crate) planned_specs: BTreeSet<String>,
    pub(crate) deprecated_specs: BTreeSet<String>,
}

impl TraceabilityItemSupport {
    fn merge_into(self, evidence: &mut ItemTraceabilityEvidence) {
        if self.current_specs.is_empty()
            && self.planned_specs.is_empty()
            && self.deprecated_specs.is_empty()
        {
            evidence.unverified_tests.insert(self.name);
            return;
        }

        evidence.verifying_tests.insert(self.name);
        evidence.current_specs.extend(self.current_specs);
        evidence.planned_specs.extend(self.planned_specs);
        evidence.deprecated_specs.extend(self.deprecated_specs);
        evidence.has_item_scoped_support |= self.has_item_scoped_support;
        evidence.has_file_scoped_support |= self.has_file_scoped_support;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TraceabilityOwnedItem {
    pub(crate) stable_id: String,
    pub(crate) kind: ModuleItemKind,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) public: bool,
    pub(crate) review_surface: bool,
    pub(crate) test_file: bool,
    pub(crate) module_ids: Vec<String>,
    pub(crate) mediated_reason: Option<&'static str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BackwardTraceAvailability {
    unavailable_reason: Option<&'static str>,
}

impl BackwardTraceAvailability {
    pub(crate) fn unavailable(reason: &'static str) -> Self {
        Self {
            unavailable_reason: Some(reason),
        }
    }

    pub(crate) fn unavailable_reason(&self) -> Option<&'static str> {
        self.unavailable_reason
    }
}

pub(crate) trait TraceabilityLanguagePack {
    fn backward_trace_availability(&self) -> BackwardTraceAvailability;

    fn owned_items_for_implementations(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &std::collections::BTreeMap<PathBuf, FileOwnership<'_>>,
    ) -> Vec<TraceabilityOwnedItem>;
}

pub(crate) fn build_traceability_analysis(inputs: TraceabilityInputs) -> TraceabilityAnalysis {
    let TraceabilityInputs {
        repo_items,
        context_items,
        graph,
    } = inputs;
    let context_items = if context_items.is_empty() {
        repo_items.clone()
    } else {
        context_items
    };
    let item_supports = collect_item_supports(
        context_items.iter().map(|item| item.stable_id.clone()),
        &graph,
    );
    let current_spec_backed_module_ids =
        collect_current_spec_backed_module_ids(&context_items, &item_supports);
    let module_connected_item_ids = collect_module_connected_item_ids(
        &context_items,
        &graph,
        &item_supports,
        &current_spec_backed_module_ids,
    );

    TraceabilityAnalysis {
        repo_items,
        item_supports,
        current_spec_backed_module_ids,
        module_connected_item_ids,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_projected_traceability_reference(
    contract: ProjectedTraceabilityContract,
    graph: &TraceGraph,
) -> ProjectedTraceabilityReference {
    let exact_reverse_closure = build_reverse_closure_reference(
        contract
            .preserved_reverse_closure_target_ids
            .iter()
            .cloned(),
        graph,
    );
    ProjectedTraceabilityReference {
        contract,
        exact_reverse_closure,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn projected_item_ids_for_inputs(inputs: &TraceabilityInputs) -> BTreeSet<String> {
    inputs
        .repo_items
        .iter()
        .map(|item| item.stable_id.clone())
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn preserved_graph_item_ids_for_reference(
    reference: &ProjectedTraceabilityReference,
) -> BTreeSet<String> {
    reference
        .contract
        .projected_item_ids
        .iter()
        .cloned()
        .chain(reference.exact_reverse_closure.node_ids.iter().cloned())
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn effective_context_item_ids_for_inputs(
    inputs: &TraceabilityInputs,
) -> BTreeSet<String> {
    let context_items = if inputs.context_items.is_empty() {
        &inputs.repo_items
    } else {
        &inputs.context_items
    };
    context_items
        .iter()
        .map(|item| item.stable_id.clone())
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn preserved_item_ids_for_reference<I>(
    reference: &ProjectedTraceabilityReference,
    owned_item_ids: I,
) -> BTreeSet<String>
where
    I: IntoIterator<Item = String>,
{
    let owned_item_ids = owned_item_ids.into_iter().collect::<BTreeSet<_>>();
    preserved_graph_item_ids_for_reference(reference)
        .iter()
        .filter(|item_id| owned_item_ids.contains(*item_id))
        .cloned()
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn projected_support_root_ids_for_inputs(
    inputs: &TraceabilityInputs,
    projected_item_ids: impl IntoIterator<Item = String>,
) -> BTreeMap<String, BTreeSet<String>> {
    collect_support_root_ids(projected_item_ids, &inputs.graph)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn projected_reverse_closure_for_inputs(
    inputs: &TraceabilityInputs,
    target_ids: impl IntoIterator<Item = String>,
) -> ReverseClosureReference {
    build_reverse_closure_reference(target_ids, &inputs.graph)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn collect_support_root_ids<I>(
    item_ids: I,
    graph: &TraceGraph,
) -> BTreeMap<String, BTreeSet<String>>
where
    I: IntoIterator<Item = String>,
{
    let mut reverse_edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (caller, callees) in &graph.edges {
        for callee in callees {
            reverse_edges
                .entry(callee.clone())
                .or_default()
                .insert(caller.clone());
        }
    }

    let mut item_support_roots = BTreeMap::new();
    for item_id in item_ids {
        let mut visited = BTreeSet::new();
        let mut support_roots = if graph.root_supports.contains_key(&item_id) {
            BTreeSet::from([item_id.clone()])
        } else {
            BTreeSet::default()
        };
        let mut pending = reverse_edges
            .get(&item_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();

        while let Some(caller_id) = pending.pop() {
            if !visited.insert(caller_id.clone()) {
                continue;
            }
            if graph.root_supports.contains_key(&caller_id) {
                support_roots.insert(caller_id.clone());
            }
            if let Some(next_callers) = reverse_edges.get(&caller_id) {
                pending.extend(next_callers.iter().cloned());
            }
        }

        item_support_roots.insert(item_id, support_roots);
    }

    item_support_roots
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn collect_reverse_reachable_ids<I>(
    item_ids: I,
    graph: &TraceGraph,
) -> BTreeMap<String, BTreeSet<String>>
where
    I: IntoIterator<Item = String>,
{
    let mut reverse_edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (caller, callees) in &graph.edges {
        for callee in callees {
            reverse_edges
                .entry(callee.clone())
                .or_default()
                .insert(caller.clone());
        }
    }

    item_ids
        .into_iter()
        .map(|item_id| {
            let mut visited = BTreeSet::new();
            let mut pending = reverse_edges
                .get(&item_id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();

            while let Some(caller_id) = pending.pop() {
                if !visited.insert(caller_id.clone()) {
                    continue;
                }
                if let Some(next_callers) = reverse_edges.get(&caller_id) {
                    pending.extend(next_callers.iter().cloned());
                }
            }

            (item_id, visited)
        })
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_reverse_closure_reference<I>(
    item_ids: I,
    graph: &TraceGraph,
) -> ReverseClosureReference
where
    I: IntoIterator<Item = String>,
{
    let target_ids = item_ids.into_iter().collect::<BTreeSet<_>>();
    let mut node_ids = target_ids.clone();
    for reachable in collect_reverse_reachable_ids(target_ids.iter().cloned(), graph).into_values()
    {
        node_ids.extend(reachable);
    }
    let internal_edges = graph
        .edges
        .iter()
        .filter(|(caller, _)| node_ids.contains(*caller))
        .map(|(caller, callees)| {
            (
                caller.clone(),
                callees
                    .iter()
                    .filter(|callee| node_ids.contains(*callee))
                    .cloned()
                    .collect(),
            )
        })
        .collect();

    ReverseClosureReference {
        target_ids,
        node_ids,
        internal_edges,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        ProjectedTraceabilityContract, ProjectedTraceabilityReference, ReverseClosureReference,
        TraceGraph, TraceabilityItemSupport, collect_support_root_ids,
        normalize_path_for_known_sources, preserved_graph_item_ids_for_reference,
        preserved_item_ids_for_reference,
    };
    use crate::model::{
        ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary, ModuleItemKind,
    };

    #[test]
    fn preserved_graph_item_ids_include_projected_items_outside_reverse_closure() {
        let reference = ProjectedTraceabilityReference {
            contract: ProjectedTraceabilityContract {
                projected_item_ids: ["projected::orphan".to_string()].into_iter().collect(),
                preserved_reverse_closure_target_ids: ["target".to_string()].into_iter().collect(),
            },
            exact_reverse_closure: ReverseClosureReference {
                target_ids: ["target".to_string()].into_iter().collect(),
                node_ids: ["target".to_string(), "test::root".to_string()]
                    .into_iter()
                    .collect(),
                internal_edges: BTreeMap::new(),
            },
        };

        assert_eq!(
            preserved_graph_item_ids_for_reference(&reference),
            [
                "projected::orphan".to_string(),
                "target".to_string(),
                "test::root".to_string()
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            preserved_item_ids_for_reference(
                &reference,
                ["projected::orphan".to_string(), "target".to_string()].into_iter(),
            ),
            ["projected::orphan".to_string(), "target".to_string()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn support_root_collection_is_reflexive_for_root_targets() {
        let graph = TraceGraph {
            edges: BTreeMap::new(),
            root_supports: [(
                "test::root".to_string(),
                TraceabilityItemSupport {
                    name: "test::root".to_string(),
                    has_item_scoped_support: true,
                    has_file_scoped_support: false,
                    current_specs: ["APP.ROOT".to_string()].into_iter().collect(),
                    planned_specs: BTreeSet::new(),
                    deprecated_specs: BTreeSet::new(),
                },
            )]
            .into_iter()
            .collect(),
        };

        assert_eq!(
            collect_support_root_ids(["test::root".to_string()], &graph),
            [(
                "test::root".to_string(),
                ["test::root".to_string()].into_iter().collect(),
            )]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn architecture_traceability_item_json_omits_internal_stable_id() {
        let summary = ArchitectureTraceabilitySummary {
            analyzed_items: 1,
            unexplained_items: vec![ArchitectureTraceabilityItem {
                path: "src/lib.rs".into(),
                line: 7,
                name: "demo::run".to_string(),
                kind: ModuleItemKind::Function,
                public: true,
                review_surface: true,
                test_file: false,
                module_backed_by_current_specs: true,
                module_connected_to_current_specs: true,
                module_ids: vec!["APP.CORE".to_string()],
                mediated_reason: None,
                verifying_tests: Vec::new(),
                unverified_tests: Vec::new(),
                current_specs: vec!["APP.RUN".to_string()],
                planned_specs: Vec::new(),
                deprecated_specs: Vec::new(),
            }],
            ..ArchitectureTraceabilitySummary::default()
        };

        let json = serde_json::to_value(summary).expect("traceability summary should serialize");
        let item = &json["unexplained_items"][0];
        assert!(
            item.get("stable_id").is_none(),
            "internal stable ids should not be exported in traceability JSON: {item:?}"
        );
        assert_eq!(item.get("line").and_then(|value| value.as_u64()), Some(7));
    }

    #[test]
    fn normalize_path_for_known_sources_prefers_unique_longest_suffix_match() {
        let normalized = normalize_path_for_known_sources(
            std::path::Path::new("/tmp/repo/go/app/main.go"),
            &["app/main.go".into(), "go/app/main.go".into()],
        );

        assert_eq!(normalized, std::path::PathBuf::from("go/app/main.go"));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpecLifecycle {
    Current,
    Planned,
    Deprecated,
}

#[derive(Debug, Clone, Default)]
struct SpecStateBuckets {
    current_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
}

pub(crate) fn build_root_supports(
    parsed_repo: &ParsedRepo,
    source_graphs: &BTreeMap<PathBuf, ParsedSourceGraph>,
    parse_body_start_line: impl Fn(&Path, &str) -> Option<usize>,
) -> BTreeMap<String, TraceabilityItemSupport> {
    let known_source_paths = source_graphs.keys().cloned().collect::<Vec<_>>();
    let spec_states = parsed_repo
        .specs
        .iter()
        .map(|spec| {
            let state = if spec.is_planned() {
                SpecLifecycle::Planned
            } else if spec.is_deprecated() {
                SpecLifecycle::Deprecated
            } else {
                SpecLifecycle::Current
            };
            (spec.id.clone(), state)
        })
        .collect::<BTreeMap<_, _>>();
    let (verify_by_item, verify_by_file) = build_verify_indexes(
        parsed_repo,
        &spec_states,
        &known_source_paths,
        parse_body_start_line,
    );

    let mut root_supports = BTreeMap::new();
    for (path, graph) in source_graphs {
        let file_specs = verify_by_file.get(path).cloned().unwrap_or_default();
        for item in graph.items.iter().filter(|item| item.is_test) {
            let item_specs = verify_by_item
                .get(&(path.clone(), item.span.start_line))
                .cloned()
                .unwrap_or_default();
            root_supports.insert(
                item.stable_id.clone(),
                TraceabilityItemSupport {
                    name: item.name.clone(),
                    has_item_scoped_support: !item_specs.current_specs.is_empty()
                        || !item_specs.planned_specs.is_empty()
                        || !item_specs.deprecated_specs.is_empty(),
                    has_file_scoped_support: !file_specs.current_specs.is_empty()
                        || !file_specs.planned_specs.is_empty()
                        || !file_specs.deprecated_specs.is_empty(),
                    current_specs: item_specs
                        .current_specs
                        .union(&file_specs.current_specs)
                        .cloned()
                        .collect(),
                    planned_specs: item_specs
                        .planned_specs
                        .union(&file_specs.planned_specs)
                        .cloned()
                        .collect(),
                    deprecated_specs: item_specs
                        .deprecated_specs
                        .union(&file_specs.deprecated_specs)
                        .cloned()
                        .collect(),
                },
            );
        }
    }

    root_supports
}

pub(crate) fn merge_trace_graph_edges(
    target: &mut BTreeMap<String, BTreeSet<String>>,
    extra: BTreeMap<String, BTreeSet<String>>,
) {
    for (caller, callees) in extra {
        target.entry(caller).or_default().extend(callees);
    }
}

pub(crate) fn owned_module_ids_for_path(
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    path: &Path,
) -> Vec<String> {
    let Some(ownership) = file_ownership.get(path) else {
        return Vec::new();
    };
    ownership
        .file_scoped
        .iter()
        .chain(ownership.item_scoped.iter())
        .map(|implementation| implementation.module_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Debug, Default)]
struct ItemTraceabilityEvidence {
    verifying_tests: BTreeSet<String>,
    unverified_tests: BTreeSet<String>,
    current_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
    has_item_scoped_support: bool,
    has_file_scoped_support: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemTraceabilityCategory {
    CurrentSpec,
    PlannedOnly,
    DeprecatedOnly,
    UnverifiedTest,
    StaticallyMediated,
    Unexplained,
}

pub(crate) fn summarize_module_traceability(
    owned_items: &[TraceabilityOwnedItem],
    analysis: &TraceabilityAnalysis,
) -> ModuleTraceabilitySummary {
    let mut summary = ModuleTraceabilitySummary {
        analyzed_items: owned_items.len(),
        ..ModuleTraceabilitySummary::default()
    };

    for item in owned_items {
        let evidence = collect_traceability_evidence(item, analysis);
        let category = classify_item_traceability_category(item, &evidence);
        push_module_traceability_item(
            &mut summary,
            ModuleTraceabilityItem {
                line: stable_id_line(&item.stable_id),
                name: item.name.clone(),
                kind: item.kind,
                mediated_reason: item.mediated_reason.map(ToString::to_string),
                verifying_tests: evidence.verifying_tests.into_iter().collect(),
                unverified_tests: evidence.unverified_tests.into_iter().collect(),
                current_specs: evidence.current_specs.into_iter().collect(),
                planned_specs: evidence.planned_specs.into_iter().collect(),
                deprecated_specs: evidence.deprecated_specs.into_iter().collect(),
            },
            category,
            evidence.has_file_scoped_support && !evidence.has_item_scoped_support,
        );
    }

    summary.sort_items();
    summary
}

pub(crate) fn summarize_repo_traceability(
    root: &Path,
    analysis: &TraceabilityAnalysis,
) -> ArchitectureTraceabilitySummary {
    let mut summary = ArchitectureTraceabilitySummary {
        analyzed_items: analysis.repo_items.len(),
        ..ArchitectureTraceabilitySummary::default()
    };

    for item in &analysis.repo_items {
        let evidence = collect_traceability_evidence(item, analysis);
        let category = classify_item_traceability_category(item, &evidence);
        push_architecture_traceability_item(
            &mut summary,
            ArchitectureTraceabilityItem {
                path: display_path(root, &item.path),
                line: stable_id_line(&item.stable_id),
                name: item.name.clone(),
                kind: item.kind,
                public: item.public,
                review_surface: item.review_surface,
                test_file: item.test_file,
                module_backed_by_current_specs: is_module_backed_by_current_specs(item, analysis),
                module_connected_to_current_specs: is_module_connected_to_current_specs(
                    item, analysis,
                ),
                module_ids: item.module_ids.clone(),
                mediated_reason: item.mediated_reason.map(ToString::to_string),
                verifying_tests: evidence.verifying_tests.into_iter().collect(),
                unverified_tests: evidence.unverified_tests.into_iter().collect(),
                current_specs: evidence.current_specs.into_iter().collect(),
                planned_specs: evidence.planned_specs.into_iter().collect(),
                deprecated_specs: evidence.deprecated_specs.into_iter().collect(),
            },
            category,
            evidence.has_file_scoped_support && !evidence.has_item_scoped_support,
        );
    }

    summary.sort_items();
    summary
}

fn stable_id_line(stable_id: &str) -> usize {
    stable_id
        .rsplit(':')
        .next()
        .and_then(|line| line.parse::<usize>().ok())
        .unwrap_or(0)
}

fn push_module_traceability_item(
    summary: &mut ModuleTraceabilitySummary,
    item: ModuleTraceabilityItem,
    category: ItemTraceabilityCategory,
    file_scoped_only: bool,
) {
    if file_scoped_only {
        summary.file_scoped_only_items.push(item.clone());
    }

    match category {
        ItemTraceabilityCategory::CurrentSpec => summary.current_spec_items.push(item),
        ItemTraceabilityCategory::PlannedOnly => summary.planned_only_items.push(item),
        ItemTraceabilityCategory::DeprecatedOnly => summary.deprecated_only_items.push(item),
        ItemTraceabilityCategory::UnverifiedTest => summary.unverified_test_items.push(item),
        ItemTraceabilityCategory::StaticallyMediated => {
            summary.statically_mediated_items.push(item);
        }
        ItemTraceabilityCategory::Unexplained => summary.unexplained_items.push(item),
    }
}

fn push_architecture_traceability_item(
    summary: &mut ArchitectureTraceabilitySummary,
    item: ArchitectureTraceabilityItem,
    category: ItemTraceabilityCategory,
    file_scoped_only: bool,
) {
    if file_scoped_only {
        summary.file_scoped_only_items.push(item.clone());
    }

    match category {
        ItemTraceabilityCategory::CurrentSpec => summary.current_spec_items.push(item),
        ItemTraceabilityCategory::PlannedOnly => summary.planned_only_items.push(item),
        ItemTraceabilityCategory::DeprecatedOnly => summary.deprecated_only_items.push(item),
        ItemTraceabilityCategory::UnverifiedTest => summary.unverified_test_items.push(item),
        ItemTraceabilityCategory::StaticallyMediated => {
            summary.statically_mediated_items.push(item);
        }
        ItemTraceabilityCategory::Unexplained => {
            summary.unexplained_items.push(item);
        }
    }
}

fn collect_traceability_evidence(
    item: &TraceabilityOwnedItem,
    analysis: &TraceabilityAnalysis,
) -> ItemTraceabilityEvidence {
    let mut evidence = ItemTraceabilityEvidence::default();
    if let Some(supports) = analysis.item_supports.get(&item.stable_id) {
        for support in supports {
            support.clone().merge_into(&mut evidence);
        }
    }
    evidence
}

fn build_verify_indexes(
    parsed_repo: &ParsedRepo,
    spec_states: &BTreeMap<String, SpecLifecycle>,
    known_source_paths: &[PathBuf],
    parse_body_start_line: impl Fn(&Path, &str) -> Option<usize>,
) -> (
    BTreeMap<(PathBuf, usize), SpecStateBuckets>,
    BTreeMap<PathBuf, SpecStateBuckets>,
) {
    let mut by_item = BTreeMap::new();
    let mut by_file = BTreeMap::new();

    for verify in &parsed_repo.verifies {
        let Some(state) = spec_states.get(&verify.spec_id).copied() else {
            continue;
        };
        if let Some(body_location) = &verify.body_location {
            let normalized_path =
                normalize_support_path_for_known_sources(&body_location.path, known_source_paths);
            let resolved_line = verify
                .body
                .as_deref()
                .and_then(|body| parse_body_start_line(&normalized_path, body))
                .map(|start_line| body_location.line + start_line - 1)
                .unwrap_or(body_location.line);
            for target_line in
                body_location.line.min(resolved_line)..=body_location.line.max(resolved_line)
            {
                accumulate_spec_state(
                    by_item
                        .entry((normalized_path.clone(), target_line))
                        .or_default(),
                    &verify.spec_id,
                    state,
                );
            }
        } else {
            let normalized_path =
                normalize_support_path_for_known_sources(&verify.location.path, known_source_paths);
            accumulate_spec_state(
                by_file.entry(normalized_path).or_default(),
                &verify.spec_id,
                state,
            );
        }
    }

    (by_item, by_file)
}

fn normalize_support_path_for_known_sources(
    path: &Path,
    known_source_paths: &[PathBuf],
) -> PathBuf {
    normalize_path_for_known_sources(path, known_source_paths)
}

pub(crate) fn normalize_path_for_known_sources(
    path: &Path,
    known_source_paths: &[PathBuf],
) -> PathBuf {
    let normalized_path = strip_private_var_prefix(path);
    if let Some(exact_match) = known_source_paths.iter().find(|known| {
        let normalized_known = strip_private_var_prefix(known);
        normalized_path == known.as_path()
            || normalized_path == normalized_known.as_path()
            || path == known.as_path()
    }) {
        return exact_match.clone();
    }

    let suffix_matches = known_source_paths
        .iter()
        .filter_map(|known| {
            let normalized_known = strip_private_var_prefix(known);
            let matches = normalized_path.ends_with(known.as_path())
                || normalized_path.ends_with(normalized_known.as_path());
            matches.then(|| {
                (
                    known.clone(),
                    known
                        .components()
                        .count()
                        .max(normalized_known.components().count()),
                )
            })
        })
        .collect::<Vec<_>>();

    let Some(longest_match_len) = suffix_matches.iter().map(|(_, len)| *len).max() else {
        return normalized_path;
    };

    let longest_matches = suffix_matches
        .into_iter()
        .filter(|(_, len)| *len == longest_match_len)
        .map(|(path, _)| path)
        .collect::<Vec<_>>();

    if longest_matches.len() == 1 {
        longest_matches
            .into_iter()
            .next()
            .expect("single longest suffix match should exist")
    } else {
        normalized_path
    }
}

fn strip_private_var_prefix(path: &Path) -> PathBuf {
    path.strip_prefix("/private")
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn accumulate_spec_state(buckets: &mut SpecStateBuckets, spec_id: &str, state: SpecLifecycle) {
    match state {
        SpecLifecycle::Current => {
            buckets.current_specs.insert(spec_id.to_string());
        }
        SpecLifecycle::Planned => {
            buckets.planned_specs.insert(spec_id.to_string());
        }
        SpecLifecycle::Deprecated => {
            buckets.deprecated_specs.insert(spec_id.to_string());
        }
    }
}

fn is_module_backed_by_current_specs(
    item: &TraceabilityOwnedItem,
    analysis: &TraceabilityAnalysis,
) -> bool {
    item.module_ids
        .iter()
        .any(|module_id| analysis.current_spec_backed_module_ids.contains(module_id))
}

fn is_module_connected_to_current_specs(
    item: &TraceabilityOwnedItem,
    analysis: &TraceabilityAnalysis,
) -> bool {
    analysis.module_connected_item_ids.contains(&item.stable_id)
}

fn collect_item_supports<I>(
    item_ids: I,
    graph: &TraceGraph,
) -> BTreeMap<String, Vec<TraceabilityItemSupport>>
where
    I: IntoIterator<Item = String>,
{
    let mut reverse_edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (caller, callees) in &graph.edges {
        for callee in callees {
            reverse_edges
                .entry(callee.clone())
                .or_default()
                .insert(caller.clone());
        }
    }

    let mut item_supports = BTreeMap::new();
    for item_id in item_ids {
        let mut visited = BTreeSet::new();
        let mut pending = reverse_edges
            .get(&item_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        let mut supports = Vec::new();

        while let Some(caller_id) = pending.pop() {
            if !visited.insert(caller_id.clone()) {
                continue;
            }
            if let Some(support) = graph.root_supports.get(&caller_id) {
                supports.push(support.clone());
            }
            if let Some(next_callers) = reverse_edges.get(&caller_id) {
                pending.extend(next_callers.iter().cloned());
            }
        }

        if !supports.is_empty() {
            supports.sort_by(|left, right| left.name.cmp(&right.name));
            item_supports.insert(item_id, supports);
        }
    }

    item_supports
}

fn collect_current_spec_backed_module_ids(
    repo_items: &[TraceabilityOwnedItem],
    item_supports: &BTreeMap<String, Vec<TraceabilityItemSupport>>,
) -> BTreeSet<String> {
    repo_items
        .iter()
        .filter(|item| {
            item_supports.get(&item.stable_id).is_some_and(|supports| {
                supports
                    .iter()
                    .any(|support| !support.current_specs.is_empty())
            })
        })
        .flat_map(|item| item.module_ids.iter().cloned())
        .collect()
}

fn collect_module_connected_item_ids(
    repo_items: &[TraceabilityOwnedItem],
    graph: &TraceGraph,
    item_supports: &BTreeMap<String, Vec<TraceabilityItemSupport>>,
    current_spec_backed_module_ids: &BTreeSet<String>,
) -> BTreeSet<String> {
    let item_modules = repo_items
        .iter()
        .map(|item| {
            let current_modules = item
                .module_ids
                .iter()
                .filter(|module_id| current_spec_backed_module_ids.contains(*module_id))
                .cloned()
                .collect::<BTreeSet<_>>();
            (item.stable_id.clone(), current_modules)
        })
        .collect::<BTreeMap<_, _>>();

    let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for item in repo_items {
        if item_modules
            .get(&item.stable_id)
            .is_some_and(|modules| !modules.is_empty())
        {
            adjacency.entry(item.stable_id.clone()).or_default();
        }
    }

    for (caller, callees) in &graph.edges {
        let Some(caller_modules) = item_modules.get(caller) else {
            continue;
        };
        if caller_modules.is_empty() {
            continue;
        }
        for callee in callees {
            let Some(callee_modules) = item_modules.get(callee) else {
                continue;
            };
            if caller_modules.is_disjoint(callee_modules) {
                continue;
            }
            adjacency
                .entry(caller.clone())
                .or_default()
                .insert(callee.clone());
            adjacency
                .entry(callee.clone())
                .or_default()
                .insert(caller.clone());
        }
    }

    let mut connected = BTreeSet::new();
    let mut pending = repo_items
        .iter()
        .filter(|item| {
            item_supports.get(&item.stable_id).is_some_and(|supports| {
                supports
                    .iter()
                    .any(|support| !support.current_specs.is_empty())
            })
        })
        .filter(|item| {
            item_modules
                .get(&item.stable_id)
                .is_some_and(|modules| !modules.is_empty())
        })
        .map(|item| item.stable_id.clone())
        .collect::<Vec<_>>();

    while let Some(item_id) = pending.pop() {
        if !connected.insert(item_id.clone()) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&item_id) {
            pending.extend(neighbors.iter().cloned());
        }
    }

    connected
}

fn classify_item_traceability_category(
    item: &TraceabilityOwnedItem,
    evidence: &ItemTraceabilityEvidence,
) -> ItemTraceabilityCategory {
    if !evidence.current_specs.is_empty() {
        ItemTraceabilityCategory::CurrentSpec
    } else if !evidence.planned_specs.is_empty() {
        ItemTraceabilityCategory::PlannedOnly
    } else if !evidence.deprecated_specs.is_empty() {
        ItemTraceabilityCategory::DeprecatedOnly
    } else if !evidence.unverified_tests.is_empty() {
        ItemTraceabilityCategory::UnverifiedTest
    } else if item.mediated_reason.is_some() {
        ItemTraceabilityCategory::StaticallyMediated
    } else {
        ItemTraceabilityCategory::Unexplained
    }
}
