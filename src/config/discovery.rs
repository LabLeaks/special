/**
@module SPECIAL.CONFIG.ROOT_DISCOVERY
Resolves the active project root from `special.toml`, VCS markers, or the current directory. This module does not parse `special.toml` key syntax.
*/
// @fileimplements SPECIAL.CONFIG.ROOT_DISCOVERY
use std::path::Path;

use anyhow::{Context, Result, bail};

use std::fs;
use std::path::PathBuf;

use super::special_toml::load_special_toml;
use super::{PatternMetricBenchmarks, RootResolution, RootSource, SpecialVersion};

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
                health_ignore_unexplained_patterns: config.health_ignore_unexplained_patterns,
                pattern_benchmarks: config.pattern_benchmarks,
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
                health_ignore_unexplained_patterns: Vec::new(),
                pattern_benchmarks: PatternMetricBenchmarks::default(),
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
        health_ignore_unexplained_patterns: Vec::new(),
        pattern_benchmarks: PatternMetricBenchmarks::default(),
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::resolve_project_root;
    use crate::config::RootSource;
    use crate::config::test_support::temp_config_test_dir;

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML
    fn prefers_special_toml_as_project_anchor() {
        let root = temp_config_test_dir("special-config-special-toml");
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
        let repo_root = temp_config_test_dir("special-config-explicit-root");
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
        let repo_root = temp_config_test_dir("special-config-file-root");
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
        let root = temp_config_test_dir("special-config-optional");

        let resolved = resolve_project_root(&root).expect("root should resolve");

        assert_eq!(resolved.source, RootSource::CurrentDir);

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.SUPPRESSES_IMPLICIT_ROOT_WARNING
    fn does_not_warn_when_special_toml_is_present() {
        let root = temp_config_test_dir("special-config-no-warning");
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
        let root = temp_config_test_dir("special-config-vcs");
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
        let root = temp_config_test_dir("special-config-cwd");
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
    // @verifies SPECIAL.CONFIG.ROOT_DISCOVERY.IMPLICIT_ROOT_WARNING
    fn warns_when_root_is_inferred() {
        let root = temp_config_test_dir("special-config-warning");
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
}
