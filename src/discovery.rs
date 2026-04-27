/**
@module SPECIAL.DISCOVERY
Discovers source and markdown annotation files under the resolved project root, applying shared VCS and `special.toml` ignore rules before spec or module parsing begins.
*/
// @fileimplements SPECIAL.DISCOVERY
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

#[derive(Debug, Clone)]
pub struct DiscoveryConfig<'a> {
    pub root: &'a Path,
    pub ignore_patterns: &'a [String],
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveredAnnotationFiles {
    pub source_files: Vec<PathBuf>,
    pub markdown_files: Vec<PathBuf>,
}

pub fn discover_annotation_files(config: DiscoveryConfig<'_>) -> Result<DiscoveredAnnotationFiles> {
    let cache_key = DiscoveryCacheKey {
        root: config.root.to_path_buf(),
        ignore_patterns: config.ignore_patterns.to_vec(),
    };
    if let Some(cached) = discovery_cache()
        .lock()
        .expect("discovery cache mutex should not be poisoned")
        .get(&cache_key)
        .cloned()
    {
        return Ok(cached);
    }

    let matcher = IgnoreMatcher::build(config.root, config.ignore_patterns)?;
    let mut discovered = DiscoveredAnnotationFiles::default();

    let walker = WalkBuilder::new(config.root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .add_custom_ignore_filename(".gitignore")
        .add_custom_ignore_filename(".jjignore")
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        if !entry
            .file_type()
            .map(|kind| kind.is_file())
            .unwrap_or(false)
        {
            continue;
        }
        if matcher.is_ignored(path) {
            continue;
        }

        if is_supported_source(path) {
            discovered.source_files.push(path.to_path_buf());
        } else if is_supported_markdown(path) {
            discovered.markdown_files.push(path.to_path_buf());
        }
    }

    discovered.source_files.sort();
    discovered.markdown_files.sort();
    discovery_cache()
        .lock()
        .expect("discovery cache mutex should not be poisoned")
        .insert(cache_key, discovered.clone());
    Ok(discovered)
}

pub(crate) fn path_matches_patterns(root: &Path, path: &Path, patterns: &[String]) -> Result<bool> {
    Ok(IgnoreMatcher::build(root, patterns)?.is_ignored(path))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DiscoveryCacheKey {
    root: PathBuf,
    ignore_patterns: Vec<String>,
}

fn discovery_cache()
-> &'static Mutex<std::collections::HashMap<DiscoveryCacheKey, DiscoveredAnnotationFiles>> {
    static CACHE: OnceLock<
        Mutex<std::collections::HashMap<DiscoveryCacheKey, DiscoveredAnnotationFiles>>,
    > = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn is_supported_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "go" | "ts" | "tsx" | "sh" | "py")
    )
}

fn is_supported_markdown(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("md"))
}

#[derive(Debug, Default)]
struct IgnoreMatcher {
    root: PathBuf,
    globset: Option<GlobSet>,
    plain_prefixes: Vec<String>,
}

impl IgnoreMatcher {
    fn build(root: &Path, patterns: &[String]) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        let mut has_globs = false;
        let mut plain_prefixes = Vec::new();

        for pattern in patterns {
            let trimmed = pattern.trim();
            if trimmed.is_empty() {
                continue;
            }

            if contains_glob_syntax(trimmed) {
                builder.add(Glob::new(trimmed)?);
                has_globs = true;
            } else {
                plain_prefixes.push(trimmed.trim_end_matches('/').to_string());
            }
        }

        let globset = if has_globs {
            Some(builder.build()?)
        } else {
            None
        };

        Ok(Self {
            root: root.to_path_buf(),
            globset,
            plain_prefixes,
        })
    }

    fn is_ignored(&self, path: &Path) -> bool {
        let relative = if let Ok(relative) = path.strip_prefix(&self.root) {
            relative
        } else {
            path
        };
        let relative = relative.to_string_lossy().replace('\\', "/");

        if self
            .plain_prefixes
            .iter()
            .any(|prefix| relative == *prefix || relative.starts_with(&format!("{prefix}/")))
        {
            return true;
        }

        self.globset
            .as_ref()
            .is_some_and(|globset| globset.is_match(&relative))
    }
}

fn contains_glob_syntax(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?') || pattern.contains('[') || pattern.contains('{')
}
