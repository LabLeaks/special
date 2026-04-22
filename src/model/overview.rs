/**
@module SPECIAL.MODEL.OVERVIEW
Rendered document, filter, metrics, and lint report domain types.
*/
// @fileimplements SPECIAL.MODEL.OVERVIEW
use serde::Serialize;

use super::{ArchitectureAnalysisSummary, Diagnostic, DiagnosticSeverity, ModuleNode, SpecNode};

#[derive(Debug, Clone, Serialize)]
pub struct SpecDocument {
    pub nodes: Vec<SpecNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<SpecMetricsSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<ArchitectureMetricsSummary>,
    #[serde(skip)]
    pub scoped: bool,
    pub nodes: Vec<ModuleNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<RepoMetricsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<ArchitectureAnalysisSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureMetricsSummary {
    pub total_modules: usize,
    pub total_areas: usize,
    pub unimplemented_modules: usize,
    pub file_scoped_implements: usize,
    pub item_scoped_implements: usize,
    pub owned_lines: usize,
    pub public_items: usize,
    pub internal_items: usize,
    pub complexity_functions: usize,
    pub total_cyclomatic: usize,
    pub max_cyclomatic: usize,
    pub total_cognitive: usize,
    pub max_cognitive: usize,
    pub quality_public_functions: usize,
    pub quality_parameters: usize,
    pub quality_bool_params: usize,
    pub quality_raw_string_params: usize,
    pub quality_panic_sites: usize,
    pub unreached_items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modules_by_area: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub owned_lines_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub max_cyclomatic_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub max_cognitive_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub panic_sites_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreached_items_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fan_in_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fan_out_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ambiguous_internal_targets_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unresolved_internal_targets_by_module: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external_dependency_targets_by_module: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecMetricsSummary {
    pub total_specs: usize,
    pub planned_specs: usize,
    pub deprecated_specs: usize,
    pub unverified_specs: usize,
    pub verified_specs: usize,
    pub attested_specs: usize,
    pub specs_with_both_supports: usize,
    pub verifies: usize,
    pub item_scoped_verifies: usize,
    pub file_scoped_verifies: usize,
    pub unattached_verifies: usize,
    pub attests: usize,
    pub block_attests: usize,
    pub file_attests: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub specs_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub current_specs_by_top_level_id: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoMetricsSummary {
    pub duplicate_items: usize,
    pub unowned_items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub duplicate_items_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unowned_items_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceability: Option<RepoTraceabilityMetrics>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupedCount {
    pub value: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepoTraceabilityMetrics {
    pub analyzed_items: usize,
    pub current_spec_items: usize,
    pub statically_mediated_items: usize,
    pub unverified_test_items: usize,
    pub unexplained_items: usize,
    pub unexplained_review_surface_items: usize,
    pub unexplained_public_items: usize,
    pub unexplained_internal_items: usize,
    pub unexplained_module_backed_items: usize,
    pub unexplained_module_connected_items: usize,
    pub unexplained_module_isolated_items: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unexplained_items_by_file: Vec<GroupedCount>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unexplained_review_surface_items_by_file: Vec<GroupedCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewDocument {
    pub lint: OverviewLintSummary,
    pub specs: OverviewSpecsSummary,
    pub arch: OverviewArchSummary,
    pub health: OverviewHealthSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceability: Option<OverviewTraceabilitySummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewLintSummary {
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewSpecsSummary {
    pub total_specs: usize,
    pub planned_specs: usize,
    pub deprecated_specs: usize,
    pub unverified_specs: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewArchSummary {
    pub total_modules: usize,
    pub total_areas: usize,
    pub unimplemented_modules: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewHealthSummary {
    pub duplicate_items: usize,
    pub unowned_items: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewTraceabilitySummary {
    pub analyzed_items: usize,
    pub current_spec_items: usize,
    pub statically_mediated_items: usize,
    pub unverified_test_items: usize,
    pub unexplained_items: usize,
    pub unexplained_review_surface_items: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
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
