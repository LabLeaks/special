/**
@module SPECIAL.CONFIG.RESOLUTION
Defines the resolved project-root result surface and warning semantics exposed after config and root discovery complete.
*/
// @fileimplements SPECIAL.CONFIG.RESOLUTION
use std::path::PathBuf;

use super::SpecialVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSource {
    SpecialToml,
    Vcs,
    CurrentDir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootResolution {
    pub root: PathBuf,
    pub source: RootSource,
    pub version: SpecialVersion,
    pub version_explicit: bool,
    pub config_path: Option<PathBuf>,
    pub ignore_patterns: Vec<String>,
}

impl RootResolution {
    pub fn warning(&self) -> Option<String> {
        match self.source {
            RootSource::SpecialToml => None,
            RootSource::Vcs => Some(format!(
                "warning: using inferred VCS root `{}`; add special.toml for predictable root selection",
                self.root.display()
            )),
            RootSource::CurrentDir => Some(format!(
                "warning: using current directory `{}` as the project root; add special.toml for predictable root selection",
                self.root.display()
            )),
        }
    }
}
