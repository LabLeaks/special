/**
@module SPECIAL.EXTRACTOR.OWNED_ITEMS
Infers the next owned source item for a retained comment block, including language-specific rules for shell, Python, and brace-delimited languages.
*/
// @fileimplements SPECIAL.EXTRACTOR.OWNED_ITEMS
use std::path::Path;

use crate::model::{OwnedItem, SourceLocation};
use crate::text_lines::skip_blank_lines;

pub(crate) fn extract_owned_item(path: &Path, lines: &[&str], index: usize) -> Option<OwnedItem> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sh") => extract_shell_owned_item(path, lines, index),
        Some("py") => extract_python_owned_item(path, lines, index),
        _ => extract_code_owned_item(path, lines, index),
    }
}

fn extract_shell_owned_item(path: &Path, lines: &[&str], mut index: usize) -> Option<OwnedItem> {
    index = skip_blank_lines(lines, index);
    index = skip_line_comments(lines, index, '#');
    index = skip_blank_lines(lines, index);

    build_owned_item(path, index, lines[index..].join("\n"))
}

fn extract_code_owned_item(path: &Path, lines: &[&str], mut index: usize) -> Option<OwnedItem> {
    index = skip_blank_lines(lines, index);

    if index >= lines.len() {
        return None;
    }

    let first = lines[index].trim_start();
    if first.starts_with("//") || first.starts_with("/*") {
        return None;
    }

    let start = index;
    let body_lines = collect_code_owned_body(lines, start);
    build_owned_item(path, start, body_lines.join("\n"))
}

fn extract_python_owned_item(path: &Path, lines: &[&str], mut index: usize) -> Option<OwnedItem> {
    index = skip_blank_lines(lines, index);

    if index >= lines.len() {
        return None;
    }

    let first = lines[index].trim_start();
    if first.starts_with('#') {
        return None;
    }

    let start = index;
    let body_lines = collect_python_owned_body(lines, start);
    build_owned_item(path, start, body_lines.join("\n"))
}

fn build_owned_item(path: &Path, start: usize, body: String) -> Option<OwnedItem> {
    let body = body.trim_end().to_string();
    if body.is_empty() {
        return None;
    }

    Some(OwnedItem {
        location: SourceLocation {
            path: path.to_path_buf(),
            line: start + 1,
        },
        body,
    })
}

fn skip_line_comments(lines: &[&str], mut index: usize, marker: char) -> usize {
    while index < lines.len() && lines[index].trim_start().starts_with(marker) {
        index += 1;
    }
    index
}

fn collect_code_owned_body<'a>(lines: &'a [&'a str], start: usize) -> Vec<&'a str> {
    let mut index = start;
    let mut body_lines = Vec::new();
    let mut brace_depth = 0_i32;
    let mut saw_open_brace = false;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if should_stop_code_body(index, start, trimmed, brace_depth, saw_open_brace) {
            break;
        }

        body_lines.push(line);
        update_brace_state(line, &mut brace_depth, &mut saw_open_brace);

        if saw_open_brace && brace_depth <= 0 {
            break;
        }
        if should_stop_code_signature(trimmed, saw_open_brace) {
            break;
        }

        index += 1;
    }

    body_lines
}

fn should_stop_code_body(
    index: usize,
    start: usize,
    trimmed: &str,
    brace_depth: i32,
    saw_open_brace: bool,
) -> bool {
    index > start && brace_depth == 0 && saw_open_brace && trimmed.is_empty()
}

fn update_brace_state(line: &str, brace_depth: &mut i32, saw_open_brace: &mut bool) {
    for ch in line.chars() {
        match ch {
            '{' => {
                *brace_depth += 1;
                *saw_open_brace = true;
            }
            '}' => *brace_depth -= 1,
            _ => {}
        }
    }
}

fn should_stop_code_signature(trimmed: &str, saw_open_brace: bool) -> bool {
    if saw_open_brace || trimmed.is_empty() {
        return false;
    }

    let continues = trimmed.ends_with(',')
        || trimmed.ends_with('(')
        || trimmed.ends_with('[')
        || trimmed.ends_with('=')
        || trimmed.starts_with("#[")
        || trimmed.starts_with('@');
    !continues && (trimmed.ends_with(';') || !trimmed.contains('('))
}

fn collect_python_owned_body<'a>(lines: &'a [&'a str], start: usize) -> Vec<&'a str> {
    let mut index = start;
    let mut body_lines = Vec::new();
    let base_indent = indentation(lines[start]);
    let mut saw_header = false;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        let indent = indentation(line);

        if index > start && should_stop_python_body(line, trimmed, indent, base_indent) {
            break;
        }
        if trimmed.is_empty() {
            if saw_header {
                body_lines.push(line);
            }
            index += 1;
            continue;
        }

        if trimmed.ends_with(':') {
            saw_header = true;
        }

        body_lines.push(line);
        index += 1;

        if !saw_header && index > start {
            break;
        }
    }

    body_lines
}

fn should_stop_python_body(line: &str, trimmed: &str, indent: usize, base_indent: usize) -> bool {
    !trimmed.is_empty()
        && indent <= base_indent
        && !line.trim_start().starts_with('@')
        && !line.trim_start().starts_with('#')
}

fn indentation(line: &str) -> usize {
    line.len() - line.trim_start().len()
}
