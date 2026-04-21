/**
@module SPECIAL.SKILLS.INSTALL.TRANSACTION
Coordinates staged bundled-skill installation transactions, including backup, commit, rollback, and temporary directory cleanup.
*/
// @fileimplements SPECIAL.SKILLS.INSTALL.TRANSACTION
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::skills::BundledSkill;

use super::fs_ops::{
    backup_path, install_path, path_entry_exists, remove_existing_path,
    remove_existing_path_if_present, stage_path, unique_install_path,
};

pub(super) struct InstallTransaction<'a> {
    destination_root: &'a Path,
    transaction_root: PathBuf,
    staging_root: PathBuf,
    backup_root: PathBuf,
    replaced: Vec<(&'static BundledSkill, PathBuf)>,
    installed: Vec<&'static BundledSkill>,
}

impl<'a> InstallTransaction<'a> {
    pub(super) fn begin(destination_root: &'a Path) -> Result<Self> {
        fs::create_dir_all(destination_root)?;

        let transaction_root = destination_root.join(unique_install_path("skills", "transaction"));
        let staging_root = transaction_root.join("staging");
        let backup_root = transaction_root.join("backup");
        fs::create_dir_all(&staging_root)?;

        Ok(Self {
            destination_root,
            transaction_root,
            staging_root,
            backup_root,
            replaced: Vec::new(),
            installed: Vec::new(),
        })
    }

    pub(super) fn staging_root(&self) -> &Path {
        &self.staging_root
    }

    pub(super) fn backup_existing(&mut self, skill: &'static BundledSkill) -> Result<()> {
        let skill_root = install_path(self.destination_root, skill);
        if !path_entry_exists(&skill_root)
            .with_context(|| format!("failed to inspect `{}`", skill_root.display()))?
        {
            return Ok(());
        }

        let backup_path = backup_path(&self.backup_root, skill);
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&skill_root, &backup_path).with_context(|| {
            format!(
                "failed to move existing skill `{}` at `{}` into backup",
                skill.id,
                skill_root.display()
            )
        })?;
        self.replaced.push((skill, backup_path));
        Ok(())
    }

    pub(super) fn install_staged(&mut self, skill: &'static BundledSkill) -> Result<()> {
        let from = stage_path(&self.staging_root, skill);
        let to = install_path(self.destination_root, skill);
        fs::rename(&from, &to).with_context(|| {
            format!(
                "failed to install skill `{}` into `{}`",
                skill.id,
                to.display()
            )
        })?;
        self.installed.push(skill);
        Ok(())
    }

    pub(super) fn rollback(&self) -> Result<()> {
        let mut rollback_errors = Vec::new();

        for skill in self.installed.iter().rev() {
            let skill_root = install_path(self.destination_root, skill);
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

        for (skill, backup_path) in self.replaced.iter().rev() {
            let skill_root = install_path(self.destination_root, skill);
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

        match path_entry_exists(&self.staging_root) {
            Ok(true) => {
                if let Err(err) = remove_existing_path(&self.staging_root) {
                    rollback_errors.push(format!(
                        "failed to remove staging directory `{}`: {err}",
                        self.staging_root.display()
                    ));
                }
            }
            Ok(false) => {}
            Err(err) => rollback_errors.push(format!(
                "failed to inspect staging directory `{}`: {err}",
                self.staging_root.display()
            )),
        }

        if rollback_errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(rollback_errors.join("; ")))
        }
    }

    pub(super) fn cleanup(&self) -> Result<()> {
        remove_existing_path_if_present(&self.transaction_root)
    }

    pub(super) fn fail(self, install_error: anyhow::Error) -> Result<usize> {
        let rollback_error = self.rollback().err();
        let cleanup_error = self.cleanup().err();

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
}
