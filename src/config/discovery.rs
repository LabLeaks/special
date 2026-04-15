/**
@module SPECIAL.CONFIG.ROOT_DISCOVERY
Resolves the active project root from `special.toml`, VCS markers, or the current directory. This module does not parse `special.toml` key syntax.
*/
// @fileimplements SPECIAL.CONFIG.ROOT_DISCOVERY
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::{RootResolution, RootSource, SpecialVersion, is_vcs_root, load_special_toml};

pub(super) fn resolve_project_root(start: &Path) -> Result<RootResolution> {
    let start = start.canonicalize()?;

    for ancestor in start.ancestors() {
        let config_path = ancestor.join("special.toml");
        if config_path.is_file() {
            let config = load_special_toml(&config_path)?;
            let root = match config.root {
                Some(configured_root) => {
                    let root =
                        ancestor
                            .join(configured_root)
                            .canonicalize()
                            .with_context(|| {
                                format!(
                                    "special.toml at `{}` points to a root that does not exist",
                                    config_path.display()
                                )
                            })?;
                    if !root.is_dir() {
                        bail!(
                            "special.toml at `{}` points to a root that is not a directory",
                            config_path.display()
                        );
                    }
                    root
                }
                None => ancestor.to_path_buf(),
            };
            return Ok(RootResolution {
                root,
                source: RootSource::SpecialToml,
                version: config.version,
                version_explicit: config.version_explicit,
                config_path: Some(config_path),
                ignore_patterns: config.ignore_patterns,
            });
        }
    }

    for ancestor in start.ancestors() {
        if is_vcs_root(ancestor) {
            return Ok(RootResolution {
                root: ancestor.to_path_buf(),
                source: RootSource::Vcs,
                version: SpecialVersion::V0,
                version_explicit: false,
                config_path: None,
                ignore_patterns: Vec::new(),
            });
        }
    }

    Ok(RootResolution {
        root: start,
        source: RootSource::CurrentDir,
        version: SpecialVersion::V0,
        version_explicit: false,
        config_path: None,
        ignore_patterns: Vec::new(),
    })
}
