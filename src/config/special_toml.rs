/**
@module SPECIAL.CONFIG.SPECIAL_TOML
Parses and loads `special.toml` root and version settings. This module does not choose VCS or current-directory fallbacks when config is absent.
*/
// @implements SPECIAL.CONFIG.SPECIAL_TOML
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use toml::Table;

use super::{SpecialToml, SpecialVersion};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpecialToml {
    root: Option<String>,
    version: Option<String>,
}

pub(super) fn load_special_toml(path: &Path) -> Result<SpecialToml> {
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
        if key != "root" && key != "version" {
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

    Ok(config)
}

fn collect_top_level_key_lines(content: &str) -> std::collections::BTreeMap<String, usize> {
    let mut key_lines = std::collections::BTreeMap::new();
    for (index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
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
