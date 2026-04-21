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
    pub(in crate::render) unowned_items: Vec<String>,
    pub(in crate::render) duplicate_items: Vec<String>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedArchitectureTraceability {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
    pub(in crate::render) items: Vec<ProjectedMetaLine>,
    pub(in crate::render) unavailable_reason: Option<String>,
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
        explanation("unowned items", MetricExplanationKey::UnownedItems),
        explanation("duplicate items", MetricExplanationKey::DuplicateItems),
    ];

    ProjectedRepoSignals {
        counts: vec![
            count("unowned items", coverage.unowned_items),
            count("duplicate items", coverage.duplicate_items),
        ],
        explanations,
        unowned_items: if verbose {
            coverage
                .unowned_item_details
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
        duplicate_items: if verbose {
            coverage
                .duplicate_item_details
                .iter()
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
                .collect()
        } else {
            Vec::new()
        },
    }
}

pub(in crate::render) fn project_repo_traceability_view(
    traceability: Option<&ArchitectureTraceabilitySummary>,
    unavailable_reason: Option<&str>,
) -> ProjectedArchitectureTraceability {
    let Some(traceability) = traceability else {
        return ProjectedArchitectureTraceability {
            counts: Vec::new(),
            explanations: Vec::new(),
            items: Vec::new(),
            unavailable_reason: unavailable_reason.map(ToString::to_string),
        };
    };
    let mut counts = vec![count(
        "traceability items analyzed",
        traceability.analyzed_items,
    )];
    let mut explanations = vec![ProjectedExplanation {
        label: "traceability items analyzed",
        plain: "this counts analyzable implementation items considered by backward trace in the current health view.",
        precise: "count of non-test implementation items included in repo traceability analysis for the active language pack.",
    }];
    if !traceability.current_spec_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "current spec items",
            traceability.current_spec_items.len(),
            "these items have a traced path from verifying tests tied to current specs.",
            "count of analyzed implementation items reached from at least one test with item- or file-scoped support for a current spec.",
        );
    }
    if !traceability.planned_only_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "planned-only items",
            traceability.planned_only_items.len(),
            "these items only trace to planned specs, not current ones.",
            "count of analyzed implementation items reached only from tests tied to planned specs and from no tests tied to current specs.",
        );
    }
    if !traceability.deprecated_only_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "deprecated-only items",
            traceability.deprecated_only_items.len(),
            "these items only trace to deprecated specs, not current ones.",
            "count of analyzed implementation items reached only from tests tied to deprecated specs and from no tests tied to current or planned specs.",
        );
    }
    if !traceability.file_scoped_only_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "file-scoped-only items",
            traceability.file_scoped_only_items.len(),
            "these items are justified only by file-scoped verification, not item-scoped verification.",
            "count of analyzed implementation items reached from tests with file-scoped support and with no item-scoped support attached to the item itself.",
        );
    }
    if !traceability.unverified_test_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "unverified-test items",
            traceability.unverified_test_items.len(),
            "these items are touched by tests that are not tied to specs.",
            "count of analyzed implementation items reached from tests with no attached current, planned, or deprecated spec support.",
        );
    }
    if !traceability.statically_mediated_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "statically mediated items",
            traceability.statically_mediated_items.len(),
            "these items are justified by a language-level static entry shape rather than a direct call chain.",
            "count of analyzed implementation items classified as traceable through mediated static edges such as trait or interface entrypoints.",
        );
    }
    if !traceability.unexplained_items.is_empty() {
        push_repo_traceability_count(
            &mut counts,
            &mut explanations,
            "unexplained items",
            traceability.unexplained_items.len(),
            "these items are still outside the currently explained spec-backed trace set.",
            "count of analyzed implementation items not classified as current, planned-only, deprecated-only, unverified-test, or statically mediated.",
        );
        for detail in unexplained_traceability_details(traceability) {
            push_repo_traceability_count(
                &mut counts,
                &mut explanations,
                detail.label,
                detail.value,
                detail.plain,
                detail.precise,
            );
        }
    }

    let mut items = Vec::new();
    push_architecture_traceability_group(
        &mut items,
        "current spec item",
        &traceability.current_spec_items,
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
    push_architecture_traceability_group(
        &mut items,
        "statically mediated item",
        &traceability.statically_mediated_items,
    );
    push_architecture_traceability_group(
        &mut items,
        "unexplained item",
        &traceability.unexplained_items,
    );

    ProjectedArchitectureTraceability {
        counts,
        explanations,
        items,
        unavailable_reason: unavailable_reason.map(ToString::to_string),
    }
}

struct RepoTraceabilityDetail {
    label: &'static str,
    value: usize,
    plain: &'static str,
    precise: &'static str,
}

fn push_repo_traceability_count(
    counts: &mut Vec<ProjectedCount>,
    explanations: &mut Vec<ProjectedExplanation>,
    label: &'static str,
    value: usize,
    plain: &'static str,
    precise: &'static str,
) {
    counts.push(count(label, value));
    explanations.push(ProjectedExplanation {
        label,
        plain,
        precise,
    });
}

fn unexplained_traceability_details(
    traceability: &ArchitectureTraceabilitySummary,
) -> [RepoTraceabilityDetail; 9] {
    [
        RepoTraceabilityDetail {
            label: "unexplained review-surface items",
            value: traceability.unexplained_review_surface_items(),
            plain: "these unexplained items are the main review pile: public API or root-visible entrypoints that behave like product surface.",
            precise: "count of unexplained implementation items marked public or root-visible by the active language pack, including process entrypoints such as `main`.",
        },
        RepoTraceabilityDetail {
            label: "unexplained public items",
            value: traceability.unexplained_public_items(),
            plain: "these unexplained items are public entrypoints or exported API surface.",
            precise: "count of unexplained implementation items marked public by the active language pack.",
        },
        RepoTraceabilityDetail {
            label: "unexplained internal items",
            value: traceability.unexplained_internal_items(),
            plain: "these unexplained items are internal implementation, not public API.",
            precise: "count of unexplained implementation items not marked public by the active language pack.",
        },
        RepoTraceabilityDetail {
            label: "unexplained test-file items",
            value: traceability.unexplained_test_file_items(),
            plain: "these unexplained items sit in files recognized as test files.",
            precise: "count of unexplained implementation items whose source path is under a tests directory or in a file named tests.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-owned items",
            value: traceability.unexplained_module_owned_items(),
            plain: "these unexplained items still belong to at least one declared module.",
            precise: "count of unexplained implementation items with one or more declared owning module ids.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-backed items",
            value: traceability.unexplained_module_backed_items(),
            plain: "these unexplained items sit in modules that already have current-spec-traced code somewhere else.",
            precise: "count of unexplained implementation items whose declared owning module ids include at least one module with current-spec-backed traced implementation.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-connected items",
            value: traceability.unexplained_module_connected_items(),
            plain: "these unexplained items also connect inside those modules to code that is already current-spec-traced.",
            precise: "count of unexplained implementation items in current-spec-backed modules that share a same-module call or reference component with current-spec-traced implementation.",
        },
        RepoTraceabilityDetail {
            label: "unexplained module-isolated items",
            value: traceability.unexplained_module_isolated_items(),
            plain: "these unexplained items are in current-spec-backed modules but still sit outside the connected traced cluster in those modules.",
            precise: "count of unexplained implementation items in current-spec-backed modules that do not share a same-module call or reference component with current-spec-traced implementation.",
        },
        RepoTraceabilityDetail {
            label: "unexplained unowned items",
            value: traceability.unexplained_unowned_items(),
            plain: "these unexplained items are outside all declared modules.",
            precise: "count of unexplained implementation items with no declared owning module ids.",
        },
    ]
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
    append_complexity(analysis, verbose, &mut counts, &mut explanations);
    append_quality(analysis, verbose, &mut counts, &mut explanations);
    append_item_signals(
        analysis,
        verbose,
        &mut counts,
        &mut meta_lines,
        &mut explanations,
    );
    append_traceability(analysis, verbose, &mut meta_lines);
    append_coupling(analysis, verbose, &mut counts, &mut explanations);
    append_dependencies(analysis, verbose, &mut counts, &mut meta_lines);

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
    verbose: bool,
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
    if verbose {
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
}

fn append_quality(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
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
    if verbose {
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
}

fn append_item_signals(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    meta_lines: &mut Vec<ProjectedMetaLine>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(item_signals) = &analysis.item_signals else {
        return;
    };

    if item_signals.unreached_item_count > 0 {
        counts.push(count("unreached items", item_signals.unreached_item_count));
        if verbose {
            explanations.push(explanation(
                "unreached items",
                MetricExplanationKey::UnreachedItems,
            ));
        }
    }
    if verbose {
        meta_lines.push(ProjectedMetaLine {
            label: "item signals analyzed",
            value: item_signals.analyzed_items.to_string(),
        });
    }
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "connected item",
        MetricExplanationKey::ConnectedItem,
        &item_signals.connected_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "outbound-heavy item",
        MetricExplanationKey::OutboundHeavyItem,
        &item_signals.outbound_heavy_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "isolated item",
        MetricExplanationKey::IsolatedItem,
        &item_signals.isolated_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "unreached item",
        MetricExplanationKey::UnreachedItem,
        &item_signals.unreached_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "highest complexity item",
        MetricExplanationKey::HighestComplexityItem,
        &item_signals.highest_complexity_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "parameter-heavy item",
        MetricExplanationKey::ParameterHeavyItem,
        &item_signals.parameter_heavy_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "stringly boundary item",
        MetricExplanationKey::StringlyBoundaryItem,
        &item_signals.stringly_boundary_items,
    );
    push_item_group(
        meta_lines,
        explanations,
        verbose,
        "panic-heavy item",
        MetricExplanationKey::PanicHeavyItem,
        &item_signals.panic_heavy_items,
    );
}

fn append_coupling(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
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
    if verbose {
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
}

fn append_traceability(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    meta_lines: &mut Vec<ProjectedMetaLine>,
) {
    if !verbose {
        return;
    }
    if let Some(reason) = &analysis.traceability_unavailable_reason {
        meta_lines.push(ProjectedMetaLine {
            label: "Rust backward trace unavailable",
            value: reason.clone(),
        });
        return;
    }

    let Some(traceability) = &analysis.traceability else {
        return;
    };

    meta_lines.push(ProjectedMetaLine {
        label: "traceability items analyzed",
        value: traceability.analyzed_items.to_string(),
    });
    push_traceability_group(
        meta_lines,
        "current spec item",
        &traceability.current_spec_items,
    );
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
    push_traceability_group(
        meta_lines,
        "statically mediated item",
        &traceability.statically_mediated_items,
    );
    push_traceability_group(
        meta_lines,
        "unexplained item",
        &traceability.unexplained_items,
    );
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
    suffix.push(if item.review_surface {
        if item.public {
            "review surface; public".to_string()
        } else {
            "review surface; root-visible entrypoint".to_string()
        }
    } else {
        "internal".to_string()
    });
    if item.module_backed_by_current_specs {
        suffix.push("module-backed".to_string());
        suffix.push(if item.module_connected_to_current_specs {
            "connected inside module".to_string()
        } else {
            "isolated inside module".to_string()
        });
    }
    if item.test_file {
        suffix.push("test file".to_string());
    }
    if item.module_ids.is_empty() {
        suffix.push("unowned".to_string());
    } else {
        suffix.push(format!("modules {}", item.module_ids.join(", ")));
    }
    if !item.current_specs.is_empty() {
        suffix.push(format!("current specs {}", item.current_specs.join(", ")));
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
    if let Some(reason) = &item.mediated_reason {
        suffix.push(format!("mediated reason {reason}"));
    }
    let base = format!("{}:{}", item.path.display(), item.name);
    if suffix.is_empty() {
        base
    } else {
        format!("{base} [{}]", suffix.join("; "))
    }
}

fn append_dependencies(
    analysis: &ModuleAnalysisSummary,
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    meta_lines: &mut Vec<ProjectedMetaLine>,
) {
    let Some(dependencies) = &analysis.dependencies else {
        return;
    };

    counts.push(count("dependency refs", dependencies.reference_count));
    counts.push(count("dependency targets", dependencies.distinct_targets));
    if verbose {
        meta_lines.extend(dependencies.targets.iter().map(|target| ProjectedMetaLine {
            label: "dependency target",
            value: format!("{} ({})", target.path, target.count),
        }));
    }
}

fn push_item_group(
    meta_lines: &mut Vec<ProjectedMetaLine>,
    explanations: &mut Vec<ProjectedExplanation>,
    verbose: bool,
    label: &'static str,
    key: MetricExplanationKey,
    items: &[ModuleItemSignal],
) {
    if items.is_empty() || !verbose {
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
    if !item.current_specs.is_empty() {
        segments.push(format!("current specs {}", item.current_specs.join(", ")));
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
    if let Some(reason) = &item.mediated_reason {
        segments.push(format!("mediated reason {reason}"));
    }

    let kind = match item.kind {
        ModuleItemKind::Function => "function",
        ModuleItemKind::Method => "method",
    };
    if segments.is_empty() {
        format!("{} [{}]", item.name, kind)
    } else {
        format!("{} [{}; {}]", item.name, kind, segments.join("; "))
    }
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
