/**
@module SPECIAL.MODEL
Canonical Rust domain types in `src/model.rs`.
*/
// @implements SPECIAL.MODEL
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
    pub location: SourceLocation,
}

impl SpecDecl {
    pub fn new(
        id: String,
        kind: NodeKind,
        text: String,
        plan: PlanState,
        location: SourceLocation,
    ) -> Result<Self, ModelInvariantError> {
        ensure_valid_plan(kind, &plan)?;
        Ok(Self {
            id,
            kind,
            text,
            plan,
            location,
        })
    }

    pub fn set_plan(&mut self, plan: PlanState) -> Result<(), ModelInvariantError> {
        ensure_valid_plan(self.kind, &plan)?;
        self.plan = plan;
        Ok(())
    }

    pub fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn planned_release(&self) -> Option<&str> {
        self.plan.release()
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
        let mut state = serializer.serialize_struct(
            "SpecDecl",
            if self.planned_release().is_some() {
                6
            } else {
                5
            },
        )?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
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

#[derive(Debug, Clone, Serialize)]
pub struct AttestRef {
    pub spec_id: String,
    pub artifact: String,
    pub owner: String,
    pub last_reviewed: String,
    pub review_interval_days: Option<u32>,
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

#[derive(Debug, Clone)]
pub struct SpecNode {
    pub id: String,
    kind: NodeKind,
    pub text: String,
    plan: PlanState,
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
            location: decl.location,
            verifies,
            attests,
            children,
        }
    }

    pub fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn planned_release(&self) -> Option<&str> {
        self.plan.release()
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
    pub children: Vec<ModuleNode>,
}

impl ModuleNode {
    pub fn new(decl: ModuleDecl, implements: Vec<ImplementRef>, children: Vec<ModuleNode>) -> Self {
        Self {
            id: decl.id,
            kind: decl.kind,
            text: decl.text,
            plan: decl.plan,
            location: decl.location,
            implements,
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
        let mut state = serializer.serialize_struct(
            "SpecNode",
            if self.planned_release().is_some() {
                9
            } else {
                8
            },
        )?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
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
        let mut state = serializer.serialize_struct(
            "ModuleNode",
            if self.planned_release().is_some() {
                8
            } else {
                7
            },
        )?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
        }
        state.serialize_field("location", &self.location)?;
        state.serialize_field("implements", &self.implements)?;
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

#[derive(Debug, Clone, Serialize)]
pub struct SpecDocument {
    pub nodes: Vec<SpecNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleDocument {
    pub nodes: Vec<ModuleNode>,
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
        ArchitectureKind, ModuleDecl, NodeKind, PlanState, PlannedRelease, SourceLocation, SpecDecl,
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
    fn rejects_planned_groups_at_construction_time() {
        let error = SpecDecl::new(
            "SPECIAL".to_string(),
            NodeKind::Group,
            "Grouping only.".to_string(),
            PlanState::planned(None),
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
    fn rejects_planned_areas_at_construction_time() {
        let error = ModuleDecl::new(
            "SPECIAL.AREA".to_string(),
            ArchitectureKind::Area,
            "Structural area.".to_string(),
            PlanState::planned(None),
            SourceLocation {
                path: "_project/ARCHITECTURE.md".into(),
                line: 1,
            },
        )
        .expect_err("areas should not accept planned state");

        assert_eq!(error.to_string(), "`@area` nodes may not be planned");
    }
}
