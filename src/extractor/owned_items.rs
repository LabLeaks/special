/**
@module SPECIAL.EXTRACTOR.OWNED_ITEMS
Infers the next owned source item for a retained comment block, including language-specific rules for shell, Python, and brace-delimited languages.
*/
// @fileimplements SPECIAL.EXTRACTOR.OWNED_ITEMS
use std::path::Path;

use crate::model::{OwnedItem, SourceLocation};

pub(crate) fn extract_owned_item(path: &Path, lines: &[&str], index: usize) -> Option<OwnedItem> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sh") => extract_shell_owned_item(path, lines, index),
        Some("py") => extract_python_owned_item(path, lines, index),
        _ => extract_code_owned_item(path, lines, index),
    }
}

fn extract_shell_owned_item(path: &Path, lines: &[&str], mut index: usize) -> Option<OwnedItem> {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }
    while index < lines.len() && lines[index].trim_start().starts_with('#') {
        index += 1;
    }
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }

    build_owned_item(path, index, lines[index..].join("\n"))
}

fn extract_code_owned_item(path: &Path, lines: &[&str], mut index: usize) -> Option<OwnedItem> {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }

    if index >= lines.len() {
        return None;
    }

    let first = lines[index].trim_start();
    if first.starts_with("//") || first.starts_with("/*") {
        return None;
    }

    let start = index;
    let mut body_lines = Vec::new();
    let mut brace_depth = 0_i32;
    let mut saw_open_brace = false;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if index > start && brace_depth == 0 && saw_open_brace && trimmed.is_empty() {
            break;
        }

        body_lines.push(line);

        for ch in line.chars() {
            match ch {
                '{' => {
                    brace_depth += 1;
                    saw_open_brace = true;
                }
                '}' => brace_depth -= 1,
                _ => {}
            }
        }

        if saw_open_brace && brace_depth <= 0 {
            break;
        }

        if !saw_open_brace && !trimmed.is_empty() {
            let continues = trimmed.ends_with(',')
                || trimmed.ends_with('(')
                || trimmed.ends_with('[')
                || trimmed.ends_with('=')
                || trimmed.starts_with("#[")
                || trimmed.starts_with('@');
            if !continues && (trimmed.ends_with(';') || !trimmed.contains('(')) {
                break;
            }
        }

        index += 1;
    }

    build_owned_item(path, start, body_lines.join("\n"))
}

fn extract_python_owned_item(path: &Path, lines: &[&str], mut index: usize) -> Option<OwnedItem> {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }

    if index >= lines.len() {
        return None;
    }

    let first = lines[index].trim_start();
    if first.starts_with('#') {
        return None;
    }

    let start = index;
    let mut body_lines = Vec::new();
    let mut base_indent: Option<usize> = None;
    let mut saw_header = false;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if index == start {
            base_indent = Some(indent);
        } else if trimmed.is_empty() {
            if saw_header {
                body_lines.push(line);
            }
            index += 1;
            continue;
        } else if let Some(base_indent) = base_indent
            && indent <= base_indent
            && !line.trim_start().starts_with('@')
            && !line.trim_start().starts_with('#')
        {
            break;
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
