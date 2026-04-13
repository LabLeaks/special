/**
@module SPECIAL.SKILLS.INSTALL
Resolves install destinations and executes staged bundled-skill filesystem installs with overwrite and rollback handling. This module does not define the bundled skill catalog or command help text.
*/
// @implements SPECIAL.SKILLS.INSTALL
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::{BundledSkill, bundled_skill, bundled_skills};

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
    fs::create_dir_all(destination_root)?;

    let transaction_root = destination_root.join(unique_install_path("skills", "transaction"));
    let staging_root = transaction_root.join("staging");
    let backup_root = transaction_root.join("backup");
    fs::create_dir_all(&staging_root)?;

    for skill in &selected {
        if let Err(err) = stage_skill_assets(&staging_root, skill) {
            return fail_install(
                destination_root,
                &staging_root,
                &transaction_root,
                &[],
                &[],
                err.context(format!(
                    "failed to stage skill `{}` into `{}`",
                    skill.id,
                    staging_root.join(skill.id).display()
                )),
            );
        }
    }

    let mut replaced: Vec<(&'static BundledSkill, PathBuf)> = Vec::new();
    for skill in &selected {
        let skill_root = destination_root.join(skill.id);
        if path_entry_exists(&skill_root)
            .with_context(|| format!("failed to inspect `{}`", skill_root.display()))?
        {
            if !overwrite_existing {
                return fail_install(
                    destination_root,
                    &staging_root,
                    &transaction_root,
                    &replaced,
                    &[],
                    anyhow::anyhow!(
                        "skill `{}` already exists at `{}`",
                        skill.id,
                        skill_root.display()
                    ),
                );
            }

            let backup_path = backup_root.join(skill.id);
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)?;
            }
            if let Err(err) = fs::rename(&skill_root, &backup_path) {
                return fail_install(
                    destination_root,
                    &staging_root,
                    &transaction_root,
                    &replaced,
                    &[],
                    anyhow::Error::new(err).context(format!(
                        "failed to move existing skill `{}` at `{}` into backup",
                        skill.id,
                        skill_root.display()
                    )),
                );
            }
            replaced.push((skill, backup_path));
        }
    }

    let mut installed: Vec<&'static BundledSkill> = Vec::new();
    for skill in &selected {
        let from = staging_root.join(skill.id);
        let to = destination_root.join(skill.id);
        if let Err(err) = fs::rename(&from, &to) {
            return fail_install(
                destination_root,
                &staging_root,
                &transaction_root,
                &replaced,
                &installed,
                anyhow::Error::new(err).context(format!(
                    "failed to install skill `{}` into `{}`",
                    skill.id,
                    to.display()
                )),
            );
        }
        installed.push(skill);
    }

    cleanup_transaction_root(&transaction_root).with_context(|| {
        format!(
            "installed {} skill(s) into `{}` but failed to remove temporary transaction directory `{}`",
            selected.len(),
            destination_root.display(),
            transaction_root.display()
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

fn stage_skill_assets(staging_root: &Path, skill: &BundledSkill) -> Result<()> {
    let skill_root = staging_root.join(skill.id);
    let temp_root = staging_root.join(unique_install_path(skill.id, "stage"));
    fs::create_dir_all(&temp_root)?;

    let write_result = (|| -> Result<()> {
        for asset in skill.assets {
            let path = temp_root.join(asset.relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, asset.contents)?;
        }
        fs::rename(&temp_root, &skill_root)?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = remove_existing_path_if_present(&temp_root);
    }

    write_result
}

fn remove_existing_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        fs::remove_file(path)?;
    } else if file_type.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn remove_existing_path_if_present(path: &Path) -> Result<()> {
    if !path_entry_exists(path)
        .with_context(|| format!("failed to inspect `{}`", path.display()))?
    {
        return Ok(());
    }
    remove_existing_path(path)
}

fn rollback_skill_install(
    destination_root: &Path,
    staging_root: &Path,
    replaced: &[(&'static BundledSkill, PathBuf)],
    installed: &[&'static BundledSkill],
) -> Result<()> {
    let mut rollback_errors = Vec::new();

    for skill in installed.iter().rev() {
        let skill_root = destination_root.join(skill.id);
        match path_entry_exists(&skill_root) {
            Ok(true) => {
                if let Err(err) = remove_existing_path(&skill_root) {
                    rollback_errors.push(format!(
                        "failed to remove partially installed skill `{}` at `{}`: {err}",
                        skill.id,
                        skill_root.display()
                    ));
                }
            }
            Ok(false) => {}
            Err(err) => rollback_errors.push(format!(
                "failed to inspect partially installed skill `{}` at `{}`: {err}",
                skill.id,
                skill_root.display()
            )),
        }
    }

    for (skill, backup_path) in replaced.iter().rev() {
        let skill_root = destination_root.join(skill.id);
        match path_entry_exists(backup_path) {
            Ok(true) => {
                if let Err(err) = fs::rename(backup_path, &skill_root) {
                    rollback_errors.push(format!(
                        "failed to restore existing skill `{}` from `{}` to `{}`: {err}",
                        skill.id,
                        backup_path.display(),
                        skill_root.display()
                    ));
                }
            }
            Ok(false) => {}
            Err(err) => rollback_errors.push(format!(
                "failed to inspect backup skill `{}` at `{}`: {err}",
                skill.id,
                backup_path.display()
            )),
        }
    }

    match path_entry_exists(staging_root) {
        Ok(true) => {
            if let Err(err) = remove_existing_path(staging_root) {
                rollback_errors.push(format!(
                    "failed to remove staging directory `{}`: {err}",
                    staging_root.display()
                ));
            }
        }
        Ok(false) => {}
        Err(err) => rollback_errors.push(format!(
            "failed to inspect staging directory `{}`: {err}",
            staging_root.display()
        )),
    }

    if !rollback_errors.is_empty() {
        bail!(rollback_errors.join("; "));
    }

    Ok(())
}

fn cleanup_transaction_root(transaction_root: &Path) -> Result<()> {
    remove_existing_path_if_present(transaction_root)
}

fn path_entry_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

fn fail_install(
    destination_root: &Path,
    staging_root: &Path,
    transaction_root: &Path,
    replaced: &[(&'static BundledSkill, PathBuf)],
    installed: &[&'static BundledSkill],
    install_error: anyhow::Error,
) -> Result<usize> {
    let rollback_error =
        rollback_skill_install(destination_root, staging_root, replaced, installed).err();
    let cleanup_error = cleanup_transaction_root(transaction_root).err();

    let mut error = install_error;
    if let Some(rollback_error) = rollback_error {
        error = error.context(format!("rollback also failed: {rollback_error}"));
    }
    if let Some(cleanup_error) = cleanup_error {
        error = error.context(format!(
            "temporary transaction cleanup also failed: {cleanup_error}"
        ));
    }
    Err(error)
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

fn unique_install_path(skill_id: &str, suffix: &str) -> String {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(".{skill_id}.{suffix}.{}.{}", std::process::id(), unique)
}
