// Askama-backed text template adapters for spec pages and nodes.
// @fileimplements SPECIAL.RENDER.TEXT.SPEC
use std::fmt::Write;

use askama::Template;

use crate::model::{NodeKind, SpecNode};
use crate::render::common::{deprecated_badge_text, planned_badge_text};
use crate::render::templates::{render_template, text_indent};

use super::super::attachments::{render_attest_section, render_verify_section};

#[derive(Template)]
#[template(path = "render/spec_page.txt", escape = "none")]
struct SpecPageTextTemplate<'a> {
    nodes: &'a [SpecNode],
    verbose: bool,
}

impl SpecPageTextTemplate<'_> {
    fn render_nodes(&self) -> String {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let rendered = render_template(&SpecNodeTextTemplate {
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
#[template(path = "render/spec_node.txt", escape = "none")]
struct SpecNodeTextTemplate<'a> {
    node: &'a SpecNode,
    depth: usize,
    verbose: bool,
}

impl SpecNodeTextTemplate<'_> {
    fn indent(&self) -> String {
        text_indent(self.depth)
    }

    fn is_group(&self) -> bool {
        self.node.kind() == NodeKind::Group
    }

    fn planned_badge(&self) -> String {
        planned_badge_text(self.node.planned_release())
    }

    fn deprecated_badge(&self) -> String {
        deprecated_badge_text(self.node.deprecated_release())
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

        render_verify_section(&mut output, &indent, self.depth, &self.node.verifies);
        render_attest_section(&mut output, &indent, self.depth, &self.node.attests);

        output
    }

    fn children_section(&self) -> String {
        self.node
            .children
            .iter()
            .map(|child| {
                render_template(&SpecNodeTextTemplate {
                    node: child,
                    depth: self.depth + 1,
                    verbose: self.verbose,
                })
            })
            .collect()
    }
}

pub(super) fn render_spec_page_text(nodes: &[SpecNode], verbose: bool) -> String {
    render_template(&SpecPageTextTemplate { nodes, verbose })
}
