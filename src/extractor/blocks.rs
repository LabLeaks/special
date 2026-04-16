/**
@module SPECIAL.EXTRACTOR.BLOCKS
Collects contiguous supported comment blocks from source files, normalizes comment syntax, and attaches each retained block to the next owned item candidate.
*/
// @fileimplements SPECIAL.EXTRACTOR.BLOCKS
use std::path::{Path, PathBuf};

use crate::annotation_syntax::is_reserved_special_annotation;
use crate::model::{BlockLine, CommentBlock};

use super::owned_items::extract_owned_item;

#[derive(Clone, Copy)]
enum LineCommentStyle {
    Slash,
    Hash,
}

pub(crate) fn extract_blocks_from_text(path: PathBuf, content: &str) -> Vec<CommentBlock> {
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

            maybe_push_comment_block(&mut blocks, &path, &lines, index, block_lines);
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

            maybe_push_comment_block(&mut blocks, &path, &lines, index, block_lines);
            continue;
        }

        index += 1;
    }

    blocks
}

fn maybe_push_comment_block(
    blocks: &mut Vec<CommentBlock>,
    path: &Path,
    lines: &[&str],
    index: usize,
    block_lines: Vec<BlockLine>,
) {
    if block_lines
        .iter()
        .any(|line| is_reserved_special_annotation(line.text.trim_start()))
    {
        blocks.push(CommentBlock {
            path: path.to_path_buf(),
            lines: block_lines,
            owned_item: extract_owned_item(path, lines, index),
        });
    }
}

fn line_comment_style_for_path(path: &Path) -> LineCommentStyle {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sh" | "py") => LineCommentStyle::Hash,
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
