// @fileimplements SPECIAL.RENDER.PROJECTION.MODULE_ANALYSIS.MODULE_VIEW
use crate::model::{ModuleItemKind, ModuleItemSignal, ModuleTraceabilityItem};
use crate::modules::analyze::explain::MetricExplanationKey;

use super::super::{ProjectedExplanation, ProjectedMetaLine, explanation};

pub(super) fn push_item_group(
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

pub(super) fn push_traceability_group(
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
