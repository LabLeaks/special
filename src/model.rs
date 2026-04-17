/**
@module SPECIAL.MODEL
Canonical Rust domain types in `src/model.rs`.
*/
// @fileimplements SPECIAL.MODEL
use std::fmt;
use std::path::PathBuf;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Spec,
    Group,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureKind {
    Module,
    Area,
}

impl ArchitectureKind {
    pub fn as_annotation(self) -> &'static str {
        match self {
            Self::Module => "@module",
            Self::Area => "@area",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct OwnedItem {
    pub location: SourceLocation,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct CommentBlock {
    pub path: PathBuf,
    pub lines: Vec<BlockLine>,
    pub owned_item: Option<OwnedItem>,
}

#[derive(Debug, Clone)]
pub struct BlockLine {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRelease(String);

impl PlannedRelease {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelInvariantError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ModelInvariantError::empty_planned_release());
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedRelease(String);

impl DeprecatedRelease {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelInvariantError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ModelInvariantError::empty_deprecated_release());
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInvariantError {
    message: String,
}

impl ModelInvariantError {
    fn planned_group(kind: NodeKind) -> Self {
        Self {
            message: format!("`{}` nodes may not be planned", kind.as_annotation()),
        }
    }

    fn empty_planned_release() -> Self {
        Self {
            message: "planned release metadata must not be empty".to_string(),
        }
    }

    fn deprecated_group(kind: NodeKind) -> Self {
        Self {
            message: format!("`{}` nodes may not be deprecated", kind.as_annotation()),
        }
    }

    fn empty_deprecated_release() -> Self {
        Self {
            message: "deprecated release metadata must not be empty".to_string(),
        }
    }

    fn conflicting_spec_lifecycle() -> Self {
        Self {
            message: "@spec may not be both planned and deprecated".to_string(),
        }
    }
}

impl fmt::Display for ModelInvariantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ModelInvariantError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanState {
    Live,
    Planned { release: Option<PlannedRelease> },
}

impl PlanState {
    pub fn live() -> Self {
        Self::Live
    }

    pub fn planned(release: Option<PlannedRelease>) -> Self {
        Self::Planned { release }
    }

    pub fn is_planned(&self) -> bool {
        matches!(self, Self::Planned { .. })
    }

    pub fn release(&self) -> Option<&str> {
        match self {
            Self::Live => None,
            Self::Planned { release } => release.as_ref().map(PlannedRelease::as_str),
        }
    }
}

impl NodeKind {
    fn as_annotation(self) -> &'static str {
        match self {
            Self::Spec => "@spec",
            Self::Group => "@group",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecDecl {
    pub id: String,
    kind: NodeKind,
    pub text: String,
    plan: PlanState,
    deprecated: bool,
    deprecated_release: Option<DeprecatedRelease>,
    pub location: SourceLocation,
}

impl SpecDecl {
    pub fn new(
        id: String,
        kind: NodeKind,
        text: String,
        plan: PlanState,
        deprecated: bool,
        deprecated_release: Option<DeprecatedRelease>,
        location: SourceLocation,
    ) -> Result<Self, ModelInvariantError> {
        ensure_valid_spec_lifecycle(kind, &plan, deprecated, deprecated_release.as_ref())?;
        Ok(Self {
            id,
            kind,
            text,
            plan,
            deprecated,
            deprecated_release,
            location,
        })
    }

    pub fn set_plan(&mut self, plan: PlanState) -> Result<(), ModelInvariantError> {
        ensure_valid_spec_lifecycle(
            self.kind,
            &plan,
            self.deprecated,
            self.deprecated_release.as_ref(),
        )?;
        self.plan = plan;
        Ok(())
    }

    pub fn set_deprecated(
        &mut self,
        is_deprecated: bool,
        deprecated_release: Option<DeprecatedRelease>,
    ) -> Result<(), ModelInvariantError> {
        ensure_valid_spec_lifecycle(
            self.kind,
            &self.plan,
            is_deprecated,
            deprecated_release.as_ref(),
        )?;
        self.deprecated = is_deprecated;
        self.deprecated_release = deprecated_release;
        Ok(())
    }

    pub fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub fn is_deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn planned_release(&self) -> Option<&str> {
        self.plan.release()
    }

    pub fn deprecated_release(&self) -> Option<&str> {
        self.deprecated_release
            .as_ref()
            .map(DeprecatedRelease::as_str)
    }

    pub fn plan(&self) -> &PlanState {
        &self.plan
    }
}

impl Serialize for SpecDecl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut fields = 6;
        if self.planned_release().is_some() {
            fields += 1;
        }
        if self.deprecated_release().is_some() {
            fields += 1;
        }
        let mut state = serializer.serialize_struct("SpecDecl", fields)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        state.serialize_field("deprecated", &self.is_deprecated())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
        }
        if let Some(deprecated_release) = self.deprecated_release() {
            state.serialize_field("deprecated_release", deprecated_release)?;
        }
        state.serialize_field("location", &self.location)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyRef {
    pub spec_id: String,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttestScope {
    Block,
    File,
}

impl AttestScope {
    pub fn as_annotation(self) -> &'static str {
        match self {
            Self::Block => "@attests",
            Self::File => "@fileattests",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AttestRef {
    pub spec_id: String,
    pub artifact: String,
    pub owner: String,
    pub last_reviewed: String,
    pub review_interval_days: Option<u32>,
    pub scope: AttestScope,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub path: PathBuf,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedRepo {
    pub specs: Vec<SpecDecl>,
    pub verifies: Vec<VerifyRef>,
    pub attests: Vec<AttestRef>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedArchitecture {
    pub modules: Vec<ModuleDecl>,
    pub implements: Vec<ImplementRef>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub id: String,
    kind: ArchitectureKind,
    pub text: String,
    plan: PlanState,
    pub location: SourceLocation,
}

impl ModuleDecl {
    pub fn new(
        id: String,
        kind: ArchitectureKind,
        text: String,
        plan: PlanState,
        location: SourceLocation,
    ) -> Result<Self, ModelInvariantError> {
        ensure_valid_architecture_plan(kind, &plan)?;
        Ok(Self {
            id,
            kind,
            text,
            plan,
            location,
        })
    }

    pub fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub fn kind(&self) -> ArchitectureKind {
        self.kind
    }

    pub fn plan(&self) -> &PlanState {
        &self.plan
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ImplementRef {
    pub module_id: String,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ArchitectureRepoSignalsSummary {
    pub unowned_unreached_items: usize,
    pub unowned_unreached_item_details: Vec<ArchitectureUnreachedItem>,
    pub duplicate_items: usize,
    pub duplicate_item_details: Vec<ArchitectureDuplicateItem>,
}

impl Serialize for ArchitectureRepoSignalsSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 2;
        if !self.unowned_unreached_item_details.is_empty() {
            field_count += 1;
        }
        if !self.duplicate_item_details.is_empty() {
            field_count += 1;
        }
        let mut state =
            serializer.serialize_struct("ArchitectureRepoSignalsSummary", field_count)?;
        state.serialize_field("unowned_unreached_items", &self.unowned_unreached_items)?;
        state.serialize_field("duplicate_items", &self.duplicate_items)?;
        if !self.unowned_unreached_item_details.is_empty() {
            state.serialize_field(
                "unowned_unreached_item_details",
                &self.unowned_unreached_item_details,
            )?;
        }
        if !self.duplicate_item_details.is_empty() {
            state.serialize_field("duplicate_item_details", &self.duplicate_item_details)?;
        }
        state.end()
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleMetricsSummary {
    pub owned_lines: usize,
    pub public_items: usize,
    pub internal_items: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleComplexitySummary {
    pub function_count: usize,
    pub total_cyclomatic: usize,
    pub max_cyclomatic: usize,
    pub total_cognitive: usize,
    pub max_cognitive: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleQualitySummary {
    pub public_function_count: usize,
    pub parameter_count: usize,
    pub bool_parameter_count: usize,
    pub raw_string_parameter_count: usize,
    pub panic_site_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ModuleItemKind {
    Function,
    Method,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleItemSignal {
    pub name: String,
    pub kind: ModuleItemKind,
    pub public: bool,
    pub parameter_count: usize,
    pub bool_parameter_count: usize,
    pub raw_string_parameter_count: usize,
    pub internal_refs: usize,
    pub inbound_internal_refs: usize,
    pub external_refs: usize,
    pub cyclomatic: usize,
    pub cognitive: usize,
    pub panic_site_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureUnreachedItem {
    pub path: PathBuf,
    pub name: String,
    pub kind: ModuleItemKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureDuplicateItem {
    pub module_id: String,
    pub path: PathBuf,
    pub name: String,
    pub kind: ModuleItemKind,
    pub duplicate_peer_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureTraceabilityItem {
    pub module_id: String,
    pub name: String,
    pub kind: ModuleItemKind,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verifying_tests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unverified_tests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub live_specs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub planned_specs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deprecated_specs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleTraceabilityItem {
    pub name: String,
    pub kind: ModuleItemKind,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verifying_tests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unverified_tests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub live_specs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub planned_specs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deprecated_specs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleItemSignalsSummary {
    pub analyzed_items: usize,
    pub unreached_item_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connected_items: Vec<ModuleItemSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outbound_heavy_items: Vec<ModuleItemSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub isolated_items: Vec<ModuleItemSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreached_items: Vec<ModuleItemSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub highest_complexity_items: Vec<ModuleItemSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameter_heavy_items: Vec<ModuleItemSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stringly_boundary_items: Vec<ModuleItemSignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub panic_heavy_items: Vec<ModuleItemSignal>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleTraceabilitySummary {
    pub analyzed_items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub live_spec_items: Vec<ModuleTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub planned_only_items: Vec<ModuleTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deprecated_only_items: Vec<ModuleTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_scoped_only_items: Vec<ModuleTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unverified_test_items: Vec<ModuleTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unknown_items: Vec<ModuleTraceabilityItem>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ArchitectureTraceabilitySummary {
    pub analyzed_items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub live_spec_items: Vec<ArchitectureTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub planned_only_items: Vec<ArchitectureTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deprecated_only_items: Vec<ArchitectureTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_scoped_only_items: Vec<ArchitectureTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unverified_test_items: Vec<ArchitectureTraceabilityItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unknown_items: Vec<ArchitectureTraceabilityItem>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleCouplingSummary {
    pub fan_in: usize,
    pub fan_out: usize,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability: f64,
    pub external_target_count: usize,
    pub unresolved_internal_target_count: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleDependencyTargetSummary {
    pub path: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleDependencySummary {
    pub reference_count: usize,
    pub distinct_targets: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<ModuleDependencyTargetSummary>,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleCoverageSummary {
    pub file_scoped_implements: usize,
    pub item_scoped_implements: usize,
}

impl Serialize for ModuleCoverageSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModuleCoverageSummary", 2)?;
        state.serialize_field("file_scoped_implements", &self.file_scoped_implements)?;
        state.serialize_field("item_scoped_implements", &self.item_scoped_implements)?;
        state.end()
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModuleAnalysisSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<ModuleCoverageSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<ModuleMetricsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<ModuleComplexitySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ModuleQualitySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_signals: Option<ModuleItemSignalsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceability: Option<ModuleTraceabilitySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling: Option<ModuleCouplingSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<ModuleDependencySummary>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ArchitectureAnalysisSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_signals: Option<ArchitectureRepoSignalsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceability: Option<ArchitectureTraceabilitySummary>,
}

#[derive(Debug, Clone)]
pub struct SpecNode {
    pub id: String,
    kind: NodeKind,
    pub text: String,
    plan: PlanState,
    deprecated: bool,
    deprecated_release: Option<DeprecatedRelease>,
    pub location: SourceLocation,
    pub verifies: Vec<VerifyRef>,
    pub attests: Vec<AttestRef>,
    pub children: Vec<SpecNode>,
}

impl SpecNode {
    pub fn new(
        decl: SpecDecl,
        verifies: Vec<VerifyRef>,
        attests: Vec<AttestRef>,
        children: Vec<SpecNode>,
    ) -> Self {
        Self {
            id: decl.id,
            kind: decl.kind,
            text: decl.text,
            plan: decl.plan,
            deprecated: decl.deprecated,
            deprecated_release: decl.deprecated_release,
            location: decl.location,
            verifies,
            attests,
            children,
        }
    }

    pub fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub fn is_deprecated(&self) -> bool {
        self.deprecated
    }

    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn planned_release(&self) -> Option<&str> {
        self.plan.release()
    }

    pub fn deprecated_release(&self) -> Option<&str> {
        self.deprecated_release
            .as_ref()
            .map(DeprecatedRelease::as_str)
    }

    pub fn is_unsupported(&self) -> bool {
        self.kind == NodeKind::Spec
            && !self.is_planned()
            && self.verifies.is_empty()
            && self.attests.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub id: String,
    kind: ArchitectureKind,
    pub text: String,
    plan: PlanState,
    pub location: SourceLocation,
    pub implements: Vec<ImplementRef>,
    pub analysis: Option<ModuleAnalysisSummary>,
    pub children: Vec<ModuleNode>,
}

impl ModuleNode {
    pub fn new(
        decl: ModuleDecl,
        implements: Vec<ImplementRef>,
        analysis: Option<ModuleAnalysisSummary>,
        children: Vec<ModuleNode>,
    ) -> Self {
        Self {
            id: decl.id,
            kind: decl.kind,
            text: decl.text,
            plan: decl.plan,
            location: decl.location,
            implements,
            analysis,
            children,
        }
    }

    pub fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub fn kind(&self) -> ArchitectureKind {
        self.kind
    }

    pub fn planned_release(&self) -> Option<&str> {
        self.plan.release()
    }

    pub fn is_unsupported(&self) -> bool {
        self.kind == ArchitectureKind::Module && !self.is_planned() && self.implements.is_empty()
    }
}

fn ensure_valid_plan(kind: NodeKind, plan: &PlanState) -> Result<(), ModelInvariantError> {
    if kind == NodeKind::Group && plan.is_planned() {
        return Err(ModelInvariantError::planned_group(kind));
    }
    Ok(())
}

fn ensure_valid_spec_lifecycle(
    kind: NodeKind,
    plan: &PlanState,
    is_deprecated: bool,
    deprecated_release: Option<&DeprecatedRelease>,
) -> Result<(), ModelInvariantError> {
    ensure_valid_plan(kind, plan)?;
    if kind == NodeKind::Group && is_deprecated {
        return Err(ModelInvariantError::deprecated_group(kind));
    }
    if !is_deprecated && deprecated_release.is_some() {
        return Err(ModelInvariantError::conflicting_spec_lifecycle());
    }
    if plan.is_planned() && is_deprecated {
        return Err(ModelInvariantError::conflicting_spec_lifecycle());
    }
    Ok(())
}

fn ensure_valid_architecture_plan(
    kind: ArchitectureKind,
    plan: &PlanState,
) -> Result<(), ModelInvariantError> {
    if kind == ArchitectureKind::Area && plan.is_planned() {
        return Err(ModelInvariantError {
            message: "`@area` nodes may not be planned".to_string(),
        });
    }
    Ok(())
}

impl Serialize for SpecNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut fields = 9;
        if self.planned_release().is_some() {
            fields += 1;
        }
        if self.deprecated_release().is_some() {
            fields += 1;
        }
        let mut state = serializer.serialize_struct("SpecNode", fields)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        state.serialize_field("deprecated", &self.is_deprecated())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
        }
        if let Some(deprecated_release) = self.deprecated_release() {
            state.serialize_field("deprecated_release", deprecated_release)?;
        }
        state.serialize_field("location", &self.location)?;
        state.serialize_field("verifies", &self.verifies)?;
        state.serialize_field("attests", &self.attests)?;
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}

impl Serialize for ModuleNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModuleNode", 9)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
        }
        state.serialize_field("location", &self.location)?;
        state.serialize_field("implements", &self.implements)?;
        if let Some(analysis) = &self.analysis {
            state.serialize_field("analysis", analysis)?;
        }
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct SpecFilter {
    pub include_planned: bool,
    pub unsupported_only: bool,
    pub scope: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleFilter {
    pub include_planned: bool,
    pub unsupported_only: bool,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModuleAnalysisOptions {
    pub coverage: bool,
    pub metrics: bool,
    pub experimental: bool,
}

impl ModuleAnalysisOptions {
    pub fn normalized(self) -> Self {
        if self.experimental {
            Self {
                coverage: true,
                metrics: true,
                experimental: true,
            }
        } else {
            self
        }
    }

    pub fn any(self) -> bool {
        let normalized = self.normalized();
        normalized.coverage || normalized.metrics || normalized.experimental
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecDocument {
    pub nodes: Vec<SpecNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleDocument {
    pub nodes: Vec<ModuleNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<ArchitectureAnalysisSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<ArchitectureAnalysisSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LintReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl LintReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArchitectureKind, DeprecatedRelease, ModuleDecl, NodeKind, PlanState, PlannedRelease,
        SourceLocation, SpecDecl,
    };

    #[test]
    fn rejects_empty_planned_release_values() {
        let error = PlannedRelease::new("   ").expect_err("empty releases should be rejected");
        assert_eq!(
            error.to_string(),
            "planned release metadata must not be empty"
        );
    }

    #[test]
    fn rejects_empty_deprecated_release_values() {
        let error = DeprecatedRelease::new("   ").expect_err("empty releases should be rejected");
        assert_eq!(
            error.to_string(),
            "deprecated release metadata must not be empty"
        );
    }

    #[test]
    fn rejects_planned_groups_at_construction_time() {
        let error = SpecDecl::new(
            "SPECIAL".to_string(),
            NodeKind::Group,
            "Grouping only.".to_string(),
            PlanState::planned(None),
            false,
            None,
            SourceLocation {
                path: "specs/special.rs".into(),
                line: 1,
            },
        )
        .expect_err("groups should not accept planned state");

        assert_eq!(error.to_string(), "`@group` nodes may not be planned");
    }

    #[test]
    fn rejects_turning_groups_planned_after_construction() {
        let mut group = SpecDecl::new(
            "SPECIAL".to_string(),
            NodeKind::Group,
            "Grouping only.".to_string(),
            PlanState::live(),
            false,
            None,
            SourceLocation {
                path: "specs/special.rs".into(),
                line: 1,
            },
        )
        .expect("live groups should be valid");

        let error = group
            .set_plan(PlanState::planned(None))
            .expect_err("groups should stay unplannable");
        assert_eq!(error.to_string(), "`@group` nodes may not be planned");
    }

    #[test]
    fn rejects_deprecated_groups_at_construction_time() {
        let error = SpecDecl::new(
            "SPECIAL".to_string(),
            NodeKind::Group,
            "Grouping only.".to_string(),
            PlanState::live(),
            true,
            Some(DeprecatedRelease::new("0.6.0").expect("release should be valid")),
            SourceLocation {
                path: "specs/special.rs".into(),
                line: 1,
            },
        )
        .expect_err("groups should not accept deprecated state");

        assert_eq!(error.to_string(), "`@group` nodes may not be deprecated");
    }

    #[test]
    fn rejects_conflicting_spec_lifecycle_metadata() {
        let error = SpecDecl::new(
            "SPECIAL".to_string(),
            NodeKind::Spec,
            "Grouping only.".to_string(),
            PlanState::planned(None),
            true,
            Some(DeprecatedRelease::new("0.6.0").expect("release should be valid")),
            SourceLocation {
                path: "specs/special.rs".into(),
                line: 1,
            },
        )
        .expect_err("specs should not accept conflicting lifecycle state");

        assert_eq!(
            error.to_string(),
            "@spec may not be both planned and deprecated"
        );
    }

    #[test]
    fn rejects_planned_areas_at_construction_time() {
        let error = ModuleDecl::new(
            "SPECIAL.AREA".to_string(),
            ArchitectureKind::Area,
            "Structural area.".to_string(),
            PlanState::planned(None),
            SourceLocation {
                path: "ARCHITECTURE.md".into(),
                line: 1,
            },
        )
        .expect_err("areas should not accept planned state");

        assert_eq!(error.to_string(), "`@area` nodes may not be planned");
    }
}
