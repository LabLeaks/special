/**
@module SPECIAL.RENDER.TEXT
Renders projected specs, modules, repo health, overviews, and lint diagnostics into human-readable text output while centralizing shared text metric helpers.
*/
// @fileimplements SPECIAL.RENDER.TEXT
use crate::model::{
    ArchitectureMetricsSummary, GroupedCount, RepoMetricsSummary, RepoTraceabilityMetrics,
    SpecMetricsSummary,
};

mod analysis;
mod attachments;
#[path = "text/lint.rs"]
mod lint;
#[path = "text/module.rs"]
mod module;
#[path = "text/overview.rs"]
mod overview;
#[path = "text/repo.rs"]
mod repo;
#[path = "text/spec.rs"]
mod spec;

pub(super) use self::lint::render_lint_text;
pub(super) use self::module::render_module_text;
pub(super) use self::overview::render_overview_text;
pub(super) use self::repo::render_repo_text;
pub(super) use self::spec::render_spec_text;

pub(super) fn render_spec_metrics_text(metrics: &SpecMetricsSummary) -> String {
    let mut output = String::from("special specs metrics\n");
    output.push_str(&format!("  total specs: {}\n", metrics.total_specs));
    output.push_str(&format!(
        "  unverified specs: {}\n",
        metrics.unverified_specs
    ));
    output.push_str(&format!("  planned specs: {}\n", metrics.planned_specs));
    output.push_str(&format!(
        "  deprecated specs: {}\n",
        metrics.deprecated_specs
    ));
    output.push_str(&format!("  verified specs: {}\n", metrics.verified_specs));
    output.push_str(&format!("  attested specs: {}\n", metrics.attested_specs));
    output.push_str(&format!(
        "  specs with both supports: {}\n",
        metrics.specs_with_both_supports
    ));
    output.push_str(&format!("  verifies: {}\n", metrics.verifies));
    output.push_str(&format!(
        "    item-scoped verifies: {}\n",
        metrics.item_scoped_verifies
    ));
    output.push_str(&format!(
        "    file-scoped verifies: {}\n",
        metrics.file_scoped_verifies
    ));
    output.push_str(&format!(
        "    unattached verifies: {}\n",
        metrics.unattached_verifies
    ));
    output.push_str(&format!("  attests: {}\n", metrics.attests));
    output.push_str(&format!("    block attests: {}\n", metrics.block_attests));
    output.push_str(&format!("    file attests: {}\n", metrics.file_attests));
    append_grouped_counts_text(&mut output, "specs by file", &metrics.specs_by_file);
    append_grouped_counts_text(
        &mut output,
        "current specs by top-level id",
        &metrics.current_specs_by_top_level_id,
    );
    output
}

pub(super) fn render_repo_metrics_text(metrics: &RepoMetricsSummary) -> String {
    let mut output = String::from("special health metrics\n");
    output.push_str(&format!("  duplicate items: {}\n", metrics.duplicate_items));
    output.push_str(&format!("  unowned items: {}\n", metrics.unowned_items));
    append_grouped_counts_text(
        &mut output,
        "duplicate items by file",
        &metrics.duplicate_items_by_file,
    );
    append_grouped_counts_text(
        &mut output,
        "unowned items by file",
        &metrics.unowned_items_by_file,
    );
    if let Some(traceability) = &metrics.traceability {
        append_repo_traceability_metrics_text(&mut output, traceability);
    }
    output
}

pub(super) fn render_arch_metrics_text(metrics: &ArchitectureMetricsSummary) -> String {
    let mut output = String::from("special arch metrics\n");
    output.push_str(&format!("  total modules: {}\n", metrics.total_modules));
    output.push_str(&format!("  total areas: {}\n", metrics.total_areas));
    output.push_str(&format!(
        "  unimplemented modules: {}\n",
        metrics.unimplemented_modules
    ));
    output.push_str(&format!(
        "  file-scoped implements: {}\n",
        metrics.file_scoped_implements
    ));
    output.push_str(&format!(
        "  item-scoped implements: {}\n",
        metrics.item_scoped_implements
    ));
    output.push_str(&format!("  owned lines: {}\n", metrics.owned_lines));
    output.push_str(&format!("  public items: {}\n", metrics.public_items));
    output.push_str(&format!("  internal items: {}\n", metrics.internal_items));
    output.push_str(&format!(
        "  complexity functions: {}\n",
        metrics.complexity_functions
    ));
    output.push_str(&format!(
        "  total cyclomatic: {}\n",
        metrics.total_cyclomatic
    ));
    output.push_str(&format!("  max cyclomatic: {}\n", metrics.max_cyclomatic));
    output.push_str(&format!("  total cognitive: {}\n", metrics.total_cognitive));
    output.push_str(&format!("  max cognitive: {}\n", metrics.max_cognitive));
    output.push_str(&format!(
        "  quality public functions: {}\n",
        metrics.quality_public_functions
    ));
    output.push_str(&format!(
        "  quality parameters: {}\n",
        metrics.quality_parameters
    ));
    output.push_str(&format!(
        "  quality bool params: {}\n",
        metrics.quality_bool_params
    ));
    output.push_str(&format!(
        "  quality raw string params: {}\n",
        metrics.quality_raw_string_params
    ));
    output.push_str(&format!(
        "  quality panic sites: {}\n",
        metrics.quality_panic_sites
    ));
    output.push_str(&format!("  unreached items: {}\n", metrics.unreached_items));
    append_grouped_counts_text(&mut output, "modules by area", &metrics.modules_by_area);
    append_grouped_counts_text(
        &mut output,
        "owned lines by module",
        &metrics.owned_lines_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "max cyclomatic by module",
        &metrics.max_cyclomatic_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "max cognitive by module",
        &metrics.max_cognitive_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "panic sites by module",
        &metrics.panic_sites_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "unreached items by module",
        &metrics.unreached_items_by_module,
    );
    append_grouped_counts_text(&mut output, "fan in by module", &metrics.fan_in_by_module);
    append_grouped_counts_text(&mut output, "fan out by module", &metrics.fan_out_by_module);
    append_grouped_counts_text(
        &mut output,
        "ambiguous internal dependency targets by module",
        &metrics.ambiguous_internal_targets_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "unresolved internal dependency targets by module",
        &metrics.unresolved_internal_targets_by_module,
    );
    append_grouped_counts_text(
        &mut output,
        "external dependency targets by module",
        &metrics.external_dependency_targets_by_module,
    );
    output
}

fn append_repo_traceability_metrics_text(output: &mut String, metrics: &RepoTraceabilityMetrics) {
    output.push_str("  traceability\n");
    output.push_str(&format!("    analyzed items: {}\n", metrics.analyzed_items));
    output.push_str(&format!(
        "    current spec items: {}\n",
        metrics.current_spec_items
    ));
    output.push_str(&format!(
        "    statically mediated items: {}\n",
        metrics.statically_mediated_items
    ));
    output.push_str(&format!(
        "    unverified test items: {}\n",
        metrics.unverified_test_items
    ));
    output.push_str(&format!(
        "    unexplained items: {}\n",
        metrics.unexplained_items
    ));
    output.push_str(&format!(
        "    unexplained review-surface items: {}\n",
        metrics.unexplained_review_surface_items
    ));
    output.push_str(&format!(
        "    unexplained public items: {}\n",
        metrics.unexplained_public_items
    ));
    output.push_str(&format!(
        "    unexplained internal items: {}\n",
        metrics.unexplained_internal_items
    ));
    output.push_str(&format!(
        "    unexplained module-backed items: {}\n",
        metrics.unexplained_module_backed_items
    ));
    output.push_str(&format!(
        "    unexplained module-connected items: {}\n",
        metrics.unexplained_module_connected_items
    ));
    output.push_str(&format!(
        "    unexplained module-isolated items: {}\n",
        metrics.unexplained_module_isolated_items
    ));
    append_grouped_counts_text(
        output,
        "unexplained items by file",
        &metrics.unexplained_items_by_file,
    );
    append_grouped_counts_text(
        output,
        "unexplained review-surface items by file",
        &metrics.unexplained_review_surface_items_by_file,
    );
}

fn append_grouped_counts_text(output: &mut String, label: &str, counts: &[GroupedCount]) {
    if counts.is_empty() {
        return;
    }
    output.push_str(&format!("  {label}\n"));
    counts.iter().for_each(|group| {
        output.push_str(&format!("    {}: {}\n", group.value, group.count));
    });
}
