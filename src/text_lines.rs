/*
@module SPECIAL.TEXT_LINES
Shared helpers for cursor-style traversal over already-split text lines.
*/
// @fileimplements SPECIAL.TEXT_LINES

pub(crate) fn skip_blank_lines(lines: &[&str], mut index: usize) -> usize {
    while index < lines.len() && lines[index].trim().is_empty() {
        index += 1;
    }
    index
}
