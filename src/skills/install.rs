/**
@module SPECIAL.SKILLS.INSTALL
Resolves install destinations and executes staged bundled-skill filesystem installs with overwrite and rollback handling. This module does not define the bundled skill catalog or command help text.
*/
// @fileimplements SPECIAL.SKILLS.INSTALL
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::{BundledSkill, bundled_skill, bundled_skills};

mod fs_ops;
mod transaction;

use fs_ops::{path_entry_exists, stage_skill_assets};
use transaction::InstallTransaction;

pub(crate) fn resolve_global_skills_root() -> Result<PathBuf> {
    if let Some(codex_home) = read_global_root_env("CODEX_HOME")? {
        return Ok(codex_home.join("skills"));
    }
    if let Some(home) = read_global_root_env("HOME")? {
        return Ok(home.join(".codex/skills"));
    }
    Err(anyhow::anyhow!(
        "global install destination unavailable: set `CODEX_HOME` or `HOME`, or pass a custom `--destination` path"
    ))
}

pub(crate) fn conflicting_skill_paths(
    destination_root: &Path,
    skill_id: Option<&str>,
) -> Result<Vec<(&'static BundledSkill, PathBuf)>> {
    let selected = selected_skills(skill_id)?;
    selected
        .into_iter()
        .map(|skill| (skill, destination_root.join(skill.id)))
        .filter_map(|(skill, path)| match path_entry_exists(&path) {
            Ok(true) => Some(Ok((skill, path))),
            Ok(false) => None,
            Err(err) => Some(Err(err.context(format!(
                "failed to check whether `{}` already exists",
                path.display()
            )))),
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn install_bundled_skills(
    destination_root: &Path,
    skill_id: Option<&str>,
    overwrite_existing: bool,
) -> Result<usize> {
    let selected = selected_skills(skill_id)?;
    let mut transaction = InstallTransaction::begin(destination_root)?;

    for skill in &selected {
        let staging_path = transaction.staging_root().join(skill.id);
        if let Err(err) = stage_skill_assets(transaction.staging_root(), skill) {
            return transaction.fail(err.context(format!(
                "failed to stage skill `{}` into `{}`",
                skill.id,
                staging_path.display()
            )));
        }
    }

    for skill in &selected {
        let skill_root = destination_root.join(skill.id);
        if path_entry_exists(&skill_root)
            .with_context(|| format!("failed to inspect `{}`", skill_root.display()))?
        {
            if !overwrite_existing {
                return transaction.fail(anyhow::anyhow!(
                    "skill `{}` already exists at `{}`",
                    skill.id,
                    skill_root.display()
                ));
            }

            if let Err(err) = transaction.backup_existing(skill) {
                return transaction.fail(err);
            }
        }
    }

    for skill in &selected {
        if let Err(err) = transaction.install_staged(skill) {
            return transaction.fail(err);
        }
    }

    transaction.cleanup().with_context(|| {
        format!(
            "installed {} skill(s) into `{}` but failed to remove temporary transaction directory",
            selected.len(),
            destination_root.display()
        )
    })?;
    Ok(selected.len())
}

fn selected_skills(skill_id: Option<&str>) -> Result<Vec<&'static BundledSkill>> {
    match skill_id {
        Some(skill_id) => {
            Ok(vec![bundled_skill(skill_id).ok_or_else(|| {
                anyhow::anyhow!("unknown skill id `{skill_id}`")
            })?])
        }
        None => Ok(bundled_skills().iter().collect()),
    }
}

fn read_global_root_env(name: &str) -> Result<Option<PathBuf>> {
    let Some(value) = env::var_os(name) else {
        return Ok(None);
    };
    if value.is_empty() {
        bail!(
            "global install destination unavailable: `{name}` is set but empty; pass a custom `--destination` path or set `{name}` to an absolute directory"
        );
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        bail!(
            "global install destination unavailable: `{name}` must be an absolute directory; pass a custom `--destination` path or set `{name}` to an absolute directory"
        );
    }
    Ok(Some(path))
}
