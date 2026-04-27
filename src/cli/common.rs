/**
@module SPECIAL.CLI.COMMON
Owns shared CLI command helpers for cache-status reporting and path normalization used across multiple command surfaces.
*/
// @fileimplements SPECIAL.CLI.COMMON
use std::path::{Path, PathBuf};

use super::status::CommandStatus;
use crate::cache::format_cache_stats_summary;

pub(super) fn report_cache_stats(status: &CommandStatus) {
    if let Some(summary) = format_cache_stats_summary() {
        status.note(&summary);
    }
}

pub(super) fn resolve_cli_paths(current_dir: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                current_dir.join(path)
            }
        })
        .collect()
}
