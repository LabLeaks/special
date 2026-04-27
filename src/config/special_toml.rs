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
    pub(crate) health_ignore_unexplained_patterns: Vec<String>,
    pub(crate) toolchain_manager: Option<ToolchainManager>,
    pub(crate) pattern_benchmarks: PatternMetricBenchmarks,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PatternMetricBenchmarks {
    pub(crate) high: f64,
    pub(crate) medium: f64,
    pub(crate) low: f64,
}

impl Default for PatternMetricBenchmarks {
    fn default() -> Self {
        Self {
            high: 0.55,
            medium: 0.45,
            low: 0.20,
        }
    }
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
    health: Option<RawHealthConfig>,
    toolchain: Option<RawToolchainConfig>,
    patterns: Option<RawPatternsConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHealthConfig {
    #[serde(rename = "ignore-unexplained")]
    ignore_unexplained: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawToolchainConfig {
    manager: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPatternsConfig {
    metrics: Option<RawPatternMetricsConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPatternMetricsConfig {
    high: Option<f64>,
    medium: Option<f64>,
    low: Option<f64>,
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
        if key != "root"
            && key != "version"
            && key != "ignore"
            && key != "health"
            && key != "toolchain"
            && key != "patterns"
        {
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

    if let Some(health) = raw.health
        && let Some(patterns) = health.ignore_unexplained
    {
        let line = key_lines.get("health").copied().unwrap_or(1);
        if patterns.iter().any(|pattern| pattern.trim().is_empty()) {
            bail!(
                "line {} must not contain an empty health ignore-unexplained pattern",
                line
            );
        }
        config.health_ignore_unexplained_patterns = patterns;
    }

    if let Some(toolchain) = raw.toolchain
        && let Some(manager) = toolchain.manager
    {
        let line = key_lines.get("toolchain").copied().unwrap_or(1);
        config.toolchain_manager = Some(ToolchainManager::parse(&manager, line)?);
    }

    if let Some(patterns) = raw.patterns
        && let Some(metrics) = patterns.metrics
    {
        let line = key_lines.get("patterns").copied().unwrap_or(1);
        config.pattern_benchmarks = parse_pattern_benchmark_config(metrics, line)?;
    }

    Ok(config)
}

fn parse_pattern_benchmark_config(
    raw: RawPatternMetricsConfig,
    line: usize,
) -> Result<PatternMetricBenchmarks> {
    let defaults = PatternMetricBenchmarks::default();
    let benchmarks = PatternMetricBenchmarks {
        high: parse_probability(raw.high, defaults.high, line)?,
        medium: parse_probability(raw.medium, defaults.medium, line)?,
        low: parse_probability(raw.low, defaults.low, line)?,
    };
    if !(benchmarks.high >= benchmarks.medium && benchmarks.medium >= benchmarks.low) {
        bail!(
            "line {} pattern metric benchmarks must be ordered high >= medium >= low",
            line
        );
    }
    Ok(benchmarks)
}

fn parse_probability(value: Option<f64>, default: f64, line: usize) -> Result<f64> {
    let Some(value) = value else {
        return Ok(default);
    };
    if !(0.0..=1.0).contains(&value) || !value.is_finite() {
        bail!(
            "line {} pattern metric benchmark values must be decimals from 0.0 through 1.0",
            line
        );
    }
    Ok(value)
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
            if trimmed == "[health]" {
                key_lines.entry("health".to_string()).or_insert(index + 1);
            }
            if trimmed == "[patterns]" || trimmed.starts_with("[patterns.") {
                key_lines.entry("patterns".to_string()).or_insert(index + 1);
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

    use super::{PatternMetricBenchmarks, ToolchainManager, parse_special_toml};
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
    fn parses_health_ignore_unexplained_patterns() {
        let config = parse_special_toml(
            "version = \"1\"\n[health]\nignore-unexplained = [\"generated/**\", \"fixtures.rs\"]\n",
        )
        .expect("config should parse");

        assert_eq!(
            config.health_ignore_unexplained_patterns,
            vec!["generated/**".to_string(), "fixtures.rs".to_string()]
        );
    }

    #[test]
    fn rejects_empty_health_ignore_unexplained_patterns() {
        let err = parse_special_toml("[health]\nignore-unexplained = [\"\"]\n")
            .expect_err("empty health ignore pattern should fail");

        assert!(
            err.to_string()
                .contains("must not contain an empty health ignore-unexplained pattern")
        );
    }

    #[test]
    fn parses_pattern_metric_benchmarks() {
        let config = parse_special_toml(
            "version = \"1\"\n[patterns.metrics]\nhigh = 0.81\nmedium = 0.44\nlow = 0.18\n",
        )
        .expect("config should parse");

        assert_eq!(
            config.pattern_benchmarks,
            PatternMetricBenchmarks {
                high: 0.81,
                medium: 0.44,
                low: 0.18,
            }
        );
    }

    #[test]
    fn rejects_out_of_range_pattern_metric_benchmarks() {
        let err = parse_special_toml("[patterns.metrics]\nhigh = 1.5\n")
            .expect_err("out of range benchmark should fail");

        assert!(
            err.to_string()
                .contains("pattern metric benchmark values must be decimals from 0.0 through 1.0")
        );
    }

    #[test]
    fn rejects_contradictory_pattern_metric_benchmarks() {
        let err = parse_special_toml(
            "version = \"1\"\n[patterns.metrics]\nhigh = 0.30\nmedium = 0.50\nlow = 0.20\n",
        )
        .expect_err("unordered benchmarks should fail");

        assert!(
            err.to_string()
                .contains("pattern metric benchmarks must be ordered high >= medium >= low")
        );
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
