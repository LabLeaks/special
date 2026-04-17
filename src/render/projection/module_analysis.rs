/**
@module SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS
Projects module analysis into a shared view model of counts, explanation rows, and supporting detail lines that text and HTML renderers can present differently.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS
use crate::model::{
    ArchitectureRepoSignalsSummary, ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary,
    ModuleAnalysisSummary, ModuleItemKind, ModuleItemSignal, ModuleNode, ModuleTraceabilityItem,
};
use crate::modules::analyze::explain::{MetricExplanationKey, metric_explanation};

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedCount {
    pub(in crate::render) label: &'static str,
    pub(in crate::render) value: String,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedMetaLine {
    pub(in crate::render) label: &'static str,
    pub(in crate::render) value: String,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedExplanation {
    pub(in crate::render) label: &'static str,
    pub(in crate::render) plain: &'static str,
    pub(in crate::render) precise: &'static str,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedRepoSignals {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
    pub(in crate::render) unowned_unreached_items: Vec<String>,
    pub(in crate::render) duplicate_items: Vec<String>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedArchitectureTraceability {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) items: Vec<ProjectedMetaLine>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedModuleAnalysis {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) meta_lines: Vec<ProjectedMetaLine>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
}

pub(in crate::render) fn project_repo_signals_view(
    coverage: &ArchitectureRepoSignalsSummary,
    verbose: bool,
) -> ProjectedRepoSignals {
    let explanations = vec![
        explanation(
            "unowned unreached items",
            MetricExplanationKey::UnownedUnreachedItems,
        ),
        explanation("duplicate items", MetricExplanationKey::DuplicateItems),
    ];

    ProjectedRepoSignals {
        counts: vec![
            count("unowned unreached items", coverage.unowned_unreached_items),
            count("duplicate items", coverage.duplicate_items),
        ],
        explanations,
        unowned_unreached_items: if verbose {
            coverage
                .unowned_unreached_item_details
                .iter()
                .map(|item| {
                    format!(
                        "{}:{} [{}]",
                        item.path.display(),
                        item.name,
                        match item.kind {
                            ModuleItemKind::Function => "function",
                            ModuleItemKind::Method => "method",
                        }
                    )
                })
                .collect()
        } else {
            Vec::new()
        },
        duplicate_items: coverage
            .duplicate_item_details
            .iter()
            .take(if verbose { usize::MAX } else { 5 })
            .map(|item| {
                format!(
                    "{}:{}:{} [{}; duplicate peers {}]",
                    item.module_id,
                    item.path.display(),
                    item.name,
                    match item.kind {
                        ModuleItemKind::Function => "function",
                        ModuleItemKind::Method => "method",
                    },
                    item.duplicate_peer_count,
                )
            })
            .collect(),
    }
}

pub(in crate::render) fn project_repo_traceability_view(
    traceability: &ArchitectureTraceabilitySummary,
) -> ProjectedArchitectureTraceability {
    let mut counts = vec![count(
        "traceability items analyzed",
        traceability.analyzed_items,
    )];
    if !traceability.live_spec_items.is_empty() {
        counts.push(count("live spec items", traceability.live_spec_items.len()));
    }
    if !traceability.planned_only_items.is_empty() {
        counts.push(count(
            "planned-only items",
            traceability.planned_only_items.len(),
        ));
    }
    if !traceability.deprecated_only_items.is_empty() {
        counts.push(count(
            "deprecated-only items",
            traceability.deprecated_only_items.len(),
        ));
    }
    if !traceability.file_scoped_only_items.is_empty() {
        counts.push(count(
            "file-scoped-only items",
            traceability.file_scoped_only_items.len(),
        ));
    }
    if !traceability.unverified_test_items.is_empty() {
        counts.push(count(
            "unverified-test items",
            traceability.unverified_test_items.len(),
        ));
    }
    if !traceability.unknown_items.is_empty() {
        counts.push(count("unknown items", traceability.unknown_items.len()));
    }

    let mut items = Vec::new();
    push_architecture_traceability_group(
        &mut items,
        "live spec item",
        &traceability.live_spec_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "planned-only item",
        &traceability.planned_only_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "deprecated-only item",
        &traceability.deprecated_only_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "file-scoped-only item",
        &traceability.file_scoped_only_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "unverified-test item",
        &traceability.unverified_test_items,
    );
    push_architecture_traceability_group(&mut items, "unknown item", &traceability.unknown_items);

    ProjectedArchitectureTraceability { counts, items }
}

pub(in crate::render) fn project_module_analysis_view(
    node: &ModuleNode,
    verbose: bool,
) -> Option<ProjectedModuleAnalysis> {
    let analysis = node.analysis.as_ref()?;
    let mut counts = Vec::new();
    let mut meta_lines = Vec::new();
    let mut explanations = Vec::new();

    append_coverage(analysis, verbose, &mut counts, &mut meta_lines);
    append_metrics(analysis, &mut counts);
    append_complexity(analysis, &mut counts, &mut explanations);
    append_quality(analysis, &mut counts, &mut explanations);
    append_item_signals(analysis, &mut counts, &mut meta_lines, &mut explanations);
    append_traceability(analysis, &mut meta_lines);
    append_coupling(analysis, &mut counts, &mut explanations);
    append_dependencies(analysis, &mut counts, &mut meta_lines);

    Some(ProjectedModuleAnalysis {
        counts,
        meta_lines,
        explanations,
    })
}

fn append_coverage(
    analysis: &ModuleAnalysisSummary,
    _verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    _meta_lines: &mut Vec<ProjectedMetaLine>,
) {
    let Some(coverage) = &analysis.coverage else {
        return;
    };

    counts.push(count(
        "file-scoped implements",
        coverage.file_scoped_implements,
    ));
    counts.push(count(
        "item-scoped implements",
        coverage.item_scoped_implements,
    ));
}

fn append_metrics(analysis: &ModuleAnalysisSummary, counts: &mut Vec<ProjectedCount>) {
    let Some(metrics) = &analysis.metrics else {
        return;
    };

    counts.push(count("owned lines", metrics.owned_lines));
    counts.push(count("public items", metrics.public_items));
    counts.push(count("internal items", metrics.internal_items));
}

fn append_complexity(
    analysis: &ModuleAnalysisSummary,
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(complexity) = &analysis.complexity else {
        return;
    };

    counts.push(count("complexity functions", complexity.function_count));
    counts.push(count("cyclomatic total", complexity.total_cyclomatic));
    counts.push(count("cyclomatic max", complexity.max_cyclomatic));
    counts.push(count("cognitive total", complexity.total_cognitive));
    counts.push(count("cognitive max", complexity.max_cognitive));
    explanations.push(explanation(
        "cyclomatic total",
        MetricExplanationKey::CyclomaticTotal,
    ));
    explanations.push(explanation(
        "cyclomatic max",
        MetricExplanationKey::CyclomaticMax,
    ));
    explanations.push(explanation(
        "cognitive total",
        MetricExplanationKey::CognitiveTotal,
    ));
    explanations.push(explanation(
        "cognitive max",
        MetricExplanationKey::CognitiveMax,
    ));
}

fn append_quality(
    analysis: &ModuleAnalysisSummary,
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(quality) = &analysis.quality else {
        return;
    };

    counts.push(count(
        "quality public functions",
        quality.public_function_count,
    ));
    counts.push(count("quality parameters", quality.parameter_count));
    counts.push(count("quality bool params", quality.bool_parameter_count));
    counts.push(count(
        "quality raw string params",
        quality.raw_string_parameter_count,
    ));
    counts.push(count("quality panic sites", quality.panic_site_count));
    explanations.push(explanation(
        "quality public functions",
        MetricExplanationKey::QualityPublicFunctions,
    ));
    explanations.push(explanation(
        "quality parameters",
        MetricExplanationKey::QualityParameters,
    ));
    explanations.push(explanation(
        "quality bool params",
        MetricExplanationKey::QualityBoolParameters,
    ));
    explanations.push(explanation(
        "quality raw string params",
        MetricExplanationKey::QualityRawStringParameters,
    ));
    explanations.push(explanation(
        "quality panic sites",
        MetricExplanationKey::QualityPanicSites,
    ));
}

fn append_item_signals(
    analysis: &ModuleAnalysisSummary,
    counts: &mut Vec<ProjectedCount>,
    meta_lines: &mut Vec<ProjectedMetaLine>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(item_signals) = &analysis.item_signals else {
        return;
    };

    if item_signals.unreached_item_count > 0 {
        counts.push(count("unreached items", item_signals.unreached_item_count));
        explanations.push(explanation(
            "unreached items",
            MetricExplanationKey::UnreachedItems,
        ));
    }
    meta_lines.push(ProjectedMetaLine {
        label: "item signals analyzed",
        value: item_signals.analyzed_items.to_string(),
    });
    push_item_group(
        meta_lines,
        explanations,
        "connected item",
        MetricExplanationKey::ConnectedItem,
        &item_signals.connected_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        "outbound-heavy item",
        MetricExplanationKey::OutboundHeavyItem,
        &item_signals.outbound_heavy_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        "isolated item",
        MetricExplanationKey::IsolatedItem,
        &item_signals.isolated_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        "unreached item",
        MetricExplanationKey::UnreachedItem,
        &item_signals.unreached_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        "highest complexity item",
        MetricExplanationKey::HighestComplexityItem,
        &item_signals.highest_complexity_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        "parameter-heavy item",
        MetricExplanationKey::ParameterHeavyItem,
        &item_signals.parameter_heavy_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        "stringly boundary item",
        MetricExplanationKey::StringlyBoundaryItem,
        &item_signals.stringly_boundary_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        "panic-heavy item",
        MetricExplanationKey::PanicHeavyItem,
        &item_signals.panic_heavy_items,
    );
}

fn append_coupling(
    analysis: &ModuleAnalysisSummary,
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(coupling) = &analysis.coupling else {
        return;
    };

    counts.push(count("fan in", coupling.fan_in));
    counts.push(count("fan out", coupling.fan_out));
    counts.push(count("afferent coupling", coupling.afferent_coupling));
    counts.push(count("efferent coupling", coupling.efferent_coupling));
    counts.push(ProjectedCount {
        label: "instability",
        value: format!("{:.2}", coupling.instability),
    });
    counts.push(count(
        "external dependency targets",
        coupling.external_target_count,
    ));
    counts.push(count(
        "unresolved internal dependency targets",
        coupling.unresolved_internal_target_count,
    ));
    explanations.push(explanation("fan in", MetricExplanationKey::FanIn));
    explanations.push(explanation("fan out", MetricExplanationKey::FanOut));
    explanations.push(explanation(
        "afferent coupling",
        MetricExplanationKey::AfferentCoupling,
    ));
    explanations.push(explanation(
        "efferent coupling",
        MetricExplanationKey::EfferentCoupling,
    ));
    explanations.push(explanation(
        "instability",
        MetricExplanationKey::Instability,
    ));
}

fn append_traceability(analysis: &ModuleAnalysisSummary, meta_lines: &mut Vec<ProjectedMetaLine>) {
    let Some(traceability) = &analysis.traceability else {
        return;
    };

    meta_lines.push(ProjectedMetaLine {
        label: "traceability items analyzed",
        value: traceability.analyzed_items.to_string(),
    });
    push_traceability_group(meta_lines, "live spec item", &traceability.live_spec_items);
    push_traceability_group(
        meta_lines,
        "planned-only item",
        &traceability.planned_only_items,
    );
    push_traceability_group(
        meta_lines,
        "deprecated-only item",
        &traceability.deprecated_only_items,
    );
    push_traceability_group(
        meta_lines,
        "file-scoped-only item",
        &traceability.file_scoped_only_items,
    );
    push_traceability_group(
        meta_lines,
        "unverified-test item",
        &traceability.unverified_test_items,
    );
    push_traceability_group(meta_lines, "unknown item", &traceability.unknown_items);
}

fn push_architecture_traceability_group(
    meta_lines: &mut Vec<ProjectedMetaLine>,
    label: &'static str,
    items: &[ArchitectureTraceabilityItem],
) {
    meta_lines.extend(items.iter().map(|item| ProjectedMetaLine {
        label,
        value: architecture_traceability_value(item),
    }));
}

fn architecture_traceability_value(item: &ArchitectureTraceabilityItem) -> String {
    let mut suffix = Vec::new();
    if !item.live_specs.is_empty() {
        suffix.push(format!("live specs {}", item.live_specs.join(", ")));
    }
    if !item.planned_specs.is_empty() {
        suffix.push(format!("planned specs {}", item.planned_specs.join(", ")));
    }
    if !item.deprecated_specs.is_empty() {
        suffix.push(format!(
            "deprecated specs {}",
            item.deprecated_specs.join(", ")
        ));
    }
    if !item.verifying_tests.is_empty() {
        suffix.push(format!(
            "verifying tests {}",
            item.verifying_tests.join(", ")
        ));
    }
    if !item.unverified_tests.is_empty() {
        suffix.push(format!(
            "unverified tests {}",
            item.unverified_tests.join(", ")
        ));
    }
    if suffix.is_empty() {
        format!("{}:{}", item.module_id, item.name)
    } else {
        format!("{}:{} [{}]", item.module_id, item.name, suffix.join("; "))
    }
}

fn append_dependencies(
    analysis: &ModuleAnalysisSummary,
    counts: &mut Vec<ProjectedCount>,
    meta_lines: &mut Vec<ProjectedMetaLine>,
) {
    let Some(dependencies) = &analysis.dependencies else {
        return;
    };

    counts.push(count("dependency refs", dependencies.reference_count));
    counts.push(count("dependency targets", dependencies.distinct_targets));
    meta_lines.extend(dependencies.targets.iter().map(|target| ProjectedMetaLine {
        label: "dependency target",
        value: format!("{} ({})", target.path, target.count),
    }));
}

fn push_item_group(
    meta_lines: &mut Vec<ProjectedMetaLine>,
    explanations: &mut Vec<ProjectedExplanation>,
    label: &'static str,
    key: MetricExplanationKey,
    items: &[ModuleItemSignal],
) {
    if items.is_empty() {
        return;
    }

    meta_lines.extend(items.iter().map(|item| ProjectedMetaLine {
        label,
        value: format_item_signal(item),
    }));
    explanations.push(explanation(label, key));
}

fn push_traceability_group(
    meta_lines: &mut Vec<ProjectedMetaLine>,
    label: &'static str,
    items: &[ModuleTraceabilityItem],
) {
    if items.is_empty() {
        return;
    }

    meta_lines.extend(items.iter().map(|item| ProjectedMetaLine {
        label,
        value: format_traceability_item(item),
    }));
}

fn format_item_signal(item: &ModuleItemSignal) -> String {
    format!(
        "{} [{}; params {} (bool {}, raw string {}), internal refs {}, inbound {}, external refs {}, cyclomatic {}, cognitive {}, panic sites {}]",
        item.name,
        match item.kind {
            ModuleItemKind::Function => "function",
            ModuleItemKind::Method => "method",
        },
        item.parameter_count,
        item.bool_parameter_count,
        item.raw_string_parameter_count,
        item.internal_refs,
        item.inbound_internal_refs,
        item.external_refs,
        item.cyclomatic,
        item.cognitive,
        item.panic_site_count,
    )
}

fn format_traceability_item(item: &ModuleTraceabilityItem) -> String {
    let mut segments = Vec::new();
    if !item.live_specs.is_empty() {
        segments.push(format!("live specs {}", item.live_specs.join(", ")));
    }
    if !item.planned_specs.is_empty() {
        segments.push(format!("planned specs {}", item.planned_specs.join(", ")));
    }
    if !item.deprecated_specs.is_empty() {
        segments.push(format!(
            "deprecated specs {}",
            item.deprecated_specs.join(", ")
        ));
    }
    if !item.verifying_tests.is_empty() {
        segments.push(format!(
            "verifying tests {}",
            item.verifying_tests.join(", ")
        ));
    }
    if !item.unverified_tests.is_empty() {
        segments.push(format!(
            "unverified tests {}",
            item.unverified_tests.join(", ")
        ));
    }

    format!(
        "{} [{}; {}]",
        item.name,
        match item.kind {
            ModuleItemKind::Function => "function",
            ModuleItemKind::Method => "method",
        },
        segments.join("; ")
    )
}

fn explanation(label: &'static str, key: MetricExplanationKey) -> ProjectedExplanation {
    let explanation = metric_explanation(key);
    ProjectedExplanation {
        label,
        plain: explanation.plain,
        precise: explanation.precise,
    }
}

fn count(label: &'static str, value: usize) -> ProjectedCount {
    ProjectedCount {
        label,
        value: value.to_string(),
    }
}
