/**
@module SPECIAL.MODULES.PARSE
Coordinates architecture parsing across markdown declarations, source-local declarations, and explicit implementation attachments.
*/
// @fileimplements SPECIAL.MODULES.PARSE
use std::path::Path;

use anyhow::Result;

use crate::model::ParsedArchitecture;

pub(super) mod declarations;
mod implements;
mod source;
use implements::parse_implements_refs;
use source::parse_source_module_decls;

use super::parse_markdown::parse_markdown_architecture_decls as parse_markdown_architecture_nodes;

pub(super) fn parse_architecture(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<ParsedArchitecture> {
    let mut parsed = ParsedArchitecture::default();
    parse_markdown_architecture_nodes(root, ignore_patterns, &mut parsed)?;
    parse_source_module_decls(root, ignore_patterns, &mut parsed)?;
    parse_implements_refs(root, ignore_patterns, &mut parsed)?;
    Ok(parsed)
}
