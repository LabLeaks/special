/**
@module SPECIAL.CLI.INIT
Initialization command boundary in `src/cli/init.rs`. This module creates starter `special.toml` files while refusing to overwrite existing config or silently create nested config beneath an active ancestor root.
*/
// @fileimplements SPECIAL.CLI.INIT
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Result, bail};

use crate::config::{RootSource, SpecialVersion, resolve_project_root};

pub(super) fn execute_init(current_dir: &Path) -> Result<ExitCode> {
    let config_path = current_dir.join("special.toml");
    if config_path.try_exists().map_err(anyhow::Error::from)? {
        bail!("special.toml already exists at `{}`", config_path.display());
    }

    let resolution = resolve_project_root(current_dir)?;
    if resolution.source == RootSource::SpecialToml
        && let Some(active_config) = resolution.config_path
        && active_config != config_path
    {
        bail!(
            "special.toml at `{}` already governs `{}`; `special init` will not create a nested config",
            active_config.display(),
            current_dir.display()
        );
    }

    fs::write(
        &config_path,
        format!(
            "version = \"{}\"\nroot = \".\"\n",
            SpecialVersion::CURRENT.as_str()
        ),
    )?;
    println!("Created {}", config_path.display());
    Ok(ExitCode::SUCCESS)
}
