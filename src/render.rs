use anyhow::Result;
use std::path::Path;
use std::sync::OnceLock;

use crate::model::{LintReport, SpecDocument, SpecNode};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

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

pub fn render_spec_html(document: &SpecDocument, _verbose: bool) -> String {
    let verbose = true;
    let mut html = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>special spec</title>\
         <style>\
         :root{color-scheme:light;--bg:#fcfcfb;--panel:#ffffff;--border:#e7e5e4;--text:#1c1917;--muted:#6b7280;--code-bg:#f5f5f4;--planned-bg:#fff7ed;--planned-text:#9a3412;--unsupported-bg:#fef2f2;--unsupported-text:#b91c1c;--group-bg:#f5f5f4;--group-text:#44403c;}\
         *{box-sizing:border-box}\
         body{margin:0;background:var(--bg);color:var(--text);font:15px/1.5 ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif}\
         main{max-width:980px;margin:0 auto;padding:32px 20px 56px}\
         h1{margin:0 0 8px;font-size:28px;line-height:1.15}\
         .lede{margin:0 0 24px;color:var(--muted)}\
         ul{list-style:none;padding-left:18px;margin:0}\
         .tree{padding-left:0}\
         li{margin:12px 0}\
         .node{background:var(--panel);border:1px solid var(--border);border-radius:10px;padding:12px 14px;box-shadow:0 1px 2px rgba(0,0,0,.03)}\
         .node-header{display:flex;align-items:center;gap:8px;flex-wrap:wrap}\
         .node-id{font:600 13px/1.4 ui-monospace,SFMono-Regular,Menlo,monospace;letter-spacing:.01em}\
         .badge{display:inline-block;padding:2px 8px;border-radius:999px;font-size:12px;line-height:1.5}\
         .badge-group{background:var(--group-bg);color:var(--group-text)}\
         .badge-planned{background:var(--planned-bg);color:var(--planned-text)}\
         .badge-unsupported{background:var(--unsupported-bg);color:var(--unsupported-text);font-weight:600}\
         .node-text{margin-top:6px;color:#292524}\
         .meta{margin-top:8px;color:var(--muted);font-size:13px}\
         .counts{display:flex;gap:12px;flex-wrap:wrap}\
         details{margin-top:10px;border-top:1px solid var(--border);padding-top:10px}\
         summary{cursor:pointer;color:#374151;font:600 13px/1.4 ui-monospace,SFMono-Regular,Menlo,monospace}\
         summary::marker{color:#9ca3af}\
         .code-block{margin:8px 0 0;white-space:pre-wrap;background:var(--code-bg);padding:12px;border-radius:8px;overflow:auto;font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,monospace}\
         </style>\
         </head><body><main><h1>special spec</h1><p class=\"lede\">Materialized semantic spec view for the current repository.</p>",
    );

    if document.nodes.is_empty() {
        html.push_str("<p>No specs found.</p></main></body></html>");
        return html;
    }

    html.push_str("<ul class=\"tree\">");
    for node in &document.nodes {
        render_node_html(node, verbose, &mut html);
    }
    html.push_str("</ul></main></body></html>");
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
    html.push_str("<li><div class=\"node\"><div class=\"node-header\"><span class=\"node-id\">");
    html.push_str(&escape_html(&node.id));
    html.push_str("</span>");
    if node.kind == crate::model::NodeKind::Group {
        html.push_str("<span class=\"badge badge-group\">group</span>");
    }
    if node.planned {
        html.push_str("<span class=\"badge badge-planned\">planned</span>");
    }
    if node.is_unsupported() {
        html.push_str("<span class=\"badge badge-unsupported\">unsupported</span>");
    }
    html.push_str("</div>");

    if !node.text.is_empty() {
        html.push_str("<div class=\"node-text\">");
        html.push_str(&escape_html(&node.text));
        html.push_str("</div>");
    }

    html.push_str("<div class=\"meta counts\">");
    html.push_str(&format!("<span>verifies: {}</span>", node.verifies.len()));
    html.push_str(&format!("<span>attests: {}</span>", node.attests.len()));
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
                let language = language_name_for_path(
                    verify
                        .body_location
                        .as_ref()
                        .map(|location| location.path.as_path())
                        .unwrap_or(verify.location.path.as_path()),
                );
                html.push_str("<pre class=\"code-block\"><code class=\"language-");
                html.push_str(language);
                html.push_str("\">");
                html.push_str(&highlight_code_html(body, language));
                html.push_str("</code></pre>");
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
                html.push_str("<pre class=\"code-block\"><code class=\"language-text\">");
                html.push_str(&escape_html(body));
                html.push_str("</code></pre>");
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

    html.push_str("</div></li>");
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn language_name_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "rust",
        Some("go") => "go",
        Some("ts" | "tsx") => "typescript",
        Some("sh") => "shell",
        Some("py") => "python",
        _ => "text",
    }
}

fn highlight_code_html(body: &str, language: &str) -> String {
    let syntax_set = syntax_set();
    let syntax = syntax_for_language(syntax_set, language);
    let mut highlighter = HighlightLines::new(syntax, theme());
    let mut rendered = String::new();

    for line in LinesWithEndings::from(body) {
        match highlighter.highlight_line(line, syntax_set) {
            Ok(regions) => match styled_line_to_highlighted_html(&regions, IncludeBackground::No) {
                Ok(html) => rendered.push_str(&html),
                Err(_) => rendered.push_str(&escape_html(line)),
            },
            Err(_) => rendered.push_str(&escape_html(line)),
        }
    }

    rendered
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme() -> &'static Theme {
    static THEMES: OnceLock<ThemeSet> = OnceLock::new();
    let themes = THEMES.get_or_init(ThemeSet::load_defaults);
    if let Some(theme) = themes.themes.get("InspiredGitHub") {
        theme
    } else {
        themes
            .themes
            .values()
            .next()
            .expect("syntect should provide at least one theme")
    }
}

fn syntax_for_language<'a>(syntax_set: &'a SyntaxSet, language: &str) -> &'a SyntaxReference {
    match language {
        "rust" => syntax_set
            .find_syntax_by_extension("rs")
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text()),
        "go" => syntax_set
            .find_syntax_by_extension("go")
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text()),
        "typescript" => syntax_set
            .find_syntax_by_extension("ts")
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text()),
        "shell" => syntax_set
            .find_syntax_by_extension("sh")
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text()),
        "python" => syntax_set
            .find_syntax_by_extension("py")
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text()),
        _ => syntax_set.find_syntax_plain_text(),
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
    use crate::model::{NodeKind, SourceLocation, SpecDocument, SpecNode, VerifyRef};

    use super::{render_spec_html, render_spec_json};

    fn sample_document() -> SpecDocument {
        SpecDocument {
            nodes: vec![SpecNode {
                id: "SPECIAL.SPEC_COMMAND".to_string(),
                kind: NodeKind::Spec,
                text: "special spec materializes the current spec view.".to_string(),
                planned: false,
                location: SourceLocation {
                    path: "/tmp/specs/special.rs".into(),
                    line: 1,
                },
                verifies: vec![VerifyRef {
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
        assert!(html.contains("<code class=\"language-rust\">"));
        assert!(html.contains("style=\"color:"));
    }
}
