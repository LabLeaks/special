/**
@module SPECIAL.CACHE.STORAGE
Serializes, deserializes, and atomically writes parsed and analysis cache entries for `SPECIAL.CACHE`.
*/
// @fileimplements SPECIAL.CACHE.STORAGE
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::model::{
    ArchitectureAnalysisSummary, ArchitectureKind, AttestRef, DeprecatedRelease, Diagnostic,
    ImplementRef, ModuleDecl, NodeKind, ParsedArchitecture, ParsedRepo, PatternApplication,
    PatternDefinition, PlanState, PlannedRelease, SourceLocation, SpecDecl, VerifyRef,
};
use crate::modules::analyze::ArchitectureAnalysis;

pub(super) fn cache_file_path(root: &Path, file_name: &str) -> PathBuf {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    root.hash(&mut hasher);
    let root_hash = hasher.finish();
    std::env::temp_dir()
        .join("special-cache")
        .join(format!("{root_hash:016x}"))
        .join(file_name)
}

pub(super) fn read_repo_cache(path: &Path, fingerprint: u64) -> Result<Option<ParsedRepo>> {
    let Ok(bytes) = fs::read(path) else {
        return Ok(None);
    };
    let Ok(envelope) = serde_json::from_slice::<RepoCacheEnvelope>(&bytes) else {
        return Ok(None);
    };
    if envelope.fingerprint != fingerprint {
        return Ok(None);
    }
    Ok(Some(envelope.value.into_parsed_repo()?))
}

pub(super) fn write_repo_cache(path: &Path, fingerprint: u64, parsed: &ParsedRepo) -> Result<()> {
    let envelope = RepoCacheEnvelope {
        fingerprint,
        value: CachedParsedRepo::from(parsed),
    };
    write_serialized_cache(path, &envelope)
}

pub(super) fn read_architecture_cache(
    path: &Path,
    fingerprint: u64,
) -> Result<Option<ParsedArchitecture>> {
    let Ok(bytes) = fs::read(path) else {
        return Ok(None);
    };
    let Ok(envelope) = serde_json::from_slice::<ArchitectureCacheEnvelope>(&bytes) else {
        return Ok(None);
    };
    if envelope.fingerprint != fingerprint {
        return Ok(None);
    }
    Ok(Some(envelope.value.into_parsed_architecture()?))
}

pub(super) fn write_architecture_cache(
    path: &Path,
    fingerprint: u64,
    parsed: &ParsedArchitecture,
) -> Result<()> {
    let envelope = ArchitectureCacheEnvelope {
        fingerprint,
        value: CachedParsedArchitecture::from(parsed),
    };
    write_serialized_cache(path, &envelope)
}

pub(super) fn read_repo_analysis_cache(
    path: &Path,
    fingerprint: u64,
) -> Result<Option<ArchitectureAnalysisSummary>> {
    read_analysis_cache::<RepoAnalysisCacheEnvelope, _>(path, fingerprint, |envelope| {
        envelope.value
    })
}

pub(super) fn write_repo_analysis_cache(
    path: &Path,
    fingerprint: u64,
    summary: &ArchitectureAnalysisSummary,
) -> Result<()> {
    write_serialized_cache(
        path,
        &RepoAnalysisCacheEnvelope {
            fingerprint,
            value: summary.clone(),
        },
    )
}

pub(super) fn read_architecture_analysis_cache(
    path: &Path,
    fingerprint: u64,
) -> Result<Option<ArchitectureAnalysis>> {
    read_analysis_cache::<ArchitectureAnalysisCacheEnvelope, _>(path, fingerprint, |envelope| {
        envelope.value
    })
}

pub(super) fn write_architecture_analysis_cache(
    path: &Path,
    fingerprint: u64,
    analysis: &ArchitectureAnalysis,
) -> Result<()> {
    write_serialized_cache(
        path,
        &ArchitectureAnalysisCacheEnvelope {
            fingerprint,
            value: analysis.clone(),
        },
    )
}

pub(super) fn read_blob_cache(path: &Path, fingerprint: u64) -> Result<Option<Vec<u8>>> {
    read_analysis_cache::<BlobCacheEnvelope, _>(path, fingerprint, |envelope| envelope.value)
}

pub(super) fn write_blob_cache(path: &Path, fingerprint: u64, value: &[u8]) -> Result<()> {
    write_serialized_cache(
        path,
        &BlobCacheEnvelope {
            fingerprint,
            value: value.to_vec(),
        },
    )
}

fn read_analysis_cache<T, U>(
    path: &Path,
    fingerprint: u64,
    into_value: impl FnOnce(T) -> U,
) -> Result<Option<U>>
where
    T: for<'de> Deserialize<'de> + AnalysisCacheEnvelopeValue,
{
    let Ok(bytes) = fs::read(path) else {
        return Ok(None);
    };
    let Ok(envelope) = serde_json::from_slice::<T>(&bytes) else {
        return Ok(None);
    };
    if envelope.fingerprint() != fingerprint {
        return Ok(None);
    }
    Ok(Some(into_value(envelope)))
}

fn write_serialized_cache<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let bytes = serde_json::to_vec(value)?;
    write_cache_bytes(path, &bytes)?;
    Ok(())
}

fn write_cache_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let temp_path = cache_temp_path(path);
    fs::write(&temp_path, bytes)?;
    if let Err(error) = replace_cache_file(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error.into());
    }
    Ok(())
}

fn replace_cache_file(temp_path: &Path, path: &Path) -> std::io::Result<()> {
    match fs::rename(temp_path, path) {
        Ok(()) => Ok(()),
        Err(error)
            if cfg!(windows)
                && path.exists()
                && matches!(
                    error.kind(),
                    ErrorKind::AlreadyExists | ErrorKind::PermissionDenied
                ) =>
        {
            fs::remove_file(path)?;
            fs::rename(temp_path, path)
        }
        Err(error) => Err(error),
    }
}

fn cache_temp_path(path: &Path) -> PathBuf {
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
    let suffix = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("cache");
    path.with_file_name(format!(
        ".{}.{}.{}.tmp",
        file_name,
        std::process::id(),
        suffix
    ))
}

#[derive(Debug, Serialize, Deserialize)]
struct RepoCacheEnvelope {
    fingerprint: u64,
    value: CachedParsedRepo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArchitectureCacheEnvelope {
    fingerprint: u64,
    value: CachedParsedArchitecture,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepoAnalysisCacheEnvelope {
    fingerprint: u64,
    value: ArchitectureAnalysisSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArchitectureAnalysisCacheEnvelope {
    fingerprint: u64,
    value: ArchitectureAnalysis,
}

trait AnalysisCacheEnvelopeValue {
    fn fingerprint(&self) -> u64;
}

impl AnalysisCacheEnvelopeValue for RepoAnalysisCacheEnvelope {
    fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

impl AnalysisCacheEnvelopeValue for ArchitectureAnalysisCacheEnvelope {
    fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

#[derive(Serialize, Deserialize)]
struct BlobCacheEnvelope {
    fingerprint: u64,
    value: Vec<u8>,
}

impl AnalysisCacheEnvelopeValue for BlobCacheEnvelope {
    fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedParsedRepo {
    specs: Vec<CachedSpecDecl>,
    verifies: Vec<VerifyRef>,
    attests: Vec<AttestRef>,
    diagnostics: Vec<Diagnostic>,
}

impl CachedParsedRepo {
    fn into_parsed_repo(self) -> Result<ParsedRepo> {
        Ok(ParsedRepo {
            specs: self
                .specs
                .into_iter()
                .map(CachedSpecDecl::into_spec_decl)
                .collect::<Result<Vec<_>>>()?,
            verifies: self.verifies,
            attests: self.attests,
            diagnostics: self.diagnostics,
        })
    }
}

impl From<&ParsedRepo> for CachedParsedRepo {
    fn from(parsed: &ParsedRepo) -> Self {
        Self {
            specs: parsed.specs.iter().map(CachedSpecDecl::from).collect(),
            verifies: parsed.verifies.clone(),
            attests: parsed.attests.clone(),
            diagnostics: parsed.diagnostics.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedParsedArchitecture {
    modules: Vec<CachedModuleDecl>,
    implements: Vec<ImplementRef>,
    #[serde(default)]
    patterns: Vec<PatternDefinition>,
    #[serde(default)]
    pattern_applications: Vec<PatternApplication>,
    diagnostics: Vec<Diagnostic>,
}

impl CachedParsedArchitecture {
    fn into_parsed_architecture(self) -> Result<ParsedArchitecture> {
        Ok(ParsedArchitecture {
            modules: self
                .modules
                .into_iter()
                .map(CachedModuleDecl::into_module_decl)
                .collect::<Result<Vec<_>>>()?,
            implements: self.implements,
            patterns: self.patterns,
            pattern_applications: self.pattern_applications,
            diagnostics: self.diagnostics,
        })
    }
}

impl From<&ParsedArchitecture> for CachedParsedArchitecture {
    fn from(parsed: &ParsedArchitecture) -> Self {
        Self {
            modules: parsed.modules.iter().map(CachedModuleDecl::from).collect(),
            implements: parsed.implements.clone(),
            patterns: parsed.patterns.clone(),
            pattern_applications: parsed.pattern_applications.clone(),
            diagnostics: parsed.diagnostics.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSpecDecl {
    id: String,
    kind: NodeKind,
    text: String,
    planned: bool,
    planned_release: Option<String>,
    deprecated: bool,
    deprecated_release: Option<String>,
    location: SourceLocation,
}

impl CachedSpecDecl {
    fn into_spec_decl(self) -> Result<SpecDecl> {
        SpecDecl::new(
            self.id,
            self.kind,
            self.text,
            if self.planned {
                PlanState::planned(self.planned_release.map(PlannedRelease::new).transpose()?)
            } else {
                PlanState::current()
            },
            self.deprecated,
            self.deprecated_release
                .map(DeprecatedRelease::new)
                .transpose()?,
            self.location,
        )
        .map_err(Into::into)
    }
}

impl From<&SpecDecl> for CachedSpecDecl {
    fn from(spec: &SpecDecl) -> Self {
        Self {
            id: spec.id.clone(),
            kind: spec.kind(),
            text: spec.text.clone(),
            planned: spec.is_planned(),
            planned_release: spec.planned_release().map(str::to_string),
            deprecated: spec.is_deprecated(),
            deprecated_release: spec.deprecated_release().map(str::to_string),
            location: spec.location.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedModuleDecl {
    id: String,
    kind: ArchitectureKind,
    text: String,
    planned: bool,
    planned_release: Option<String>,
    location: SourceLocation,
}

impl CachedModuleDecl {
    fn into_module_decl(self) -> Result<ModuleDecl> {
        ModuleDecl::new(
            self.id,
            self.kind,
            self.text,
            if self.planned {
                PlanState::planned(self.planned_release.map(PlannedRelease::new).transpose()?)
            } else {
                PlanState::current()
            },
            self.location,
        )
        .map_err(Into::into)
    }
}

impl From<&ModuleDecl> for CachedModuleDecl {
    fn from(module: &ModuleDecl) -> Self {
        Self {
            id: module.id.clone(),
            kind: module.kind(),
            text: module.text.clone(),
            planned: module.is_planned(),
            planned_release: module.plan().release().map(str::to_string),
            location: module.location.clone(),
        }
    }
}
