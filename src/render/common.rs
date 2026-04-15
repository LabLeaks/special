/**
@module SPECIAL.RENDER.COMMON
Provides shared render constants plus escaping and syntax-highlighting helpers used by text and HTML backends.
*/
// @fileimplements SPECIAL.RENDER.COMMON
use std::path::Path;
use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

pub(super) const SPEC_HTML_STYLE: &str = r#":root{color-scheme:light;--bg:#fcfcfb;--panel:#ffffff;--border:#e7e5e4;--text:#1c1917;--muted:#6b7280;--code-bg:#f5f5f4;--planned-bg:#fff7ed;--planned-text:#9a3412;--unsupported-bg:#fef2f2;--unsupported-text:#b91c1c;--group-bg:#f5f5f4;--group-text:#44403c;}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--text);font:15px/1.5 ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
main{max-width:980px;margin:0 auto;padding:32px 20px 56px}
h1{margin:0 0 8px;font-size:28px;line-height:1.15}
.lede{margin:0 0 24px;color:var(--muted)}
ul{list-style:none;padding-left:18px;margin:0}
.tree{padding-left:0}
li{margin:12px 0}
.node{background:var(--panel);border:1px solid var(--border);border-radius:10px;padding:12px 14px;box-shadow:0 1px 2px rgba(0,0,0,.03)}
.node-header{display:flex;align-items:center;gap:8px;flex-wrap:wrap}
.node-id{font:600 13px/1.4 ui-monospace,SFMono-Regular,Menlo,monospace;letter-spacing:.01em}
.badge{display:inline-block;padding:2px 8px;border-radius:999px;font-size:12px;line-height:1.5}
.badge-group{background:var(--group-bg);color:var(--group-text)}
.badge-planned{background:var(--planned-bg);color:var(--planned-text)}
.badge-unsupported{background:var(--unsupported-bg);color:var(--unsupported-text);font-weight:600}
.node-text{margin-top:6px;color:#292524}
.meta{margin-top:8px;color:var(--muted);font-size:13px}
.counts{display:flex;gap:12px;flex-wrap:wrap}
details{margin-top:10px;border-top:1px solid var(--border);padding-top:10px}
summary{cursor:pointer;color:#374151;font:600 13px/1.4 ui-monospace,SFMono-Regular,Menlo,monospace}
summary::marker{color:#9ca3af}
.code-block{margin:8px 0 0;white-space:pre-wrap;background:var(--code-bg);padding:12px;border-radius:8px;overflow:auto;font:13px/1.45 ui-monospace,SFMono-Regular,Menlo,monospace}"#;

pub(super) const SPEC_HTML_EMPTY: &str = "<p>No specs found.</p></main></body></html>";
pub(super) const MODULES_HTML_EMPTY: &str = "<p>No modules found.</p></main></body></html>";

pub(super) fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(super) fn planned_badge_text(planned_release: Option<&str>) -> String {
    match planned_release {
        Some(release) => format!("planned: {release}"),
        None => "planned".to_string(),
    }
}

pub(super) fn language_name_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "rust",
        Some("go") => "go",
        Some("ts" | "tsx") => "typescript",
        Some("sh") => "shell",
        Some("py") => "python",
        _ => "text",
    }
}

pub(super) fn highlight_code_html(body: &str, language: &str) -> String {
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
