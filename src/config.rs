use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

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
}

#[derive(Debug, Default)]
struct SpecialToml {
    root: Option<PathBuf>,
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
    let start = start.canonicalize()?;

    for ancestor in start.ancestors() {
        let config_path = ancestor.join("special.toml");
        if config_path.is_file() {
            let config = load_special_toml(&config_path)?;
            let root = match config.root {
                Some(configured_root) => ancestor
                    .join(configured_root)
                    .canonicalize()
                    .with_context(|| {
                        format!(
                            "special.toml at `{}` points to a root that does not exist",
                            config_path.display()
                        )
                    })?,
                None => ancestor.to_path_buf(),
            };
            return Ok(RootResolution {
                root,
                source: RootSource::SpecialToml,
            });
        }
    }

    for ancestor in start.ancestors() {
        if is_vcs_root(ancestor) {
            return Ok(RootResolution {
                root: ancestor.to_path_buf(),
                source: RootSource::Vcs,
            });
        }
    }

    Ok(RootResolution {
        root: start,
        source: RootSource::CurrentDir,
    })
}

fn is_vcs_root(path: &Path) -> bool {
    is_marker_present(path.join(".git")) || is_marker_present(path.join(".jj"))
}

fn is_marker_present(path: PathBuf) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_dir() || metadata.is_file())
        .unwrap_or(false)
}

fn load_special_toml(path: &Path) -> Result<SpecialToml> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read special.toml at `{}`", path.display()))?;
    parse_special_toml(&content)
        .with_context(|| format!("failed to parse special.toml at `{}`", path.display()))
}

fn parse_special_toml(content: &str) -> Result<SpecialToml> {
    let mut config = SpecialToml::default();

    for (index, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            bail!("line {} must use `key = \"value\"` syntax", index + 1);
        };

        let key = key.trim();
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .ok_or_else(|| anyhow::anyhow!("line {} must use a quoted string value", index + 1))?;

        match key {
            "root" => config.root = Some(PathBuf::from(value)),
            _ => bail!("line {} uses unknown key `{key}`", index + 1),
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{resolve_project_root, RootSource};

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
        fs::write(repo_root.join("special.toml"), "root = \"workspace/specs\"\n")
            .expect("special.toml should be created");

        let resolved = resolve_project_root(&nested).expect("root should resolve");

        assert_eq!(resolved.root, configured_root.canonicalize().expect("root should canonicalize"));
        assert_eq!(resolved.source, RootSource::SpecialToml);

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

        assert_eq!(resolved.root, nested.canonicalize().expect("path should canonicalize"));
        assert_eq!(resolved.source, RootSource::CurrentDir);
        assert!(resolved.warning().is_some());

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
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

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }
}
