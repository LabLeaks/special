#[cfg(test)]
use std::cell::Cell;
/**
@module SPECIAL.MODULES.ANALYZE.TRACEABILITY_CORE
Defines the shared item-evidence traceability IR used by language packs to contribute one combined test-rooted trace graph without hardcoding parser or toolchain details into repo or module projections. This core owns the portable item/test/evidence shape, graph propagation, availability contract, and classification rules, while language-specific adapters decide whether backward trace can run at all and only populate the graph when their required local tool is available.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.TRACEABILITY_CORE
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use flate2::read::GzDecoder;

use crate::model::{
    ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary, ImplementRef, ModuleItemKind,
    ModuleTraceabilityItem, ModuleTraceabilitySummary, ParsedRepo,
};
use crate::syntax::ParsedSourceGraph;

use super::{FileOwnership, display_path};

#[cfg(test)]
thread_local! {
    static RUST_REFERENCE_KERNEL_TEST_OPT_IN: Cell<bool> = const { Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn use_rust_reference_traceability_kernel_for_tests() {
    RUST_REFERENCE_KERNEL_TEST_OPT_IN.with(|opt_in| opt_in.set(true));
}

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
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectedTraceabilityContract {
    pub(crate) projected_item_ids: BTreeSet<String>,
    pub(crate) preserved_reverse_closure_target_ids: BTreeSet<String>,
}

/// Shared exact reference object derived from a projected-contract proof
/// object plus the full graph.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectedTraceabilityReference {
    pub(crate) contract: ProjectedTraceabilityContract,
    pub(crate) exact_reverse_closure: ReverseClosureReference,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectedTraceabilityKernelInput {
    pub(crate) schema_version: u32,
    pub(crate) projected_item_ids: BTreeSet<String>,
    pub(crate) preserved_reverse_closure_target_ids: Option<BTreeSet<String>>,
    pub(crate) edges: BTreeMap<String, BTreeSet<String>>,
    pub(crate) support_root_ids: BTreeSet<String>,
}

impl ProjectedTraceabilityKernelInput {
    pub(crate) const SCHEMA_VERSION: u32 = 1;

    fn from_projected_items_and_graph(
        projected_item_ids: BTreeSet<String>,
        graph: &TraceGraph,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            projected_item_ids,
            preserved_reverse_closure_target_ids: None,
            edges: graph.edges.clone(),
            support_root_ids: graph.root_supports.keys().cloned().collect(),
        }
    }

    fn from_contract_and_graph(
        contract: ProjectedTraceabilityContract,
        graph: &TraceGraph,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            projected_item_ids: contract.projected_item_ids,
            preserved_reverse_closure_target_ids: Some(
                contract.preserved_reverse_closure_target_ids,
            ),
            edges: graph.edges.clone(),
            support_root_ids: graph.root_supports.keys().cloned().collect(),
        }
    }

    fn as_trace_graph(&self) -> TraceGraph {
        TraceGraph {
            edges: self.edges.clone(),
            root_supports: self
                .support_root_ids
                .iter()
                .map(|support_root_id| {
                    (
                        support_root_id.clone(),
                        TraceabilityItemSupport::kernel_placeholder(support_root_id),
                    )
                })
                .collect(),
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectedTraceabilityKernelOutput {
    pub(crate) schema_version: u32,
    pub(crate) reference: ProjectedTraceabilityReference,
}

/// Shared projected traceability kernel.
///
/// Language packs adapt facts into `TraceGraph` plus projected item ids, then
/// call this boundary instead of reimplementing the support-root and
/// reverse-closure rules pack-by-pack. Production execution must go through the
/// Lean kernel; the Rust implementation is a debug/test oracle only.
pub(crate) trait ProjectedTraceabilityKernel {
    fn derive(
        &self,
        input: ProjectedTraceabilityKernelInput,
    ) -> Result<ProjectedTraceabilityKernelOutput, String>;
}

#[derive(Debug, Clone, Copy)]
struct RustReferenceTraceabilityKernel;

impl ProjectedTraceabilityKernel for RustReferenceTraceabilityKernel {
    fn derive(
        &self,
        input: ProjectedTraceabilityKernelInput,
    ) -> Result<ProjectedTraceabilityKernelOutput, String> {
        if input.schema_version != ProjectedTraceabilityKernelInput::SCHEMA_VERSION {
            return Err(format!(
                "unsupported traceability kernel schema version {}",
                input.schema_version
            ));
        }
        let graph = input.as_trace_graph();
        let preserved_reverse_closure_target_ids = input
            .preserved_reverse_closure_target_ids
            .unwrap_or_else(|| {
                collect_support_root_ids(input.projected_item_ids.iter().cloned(), &graph)
                    .into_iter()
                    .filter_map(|(item_id, supports)| (!supports.is_empty()).then_some(item_id))
                    .collect()
            });

        let contract = ProjectedTraceabilityContract {
            projected_item_ids: input.projected_item_ids,
            preserved_reverse_closure_target_ids,
        };
        let exact_reverse_closure = build_reverse_closure_reference(
            contract
                .preserved_reverse_closure_target_ids
                .iter()
                .cloned(),
            &graph,
        );
        let reference = ProjectedTraceabilityReference {
            contract,
            exact_reverse_closure,
        };

        Ok(ProjectedTraceabilityKernelOutput {
            schema_version: ProjectedTraceabilityKernelInput::SCHEMA_VERSION,
            reference,
        })
    }
}

static RUST_REFERENCE_TRACEABILITY_KERNEL: RustReferenceTraceabilityKernel =
    RustReferenceTraceabilityKernel;

#[derive(Debug, Clone, Copy)]
struct LeanProcessTraceabilityKernel;

impl ProjectedTraceabilityKernel for LeanProcessTraceabilityKernel {
    fn derive(
        &self,
        input: ProjectedTraceabilityKernelInput,
    ) -> Result<ProjectedTraceabilityKernelOutput, String> {
        derive_with_lean_process(input)
    }
}

fn derive_projected_traceability_kernel(
    input: ProjectedTraceabilityKernelInput,
) -> Result<ProjectedTraceabilityKernelOutput, String> {
    match env::var("SPECIAL_TRACEABILITY_KERNEL") {
        Ok(value) => derive_projected_traceability_kernel_for_selector(input, Some(value.as_str())),
        Err(env::VarError::NotPresent) => {
            derive_projected_traceability_kernel_for_selector(input, None)
        }
        Err(error) => Err(format!(
            "could not read SPECIAL_TRACEABILITY_KERNEL: {error}"
        )),
    }
}

fn derive_projected_traceability_kernel_for_selector(
    input: ProjectedTraceabilityKernelInput,
    selector: Option<&str>,
) -> Result<ProjectedTraceabilityKernelOutput, String> {
    match selector {
        Some("rust-reference") => derive_rust_reference_traceability_kernel(input),
        Some(value) if value.is_empty() || value == "lean" => {
            LeanProcessTraceabilityKernel.derive(input)
        }
        Some(value) => Err(format!(
            "unsupported SPECIAL_TRACEABILITY_KERNEL value `{value}`; expected `lean` or debug/test-only `rust-reference`"
        )),
        None => {
            #[cfg(test)]
            {
                if RUST_REFERENCE_KERNEL_TEST_OPT_IN.with(Cell::get) {
                    return RUST_REFERENCE_TRACEABILITY_KERNEL.derive(input);
                }
            }
            LeanProcessTraceabilityKernel.derive(input)
        }
    }
}

fn derive_rust_reference_traceability_kernel(
    input: ProjectedTraceabilityKernelInput,
) -> Result<ProjectedTraceabilityKernelOutput, String> {
    if cfg!(debug_assertions) {
        RUST_REFERENCE_TRACEABILITY_KERNEL.derive(input)
    } else {
        Err(
            "SPECIAL_TRACEABILITY_KERNEL=rust-reference is only available in debug/test builds; production traceability requires the Lean kernel"
                .to_string(),
        )
    }
}

fn derive_with_lean_process(
    input: ProjectedTraceabilityKernelInput,
) -> Result<ProjectedTraceabilityKernelOutput, String> {
    let payload = serde_json::to_string(&input)
        .map_err(|error| format!("could not encode Lean kernel input: {error}"))?;
    let mut command = lean_kernel_command()?;
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start Lean traceability kernel: {error}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "Lean traceability kernel stdin was unavailable".to_string())?
        .write_all(payload.as_bytes())
        .map_err(|error| format!("could not send input to Lean traceability kernel: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("could not read Lean traceability kernel output: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Lean traceability kernel exited with {}; stderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("Lean traceability kernel emitted non-UTF-8 output: {error}"))?;
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .ok_or_else(|| format!("Lean traceability kernel did not emit JSON output:\n{stdout}"))?;
    serde_json::from_str(json_line)
        .map_err(|error| format!("could not decode Lean traceability kernel output: {error}"))
}

fn lean_kernel_command() -> Result<Command, String> {
    if let Ok(path) = env::var("SPECIAL_TRACEABILITY_KERNEL_EXE") {
        if path.is_empty() {
            return Err("SPECIAL_TRACEABILITY_KERNEL_EXE was set to an empty path".to_string());
        }
        return Ok(Command::new(path));
    }

    if let Some(path) = embedded_lean_kernel_executable()? {
        return Ok(Command::new(path));
    }

    Err(
        "Lean traceability kernel was requested, but this binary was not built with \
         SPECIAL_BUILD_LEAN_KERNEL=1 and SPECIAL_TRACEABILITY_KERNEL_EXE was not set"
            .to_string(),
    )
}

#[cfg(special_embedded_lean_kernel)]
fn embedded_lean_kernel_bytes() -> &'static [u8] {
    include_bytes!(env!("SPECIAL_EMBEDDED_LEAN_KERNEL_PATH"))
}

#[cfg(not(special_embedded_lean_kernel))]
fn embedded_lean_kernel_bytes() -> &'static [u8] {
    &[]
}

#[cfg(special_embedded_lean_kernel)]
fn embedded_lean_kernel_filename() -> &'static str {
    env!("SPECIAL_EMBEDDED_LEAN_KERNEL_FILENAME")
}

#[cfg(not(special_embedded_lean_kernel))]
fn embedded_lean_kernel_filename() -> &'static str {
    ""
}

#[cfg(special_embedded_lean_kernel)]
fn embedded_lean_kernel_uncompressed_len() -> u64 {
    env!("SPECIAL_EMBEDDED_LEAN_KERNEL_UNCOMPRESSED_LEN")
        .parse()
        .expect("embedded Lean kernel uncompressed length should be a u64")
}

#[cfg(not(special_embedded_lean_kernel))]
fn embedded_lean_kernel_uncompressed_len() -> u64 {
    0
}

fn embedded_lean_kernel_executable() -> Result<Option<PathBuf>, String> {
    let bytes = embedded_lean_kernel_bytes();
    if bytes.is_empty() {
        return Ok(None);
    }
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let kernel_hash = hasher.finish();

    let path = env::temp_dir().join(format!(
        "special-{}-{kernel_hash:016x}-{}",
        env!("CARGO_PKG_VERSION"),
        embedded_lean_kernel_filename()
    ));
    let expected_len = embedded_lean_kernel_uncompressed_len();
    if fs::metadata(&path)
        .map(|metadata| metadata.len() == expected_len)
        .unwrap_or(false)
    {
        return Ok(Some(path));
    }
    let temp_path = path.with_extension(format!("tmp-{}", std::process::id()));
    let decompressed = decompress_embedded_lean_kernel(bytes, expected_len)?;
    fs::write(&temp_path, decompressed).map_err(|error| {
        format!(
            "could not materialize embedded Lean traceability kernel at {}: {error}",
            temp_path.display()
        )
    })?;
    make_executable(&temp_path)?;
    if let Err(error) = fs::rename(&temp_path, &path) {
        if fs::metadata(&path)
            .map(|metadata| metadata.len() == expected_len)
            .unwrap_or(false)
        {
            let _ = fs::remove_file(&temp_path);
            return Ok(Some(path));
        }
        return Err(format!(
            "could not install embedded Lean traceability kernel at {}: {error}",
            path.display()
        ));
    }
    Ok(Some(path))
}

fn decompress_embedded_lean_kernel(bytes: &[u8], expected_len: u64) -> Result<Vec<u8>, String> {
    let mut decoder = GzDecoder::new(bytes);
    let mut decompressed = Vec::with_capacity(expected_len.min(usize::MAX as u64) as usize);
    decoder.read_to_end(&mut decompressed).map_err(|error| {
        format!("could not decompress embedded Lean traceability kernel: {error}")
    })?;
    if decompressed.len() as u64 != expected_len {
        return Err(format!(
            "embedded Lean traceability kernel decompressed to {} bytes, expected {expected_len}",
            decompressed.len()
        ));
    }
    Ok(decompressed)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("could not read permissions for {}: {error}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|error| {
        format!(
            "could not make embedded Lean traceability kernel executable at {}: {error}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn build_projected_traceability_contract(
    projected_item_ids: BTreeSet<String>,
    graph: &TraceGraph,
) -> Result<ProjectedTraceabilityContract, String> {
    Ok(
        build_projected_traceability_reference_from_projected_items(projected_item_ids, graph)?
            .contract,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
// @applies TRACEABILITY.SCOPED_PROJECTED_KERNEL
pub(crate) fn build_projected_traceability_reference_from_projected_items(
    projected_item_ids: BTreeSet<String>,
    graph: &TraceGraph,
) -> Result<ProjectedTraceabilityReference, String> {
    let input =
        ProjectedTraceabilityKernelInput::from_projected_items_and_graph(projected_item_ids, graph);
    derive_projected_traceability_kernel(input).map(|output| output.reference)
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
    fn kernel_placeholder(stable_id: &str) -> Self {
        Self {
            name: stable_id.to_string(),
            has_item_scoped_support: false,
            has_file_scoped_support: false,
            current_specs: BTreeSet::new(),
            planned_specs: BTreeSet::new(),
            deprecated_specs: BTreeSet::new(),
        }
    }

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

pub(crate) trait TraceabilityLanguagePack {
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
) -> Result<ProjectedTraceabilityReference, String> {
    let input = ProjectedTraceabilityKernelInput::from_contract_and_graph(contract, graph);
    derive_projected_traceability_kernel(input).map(|output| output.reference)
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
    use std::env;

    use super::{
        ProjectedTraceabilityContract, ProjectedTraceabilityKernel,
        ProjectedTraceabilityKernelInput, ProjectedTraceabilityReference,
        RUST_REFERENCE_TRACEABILITY_KERNEL, ReverseClosureReference, TraceGraph,
        TraceabilityItemSupport, build_projected_traceability_contract,
        build_projected_traceability_reference, collect_support_root_ids,
        derive_projected_traceability_kernel_for_selector, normalize_path_for_known_sources,
        preserved_graph_item_ids_for_reference, preserved_item_ids_for_reference,
        use_rust_reference_traceability_kernel_for_tests,
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

    // @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.LEAN_KERNEL
    #[test]
    fn default_projected_kernel_requires_lean_without_test_opt_in() {
        let input = ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
            ["app::live".to_string()].into_iter().collect(),
            &TraceGraph::default(),
        );

        let result = derive_projected_traceability_kernel_for_selector(input, None);
        if let Err(error) = result {
            assert!(
                error.contains("Lean traceability kernel was requested")
                    || error.contains("Lean traceability kernel"),
                "default kernel failure should be a Lean-kernel failure, got {error}"
            );
        }
    }

    // @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.LEAN_KERNEL
    #[test]
    fn old_rust_and_auto_kernel_selectors_are_rejected() {
        for selector in ["rust", "auto"] {
            let input = ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                ["app::live".to_string()].into_iter().collect(),
                &TraceGraph::default(),
            );

            let error = derive_projected_traceability_kernel_for_selector(input, Some(selector))
                .expect_err("old non-Lean selectors should be rejected");
            assert!(
                error.contains("expected `lean` or debug/test-only `rust-reference`"),
                "unexpected selector error for {selector}: {error}"
            );
        }
    }

    #[test]
    fn projected_traceability_kernel_targets_only_supported_projected_items() {
        use_rust_reference_traceability_kernel_for_tests();

        let graph = TraceGraph {
            edges: [
                (
                    "tests::test_live".to_string(),
                    ["app::live".to_string()].into_iter().collect(),
                ),
                (
                    "tests::test_helper".to_string(),
                    ["app::helper".to_string()].into_iter().collect(),
                ),
                (
                    "app::helper".to_string(),
                    ["app::live".to_string()].into_iter().collect(),
                ),
            ]
            .into_iter()
            .collect(),
            root_supports: [(
                "tests::test_live".to_string(),
                TraceabilityItemSupport {
                    name: "tests::test_live".to_string(),
                    has_item_scoped_support: true,
                    has_file_scoped_support: false,
                    current_specs: ["APP.LIVE".to_string()].into_iter().collect(),
                    planned_specs: BTreeSet::new(),
                    deprecated_specs: BTreeSet::new(),
                },
            )]
            .into_iter()
            .collect(),
        };
        let contract = build_projected_traceability_contract(
            ["app::live".to_string(), "app::orphan".to_string()]
                .into_iter()
                .collect(),
            &graph,
        )
        .expect("projected contract should derive");

        assert_eq!(
            contract.projected_item_ids,
            ["app::live".to_string(), "app::orphan".to_string()]
                .into_iter()
                .collect()
        );
        assert_eq!(
            contract.preserved_reverse_closure_target_ids,
            ["app::live".to_string()].into_iter().collect()
        );

        let reference = build_projected_traceability_reference(contract, &graph)
            .expect("reference should derive");

        assert_eq!(
            reference.exact_reverse_closure.node_ids,
            [
                "app::live".to_string(),
                "app::helper".to_string(),
                "tests::test_live".to_string(),
                "tests::test_helper".to_string(),
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn projected_traceability_kernel_wire_schema_round_trips() {
        let graph = TraceGraph {
            edges: [(
                "tests::test_live".to_string(),
                ["app::live".to_string()].into_iter().collect(),
            )]
            .into_iter()
            .collect(),
            root_supports: [(
                "tests::test_live".to_string(),
                TraceabilityItemSupport {
                    name: "tests::test_live".to_string(),
                    has_item_scoped_support: true,
                    has_file_scoped_support: false,
                    current_specs: ["APP.LIVE".to_string()].into_iter().collect(),
                    planned_specs: BTreeSet::new(),
                    deprecated_specs: BTreeSet::new(),
                },
            )]
            .into_iter()
            .collect(),
        };
        let input = ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
            ["app::live".to_string(), "app::orphan".to_string()]
                .into_iter()
                .collect(),
            &graph,
        );
        let input_json = serde_json::to_value(&input).expect("kernel input should serialize");

        assert_eq!(input_json["schema_version"], 1);
        assert_eq!(
            input_json["preserved_reverse_closure_target_ids"],
            serde_json::Value::Null
        );
        assert_eq!(
            input_json["support_root_ids"],
            serde_json::json!(["tests::test_live"])
        );

        let decoded_input: ProjectedTraceabilityKernelInput =
            serde_json::from_value(input_json).expect("kernel input should deserialize");
        let output = RUST_REFERENCE_TRACEABILITY_KERNEL
            .derive(decoded_input)
            .expect("reference kernel should derive output");
        let output_json = serde_json::to_value(&output).expect("kernel output should serialize");

        assert_eq!(output_json["schema_version"], 1);
        assert_eq!(
            output_json["reference"]["contract"]["preserved_reverse_closure_target_ids"],
            serde_json::json!(["app::live"])
        );

        let decoded_output: super::ProjectedTraceabilityKernelOutput =
            serde_json::from_value(output_json).expect("kernel output should deserialize");
        assert_eq!(decoded_output, output);
    }

    // @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.LEAN_KERNEL
    #[test]
    fn projected_traceability_lean_kernel_matches_rust_reference_cases() {
        if !lean_kernel_available_for_equivalence_test() {
            return;
        }

        for input in projected_kernel_equivalence_cases() {
            assert_lean_kernel_matches_rust_reference(input);
        }
    }

    // @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.LEAN_KERNEL.PROVEN_EXECUTABLE
    #[test]
    fn projected_traceability_kernel_default_uses_lean_when_available() {
        if !lean_kernel_available_for_equivalence_test() {
            return;
        }

        let input = ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
            ["app::live".to_string()].into_iter().collect(),
            &TraceGraph::default(),
        );
        super::derive_projected_traceability_kernel(input)
            .expect("available Lean kernel must run by default");
    }

    fn lean_kernel_available_for_equivalence_test() -> bool {
        if cfg!(special_embedded_lean_kernel) {
            return true;
        }
        if env::var("SPECIAL_TRACEABILITY_KERNEL_EXE").is_ok_and(|path| !path.is_empty()) {
            return true;
        }
        if env::var("SPECIAL_REQUIRE_LEAN_KERNEL_TESTS").is_ok() {
            panic!(
                "SPECIAL_REQUIRE_LEAN_KERNEL_TESTS was set, but no embedded Lean kernel is present \
                 and SPECIAL_TRACEABILITY_KERNEL_EXE was not set"
            );
        }
        eprintln!(
            "skipping Lean kernel equivalence test: set SPECIAL_TRACEABILITY_KERNEL_EXE or build \
             with SPECIAL_BUILD_LEAN_KERNEL=1"
        );
        false
    }

    fn assert_lean_kernel_matches_rust_reference(input: ProjectedTraceabilityKernelInput) {
        let expected = RUST_REFERENCE_TRACEABILITY_KERNEL
            .derive(input.clone())
            .expect("Rust reference kernel should derive output");
        let actual = super::LeanProcessTraceabilityKernel
            .derive(input)
            .expect("Lean kernel should derive output");

        assert_eq!(actual, expected);
    }

    fn projected_kernel_equivalence_cases() -> Vec<ProjectedTraceabilityKernelInput> {
        vec![
            ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                ["app::live".to_string(), "app::orphan".to_string()]
                    .into_iter()
                    .collect(),
                &TraceGraph {
                    edges: [
                        (
                            "tests::test_live".to_string(),
                            ["app::live".to_string()].into_iter().collect(),
                        ),
                        (
                            "tests::test_helper".to_string(),
                            ["app::helper".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::helper".to_string(),
                            ["app::live".to_string()].into_iter().collect(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    root_supports: [(
                        "tests::test_live".to_string(),
                        TraceabilityItemSupport::kernel_placeholder("tests::test_live"),
                    )]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_contract_and_graph(
                ProjectedTraceabilityContract {
                    projected_item_ids: ["app::projected".to_string()].into_iter().collect(),
                    preserved_reverse_closure_target_ids: ["app::leaf".to_string()]
                        .into_iter()
                        .collect(),
                },
                &TraceGraph {
                    edges: [
                        (
                            "tests::test_entry".to_string(),
                            ["app::entry".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::entry".to_string(),
                            ["app::leaf".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::noise".to_string(),
                            ["app::leaf".to_string()].into_iter().collect(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    root_supports: [(
                        "tests::test_entry".to_string(),
                        TraceabilityItemSupport::kernel_placeholder("tests::test_entry"),
                    )]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                ["app::left".to_string(), "app::right".to_string()]
                    .into_iter()
                    .collect(),
                &TraceGraph {
                    edges: [
                        (
                            "tests::test_left".to_string(),
                            ["app::left".to_string()].into_iter().collect(),
                        ),
                        (
                            "tests::test_right".to_string(),
                            ["app::right".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::left".to_string(),
                            ["app::right".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::right".to_string(),
                            ["app::left".to_string()].into_iter().collect(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    root_supports: [
                        (
                            "tests::test_left".to_string(),
                            TraceabilityItemSupport::kernel_placeholder("tests::test_left"),
                        ),
                        (
                            "tests::test_right".to_string(),
                            TraceabilityItemSupport::kernel_placeholder("tests::test_right"),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                ["app::orphan".to_string()].into_iter().collect(),
                &TraceGraph {
                    edges: BTreeMap::new(),
                    root_supports: BTreeMap::new(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                ["app::root".to_string()].into_iter().collect(),
                &TraceGraph {
                    edges: BTreeMap::new(),
                    root_supports: [(
                        "app::root".to_string(),
                        TraceabilityItemSupport::kernel_placeholder("app::root"),
                    )]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                ["app::recursive".to_string()].into_iter().collect(),
                &TraceGraph {
                    edges: [(
                        "app::recursive".to_string(),
                        ["app::recursive".to_string()].into_iter().collect(),
                    )]
                    .into_iter()
                    .collect(),
                    root_supports: [(
                        "app::recursive".to_string(),
                        TraceabilityItemSupport::kernel_placeholder("app::recursive"),
                    )]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                ["app::leaf".to_string()].into_iter().collect(),
                &TraceGraph {
                    edges: [
                        (
                            "tests::left".to_string(),
                            ["app::mid_left".to_string()].into_iter().collect(),
                        ),
                        (
                            "tests::right".to_string(),
                            ["app::mid_right".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::mid_left".to_string(),
                            ["app::leaf".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::mid_right".to_string(),
                            ["app::leaf".to_string()].into_iter().collect(),
                        ),
                        (
                            "tests::noise".to_string(),
                            ["app::noise".to_string()].into_iter().collect(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    root_supports: [
                        (
                            "tests::left".to_string(),
                            TraceabilityItemSupport::kernel_placeholder("tests::left"),
                        ),
                        (
                            "tests::right".to_string(),
                            TraceabilityItemSupport::kernel_placeholder("tests::right"),
                        ),
                        (
                            "tests::noise".to_string(),
                            TraceabilityItemSupport::kernel_placeholder("tests::noise"),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_contract_and_graph(
                ProjectedTraceabilityContract {
                    projected_item_ids: ["app::projected".to_string()].into_iter().collect(),
                    preserved_reverse_closure_target_ids: BTreeSet::new(),
                },
                &TraceGraph {
                    edges: [(
                        "tests::projected".to_string(),
                        ["app::projected".to_string()].into_iter().collect(),
                    )]
                    .into_iter()
                    .collect(),
                    root_supports: [(
                        "tests::projected".to_string(),
                        TraceabilityItemSupport::kernel_placeholder("tests::projected"),
                    )]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_contract_and_graph(
                ProjectedTraceabilityContract {
                    projected_item_ids: ["app::projected".to_string()].into_iter().collect(),
                    preserved_reverse_closure_target_ids: ["app::external".to_string()]
                        .into_iter()
                        .collect(),
                },
                &TraceGraph {
                    edges: [(
                        "tests::external".to_string(),
                        ["app::external".to_string()].into_iter().collect(),
                    )]
                    .into_iter()
                    .collect(),
                    root_supports: [(
                        "tests::external".to_string(),
                        TraceabilityItemSupport::kernel_placeholder("tests::external"),
                    )]
                    .into_iter()
                    .collect(),
                },
            ),
            ProjectedTraceabilityKernelInput::from_projected_items_and_graph(
                [
                    "app::supported_cycle".to_string(),
                    "app::unsupported_cycle".to_string(),
                ]
                .into_iter()
                .collect(),
                &TraceGraph {
                    edges: [
                        (
                            "tests::supported_cycle".to_string(),
                            ["app::supported_cycle".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::supported_cycle".to_string(),
                            ["app::unsupported_cycle".to_string()].into_iter().collect(),
                        ),
                        (
                            "app::unsupported_cycle".to_string(),
                            ["app::supported_cycle".to_string()].into_iter().collect(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    root_supports: [(
                        "tests::supported_cycle".to_string(),
                        TraceabilityItemSupport::kernel_placeholder("tests::supported_cycle"),
                    )]
                    .into_iter()
                    .collect(),
                },
            ),
        ]
    }

    #[test]
    fn projected_traceability_kernel_wire_schema_honors_explicit_contract_targets() {
        use_rust_reference_traceability_kernel_for_tests();

        let graph = TraceGraph {
            edges: BTreeMap::new(),
            root_supports: BTreeMap::new(),
        };
        let contract = ProjectedTraceabilityContract {
            projected_item_ids: ["app::orphan".to_string()].into_iter().collect(),
            preserved_reverse_closure_target_ids: ["app::orphan".to_string()].into_iter().collect(),
        };
        let reference = build_projected_traceability_reference(contract, &graph)
            .expect("reference should derive");

        assert_eq!(
            reference.exact_reverse_closure.node_ids,
            ["app::orphan".to_string()].into_iter().collect()
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
            let resolved_line = match verify
                .body
                .as_deref()
                .and_then(|body| parse_body_start_line(&normalized_path, body))
            {
                Some(0) => {
                    panic!("parse_body_start_line must return 1-based source lines, got 0")
                }
                Some(start_line) => body_location
                    .line
                    .checked_add(start_line)
                    .and_then(|line| line.checked_sub(1))
                    .expect("resolved verify body line should fit usize"),
                None => body_location.line,
            };
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
