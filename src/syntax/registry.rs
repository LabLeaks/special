/**
@module SPECIAL.SYNTAX.REGISTRY
Projects the shared `SPECIAL.LANGUAGE_PACKS` registry onto syntax parsing so shared parser entrypoints can discover language packs by path and dispatch to the right provider without hardcoding one parse branch per language in the syntax core.
*/
// @fileimplements SPECIAL.SYNTAX.REGISTRY
use std::path::Path;

use crate::language_packs;

use super::{ParsedSourceGraph, SourceLanguage};

pub(super) fn language_for_path(path: &Path) -> Option<SourceLanguage> {
    descriptor_for_path(path).map(|descriptor| descriptor.language)
}

pub(super) fn parse_source_graph_at_path(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    descriptor_for_path(path).and_then(|descriptor| (descriptor.parse_source_graph)(path, text))
}

#[cfg(test)]
pub(super) fn parse_source_graph_for_language(
    language: SourceLanguage,
    path: &Path,
    text: &str,
) -> Option<ParsedSourceGraph> {
    descriptor_for_language(language)
        .and_then(|descriptor| (descriptor.parse_source_graph)(path, text))
}

// @applies REGISTRY.PROVIDER_DESCRIPTOR
fn descriptor_for_path(path: &Path) -> Option<&'static language_packs::LanguagePackDescriptor> {
    language_packs::descriptors()
        .iter()
        .copied()
        .find(|descriptor| (descriptor.matches_path)(path))
}

#[cfg(test)]
fn descriptor_for_language(
    language: SourceLanguage,
) -> Option<&'static language_packs::LanguagePackDescriptor> {
    language_packs::descriptors()
        .iter()
        .copied()
        .find(|descriptor| descriptor.language == language)
}
