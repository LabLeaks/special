use anyhow::Result;

use crate::model::{LintReport, SpecDocument, SpecNode};

pub fn render_spec_text(document: &SpecDocument, verbose: bool) -> String {
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

pub fn render_spec_json(document: &SpecDocument, verbose: bool) -> Result<String> {
    let document = if verbose {
        document.clone()
    } else {
        strip_support_bodies(document)
    };
    Ok(serde_json::to_string_pretty(&document)?)
}

pub fn render_spec_html(document: &SpecDocument, verbose: bool) -> String {
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>special spec</title>\
         <style>body{font-family:ui-monospace,monospace;padding:24px;line-height:1.5}\
         ul{list-style:none;padding-left:20px}li{margin:10px 0}\
         .meta{color:#555}.planned{color:#8a5a00}.unsupported{color:#a40000;font-weight:700}\
         details{margin-top:8px}pre{white-space:pre-wrap;background:#f7f7f7;padding:12px;border-radius:6px;overflow:auto}</style>\
         </head><body><h1>special spec</h1>",
    );

    if document.nodes.is_empty() {
        html.push_str("<p>No specs found.</p></body></html>");
        return html;
    }

    html.push_str("<ul>");
    for node in &document.nodes {
        render_node_html(node, verbose, &mut html);
    }
    html.push_str("</ul></body></html>");
    html
}

pub fn render_lint_text(report: &LintReport) -> String {
    if report.diagnostics.is_empty() {
        return "Lint clean.".to_string();
    }

    let mut output = String::new();
    for diagnostic in &report.diagnostics {
        output.push_str(&format!(
            "{}:{}: {}\n",
            diagnostic.path.display(),
            diagnostic.line,
            diagnostic.message
        ));
    }
    output.trim_end().to_string()
}

fn render_node_text(node: &SpecNode, depth: usize, verbose: bool, output: &mut String) {
    let indent = "  ".repeat(depth);
    output.push_str(&indent);
    output.push_str(&node.id);
    if node.kind == crate::model::NodeKind::Group {
        output.push_str(" [group]");
    }
    if node.planned {
        output.push_str(" [planned]");
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

fn render_node_html(node: &SpecNode, verbose: bool, html: &mut String) {
    html.push_str("<li><div>");
    html.push_str(&escape_html(&node.id));
    if node.kind == crate::model::NodeKind::Group {
        html.push_str(" <span class=\"meta\">[group]</span>");
    }
    if node.planned {
        html.push_str(" <span class=\"planned\">[planned]</span>");
    }
    if node.is_unsupported() {
        html.push_str(" <span class=\"unsupported\">[unsupported]</span>");
    }
    html.push_str("</div>");

    if !node.text.is_empty() {
        html.push_str("<div class=\"meta\">");
        html.push_str(&escape_html(&node.text));
        html.push_str("</div>");
    }

    html.push_str("<div class=\"meta\">");
    html.push_str(&format!(
        "verifies: {} | attests: {}",
        node.verifies.len(),
        node.attests.len()
    ));
    html.push_str("</div>");

    if verbose {
        html.push_str("<div class=\"meta\">declared at ");
        html.push_str(&escape_html(&format!(
            "{}:{}",
            node.location.path.display(),
            node.location.line
        )));
        html.push_str("</div>");

        for verify in &node.verifies {
            html.push_str("<details><summary>@verifies ");
            html.push_str(&escape_html(&format!(
                "{}:{}",
                verify.location.path.display(),
                verify.location.line
            )));
            html.push_str("</summary>");
            if let Some(body_location) = &verify.body_location {
                html.push_str("<div class=\"meta\">body at ");
                html.push_str(&escape_html(&format!(
                    "{}:{}",
                    body_location.path.display(),
                    body_location.line
                )));
                html.push_str("</div>");
            }
            if let Some(body) = &verify.body {
                html.push_str("<pre>");
                html.push_str(&escape_html(body));
                html.push_str("</pre>");
            }
            html.push_str("</details>");
        }

        for attest in &node.attests {
            html.push_str("<details><summary>@attests ");
            html.push_str(&escape_html(&format!(
                "{}:{}",
                attest.location.path.display(),
                attest.location.line
            )));
            html.push_str("</summary>");
            if let Some(body) = &attest.body {
                html.push_str("<pre>");
                html.push_str(&escape_html(body));
                html.push_str("</pre>");
            }
            html.push_str("</details>");
        }
    }

    if !node.children.is_empty() {
        html.push_str("<ul>");
        for child in &node.children {
            render_node_html(child, verbose, html);
        }
        html.push_str("</ul>");
    }

    html.push_str("</li>");
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn render_block_text(body: &str, depth: usize, output: &mut String) {
    let indent = "  ".repeat(depth);
    for line in body.lines() {
        output.push_str(&indent);
        output.push_str(line);
        output.push('\n');
    }
}

fn strip_support_bodies(document: &SpecDocument) -> SpecDocument {
    SpecDocument {
        nodes: document
            .nodes
            .iter()
            .cloned()
            .map(strip_node_support_bodies)
            .collect(),
    }
}

fn strip_node_support_bodies(mut node: SpecNode) -> SpecNode {
    for verify in &mut node.verifies {
        verify.body_location = None;
        verify.body = None;
    }
    for attest in &mut node.attests {
        attest.body = None;
    }
    node.children = node
        .children
        .into_iter()
        .map(strip_node_support_bodies)
        .collect();
    node
}

#[cfg(test)]
mod tests {
    use crate::model::{NodeKind, SourceLocation, SpecDocument, SpecNode};

    use super::{render_spec_html, render_spec_json};

    fn sample_document() -> SpecDocument {
        SpecDocument {
            nodes: vec![SpecNode {
                id: "SPECIAL.SPEC_COMMAND".to_string(),
                kind: NodeKind::Spec,
                text: "special spec materializes the current spec view.".to_string(),
                planned: false,
                location: SourceLocation {
                    path: "specs/special.rs".into(),
                    line: 1,
                },
                verifies: Vec::new(),
                attests: Vec::new(),
                children: Vec::new(),
            }],
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
    }
}
