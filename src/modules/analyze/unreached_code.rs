/**
@module SPECIAL.MODULES.ANALYZE.UNREACHED_CODE
Surfaces conservative unreached-code indicators from built-in language analyzers, including unowned source items with no observed path from public or test roots.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.UNREACHED_CODE
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::model::{ArchitectureRepoSignalsSummary, ArchitectureUnreachedItem};

use super::FileOwnership;
use super::go::summarize_go_item_signals;
use super::read_owned_file_text;
use super::rust::item_signals::summarize_rust_item_signals;
use super::typescript::summarize_typescript_item_signals;

pub(super) fn apply_unowned_unreached_summary(
    root: &Path,
    files: &[PathBuf],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    coverage: &mut ArchitectureRepoSignalsSummary,
) {
    let mut details = Vec::new();

    for path in files {
        if file_ownership.contains_key(path) {
            continue;
        }
        let text = read_owned_file_text(root, path);
        let summary = match path.extension().and_then(|ext| ext.to_str()) {
            Some("go") => summarize_go_item_signals(path, &text),
            Some("rs") => summarize_rust_item_signals(path, &text),
            Some("ts" | "tsx") => summarize_typescript_item_signals(path, &text),
            _ => continue,
        };
        coverage.unowned_unreached_items += summary.unreached_item_count;
        for item in summary.unreached_items {
            details.push(ArchitectureUnreachedItem {
                path: super::display_path(root, path),
                name: item.name,
                kind: item.kind,
            });
        }
    }

    details.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.kind.cmp(&right.kind))
    });

    coverage.unowned_unreached_item_details = details;
}
