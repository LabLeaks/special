/**
@module SPECIAL.CONFIG
Re-exports configuration versioning and resolved-root entrypoints while delegating `special.toml` parsing and project-root discovery to narrower submodules.
*/
// @fileimplements SPECIAL.CONFIG
use std::path::Path;

use anyhow::Result;

mod discovery;
mod resolution;
mod special_toml;
mod version;

pub use resolution::{RootResolution, RootSource};
pub use version::SpecialVersion;

pub fn resolve_project_root(start: &Path) -> Result<RootResolution> {
    discovery::resolve_project_root(start)
}
