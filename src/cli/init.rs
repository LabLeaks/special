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
            "version = \"{}\"\nroot = \".\"\n\n# Optional: keep generated or fixture-heavy paths out of health's\n# unexplained-by-spec bucket without hiding them from discovery or architecture.\n#\n# [health]\n# ignore-unexplained = [\"generated/**\"]\n#\n# Optional: tell tool-backed traceability to use the project's declared toolchain.\n# Out of the box, special understands these project contracts:\n#   - `mise.toml`\n#   - `.tool-versions` (asdf-compatible)\n#\n# If your project root is not where the toolchain file lives, or you want to pin the\n# contract explicitly, uncomment this block:\n#\n# [toolchain]\n# manager = \"mise\" # or \"asdf\"\n#\n# Optional: tune advisory pattern similarity benchmark centers.\n# Leave this commented out unless the default estimates are noisy for your codebase.\n#\n# [patterns.metrics]\n# high = 0.55\n# medium = 0.45\n# low = 0.20\n",
            SpecialVersion::CURRENT.as_str()
        ),
    )?;
    println!("Created {}", config_path.display());
    Ok(ExitCode::SUCCESS)
}
