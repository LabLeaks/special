/**
@module SPECIAL.MODULES.ANALYZE.RUST.TOOLCHAIN
Discovers local Rust toolchain project context needed by richer Rust language-pack backends without forcing higher analysis layers to know about Cargo metadata details. This module should only surface stable project facts useful to target-aware and future semantic backends, such as workspace roots, package manifests, and Rust target source files.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.TOOLCHAIN
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RustToolchainProject {
    pub(super) workspace_root: PathBuf,
    pub(super) target_sources: BTreeMap<PathBuf, RustToolchainTarget>,
    pub(super) capabilities: RustToolchainCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RustToolchainTarget {
    pub(super) package_name: String,
    pub(super) target_name: String,
    pub(super) kind: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct RustToolchainCapabilities {
    pub(super) active_channel: RustToolchainChannel,
    pub(super) rustdoc_json: RustdocJsonAvailability,
    pub(super) rust_analyzer_available: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum RustToolchainChannel {
    Stable,
    Beta,
    Nightly,
    Dev,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum RustdocJsonAvailability {
    Available,
    RequiresNightly,
    #[default]
    Unknown,
}

impl RustToolchainProject {
    pub(super) fn crate_root_aliases(&self) -> BTreeSet<String> {
        let mut aliases = BTreeSet::new();
        for target in self.target_sources.values() {
            if target
                .kind
                .iter()
                .any(|kind| kind == "lib" || kind == "proc-macro")
            {
                aliases.insert(target.target_name.clone());
                aliases.insert(target.package_name.replace('-', "_"));
            }
        }
        aliases
    }

    pub(super) fn binary_target_sources(&self) -> BTreeMap<String, PathBuf> {
        self.target_sources
            .iter()
            .filter(|(_, target)| target.kind.iter().any(|kind| kind == "bin"))
            .map(|(path, target)| (target.target_name.clone(), path.clone()))
            .collect()
    }
}

pub(super) fn probe_local_toolchain_project(root: &Path) -> Option<RustToolchainProject> {
    if !root.join("Cargo.toml").exists() {
        return None;
    }

    let output = Command::new("mise")
        .args([
            "exec",
            "--",
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let metadata: Value = serde_json::from_slice(&output.stdout).ok()?;
    let capabilities = probe_local_toolchain_capabilities(root);
    parse_cargo_metadata(root, &metadata, capabilities)
}

pub(super) fn analysis_environment_fingerprint(root: &Path) -> String {
    let capabilities = probe_local_toolchain_capabilities(root);
    format!(
        "cargo_toml={};channel={:?};rustdoc_json={:?};rust_analyzer={}",
        root.join("Cargo.toml").exists(),
        capabilities.active_channel,
        capabilities.rustdoc_json,
        capabilities.rust_analyzer_available
    )
}

fn probe_local_toolchain_capabilities(root: &Path) -> RustToolchainCapabilities {
    let rust_analyzer_available = probe_rust_analyzer_available(root);
    let output = Command::new("mise")
        .args(["exec", "--", "rustc", "--version", "--verbose"])
        .current_dir(root)
        .output();
    let Ok(output) = output else {
        return RustToolchainCapabilities {
            rust_analyzer_available,
            ..RustToolchainCapabilities::default()
        };
    };
    if !output.status.success() {
        return RustToolchainCapabilities {
            rust_analyzer_available,
            ..RustToolchainCapabilities::default()
        };
    }
    let Ok(stdout) = String::from_utf8(output.stdout) else {
        return RustToolchainCapabilities {
            rust_analyzer_available,
            ..RustToolchainCapabilities::default()
        };
    };
    let mut capabilities = parse_rustc_verbose_capabilities(&stdout);
    capabilities.rust_analyzer_available = rust_analyzer_available;
    capabilities
}

fn probe_rust_analyzer_available(root: &Path) -> bool {
    Command::new("mise")
        .args(["exec", "--", "rust-analyzer", "--version"])
        .current_dir(root)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn parse_rustc_verbose_capabilities(stdout: &str) -> RustToolchainCapabilities {
    let active_channel = stdout
        .lines()
        .find_map(|line| line.strip_prefix("release: "))
        .map(parse_rust_release_channel)
        .unwrap_or_default();
    let rustdoc_json = match active_channel {
        RustToolchainChannel::Nightly | RustToolchainChannel::Dev => {
            RustdocJsonAvailability::Available
        }
        RustToolchainChannel::Stable | RustToolchainChannel::Beta => {
            RustdocJsonAvailability::RequiresNightly
        }
        RustToolchainChannel::Unknown => RustdocJsonAvailability::Unknown,
    };
    RustToolchainCapabilities {
        active_channel,
        rustdoc_json,
        rust_analyzer_available: false,
    }
}

fn parse_rust_release_channel(release: &str) -> RustToolchainChannel {
    if release.contains("nightly") {
        RustToolchainChannel::Nightly
    } else if release.contains("beta") {
        RustToolchainChannel::Beta
    } else if release.contains("dev") {
        RustToolchainChannel::Dev
    } else if release
        .chars()
        .next()
        .is_some_and(|char| char.is_ascii_digit())
    {
        RustToolchainChannel::Stable
    } else {
        RustToolchainChannel::Unknown
    }
}

fn parse_cargo_metadata(
    root: &Path,
    metadata: &Value,
    capabilities: RustToolchainCapabilities,
) -> Option<RustToolchainProject> {
    let workspace_root = PathBuf::from(metadata.get("workspace_root")?.as_str()?);
    let packages = metadata.get("packages")?.as_array()?;

    let mut target_sources = BTreeMap::new();
    for package in packages {
        let package_name = package.get("name")?.as_str()?.to_string();
        let targets = package.get("targets")?.as_array()?;
        for target in targets {
            let src_path = PathBuf::from(target.get("src_path")?.as_str()?);
            if src_path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let path = src_path
                .strip_prefix(root)
                .map(Path::to_path_buf)
                .unwrap_or(src_path);
            let kind = target
                .get("kind")
                .and_then(Value::as_array)
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|entry| entry.as_str().map(ToString::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            target_sources.insert(
                path,
                RustToolchainTarget {
                    package_name: package_name.clone(),
                    target_name: target.get("name")?.as_str()?.to_string(),
                    kind,
                },
            );
        }
    }

    Some(RustToolchainProject {
        workspace_root,
        target_sources,
        capabilities,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use super::{
        RustToolchainCapabilities, RustToolchainChannel, RustToolchainTarget,
        RustdocJsonAvailability, parse_cargo_metadata, parse_rust_release_channel,
        parse_rustc_verbose_capabilities,
    };

    #[test]
    fn parses_workspace_root_and_target_sources_from_cargo_metadata() {
        let metadata = json!({
            "workspace_root": "/tmp/demo",
            "packages": [{
                "name": "demo",
                "targets": [
                    {
                        "name": "demo",
                        "kind": ["lib"],
                        "src_path": "/tmp/demo/src/lib.rs"
                    },
                    {
                        "name": "demo",
                        "kind": ["bin"],
                        "src_path": "/tmp/demo/src/main.rs"
                    }
                ]
            }]
        });

        let project = parse_cargo_metadata(
            Path::new("/tmp/demo"),
            &metadata,
            RustToolchainCapabilities {
                active_channel: RustToolchainChannel::Stable,
                rustdoc_json: RustdocJsonAvailability::RequiresNightly,
                rust_analyzer_available: false,
            },
        )
        .expect("metadata should parse");
        assert_eq!(project.workspace_root, PathBuf::from("/tmp/demo"));
        assert_eq!(
            project.target_sources.get(Path::new("src/lib.rs")),
            Some(&RustToolchainTarget {
                package_name: "demo".to_string(),
                target_name: "demo".to_string(),
                kind: vec!["lib".to_string()],
            })
        );
        assert_eq!(
            project.target_sources.get(Path::new("src/main.rs")),
            Some(&RustToolchainTarget {
                package_name: "demo".to_string(),
                target_name: "demo".to_string(),
                kind: vec!["bin".to_string()],
            })
        );
        assert_eq!(
            project.capabilities,
            RustToolchainCapabilities {
                active_channel: RustToolchainChannel::Stable,
                rustdoc_json: RustdocJsonAvailability::RequiresNightly,
                rust_analyzer_available: false,
            }
        );
    }

    #[test]
    fn derives_crate_root_aliases_and_binary_targets() {
        let project = super::RustToolchainProject {
            workspace_root: PathBuf::from("/tmp/demo"),
            target_sources: BTreeMap::from([
                (
                    PathBuf::from("src/lib.rs"),
                    RustToolchainTarget {
                        package_name: "demo-cli".to_string(),
                        target_name: "demo".to_string(),
                        kind: vec!["lib".to_string()],
                    },
                ),
                (
                    PathBuf::from("src/main.rs"),
                    RustToolchainTarget {
                        package_name: "demo-cli".to_string(),
                        target_name: "demo-cli".to_string(),
                        kind: vec!["bin".to_string()],
                    },
                ),
            ]),
            capabilities: RustToolchainCapabilities::default(),
        };

        assert_eq!(
            project.crate_root_aliases(),
            BTreeSet::from(["demo".to_string(), "demo_cli".to_string()])
        );
        assert_eq!(
            project.binary_target_sources(),
            BTreeMap::from([("demo-cli".to_string(), PathBuf::from("src/main.rs"),)])
        );
    }

    #[test]
    fn parses_release_channels_from_rustc_release_strings() {
        assert_eq!(
            parse_rust_release_channel("1.94.1"),
            RustToolchainChannel::Stable
        );
        assert_eq!(
            parse_rust_release_channel("1.95.0-beta.1"),
            RustToolchainChannel::Beta
        );
        assert_eq!(
            parse_rust_release_channel("1.95.0-nightly"),
            RustToolchainChannel::Nightly
        );
        assert_eq!(
            parse_rust_release_channel("1.95.0-dev"),
            RustToolchainChannel::Dev
        );
    }

    #[test]
    fn derives_rustdoc_json_capability_from_rustc_verbose_output() {
        let stable = parse_rustc_verbose_capabilities(
            "rustc 1.94.1 (e408947bf 2026-03-25)\nrelease: 1.94.1\n",
        );
        assert_eq!(
            stable,
            RustToolchainCapabilities {
                active_channel: RustToolchainChannel::Stable,
                rustdoc_json: RustdocJsonAvailability::RequiresNightly,
                rust_analyzer_available: false,
            }
        );

        let nightly = parse_rustc_verbose_capabilities(
            "rustc 1.95.0-nightly (aaaaaaaaa 2026-04-01)\nrelease: 1.95.0-nightly\n",
        );
        assert_eq!(
            nightly,
            RustToolchainCapabilities {
                active_channel: RustToolchainChannel::Nightly,
                rustdoc_json: RustdocJsonAvailability::Available,
                rust_analyzer_available: false,
            }
        );
    }
}
