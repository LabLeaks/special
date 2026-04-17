/**
@module SPECIAL.RENDER
Coordinates shared projection policy and output backends for text, JSON, and HTML views over specs, modules, and lint diagnostics.
*/
// @fileimplements SPECIAL.RENDER
use anyhow::Result;

use crate::model::{LintReport, ModuleDocument, RepoDocument, SpecDocument};

mod common;
mod html;
mod json;
mod projection;
mod templates;
mod text;

pub fn render_spec_text(document: &SpecDocument, verbose: bool) -> String {
    text::render_spec_text(document, verbose)
}

pub fn render_module_text(document: &ModuleDocument, verbose: bool) -> String {
    text::render_module_text(document, verbose)
}

pub fn render_repo_text(document: &RepoDocument, verbose: bool) -> String {
    text::render_repo_text(document, verbose)
}

pub fn render_spec_json(document: &SpecDocument, verbose: bool) -> Result<String> {
    json::render_spec_json(document, verbose)
}

pub fn render_module_json(document: &ModuleDocument, verbose: bool) -> Result<String> {
    json::render_module_json(document, verbose)
}

pub fn render_repo_json(document: &RepoDocument, verbose: bool) -> Result<String> {
    json::render_repo_json(document, verbose)
}

pub fn render_spec_html(document: &SpecDocument, verbose: bool) -> String {
    html::render_spec_html(document, verbose)
}

pub fn render_module_html(document: &ModuleDocument, verbose: bool) -> String {
    html::render_module_html(document, verbose)
}

pub fn render_repo_html(document: &RepoDocument, verbose: bool) -> String {
    html::render_repo_html(document, verbose)
}

pub fn render_lint_text(report: &LintReport) -> String {
    text::render_lint_text(report)
}

#[cfg(test)]
mod tests {
    use crate::model::{
        NodeKind, PlanState, SourceLocation, SpecDecl, SpecDocument, SpecNode, VerifyRef,
    };

    use super::{render_spec_html, render_spec_json};

    fn sample_document() -> SpecDocument {
        SpecDocument {
            nodes: vec![SpecNode::new(
                SpecDecl::new(
                    "SPECIAL.SPEC_COMMAND".to_string(),
                    NodeKind::Spec,
                    "special specs materializes the current spec view.".to_string(),
                    PlanState::live(),
                    false,
                    None,
                    SourceLocation {
                        path: "/tmp/specs/special.rs".into(),
                        line: 1,
                    },
                )
                .expect("test should construct valid spec decl"),
                vec![VerifyRef {
                    spec_id: "SPECIAL.SPEC_COMMAND".to_string(),
                    location: SourceLocation {
                        path: "/tmp/tests/cli.rs".into(),
                        line: 12,
                    },
                    body_location: Some(SourceLocation {
                        path: "/tmp/tests/cli.rs".into(),
                        line: 13,
                    }),
                    body: Some("fn verifies_spec_command() {}".to_string()),
                }],
                Vec::new(),
                Vec::new(),
            )],
        }
    }

    #[test]
    fn renders_json_output() {
        let json = render_spec_json(&sample_document(), false).expect("json render should succeed");
        assert!(json.contains("\"SPECIAL.SPEC_COMMAND\""));
    }

    #[test]
    fn renders_html_output() {
        let html = render_spec_html(&sample_document(), false);
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("SPECIAL.SPEC_COMMAND"));
        assert!(!html.contains("<details><summary>@verifies"));
    }

    #[test]
    fn renders_verbose_html_output() {
        let html = render_spec_html(&sample_document(), true);
        assert!(html.contains("<code class=\"language-rust\">"));
        assert!(html.contains("style=\"color:"));
        assert!(html.contains("<details><summary>@verifies"));
    }
}
