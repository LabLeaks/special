use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::model::{BlockLine, CommentBlock, OwnedItem, SourceLocation};

const SUPPORTED_EXTENSIONS: &[&str] = &["rs", "go", "ts", "tsx", "sh"];

#[derive(Clone, Copy)]
enum LineCommentStyle {
    Slash,
    Hash,
}

pub fn collect_comment_blocks(root: &Path) -> Result<Vec<CommentBlock>> {
    let mut blocks = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        if !entry
            .file_type()
            .map(|kind| kind.is_file())
            .unwrap_or(false)
        {
            continue;
        }
        if !is_supported(path) {
            continue;
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("reading source file {}", path.display()))?;
        blocks.extend(extract_blocks_from_text(path.to_path_buf(), &content));
    }

    Ok(blocks)
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

fn extract_blocks_from_text(path: PathBuf, content: &str) -> Vec<CommentBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut index = 0;
    let line_comment_style = line_comment_style_for_path(&path);

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();

        if is_line_comment(trimmed, line_comment_style) {
            let mut block_lines = Vec::new();

            while index < lines.len()
                && is_line_comment(lines[index].trim_start(), line_comment_style)
            {
                block_lines.push(BlockLine {
                    line: index + 1,
                    text: strip_line_comment(lines[index].trim_start(), line_comment_style)
                        .to_string(),
                });
                index += 1;
            }

            if block_lines
                .iter()
                .any(|line| line.text.trim_start().starts_with('@'))
            {
                blocks.push(CommentBlock {
                    path: path.clone(),
                    lines: block_lines,
                    owned_item: extract_owned_item(&path, &lines, index),
                });
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            let start = index + 1;
            let mut block_lines = Vec::new();
            let mut current = trimmed;

            loop {
                let line_number = index + 1;
                let is_first = line_number == start;
                let mut segment = if is_first {
                    strip_block_start(current)
                } else {
                    current
                };
                let mut ended = false;

                if let Some((before_end, _)) = segment.split_once("*/") {
                    segment = before_end;
                    ended = true;
                }

                block_lines.push(BlockLine {
                    line: line_number,
                    text: strip_block_line_prefix(segment).to_string(),
                });

                index += 1;
                if ended || index >= lines.len() {
                    break;
                }
                current = lines[index];
            }

            if block_lines
                .iter()
                .any(|line| line.text.trim_start().starts_with('@'))
            {
                blocks.push(CommentBlock {
                    path: path.clone(),
                    lines: block_lines,
                    owned_item: extract_owned_item(&path, &lines, index),
                });
            }
            continue;
        }

        index += 1;
    }

    blocks
}

fn line_comment_style_for_path(path: &Path) -> LineCommentStyle {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sh") => LineCommentStyle::Hash,
        _ => LineCommentStyle::Slash,
    }
}

fn is_line_comment(line: &str, style: LineCommentStyle) -> bool {
    match style {
        LineCommentStyle::Slash => line.starts_with("//"),
        LineCommentStyle::Hash => line.starts_with('#'),
    }
}

fn strip_line_comment(line: &str, style: LineCommentStyle) -> &str {
    let stripped = match style {
        LineCommentStyle::Slash => line
            .strip_prefix("///")
            .or_else(|| line.strip_prefix("//!"))
            .or_else(|| line.strip_prefix("//"))
            .unwrap_or(line),
        LineCommentStyle::Hash => line
            .strip_prefix("#!")
            .map(|_| "")
            .or_else(|| line.strip_prefix('#'))
            .unwrap_or(line),
    };
    stripped.strip_prefix(' ').unwrap_or(stripped)
}

fn strip_block_start(line: &str) -> &str {
    let stripped = line
        .strip_prefix("/**")
        .or_else(|| line.strip_prefix("/*"))
        .unwrap_or(line);
    stripped.strip_prefix(' ').unwrap_or(stripped)
}

fn strip_block_line_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();
    let stripped = trimmed.strip_prefix('*').unwrap_or(trimmed);
    stripped.strip_prefix(' ').unwrap_or(stripped)
}

fn extract_owned_item(path: &Path, lines: &[&str], index: usize) -> Option<OwnedItem> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sh") => extract_shell_owned_item(path, lines, index),
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

    if index >= lines.len() {
        return None;
    }

    let body = lines[index..].join("\n").trim_end().to_string();
    if body.is_empty() {
        return None;
    }

    Some(OwnedItem {
        location: SourceLocation {
            path: path.to_path_buf(),
            line: index + 1,
        },
        body,
    })
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

    let body = body_lines.join("\n").trim_end().to_string();
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::extract_blocks_from_text;

    #[test]
    // @verifies SPECIAL.PARSE.LINE_COMMENTS
    fn extracts_line_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.rs"),
            "// @spec AUTH\n// Auth works.\nfn main() {}\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[0].text, "@spec AUTH");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "fn main() {}"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.BLOCK_COMMENTS
    fn extracts_block_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.ts"),
            "/**\n * @verifies AUTH.LOGIN\n */\nexport {};\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[1].text, "@verifies AUTH.LOGIN");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "export {};"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.SHELL_COMMENTS
    fn extracts_shell_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("scripts/verify.sh"),
            "#!/usr/bin/env bash\n# @verifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED\nset -euo pipefail\n\nexec mise exec -- cargo clippy --all-targets --all-features -- -D warnings\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].lines[1].text,
            "@verifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED"
        );
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "set -euo pipefail\n\nexec mise exec -- cargo clippy --all-targets --all-features -- -D warnings"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.VERIFIES.ATTACHES_TO_NEXT_ITEM
    fn attaches_verify_block_to_next_item() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.rs"),
            "// @verifies AUTH.LOGIN\n#[test]\nfn verifies_auth_login() {\n    assert!(true);\n}\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "#[test]\nfn verifies_auth_login() {\n    assert!(true);\n}"
        );
    }
}
