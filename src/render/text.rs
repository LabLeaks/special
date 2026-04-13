/**
@module SPECIAL.RENDER.TEXT
Renders projected specs, modules, and lint diagnostics into human-readable text output.
*/
// @implements SPECIAL.RENDER.TEXT
use crate::model::{
    ArchitectureKind, DiagnosticSeverity, LintReport, ModuleDocument, ModuleNode, SpecDocument,
    SpecNode,
};

use super::common::planned_badge_text;
use super::projection::{project_document, project_module_document};

pub(super) fn render_spec_text(document: &SpecDocument, verbose: bool) -> String {
    let document = project_document(document, verbose);
    if document.nodes.is_empty() {
        return "No specs found.".to_string();
    }

    let mut output = String::new();
    for (index, node) in document.nodes.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        render_node_text(node, 0, verbose, &mut output);
    }
    output
}

pub(super) fn render_module_text(document: &ModuleDocument, verbose: bool) -> String {
    let document = project_module_document(document, verbose);
    if document.nodes.is_empty() {
        return "No modules found.".to_string();
    }

    let mut output = String::new();
    for (index, node) in document.nodes.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        render_module_node_text(node, 0, verbose, &mut output);
    }
    output
}

pub(super) fn render_lint_text(report: &LintReport) -> String {
    if report.diagnostics.is_empty() {
        return "Lint clean.".to_string();
    }

    let mut output = String::new();
    for diagnostic in &report.diagnostics {
        let severity = match diagnostic.severity {
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Error => "error",
        };
        output.push_str(&format!(
            "{}:{}: {}: {}\n",
            diagnostic.path.display(),
            diagnostic.line,
            severity,
            diagnostic.message
        ));
    }
    output.trim_end().to_string()
}

fn render_node_text(node: &SpecNode, depth: usize, verbose: bool, output: &mut String) {
    let indent = "  ".repeat(depth);
    output.push_str(&indent);
    output.push_str(&node.id);
    if node.kind() == crate::model::NodeKind::Group {
        output.push_str(" [group]");
    }
    if node.is_planned() {
        output.push_str(" [");
        output.push_str(&planned_badge_text(node.planned_release()));
        output.push(']');
    }
    if node.is_unsupported() {
        output.push_str(" [unsupported]");
    }
    output.push('\n');

    if !node.text.is_empty() {
        output.push_str(&indent);
        output.push_str("  ");
        output.push_str(&node.text);
        output.push('\n');
    }

    output.push_str(&indent);
    output.push_str(&format!("  verifies: {}\n", node.verifies.len()));
    output.push_str(&indent);
    output.push_str(&format!("  attests: {}\n", node.attests.len()));

    if verbose {
        output.push_str(&indent);
        output.push_str("  declared at: ");
        output.push_str(&format!(
            "{}:{}\n",
            node.location.path.display(),
            node.location.line
        ));

        for verify in &node.verifies {
            output.push_str(&indent);
            output.push_str("  @verifies ");
            output.push_str(&format!(
                "{}:{}\n",
                verify.location.path.display(),
                verify.location.line
            ));

            if let Some(body_location) = &verify.body_location {
                output.push_str(&indent);
                output.push_str("    body at: ");
                output.push_str(&format!(
                    "{}:{}\n",
                    body_location.path.display(),
                    body_location.line
                ));
            }

            if let Some(body) = &verify.body {
                render_block_text(body, depth + 2, output);
            }
        }

        for attest in &node.attests {
            output.push_str(&indent);
            output.push_str("  @attests ");
            output.push_str(&format!(
                "{}:{}\n",
                attest.location.path.display(),
                attest.location.line
            ));
            if let Some(body) = &attest.body {
                render_block_text(body, depth + 2, output);
            }
        }
    }

    for child in &node.children {
        render_node_text(child, depth + 1, verbose, output);
    }
}

fn render_module_node_text(node: &ModuleNode, depth: usize, verbose: bool, output: &mut String) {
    let indent = "  ".repeat(depth);
    output.push_str(&indent);
    output.push_str(&node.id);
    if node.kind() == ArchitectureKind::Area {
        output.push_str(" [area]");
    }
    if node.is_planned() {
        output.push_str(" [");
        output.push_str(&planned_badge_text(node.planned_release()));
        output.push(']');
    }
    if node.is_unsupported() {
        output.push_str(" [unsupported]");
    }
    output.push('\n');

    if !node.text.is_empty() {
        output.push_str(&indent);
        output.push_str("  ");
        output.push_str(&node.text);
        output.push('\n');
    }

    output.push_str(&indent);
    output.push_str(&format!("  implements: {}\n", node.implements.len()));

    if verbose {
        output.push_str(&indent);
        output.push_str("  declared at: ");
        output.push_str(&format!(
            "{}:{}\n",
            node.location.path.display(),
            node.location.line
        ));

        for implementation in &node.implements {
            output.push_str(&indent);
            output.push_str("  @implements ");
            output.push_str(&format!(
                "{}:{}\n",
                implementation.location.path.display(),
                implementation.location.line
            ));

            if let Some(body_location) = &implementation.body_location {
                output.push_str(&indent);
                output.push_str("    body at: ");
                output.push_str(&format!(
                    "{}:{}\n",
                    body_location.path.display(),
                    body_location.line
                ));
            }

            if let Some(body) = &implementation.body {
                render_block_text(body, depth + 2, output);
            }
        }
    }

    for child in &node.children {
        render_module_node_text(child, depth + 1, verbose, output);
    }
}

fn render_block_text(body: &str, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    for line in body.lines() {
        output.push_str(&indent);
        output.push_str(line);
        output.push('\n');
    }
}
