// Askama-backed HTML template adapters for spec pages and nodes.
// @fileimplements SPECIAL.RENDER.HTML.SPEC
use askama::Template;

use crate::model::{NodeKind, SpecNode};
use crate::render::common::{deprecated_badge_text, planned_badge_text};
use crate::render::html_common::{
    SPEC_HTML_STYLE, escape_html, highlight_code_html, language_name_for_path,
};
use crate::render::templates::render_template;

use super::super::{
    DetailSectionsHtmlTemplate, HtmlDetailSection, SpecVerboseHtmlTemplate, attest_label,
    verify_label,
};

#[derive(Template)]
#[template(path = "render/spec_page.html")]
struct SpecPageHtmlTemplate<'a> {
    nodes: &'a [SpecNode],
    verbose: bool,
    style: &'static str,
    metrics_html: String,
}

impl SpecPageHtmlTemplate<'_> {
    fn tree_html(&self) -> String {
        self.nodes
            .iter()
            .map(|node| {
                render_template(&SpecNodeHtmlTemplate {
                    node,
                    verbose: self.verbose,
                })
            })
            .collect()
    }
}

#[derive(Template)]
#[template(path = "render/spec_node.html")]
struct SpecNodeHtmlTemplate<'a> {
    node: &'a SpecNode,
    verbose: bool,
}

impl SpecNodeHtmlTemplate<'_> {
    fn node_id(&self) -> String {
        self.node.id.clone()
    }

    fn node_text(&self) -> String {
        self.node.text.clone()
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

    fn declared_at(&self) -> String {
        format!(
            "{}:{}",
            self.node.location.path.display(),
            self.node.location.line
        )
    }

    fn verbose_section(&self) -> String {
        if !self.verbose {
            return String::new();
        }
        let verifies = self
            .node
            .verifies
            .iter()
            .map(|verify| HtmlDetailSection {
                label: verify_label(verify),
                location: format!(
                    "{}:{}",
                    verify.location.path.display(),
                    verify.location.line
                ),
                body_at: verify
                    .body_location
                    .as_ref()
                    .map(|location| format!("{}:{}", location.path.display(), location.line)),
                body_html: verify.body.as_ref().map(|body| {
                    let language = language_name_for_path(
                        verify
                            .body_location
                            .as_ref()
                            .map(|location| location.path.as_path())
                            .unwrap_or(verify.location.path.as_path()),
                    );
                    highlight_code_html(body, language)
                }),
                language_class: language_name_for_path(
                    verify
                        .body_location
                        .as_ref()
                        .map(|location| location.path.as_path())
                        .unwrap_or(verify.location.path.as_path()),
                )
                .to_string(),
            })
            .collect::<Vec<_>>();
        let attests = self
            .node
            .attests
            .iter()
            .map(|attest| HtmlDetailSection {
                label: attest_label(attest),
                location: format!(
                    "{}:{}",
                    attest.location.path.display(),
                    attest.location.line
                ),
                body_at: None,
                body_html: attest.body.as_ref().map(|body| escape_html(body)),
                language_class: "text".to_string(),
            })
            .collect::<Vec<_>>();

        render_template(&SpecVerboseHtmlTemplate {
            declared_at: self.declared_at(),
            verifies_html: render_template(&DetailSectionsHtmlTemplate { details: &verifies }),
            attests_html: render_template(&DetailSectionsHtmlTemplate { details: &attests }),
        })
    }

    fn children_section(&self) -> String {
        if self.node.children.is_empty() {
            return String::new();
        }

        let children: String = self
            .node
            .children
            .iter()
            .map(|child| {
                render_template(&SpecNodeHtmlTemplate {
                    node: child,
                    verbose: self.verbose,
                })
            })
            .collect();
        format!("<ul>{children}</ul>")
    }
}

pub(super) fn render_spec_page_html(
    nodes: &[SpecNode],
    verbose: bool,
    metrics_html: String,
) -> String {
    render_template(&SpecPageHtmlTemplate {
        nodes,
        verbose,
        style: SPEC_HTML_STYLE,
        metrics_html,
    })
}
