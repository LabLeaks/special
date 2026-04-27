/**
@module SPECIAL.RENDER.HTML_COMMON
Provides HTML-only styling, escaping, and syntax-highlighting helpers for render backends.
*/
// @fileimplements SPECIAL.RENDER.HTML_COMMON
#[path = "html/code_templates.rs"]
mod code_templates;

pub(super) use code_templates::{highlight_code_html, language_name_for_path};

pub(super) const SPEC_HTML_STYLE: &str = r#":root{color-scheme:light;--bg:#fcfcfb;--panel:#ffffff;--border:#e7e5e4;--text:#1c1917;--muted:#6b7280;--code-bg:#f5f5f4;--planned-bg:#fff7ed;--planned-text:#9a3412;--deprecated-bg:#eff6ff;--deprecated-text:#1d4ed8;--unsupported-bg:#fef2f2;--unsupported-text:#b91c1c;--group-bg:#f5f5f4;--group-text:#44403c;}
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
.badge-deprecated{background:var(--deprecated-bg);color:var(--deprecated-text)}
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
