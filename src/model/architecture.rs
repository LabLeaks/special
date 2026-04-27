/**
@module SPECIAL.MODEL.ARCHITECTURE
Architecture declaration, attachment, and rendered module-tree domain types.
*/
// @fileimplements SPECIAL.MODEL.ARCHITECTURE
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use super::{
    ArchitectureKind, DeclaredStateFilter, ModelInvariantError, ModuleAnalysisSummary, PlanState,
    SourceLocation,
};

#[derive(Debug, Default, Clone)]
pub struct ParsedArchitecture {
    pub modules: Vec<ModuleDecl>,
    pub implements: Vec<ImplementRef>,
    pub patterns: Vec<PatternDefinition>,
    pub pattern_applications: Vec<PatternApplication>,
    pub diagnostics: Vec<super::Diagnostic>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementRef {
    pub module_id: String,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDefinition {
    pub pattern_id: String,
    #[serde(default)]
    pub strictness: PatternStrictness,
    pub text: String,
    pub location: SourceLocation,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternStrictness {
    High,
    #[default]
    Medium,
    Low,
}

impl PatternStrictness {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternApplication {
    pub pattern_id: String,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub id: String,
    kind: ArchitectureKind,
    pub text: String,
    plan: PlanState,
    pub location: SourceLocation,
    pub implements: Vec<ImplementRef>,
    pub pattern_applications: Vec<PatternApplication>,
    pub analysis: Option<ModuleAnalysisSummary>,
    pub children: Vec<ModuleNode>,
}

impl ModuleNode {
    pub fn new(
        decl: ModuleDecl,
        implements: Vec<ImplementRef>,
        pattern_applications: Vec<PatternApplication>,
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
            pattern_applications,
            analysis,
            children,
        }
    }

    pub(crate) fn is_planned(&self) -> bool {
        self.plan.is_planned()
    }

    pub(crate) fn kind(&self) -> ArchitectureKind {
        self.kind
    }

    pub(crate) fn planned_release(&self) -> Option<&str> {
        self.plan.release()
    }

    pub(crate) fn is_unimplemented(&self) -> bool {
        self.kind == ArchitectureKind::Module && !self.is_planned() && self.implements.is_empty()
    }
}

impl Serialize for ModuleNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ModuleNode", 10)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("planned", &self.is_planned())?;
        if let Some(planned_release) = self.planned_release() {
            state.serialize_field("planned_release", planned_release)?;
        }
        state.serialize_field("location", &self.location)?;
        state.serialize_field("implements", &self.implements)?;
        state.serialize_field("pattern_applications", &self.pattern_applications)?;
        if let Some(analysis) = &self.analysis {
            state.serialize_field("analysis", analysis)?;
        }
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleFilter {
    pub state: DeclaredStateFilter,
    pub unimplemented_only: bool,
    pub scope: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PatternFilter {
    pub scope: Option<String>,
    pub metrics: bool,
    pub target_paths: Vec<std::path::PathBuf>,
    pub comparison_paths: Vec<std::path::PathBuf>,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<PatternMetricsSummary>,
    #[serde(skip)]
    pub scoped: bool,
    pub patterns: Vec<PatternNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternNode {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<PatternDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<PatternSimilarityMetrics>,
    pub applications: Vec<PatternApplicationNode>,
    pub modules: Vec<PatternModuleRef>,
    pub children: Vec<PatternNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternModuleRef {
    pub id: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternApplicationNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    pub location: SourceLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternMetricsSummary {
    pub total_patterns: usize,
    pub total_definitions: usize,
    pub total_applications: usize,
    pub modules_with_applications: usize,
    pub possible_missing_applications: Vec<PatternMissingApplicationCandidate>,
    pub possible_pattern_clusters: Vec<PatternClusterCandidate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternMissingApplicationCandidate {
    pub pattern_id: String,
    pub strictness: PatternStrictness,
    pub confidence: PatternCandidateConfidence,
    pub score: f64,
    pub item_name: String,
    pub location: SourceLocation,
    pub matched_terms: Vec<String>,
    pub missing_terms: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternCandidateConfidence {
    Probable,
    Possible,
}

impl PatternCandidateConfidence {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Probable => "probable",
            Self::Possible => "possible",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternClusterCandidate {
    pub score: f64,
    pub suggested_strictness: PatternStrictness,
    pub interpretation: PatternClusterInterpretation,
    pub meaning: &'static str,
    pub precise: &'static str,
    pub item_count: usize,
    pub shared_terms: Vec<String>,
    pub items: Vec<PatternClusterItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternClusterInterpretation {
    PossiblePattern,
    ExtractionCandidate,
}

impl PatternClusterInterpretation {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PossiblePattern => "possible pattern",
            Self::ExtractionCandidate => "helper/component extraction candidate",
        }
    }

    pub(crate) fn meaning(self) -> &'static str {
        match self {
            Self::PossiblePattern => {
                "these items share a recurring source shape that may reflect an adopted implementation approach."
            }
            Self::ExtractionCandidate => {
                "these items are very tightly shaped, so the better move may be extracting a helper or component instead of naming a pattern."
            }
        }
    }

    pub(crate) fn precise(self) -> &'static str {
        match self {
            Self::PossiblePattern => {
                "deterministic source-feature clustering found shared calls, control-flow, names, or structural terms with enough variation to warrant human pattern review."
            }
            Self::ExtractionCandidate => {
                "deterministic source-feature clustering found low-variance repeated implementation, which is often duplicated logic rather than an adaptable pattern."
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternClusterItem {
    pub item_name: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternSimilarityMetrics {
    pub strictness: PatternStrictness,
    pub scored_applications: usize,
    pub pair_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark_estimate: Option<PatternBenchmarkEstimate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternBenchmarkEstimate {
    DuplicateLike,
    TighterThanExpected,
    NearExpected,
    LooserThanExpected,
    Diffuse,
}

impl PatternBenchmarkEstimate {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::DuplicateLike => "duplicate-like",
            Self::TighterThanExpected => "tighter than expected",
            Self::NearExpected => "near expected",
            Self::LooserThanExpected => "looser than expected",
            Self::Diffuse => "diffuse",
        }
    }
}

fn ensure_valid_architecture_plan(
    kind: ArchitectureKind,
    plan: &PlanState,
) -> Result<(), ModelInvariantError> {
    if kind == ArchitectureKind::Area && plan.is_planned() {
        return Err(ModelInvariantError::planned_area());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ArchitectureKind, ModuleDecl, PlanState, SourceLocation};

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
