/**
@module SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS
Projects module analysis into a shared view model of counts, explanation rows, and supporting detail lines that text and HTML renderers can present differently.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS
use crate::model::{
    ArchitectureCoverageSummary, ModuleAnalysisSummary, ModuleItemKind, ModuleItemSignal,
    ModuleNode,
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
pub(in crate::render) struct ProjectedArchitectureCoverage {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) uncovered_paths: Vec<String>,
    pub(in crate::render) weak_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub(in crate::render) struct ProjectedModuleAnalysis {
    pub(in crate::render) counts: Vec<ProjectedCount>,
    pub(in crate::render) meta_lines: Vec<ProjectedMetaLine>,
    pub(in crate::render) explanations: Vec<ProjectedExplanation>,
}

pub(in crate::render) fn project_architecture_coverage_view(
    coverage: &ArchitectureCoverageSummary,
    verbose: bool,
) -> ProjectedArchitectureCoverage {
    ProjectedArchitectureCoverage {
        counts: vec![
            count("analyzed files", coverage.analyzed_files),
            count("covered files", coverage.covered_files),
            count("uncovered files", coverage.uncovered_files),
            count("weak files", coverage.weak_files),
        ],
        uncovered_paths: if verbose {
            coverage
                .uncovered_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect()
        } else {
            Vec::new()
        },
        weak_paths: if verbose {
            coverage
                .weak_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect()
        } else {
            Vec::new()
        },
    }
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
    append_item_signals(analysis, &mut meta_lines, &mut explanations);
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
    verbose: bool,
    counts: &mut Vec<ProjectedCount>,
    meta_lines: &mut Vec<ProjectedMetaLine>,
) {
    let Some(coverage) = &analysis.coverage else {
        return;
    };

    counts.push(count("covered files", coverage.covered_files));
    counts.push(count("weak files", coverage.weak_files));
    counts.push(count(
        "file-scoped implements",
        coverage.file_scoped_implements,
    ));
    counts.push(count(
        "item-scoped implements",
        coverage.item_scoped_implements,
    ));

    if verbose {
        meta_lines.extend(coverage.covered_paths.iter().map(|path| ProjectedMetaLine {
            label: "covered file",
            value: path.display().to_string(),
        }));
        meta_lines.extend(coverage.weak_paths.iter().map(|path| ProjectedMetaLine {
            label: "weak file",
            value: path.display().to_string(),
        }));
    }
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
    meta_lines: &mut Vec<ProjectedMetaLine>,
    explanations: &mut Vec<ProjectedExplanation>,
) {
    let Some(item_signals) = &analysis.item_signals else {
        return;
    };

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
