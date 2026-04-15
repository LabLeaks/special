/**
@module SPECIAL.CONFIG
Coordinates `special.toml` parsing and project-root discovery for the rest of the application.
*/
// @fileimplements SPECIAL.CONFIG
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

mod discovery;
mod special_toml;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpecialVersion {
    #[default]
    V0,
    V1,
}

impl SpecialVersion {
    pub const CURRENT: Self = Self::V1;

    pub fn as_str(self) -> &'static str {
        match self {
            Self::V0 => "0",
            Self::V1 => "1",
        }
    }

    fn parse(value: &str, line: Option<usize>) -> Result<Self> {
        match value {
            "0" => Ok(Self::V0),
            "1" => Ok(Self::V1),
            _ => {
                if let Some(line) = line {
                    bail!("line {line} uses unsupported `special.toml` version `{value}`");
                }
                bail!("unsupported `special.toml` version `{value}`");
            }
        }
    }
}

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

#[derive(Debug, Default)]
pub(super) struct SpecialToml {
    root: Option<PathBuf>,
    version: SpecialVersion,
    version_explicit: bool,
    ignore_patterns: Vec<String>,
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

pub fn resolve_project_root(start: &Path) -> Result<RootResolution> {
    discovery::resolve_project_root(start)
}

pub(super) fn is_vcs_root(path: &Path) -> bool {
    is_marker_present(path.join(".git")) || is_marker_present(path.join(".jj"))
}

pub(super) fn is_marker_present(path: PathBuf) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_dir() || metadata.is_file())
        .unwrap_or(false)
}

fn load_special_toml(path: &Path) -> Result<SpecialToml> {
    special_toml::load_special_toml(path)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{RootSource, SpecialVersion, resolve_project_root, special_toml};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path.canonicalize().expect("temp dir should canonicalize")
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML
    fn prefers_special_toml_as_project_anchor() {
        let root = temp_dir("special-config-special-toml");
        let nested = root.join("a/b/c");
        fs::create_dir_all(&nested).expect("nested dir should be created");
        fs::write(root.join("special.toml"), "").expect("special.toml should be created");
        fs::create_dir_all(root.join(".git")).expect(".git dir should be created");

        let resolved = resolve_project_root(&nested).expect("root should resolve");

        assert_eq!(resolved.root, root);
        assert_eq!(resolved.source, RootSource::SpecialToml);

        fs::remove_dir_all(&resolved.root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.EXPLICIT_ROOT
    fn uses_root_from_special_toml() {
        let repo_root = temp_dir("special-config-explicit-root");
        let configured_root = repo_root.join("workspace/specs");
        let nested = repo_root.join("workspace/specs/a/b");
        fs::create_dir_all(&nested).expect("nested dir should be created");
        fs::write(
            repo_root.join("special.toml"),
            "root = \"workspace/specs\"\n",
        )
        .expect("special.toml should be created");

        let resolved = resolve_project_root(&nested).expect("root should resolve");

        assert_eq!(
            resolved.root,
            configured_root
                .canonicalize()
                .expect("root should canonicalize")
        );
        assert_eq!(resolved.source, RootSource::SpecialToml);

        fs::remove_dir_all(&repo_root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_BE_DIRECTORY
    fn rejects_file_root_from_special_toml() {
        let repo_root = temp_dir("special-config-file-root");
        let nested = repo_root.join("workspace/specs");
        fs::create_dir_all(&nested).expect("nested dir should be created");
        fs::write(repo_root.join("workspace/specs.rs"), "// fixture")
            .expect("fixture file should be written");
        fs::write(
            repo_root.join("special.toml"),
            "root = \"workspace/specs.rs\"\n",
        )
        .expect("special.toml should be created");

        let err = resolve_project_root(&nested).expect_err("file root should fail");

        assert!(
            err.to_string()
                .contains("points to a root that is not a directory")
        );

        fs::remove_dir_all(&repo_root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.OPTIONAL
    fn does_not_require_special_toml_for_resolution() {
        let root = temp_dir("special-config-optional");

        let resolved = resolve_project_root(&root).expect("root should resolve");

        assert_eq!(resolved.source, RootSource::CurrentDir);

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.SUPPRESSES_IMPLICIT_ROOT_WARNING
    fn does_not_warn_when_special_toml_is_present() {
        let root = temp_dir("special-config-no-warning");
        let nested = root.join("a/b/c");
        fs::create_dir_all(&nested).expect("nested dir should be created");
        fs::write(root.join("special.toml"), "").expect("special.toml should be created");

        let resolved = resolve_project_root(&nested).expect("root should resolve");

        assert!(resolved.warning().is_none());

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.ROOT_DISCOVERY.VCS_DEFAULT
    fn falls_back_to_vcs_root_without_special_toml() {
        let root = temp_dir("special-config-vcs");
        let nested = root.join("a/b");
        fs::create_dir_all(&nested).expect("nested dir should be created");
        fs::write(root.join(".git"), "gitdir: /tmp/example\n").expect(".git file should exist");

        let resolved = resolve_project_root(&nested).expect("root should resolve");

        assert_eq!(resolved.root, root);
        assert_eq!(resolved.source, RootSource::Vcs);
        assert!(resolved.warning().is_some());

        fs::remove_dir_all(&resolved.root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.ROOT_DISCOVERY.CWD_FALLBACK
    fn falls_back_to_current_directory_without_config_or_vcs() {
        let root = temp_dir("special-config-cwd");
        let nested = root.join("a/b");
        fs::create_dir_all(&nested).expect("nested dir should be created");

        let resolved = resolve_project_root(&nested).expect("root should resolve");

        assert_eq!(
            resolved.root,
            nested.canonicalize().expect("path should canonicalize")
        );
        assert_eq!(resolved.source, RootSource::CurrentDir);
        assert!(resolved.warning().is_some());

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn reports_unsupported_special_toml_versions_with_line_context() {
        let err = special_toml::parse_special_toml("root = \".\"\nversion = \"9\"\n")
            .expect_err("unsupported versions should fail");

        assert!(
            err.to_string()
                .contains("line 2 uses unsupported `special.toml` version `9`")
        );
    }

    #[test]
    // @verifies SPECIAL.CONFIG.ROOT_DISCOVERY.IMPLICIT_ROOT_WARNING
    fn warns_when_root_is_inferred() {
        let root = temp_dir("special-config-warning");
        let nested = root.join("a/b");
        fs::create_dir_all(&nested).expect("nested dir should be created");
        fs::write(root.join(".git"), "gitdir: /tmp/example\n").expect(".git file should exist");

        let resolved = resolve_project_root(&nested).expect("root should resolve");
        let warning = resolved.warning().expect("warning should be present");

        assert!(warning.contains("warning: using inferred VCS root"));
        assert!(warning.contains("add special.toml for predictable root selection"));
        assert!(!resolved.version_explicit);

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.VERSION
    fn parses_special_toml_version() {
        let config = special_toml::parse_special_toml("version = \"1\"\nroot = \".\"\n")
            .expect("config should parse");

        assert_eq!(config.version, SpecialVersion::V1);
        assert_eq!(config.root, Some(PathBuf::from(".")));
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.VERSION.DEFAULTS_TO_LEGACY
    fn defaults_special_toml_version_to_legacy() {
        let config = special_toml::parse_special_toml("root = \".\"\n")
            .expect("config without version should parse");

        assert_eq!(config.version, SpecialVersion::V0);
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.VERSION.UNKNOWN_REJECTED
    fn rejects_unknown_special_toml_version() {
        let err =
            special_toml::parse_special_toml("version = \"2\"\n").expect_err("config should fail");

        assert!(
            err.to_string()
                .contains("unsupported `special.toml` version `2`")
        );
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.DUPLICATE_KEYS_REJECTED
    fn rejects_duplicate_special_toml_keys() {
        let err = special_toml::parse_special_toml("root = \".\"\nroot = \"workspace\"\n")
            .expect_err("duplicate root should fail");

        let message = err.to_string();
        assert!(message.contains("line 2"));
        assert!(message.contains("root"));

        let err = special_toml::parse_special_toml("version = \"1\"\nversion = \"0\"\n")
            .expect_err("duplicate version should fail");

        let message = err.to_string();
        assert!(message.contains("line 2"));
        assert!(message.contains("version"));
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_NOT_BE_EMPTY
    fn rejects_empty_special_toml_root() {
        let err =
            special_toml::parse_special_toml("root = \"\"\n").expect_err("empty root should fail");

        assert!(err.to_string().contains("must not use an empty root path"));
    }
}
