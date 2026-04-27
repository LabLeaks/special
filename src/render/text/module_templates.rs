// Askama-backed text template adapters for architecture module pages and nodes.
// @fileimplements SPECIAL.RENDER.TEXT.MODULE
use std::fmt::Write;

use askama::Template;

use crate::model::{ArchitectureKind, ModuleDocument, ModuleNode};
use crate::render::common::planned_badge_text;
use crate::render::projection::project_module_analysis_view;
use crate::render::templates::{render_template, text_indent};

use super::super::analysis::render_projected_module_analysis;
use super::super::attachments::render_implementation_section;

#[derive(Template)]
#[template(path = "render/module_page.txt", escape = "none")]
struct ModulePageTextTemplate<'a> {
    document: &'a ModuleDocument,
    verbose: bool,
}

impl ModulePageTextTemplate<'_> {
    fn render_nodes(&self) -> String {
        self.document
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let rendered = render_template(&ModuleNodeTextTemplate {
                    node,
                    depth: 0,
                    verbose: self.verbose,
                });
                if index == 0 {
                    rendered
                } else {
                    format!("\n{rendered}")
                }
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/module_node.txt", escape = "none")]
struct ModuleNodeTextTemplate<'a> {
    node: &'a ModuleNode,
    depth: usize,
    verbose: bool,
}

impl ModuleNodeTextTemplate<'_> {
    fn indent(&self) -> String {
        text_indent(self.depth)
    }

    fn is_area(&self) -> bool {
        self.node.kind() == ArchitectureKind::Area
    }

    fn planned_badge(&self) -> String {
        planned_badge_text(self.node.planned_release())
    }

    fn analysis_section(&self) -> String {
        let indent = self.indent();
        project_module_analysis_view(self.node, self.verbose)
            .map(|analysis| render_projected_module_analysis(&indent, &analysis))
            .unwrap_or_default()
    }

    fn patterns_section(&self) -> String {
        let indent = self.indent();
        let mut output = String::new();
        if !self.node.pattern_applications.is_empty() {
            writeln!(
                output,
                "{}  pattern applications: {}",
                indent,
                self.node.pattern_applications.len()
            )
            .expect("string writes should succeed");
            for application in &self.node.pattern_applications {
                writeln!(output, "{}    {}", indent, application.pattern_id)
                    .expect("string writes should succeed");
            }
        }
        output
    }

    fn verbose_section(&self) -> String {
        if !self.verbose {
            return String::new();
        }

        let indent = self.indent();
        let mut output = String::new();
        writeln!(
            output,
            "{}  declared at: {}:{}",
            indent,
            self.node.location.path.display(),
            self.node.location.line
        )
        .expect("string writes should succeed");

        render_implementation_section(&mut output, &indent, self.depth, &self.node.implements);

        output
    }

    fn children_section(&self) -> String {
        self.node
            .children
            .iter()
            .map(|child| {
                render_template(&ModuleNodeTextTemplate {
                    node: child,
                    depth: self.depth + 1,
                    verbose: self.verbose,
                })
            })
            .collect()
    }
}

pub(super) fn render_module_page_text(document: &ModuleDocument, verbose: bool) -> String {
    render_template(&ModulePageTextTemplate { document, verbose })
}
