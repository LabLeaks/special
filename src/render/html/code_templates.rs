// Syntax-highlighting support used by HTML template adapters.
// @fileimplements SPECIAL.RENDER.HTML_COMMON
use std::path::Path;
use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{IncludeBackground, styled_line_to_highlighted_html};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

use crate::source_paths::has_extension;

use super::escape_html;

pub(in crate::render) fn language_name_for_path(path: &Path) -> &'static str {
    if has_extension(path, &["rs"]) {
        "rust"
    } else if has_extension(path, &["go"]) {
        "go"
    } else if has_extension(path, &["ts", "tsx"]) {
        "typescript"
    } else if has_extension(path, &["sh"]) {
        "shell"
    } else if has_extension(path, &["py"]) {
        "python"
    } else {
        "text"
    }
}

pub(in crate::render) fn highlight_code_html(body: &str, language: &str) -> String {
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
