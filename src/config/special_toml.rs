/**
@module SPECIAL.CONFIG.SPECIAL_TOML
Parses and loads `special.toml` root, version, and shared discovery ignore settings. This module does not choose VCS or current-directory fallbacks when config is absent.
*/
// @fileimplements SPECIAL.CONFIG.SPECIAL_TOML
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use toml::Table;

use super::SpecialVersion;

#[derive(Debug, Default)]
pub(crate) struct SpecialToml {
    pub(crate) root: Option<PathBuf>,
    pub(crate) version: SpecialVersion,
    pub(crate) version_explicit: bool,
    pub(crate) ignore_patterns: Vec<String>,
    pub(crate) toolchain_manager: Option<ToolchainManager>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolchainManager {
    Mise,
    Asdf,
}

impl ToolchainManager {
    fn parse(value: &str, line: usize) -> Result<Self> {
        match value {
            "mise" => Ok(Self::Mise),
            "asdf" => Ok(Self::Asdf),
            _ => bail!(
                "line {} uses unsupported toolchain manager `{}`; expected `mise` or `asdf`",
                line,
                value
            ),
        }
    }

    pub(crate) fn command(self) -> &'static str {
        match self {
            Self::Mise => "mise",
            Self::Asdf => "asdf",
        }
    }

    pub(crate) fn exec_prefix(self) -> &'static [&'static str] {
        match self {
            Self::Mise => &["exec", "--"],
            Self::Asdf => &["exec"],
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpecialToml {
    root: Option<String>,
    version: Option<String>,
    ignore: Option<Vec<String>>,
    toolchain: Option<RawToolchainConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawToolchainConfig {
    manager: Option<String>,
}

pub(crate) fn load_special_toml(path: &Path) -> Result<SpecialToml> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read special.toml at `{}`", path.display()))?;
    parse_special_toml(&content)
        .with_context(|| format!("failed to parse special.toml at `{}`", path.display()))
}

pub(super) fn parse_special_toml(content: &str) -> Result<SpecialToml> {
    let table: Table =
        toml::from_str(content).map_err(|err| anyhow!(format_toml_parse_error(content, &err)))?;
    let key_lines = collect_top_level_key_lines(content);

    for key in table.keys() {
        if key != "root" && key != "version" && key != "ignore" && key != "toolchain" {
            let line = key_lines.get(key.as_str()).copied().unwrap_or(1);
            bail!("line {} uses unknown key `{key}`", line);
        }
    }

    let raw: RawSpecialToml =
        toml::from_str(content).map_err(|err| anyhow!(format_toml_parse_error(content, &err)))?;

    let mut config = SpecialToml::default();

    if let Some(root) = raw.root {
        let line = key_lines.get("root").copied().unwrap_or(1);
        if root.trim().is_empty() {
            bail!("line {} must not use an empty root path", line);
        }
        config.root = Some(PathBuf::from(root));
    }

    if let Some(version) = raw.version {
        let line = key_lines.get("version").copied().unwrap_or(1);
        config.version = SpecialVersion::parse(&version, Some(line))?;
        config.version_explicit = true;
    }

    if let Some(ignore_patterns) = raw.ignore {
        let line = key_lines.get("ignore").copied().unwrap_or(1);
        if ignore_patterns
            .iter()
            .any(|pattern| pattern.trim().is_empty())
        {
            bail!("line {} must not contain an empty ignore pattern", line);
        }
        config.ignore_patterns = ignore_patterns;
    }

    if let Some(toolchain) = raw.toolchain
        && let Some(manager) = toolchain.manager
    {
        let line = key_lines.get("toolchain").copied().unwrap_or(1);
        config.toolchain_manager = Some(ToolchainManager::parse(&manager, line)?);
    }

    Ok(config)
}

fn collect_top_level_key_lines(content: &str) -> std::collections::BTreeMap<String, usize> {
    let mut key_lines = std::collections::BTreeMap::new();
    for (index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') {
            if trimmed == "[toolchain]" {
                key_lines
                    .entry("toolchain".to_string())
                    .or_insert(index + 1);
            }
            continue;
        }

        let Some((raw_key, _)) = raw_line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim().trim_matches('"').trim_matches('\'');
        if !key.is_empty() {
            key_lines.entry(key.to_string()).or_insert(index + 1);
        }
    }
    key_lines
}

fn format_toml_parse_error(content: &str, err: &toml::de::Error) -> String {
    let line = err
        .span()
        .map(|span| line_for_offset(content, span.start))
        .unwrap_or(1);
    let message = err.message().trim();

    if let Some(key) = extract_quoted_identifier(message) {
        if message.contains("duplicate") {
            return format!("line {} repeats `{}`", line, key);
        }
        if message.contains("unknown field") {
            return format!("line {} uses unknown key `{}`", line, key);
        }
    }

    if message.contains("invalid string") || message.contains("invalid type") {
        return format!("line {} must use a quoted string value", line);
    }
    if message.contains("expected an equals")
        || message.contains("missing an equals")
        || message.contains("expected `=`")
    {
        return format!("line {} must use `key = \"value\"` syntax", line);
    }

    format!("line {} {message}", line)
}

fn extract_quoted_identifier(message: &str) -> Option<&str> {
    for delimiter in ['`', '\'', '"'] {
        if let Some((_, remainder)) = message.split_once(delimiter)
            && let Some((identifier, _)) = remainder.split_once(delimiter)
            && !identifier.is_empty()
        {
            return Some(identifier);
        }
    }
    None
}

fn line_for_offset(content: &str, offset: usize) -> usize {
    content[..offset.min(content.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ToolchainManager, parse_special_toml};
    use crate::config::SpecialVersion;

    #[test]
    fn reports_unsupported_special_toml_versions_with_line_context() {
        let err = parse_special_toml("root = \".\"\nversion = \"9\"\n")
            .expect_err("unsupported versions should fail");

        assert!(
            err.to_string()
                .contains("line 2 uses unsupported `special.toml` version `9`")
        );
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.VERSION
    fn parses_special_toml_version() {
        let config =
            parse_special_toml("version = \"1\"\nroot = \".\"\n").expect("config should parse");

        assert_eq!(config.version, SpecialVersion::V1);
        assert_eq!(config.root, Some(PathBuf::from(".")));
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.VERSION.DEFAULTS_TO_LEGACY
    fn defaults_special_toml_version_to_legacy() {
        let config =
            parse_special_toml("root = \".\"\n").expect("config without version should parse");

        assert_eq!(config.version, SpecialVersion::V0);
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.VERSION.UNKNOWN_REJECTED
    fn rejects_unknown_special_toml_version() {
        let err = parse_special_toml("version = \"2\"\n").expect_err("config should fail");

        assert!(
            err.to_string()
                .contains("unsupported `special.toml` version `2`")
        );
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.DUPLICATE_KEYS_REJECTED
    fn rejects_duplicate_special_toml_keys() {
        let err = parse_special_toml("root = \".\"\nroot = \"workspace\"\n")
            .expect_err("duplicate root should fail");

        let message = err.to_string();
        assert!(message.contains("line 2"));
        assert!(message.contains("root"));

        let err = parse_special_toml("version = \"1\"\nversion = \"0\"\n")
            .expect_err("duplicate version should fail");

        let message = err.to_string();
        assert!(message.contains("line 2"));
        assert!(message.contains("version"));
    }

    #[test]
    // @verifies SPECIAL.CONFIG.SPECIAL_TOML.ROOT_MUST_NOT_BE_EMPTY
    fn rejects_empty_special_toml_root() {
        let err = parse_special_toml("root = \"\"\n").expect_err("empty root should fail");

        assert!(err.to_string().contains("must not use an empty root path"));
    }

    #[test]
    fn parses_supported_toolchain_manager() {
        let config =
            parse_special_toml("[toolchain]\nmanager = \"mise\"\n").expect("config should parse");

        assert_eq!(config.toolchain_manager, Some(ToolchainManager::Mise));
    }

    #[test]
    fn rejects_unknown_toolchain_manager() {
        let err = parse_special_toml("[toolchain]\nmanager = \"npm\"\n")
            .expect_err("unknown manager should fail");

        assert!(
            err.to_string()
                .contains("unsupported toolchain manager `npm`")
        );
    }
}
