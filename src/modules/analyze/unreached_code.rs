/**
@module SPECIAL.MODULES.ANALYZE.UNREACHED_CODE
Surfaces repo-wide ownership gaps for analyzable source items outside declared modules.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.UNREACHED_CODE
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{ArchitectureRepoSignalsSummary, ArchitectureUnownedItem, ModuleItemKind};
use crate::syntax::{SourceItemKind, parse_source_graph};

use super::FileOwnership;
use super::read_owned_file_text;

pub(super) fn apply_unowned_item_summary(
    root: &Path,
    files: &[PathBuf],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    coverage: &mut ArchitectureRepoSignalsSummary,
) -> Result<()> {
    let mut details = Vec::new();

    for path in files {
        if file_ownership.contains_key(path) {
            continue;
        }
        let text = read_owned_file_text(root, path)?;
        let Some(graph) = parse_source_graph(path, &text) else {
            continue;
        };
        coverage.unowned_items += graph.items.len();
        for item in graph.items {
            details.push(ArchitectureUnownedItem {
                path: super::display_path(root, path),
                name: item.name,
                kind: match item.kind {
                    SourceItemKind::Function => ModuleItemKind::Function,
                    SourceItemKind::Method => ModuleItemKind::Method,
                },
            });
        }
    }

    details.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.kind.cmp(&right.kind))
    });

    coverage.unowned_item_details = details;
    Ok(())
}
