/**
@module SPECIAL.MODULES.ANALYZE.COVERAGE
Computes architecture-wide and per-module ownership coverage so hidden or weakly owned implementation can be inspected directly.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.COVERAGE
use std::fs;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use crate::model::ArchitectureCoverageSummary;

use super::{FileOwnership, display_path};

pub(super) fn build_coverage_summary(
    root: &Path,
    files: &[PathBuf],
    file_ownership: &std::collections::BTreeMap<PathBuf, FileOwnership<'_>>,
) -> ArchitectureCoverageSummary {
    let mut uncovered_paths = Vec::new();
    let mut weak_paths = Vec::new();
    let mut covered_files = 0;

    for path in files {
        match file_ownership.get(path) {
            None => uncovered_paths.push(display_path(root, path)),
            Some(ownership) => {
                covered_files += 1;
                if file_is_weakly_covered(path, ownership) {
                    weak_paths.push(display_path(root, path));
                }
            }
        }
    }

    ArchitectureCoverageSummary {
        analyzed_files: files.len(),
        covered_files,
        uncovered_files: uncovered_paths.len(),
        weak_files: weak_paths.len(),
        uncovered_paths,
        weak_paths,
    }
}

fn file_is_weakly_covered(path: &Path, ownership: &FileOwnership<'_>) -> bool {
    if !ownership.file_scoped.is_empty() {
        return true;
    }
    if ownership.item_scoped.is_empty() {
        return false;
    }
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let covered_ranges: Vec<RangeInclusive<usize>> = ownership
        .item_scoped
        .iter()
        .filter_map(|implementation| {
            let body_location = implementation.body_location.as_ref()?;
            let body = implementation.body.as_ref()?;
            let lines = line_count(body);
            Some(body_location.line..=body_location.line + lines.saturating_sub(1))
        })
        .collect();

    has_significant_lines_outside_ranges(path, &content, &covered_ranges)
}

fn has_significant_lines_outside_ranges(
    path: &Path,
    content: &str,
    covered_ranges: &[RangeInclusive<usize>],
) -> bool {
    content.lines().enumerate().any(|(index, line)| {
        let line_number = index + 1;
        if covered_ranges
            .iter()
            .any(|range| range.contains(&line_number))
        {
            return false;
        }
        is_significant_line(path, line)
    })
}

fn is_significant_line(path: &Path, line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sh" | "py") => !trimmed.starts_with('#'),
        Some("rs" | "go" | "ts" | "tsx") => {
            !trimmed.starts_with("//")
                && trimmed != "/*"
                && trimmed != "/**"
                && trimmed != "*/"
                && !trimmed.starts_with('*')
        }
        _ => true,
    }
}

fn line_count(text: &str) -> usize {
    text.lines().count().max(1)
}
