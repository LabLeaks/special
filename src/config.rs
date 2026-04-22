use anyhow::Result;
/**
@module SPECIAL.CONFIG
Re-exports configuration versioning and resolved-root entrypoints while delegating `special.toml` parsing and project-root discovery to narrower submodules.
*/
// @fileimplements SPECIAL.CONFIG
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod discovery;
mod resolution;
mod special_toml;
#[cfg(test)]
mod test_support;
mod version;

pub use resolution::{RootResolution, RootSource};
pub(crate) use special_toml::{ToolchainManager, load_special_toml};
pub use version::SpecialVersion;

pub fn resolve_project_root(start: &Path) -> Result<RootResolution> {
    discovery::resolve_project_root(start)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectToolchain {
    root: PathBuf,
    binding: ToolchainBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ToolchainBinding {
    Explicit(ToolchainManager),
    SpecialToml,
    MiseToml(PathBuf),
    ToolVersions(PathBuf),
}

#[derive(Debug, Clone)]
pub(crate) enum ProjectToolStatus {
    NoDeclaredContract,
    MissingTool,
    Available,
}

impl ProjectToolchain {
    pub(crate) fn discover(root: &Path) -> Result<Option<Self>> {
        let root = root.canonicalize()?;

        for ancestor in root.ancestors() {
            let config_path = ancestor.join("special.toml");
            if config_path.is_file() {
                let config = load_special_toml(&config_path)?;
                if let Some(manager) = config.toolchain_manager {
                    return Ok(Some(Self {
                        root,
                        binding: ToolchainBinding::Explicit(manager),
                    }));
                }
                if ancestor.join("mise.toml").is_file() {
                    return Ok(Some(Self {
                        root: root.clone(),
                        binding: ToolchainBinding::MiseToml(ancestor.to_path_buf()),
                    }));
                }
                if ancestor.join(".tool-versions").is_file() {
                    return Ok(Some(Self {
                        root: root.clone(),
                        binding: ToolchainBinding::ToolVersions(ancestor.to_path_buf()),
                    }));
                }
                return Ok(Some(Self {
                    root: root.clone(),
                    binding: ToolchainBinding::SpecialToml,
                }));
            }
        }

        if root.join("mise.toml").is_file() {
            return Ok(Some(Self {
                root: root.clone(),
                binding: ToolchainBinding::MiseToml(root.clone()),
            }));
        }
        if root.join(".tool-versions").is_file() {
            return Ok(Some(Self {
                root: root.clone(),
                binding: ToolchainBinding::ToolVersions(root.clone()),
            }));
        }

        Ok(None)
    }

    pub(crate) fn selected_manager(&self, tool: &str) -> Option<ToolchainManager> {
        match &self.binding {
            ToolchainBinding::Explicit(manager) => Some(*manager),
            ToolchainBinding::SpecialToml => Some(ToolchainManager::Mise),
            ToolchainBinding::MiseToml(contract_root) => {
                mise_contract_supports_tool(contract_root, tool).then_some(ToolchainManager::Mise)
            }
            ToolchainBinding::ToolVersions(contract_root) => {
                resolve_tool_versions_manager_for_tool(contract_root, tool)
            }
        }
    }

    pub(crate) fn launcher_label(&self, tool: &str) -> &'static str {
        match (&self.binding, self.selected_manager(tool)) {
            (_, Some(manager)) => manager.command(),
            _ => "undeclared",
        }
    }

    pub(crate) fn command(&self, tool: &str) -> Command {
        let mut command = match self.selected_manager(tool) {
            Some(manager) => {
                let mut command = Command::new(manager.command());
                command.args(manager.exec_prefix()).arg(tool);
                command
            }
            None => Command::new("__special_missing_declared_tool__"),
        };
        command.current_dir(&self.root);
        command
    }

    pub(crate) fn tool_available(&self, tool: &str, args: &[&str]) -> bool {
        self.command(tool)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

pub(crate) fn supported_project_toolchain_contracts() -> &'static str {
    "`special.toml`, `mise.toml`, or `.tool-versions`"
}

fn resolve_tool_versions_manager_for_tool(root: &Path, tool: &str) -> Option<ToolchainManager> {
    if !tool_versions_declares_tool(root, tool) {
        return None;
    }
    if tool_versions_requires_mise(root) {
        return Some(ToolchainManager::Mise);
    }
    Some(ToolchainManager::Mise)
}

fn tool_versions_requires_mise(root: &Path) -> bool {
    fs::read_to_string(root.join(".tool-versions"))
        .ok()
        .is_some_and(|contents| {
            contents.lines().any(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return false;
                }
                line.split_whitespace()
                    .next()
                    .is_some_and(|tool| tool.contains(':'))
            })
        })
}

fn tool_versions_declares_tool(root: &Path, requested_tool: &str) -> bool {
    let Ok(contents) = fs::read_to_string(root.join(".tool-versions")) else {
        return false;
    };
    contents.lines().any(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return false;
        }
        line.split_whitespace()
            .next()
            .is_some_and(|tool| contract_declares_tool(tool, requested_tool))
    })
}

fn contract_declares_tool(contract_tool: &str, requested_tool: &str) -> bool {
    match (contract_tool, requested_tool) {
        ("nodejs", "node" | "npm" | "npx") => true,
        ("golang" | "go", "go") => true,
        ("python" | "python3", "python" | "python3") => true,
        ("rust" | "rustc", "rustc" | "cargo" | "rust-analyzer") => true,
        ("ruby", "ruby") => true,
        _ if contract_tool == requested_tool => true,
        _ if contract_tool.contains(':') => contract_tool.ends_with(&format!("/{requested_tool}")),
        _ => false,
    }
}

fn mise_contract_supports_tool(root: &Path, tool: &str) -> bool {
    let Ok(contents) = fs::read_to_string(root.join("mise.toml")) else {
        return false;
    };
    let Ok(table) = contents.parse::<toml::Table>() else {
        return false;
    };
    let Some(tools) = table.get("tools").and_then(|value| value.as_table()) else {
        return false;
    };
    tools.iter().any(|(contract_tool, value)| {
        contract_declares_tool(contract_tool, tool)
            || value
                .as_table()
                .and_then(|inline| inline.get("exe"))
                .and_then(|exe| exe.as_str())
                .is_some_and(|exe| exe == tool)
    })
}

pub(crate) fn probe_project_tool(
    root: &Path,
    tool: &str,
    args: &[&str],
) -> Result<ProjectToolStatus> {
    if let Some(toolchain) = ProjectToolchain::discover(root)? {
        if toolchain.tool_available(tool, args) {
            Ok(ProjectToolStatus::Available)
        } else {
            Ok(ProjectToolStatus::MissingTool)
        }
    } else {
        Ok(ProjectToolStatus::NoDeclaredContract)
    }
}

pub(crate) fn standard_tool_unavailable_reason(
    feature: &str,
    tool: &str,
    status: &ProjectToolStatus,
) -> String {
    match status {
        ProjectToolStatus::NoDeclaredContract => format!(
            "{feature} is unavailable because the analyzed project does not declare a supported {} contract with a resolvable `{tool}` tool",
            supported_project_toolchain_contracts()
        ),
        ProjectToolStatus::MissingTool => format!(
            "{feature} is unavailable because the project's declared toolchain does not provide `{tool}`"
        ),
        ProjectToolStatus::Available => {
            format!("{feature} is unavailable despite a resolvable `{tool}` tool")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{ProjectToolchain, ToolchainManager};
    use crate::config::test_support::temp_config_test_dir;

    #[test]
    fn resolves_mise_toml_as_project_toolchain_contract() {
        let root = temp_config_test_dir("special-config-toolchain-mise");
        fs::write(root.join("mise.toml"), "[tools]\nnode = \"24\"\n")
            .expect("mise.toml should be written");

        let toolchain = ProjectToolchain::discover(&root).expect("toolchain should resolve");
        assert_eq!(
            toolchain.map(|toolchain| toolchain.selected_manager("node")),
            Some(Some(ToolchainManager::Mise))
        );

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn resolves_tool_versions_as_project_toolchain_contract() {
        let root = temp_config_test_dir("special-config-toolchain-asdf");
        fs::write(root.join(".tool-versions"), "nodejs 24.15.0\n")
            .expect(".tool-versions should be written");

        let toolchain = ProjectToolchain::discover(&root).expect("toolchain should resolve");
        let manager = toolchain
            .expect("toolchain should exist")
            .selected_manager("node");
        assert_eq!(manager, Some(ToolchainManager::Mise));

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn resolves_tool_versions_with_mise_only_syntax_to_mise() {
        let root = temp_config_test_dir("special-config-toolchain-mise-only-syntax");
        fs::write(
            root.join(".tool-versions"),
            "nodejs 24.15.0\ngo:golang.org/x/tools/gopls v0.16.2\n",
        )
        .expect(".tool-versions should be written");

        let toolchain = ProjectToolchain::discover(&root).expect("toolchain should resolve");
        assert_eq!(
            toolchain
                .expect("toolchain should exist")
                .selected_manager("gopls"),
            Some(ToolchainManager::Mise)
        );

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn special_toml_toolchain_override_applies_to_configured_root() {
        let repo_root = temp_config_test_dir("special-config-toolchain-override");
        let workspace = repo_root.join("workspace");
        fs::create_dir_all(&workspace).expect("workspace should be created");
        fs::write(
            repo_root.join("special.toml"),
            "version = \"1\"\nroot = \"workspace\"\n\n[toolchain]\nmanager = \"asdf\"\n",
        )
        .expect("special.toml should be written");

        let toolchain = ProjectToolchain::discover(&workspace).expect("toolchain should resolve");
        assert_eq!(
            toolchain.map(|toolchain| toolchain.selected_manager("node")),
            Some(Some(ToolchainManager::Asdf))
        );

        fs::remove_dir_all(&repo_root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn discovered_project_toolchain_builds_commands_at_configured_root() {
        let root = temp_config_test_dir("special-config-toolchain-command");
        fs::write(root.join("mise.toml"), "[tools]\nnode = \"24\"\n")
            .expect("mise.toml should be written");

        let toolchain = ProjectToolchain::discover(&root)
            .expect("toolchain should resolve")
            .expect("toolchain should exist");
        let command = toolchain.command("node");

        assert_eq!(
            toolchain.selected_manager("node"),
            Some(ToolchainManager::Mise)
        );
        assert_eq!(command.get_program(), std::ffi::OsStr::new("mise"));
        assert_eq!(command.get_current_dir(), Some(root.as_path()));

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn bare_mise_toml_does_not_hijack_unrelated_tools() {
        let root = temp_config_test_dir("special-config-bare-mise-fallback");
        fs::write(
            root.join("mise.toml"),
            "[tools]\n\"ubi:leanprover/elan\" = { version = \"v4.2.1\", exe = \"elan\" }\n",
        )
        .expect("mise.toml should be written");

        let toolchain = ProjectToolchain::discover(&root)
            .expect("toolchain should resolve")
            .expect("toolchain should exist");

        assert_eq!(
            toolchain.selected_manager("elan"),
            Some(ToolchainManager::Mise)
        );
        assert_eq!(toolchain.selected_manager("node"), None);
        assert_eq!(toolchain.launcher_label("node"), "undeclared");
        assert!(!toolchain.tool_available("node", &["--version"]));

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn special_toml_without_toolchain_override_defaults_to_mise() {
        let root = temp_config_test_dir("special-config-special-toml-default-manager");
        fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
            .expect("special.toml should be written");

        let toolchain = ProjectToolchain::discover(&root)
            .expect("toolchain should resolve")
            .expect("toolchain should exist");

        assert_eq!(
            toolchain.selected_manager("node"),
            Some(ToolchainManager::Mise)
        );
        assert_eq!(toolchain.launcher_label("node"), "mise");

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }

    #[test]
    fn no_declared_contract_returns_none() {
        let root = temp_config_test_dir("special-config-no-toolchain-contract");

        assert_eq!(
            ProjectToolchain::discover(&root).expect("discovery should succeed"),
            None
        );

        fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }
}
