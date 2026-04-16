/**
@module SPECIAL.SKILLS.INSTALL.FS
Shared filesystem operations for bundled-skill installation, including staging directories, existence checks, removals, and unique temporary path generation.
*/
// @fileimplements SPECIAL.SKILLS.INSTALL.FS
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::skills::BundledSkill;

pub(super) fn stage_skill_assets(staging_root: &Path, skill: &BundledSkill) -> Result<()> {
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

pub(super) fn remove_existing_path(path: &Path) -> Result<()> {
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

pub(super) fn remove_existing_path_if_present(path: &Path) -> Result<()> {
    if !path_entry_exists(path)? {
        return Ok(());
    }
    remove_existing_path(path)
}

pub(super) fn path_entry_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn unique_install_path(skill_id: &str, suffix: &str) -> String {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(".{skill_id}.{suffix}.{}.{}", std::process::id(), unique)
}

pub(super) fn stage_path(staging_root: &Path, skill: &BundledSkill) -> PathBuf {
    staging_root.join(skill.id)
}

pub(super) fn install_path(destination_root: &Path, skill: &BundledSkill) -> PathBuf {
    destination_root.join(skill.id)
}

pub(super) fn backup_path(backup_root: &Path, skill: &BundledSkill) -> PathBuf {
    backup_root.join(skill.id)
}
