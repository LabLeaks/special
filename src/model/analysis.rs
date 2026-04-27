/**
@module SPECIAL.MODEL.ANALYSIS
Architecture analysis, traceability, dependency, and quality summary domain types.
*/
// @fileimplements SPECIAL.MODEL.ANALYSIS
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchitectureRepoSignalsSummary {
    pub unowned_items: usize,
    #[serde(default)]
    pub unowned_item_details: Vec<ArchitectureUnownedItem>,
    pub duplicate_items: usize,
    #[serde(default)]
    pub duplicate_item_details: Vec<ArchitectureDuplicateItem>,
}

impl Serialize for ArchitectureRepoSignalsSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 2;
        if !self.unowned_item_details.is_empty() {
            field_count += 1;
        }
        if !self.duplicate_item_details.is_empty() {
            field_count += 1;
        }
        let mut state =
            serializer.serialize_struct("ArchitectureRepoSignalsSummary", field_count)?;
        state.serialize_field("unowned_items", &self.unowned_items)?;
        state.serialize_field("duplicate_items", &self.duplicate_items)?;
        if !self.unowned_item_details.is_empty() {
            state.serialize_field("unowned_item_details", &self.unowned_item_details)?;
        }
        if !self.duplicate_item_details.is_empty() {
            state.serialize_field("duplicate_item_details", &self.duplicate_item_details)?;
        }
        state.end()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleMetricsSummary {
    pub owned_lines: usize,
    pub public_items: usize,
    pub internal_items: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleComplexitySummary {
    pub function_count: usize,
    pub total_cyclomatic: usize,
    pub max_cyclomatic: usize,
    pub total_cognitive: usize,
    pub max_cognitive: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleQualitySummary {
    pub public_function_count: usize,
    pub parameter_count: usize,
    pub bool_parameter_count: usize,
    pub raw_string_parameter_count: usize,
    pub panic_site_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ModuleItemKind {
    Function,
    Method,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureUnownedItem {
    pub path: std::path::PathBuf,
    pub name: String,
    pub kind: ModuleItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDuplicateItem {
    pub module_id: String,
    pub path: std::path::PathBuf,
    pub name: String,
    pub kind: ModuleItemKind,
    pub duplicate_peer_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureTraceabilityItem {
    pub path: std::path::PathBuf,
    pub line: usize,
    pub name: String,
    pub kind: ModuleItemKind,
    pub public: bool,
    pub review_surface: bool,
    pub test_file: bool,
    pub module_backed_by_current_specs: bool,
    pub module_connected_to_current_specs: bool,
    #[serde(default)]
    pub module_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mediated_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verifying_tests: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unverified_tests: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub current_specs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planned_specs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecated_specs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTraceabilityItem {
    pub line: usize,
    pub name: String,
    pub kind: ModuleItemKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mediated_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verifying_tests: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unverified_tests: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub current_specs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planned_specs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecated_specs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleItemSignalsSummary {
    pub analyzed_items: usize,
    pub unreached_item_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connected_items: Vec<ModuleItemSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outbound_heavy_items: Vec<ModuleItemSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub isolated_items: Vec<ModuleItemSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unreached_items: Vec<ModuleItemSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highest_complexity_items: Vec<ModuleItemSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_heavy_items: Vec<ModuleItemSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stringly_boundary_items: Vec<ModuleItemSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panic_heavy_items: Vec<ModuleItemSignal>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleTraceabilitySummary {
    pub analyzed_items: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub current_spec_items: Vec<ModuleTraceabilityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planned_only_items: Vec<ModuleTraceabilityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecated_only_items: Vec<ModuleTraceabilityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_scoped_only_items: Vec<ModuleTraceabilityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unverified_test_items: Vec<ModuleTraceabilityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statically_mediated_items: Vec<ModuleTraceabilityItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unexplained_items: Vec<ModuleTraceabilityItem>,
}

impl ModuleTraceabilitySummary {
    pub(crate) fn sort_items(&mut self) {
        for items in [
            &mut self.current_spec_items,
            &mut self.planned_only_items,
            &mut self.deprecated_only_items,
            &mut self.file_scoped_only_items,
            &mut self.unverified_test_items,
            &mut self.statically_mediated_items,
            &mut self.unexplained_items,
        ] {
            items.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.line.cmp(&right.line))
                    .then_with(|| left.kind.cmp(&right.kind))
            });
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchitectureTraceabilitySummary {
    pub analyzed_items: usize,
    pub current_spec_items: Vec<ArchitectureTraceabilityItem>,
    pub planned_only_items: Vec<ArchitectureTraceabilityItem>,
    pub deprecated_only_items: Vec<ArchitectureTraceabilityItem>,
    pub file_scoped_only_items: Vec<ArchitectureTraceabilityItem>,
    pub unverified_test_items: Vec<ArchitectureTraceabilityItem>,
    pub statically_mediated_items: Vec<ArchitectureTraceabilityItem>,
    pub unexplained_items: Vec<ArchitectureTraceabilityItem>,
}

impl ArchitectureTraceabilitySummary {
    pub fn extend_from(&mut self, delta: Self) {
        self.analyzed_items += delta.analyzed_items;
        self.current_spec_items.extend(delta.current_spec_items);
        self.planned_only_items.extend(delta.planned_only_items);
        self.deprecated_only_items
            .extend(delta.deprecated_only_items);
        self.file_scoped_only_items
            .extend(delta.file_scoped_only_items);
        self.unverified_test_items
            .extend(delta.unverified_test_items);
        self.statically_mediated_items
            .extend(delta.statically_mediated_items);
        self.unexplained_items.extend(delta.unexplained_items);
    }

    pub fn sort_items(&mut self) {
        for items in [
            &mut self.current_spec_items,
            &mut self.planned_only_items,
            &mut self.deprecated_only_items,
            &mut self.file_scoped_only_items,
            &mut self.unverified_test_items,
            &mut self.statically_mediated_items,
            &mut self.unexplained_items,
        ] {
            items.sort_by(|left, right| {
                left.path
                    .cmp(&right.path)
                    .then_with(|| left.line.cmp(&right.line))
                    .then_with(|| left.name.cmp(&right.name))
                    .then_with(|| left.kind.cmp(&right.kind))
            });
        }
    }

    pub fn unexplained_review_surface_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| item.review_surface)
            .count()
    }

    pub fn unexplained_public_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| item.public)
            .count()
    }

    pub fn unexplained_internal_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| !item.public)
            .count()
    }

    pub fn unexplained_test_file_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| item.test_file)
            .count()
    }

    pub fn unexplained_module_owned_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| !item.module_ids.is_empty())
            .count()
    }

    pub fn unexplained_unowned_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| item.module_ids.is_empty())
            .count()
    }

    pub fn unexplained_module_backed_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| item.module_backed_by_current_specs)
            .count()
    }

    pub fn unexplained_module_connected_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| {
                item.module_backed_by_current_specs && item.module_connected_to_current_specs
            })
            .count()
    }

    pub fn unexplained_module_isolated_items(&self) -> usize {
        self.unexplained_items
            .iter()
            .filter(|item| {
                item.module_backed_by_current_specs && !item.module_connected_to_current_specs
            })
            .count()
    }
}

impl Serialize for ArchitectureTraceabilitySummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ArchitectureTraceabilitySummary", 17)?;
        state.serialize_field("analyzed_items", &self.analyzed_items)?;
        state.serialize_field(
            "unexplained_review_surface_items",
            &self.unexplained_review_surface_items(),
        )?;
        state.serialize_field("unexplained_public_items", &self.unexplained_public_items())?;
        state.serialize_field(
            "unexplained_internal_items",
            &self.unexplained_internal_items(),
        )?;
        state.serialize_field(
            "unexplained_test_file_items",
            &self.unexplained_test_file_items(),
        )?;
        state.serialize_field(
            "unexplained_module_owned_items",
            &self.unexplained_module_owned_items(),
        )?;
        state.serialize_field(
            "unexplained_unowned_items",
            &self.unexplained_unowned_items(),
        )?;
        state.serialize_field(
            "unexplained_module_backed_items",
            &self.unexplained_module_backed_items(),
        )?;
        state.serialize_field(
            "unexplained_module_connected_items",
            &self.unexplained_module_connected_items(),
        )?;
        state.serialize_field(
            "unexplained_module_isolated_items",
            &self.unexplained_module_isolated_items(),
        )?;
        state.serialize_field("current_spec_items", &self.current_spec_items)?;
        state.serialize_field("planned_only_items", &self.planned_only_items)?;
        state.serialize_field("deprecated_only_items", &self.deprecated_only_items)?;
        state.serialize_field("file_scoped_only_items", &self.file_scoped_only_items)?;
        state.serialize_field("unverified_test_items", &self.unverified_test_items)?;
        state.serialize_field("statically_mediated_items", &self.statically_mediated_items)?;
        state.serialize_field("unexplained_items", &self.unexplained_items)?;
        state.end()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleCouplingSummary {
    pub fan_in: usize,
    pub fan_out: usize,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability: f64,
    pub external_target_count: usize,
    pub ambiguous_internal_target_count: usize,
    pub unresolved_internal_target_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleDependencyTargetSummary {
    pub path: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleDependencySummary {
    pub reference_count: usize,
    pub distinct_targets: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<ModuleDependencyTargetSummary>,
}

#[derive(Debug, Clone, Default, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub traceability_unavailable_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling: Option<ModuleCouplingSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<ModuleDependencySummary>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchitectureAnalysisSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_signals: Option<ArchitectureRepoSignalsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceability: Option<ArchitectureTraceabilitySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceability_unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModuleAnalysisOptions {
    pub coverage: bool,
    pub metrics: bool,
    pub traceability: bool,
}

impl ModuleAnalysisOptions {
    pub fn normalized(self) -> Self {
        if self.traceability {
            Self {
                coverage: true,
                metrics: true,
                traceability: true,
            }
        } else {
            self
        }
    }

    pub fn any(self) -> bool {
        let normalized = self.normalized();
        normalized.coverage || normalized.metrics || normalized.traceability
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary, ModuleItemKind,
        ModuleTraceabilityItem, ModuleTraceabilitySummary,
    };

    fn module_item(name: &str, line: usize, kind: ModuleItemKind) -> ModuleTraceabilityItem {
        ModuleTraceabilityItem {
            line,
            name: name.to_string(),
            kind,
            mediated_reason: None,
            verifying_tests: Vec::new(),
            unverified_tests: Vec::new(),
            current_specs: Vec::new(),
            planned_specs: Vec::new(),
            deprecated_specs: Vec::new(),
        }
    }

    fn architecture_item(
        path: &str,
        name: &str,
        line: usize,
        kind: ModuleItemKind,
    ) -> ArchitectureTraceabilityItem {
        ArchitectureTraceabilityItem {
            path: path.into(),
            line,
            name: name.to_string(),
            kind,
            public: false,
            review_surface: false,
            test_file: false,
            module_backed_by_current_specs: false,
            module_connected_to_current_specs: false,
            module_ids: Vec::new(),
            mediated_reason: None,
            verifying_tests: Vec::new(),
            unverified_tests: Vec::new(),
            current_specs: Vec::new(),
            planned_specs: Vec::new(),
            deprecated_specs: Vec::new(),
        }
    }

    #[test]
    // @verifies SPECIAL.HEALTH_COMMAND.TRACEABILITY.DETERMINISTIC_ORDERING
    fn traceability_summaries_sort_items_deterministically() {
        let mut module = ModuleTraceabilitySummary {
            current_spec_items: vec![
                module_item("zeta", 3, ModuleItemKind::Function),
                module_item("alpha", 9, ModuleItemKind::Method),
                module_item("alpha", 2, ModuleItemKind::Function),
            ],
            ..ModuleTraceabilitySummary::default()
        };

        module.sort_items();

        assert_eq!(
            module
                .current_spec_items
                .iter()
                .map(|item| (item.name.clone(), item.line, item.kind))
                .collect::<Vec<_>>(),
            vec![
                ("alpha".to_string(), 2, ModuleItemKind::Function),
                ("alpha".to_string(), 9, ModuleItemKind::Method),
                ("zeta".to_string(), 3, ModuleItemKind::Function),
            ]
        );

        let mut repo = ArchitectureTraceabilitySummary {
            current_spec_items: vec![
                architecture_item("src/z.rs", "same", 2, ModuleItemKind::Function),
                architecture_item("src/a.rs", "zeta", 4, ModuleItemKind::Function),
                architecture_item("src/a.rs", "alpha", 1, ModuleItemKind::Method),
            ],
            ..ArchitectureTraceabilitySummary::default()
        };

        repo.sort_items();

        assert_eq!(
            repo.current_spec_items
                .iter()
                .map(|item| {
                    (
                        item.path.to_string_lossy().to_string(),
                        item.name.clone(),
                        item.line,
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                ("src/a.rs".to_string(), "alpha".to_string(), 1),
                ("src/a.rs".to_string(), "zeta".to_string(), 4),
                ("src/z.rs".to_string(), "same".to_string(), 2),
            ]
        );
    }
}
