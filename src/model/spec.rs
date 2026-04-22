/**
@module SPECIAL.MODEL.SPEC
Spec declaration, support attachment, and rendered spec-tree domain types.
*/
// @fileimplements SPECIAL.MODEL.SPEC
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use super::{
    DeclaredStateFilter, DeprecatedRelease, DiagnosticSeverity, ModelInvariantError, NodeKind,
    PlanState, SourceLocation,
};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRef {
    pub spec_id: String,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub path: std::path::PathBuf,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ParsedRepo {
    pub specs: Vec<SpecDecl>,
    pub verifies: Vec<VerifyRef>,
    pub attests: Vec<AttestRef>,
    pub diagnostics: Vec<Diagnostic>,
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

    pub(crate) fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub(crate) fn is_deprecated(&self) -> bool {
        self.deprecated
    }

    pub(crate) fn kind(&self) -> NodeKind {
        self.kind
    }

    pub(crate) fn planned_release(&self) -> Option<&str> {
        self.plan.release()
    }

    pub(crate) fn deprecated_release(&self) -> Option<&str> {
        self.deprecated_release
            .as_ref()
            .map(DeprecatedRelease::as_str)
    }

    pub(crate) fn is_unverified(&self) -> bool {
        self.kind == NodeKind::Spec
            && !self.is_planned()
            && self.verifies.is_empty()
            && self.attests.is_empty()
    }
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

#[derive(Debug, Clone)]
pub struct SpecFilter {
    pub state: DeclaredStateFilter,
    pub unverified_only: bool,
    pub scope: Option<String>,
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
        return Err(ModelInvariantError::deprecated_release_without_deprecated());
    }
    if plan.is_planned() && is_deprecated {
        return Err(ModelInvariantError::conflicting_spec_lifecycle());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{NodeKind, PlanState, SourceLocation, SpecDecl};
    use crate::model::{DeprecatedRelease, PlannedRelease};

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
            PlanState::current(),
            false,
            None,
            SourceLocation {
                path: "specs/special.rs".into(),
                line: 1,
            },
        )
        .expect("current groups should be valid");

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
            PlanState::current(),
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
    fn rejects_deprecated_release_without_deprecated_state() {
        let error = SpecDecl::new(
            "SPECIAL".to_string(),
            NodeKind::Spec,
            "Grouping only.".to_string(),
            PlanState::Current,
            false,
            Some(DeprecatedRelease::new("0.6.0").expect("release should be valid")),
            SourceLocation {
                path: "specs/special.rs".into(),
                line: 1,
            },
        )
        .expect_err("deprecated release metadata should require deprecated state");

        assert_eq!(
            error.to_string(),
            "@deprecated release metadata requires @deprecated"
        );
    }
}
