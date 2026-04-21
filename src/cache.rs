use std::cell::RefCell;
/**
@module SPECIAL.CACHE
Persistent parsed and analysis cache for health and architecture surfaces. This module memoizes parsed repo annotations, parsed architecture declarations, and whole unscoped analysis summaries across command invocations using discovered-file metadata plus language-pack environment fingerprints as invalidation inputs. It should stay a reusable substrate underneath `special`, `special specs`, `special arch`, and `special health` rather than caching rendered output shapes.
*/
// @fileimplements SPECIAL.CACHE
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::SpecialVersion;
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::{
    ArchitectureAnalysisSummary, ArchitectureKind, AttestRef, DeprecatedRelease, Diagnostic,
    ImplementRef, ModuleAnalysisOptions, ModuleDecl, NodeKind, ParsedArchitecture, ParsedRepo,
    PlanState, PlannedRelease, SourceLocation, SpecDecl, VerifyRef,
};
use crate::modules;
use crate::modules::analyze::{self, ArchitectureAnalysis};
use crate::parser::{self, ParseDialect};

const CACHE_SCHEMA_VERSION: u32 = 3;
const CACHE_LOCK_STALE_AFTER: Duration = Duration::from_secs(300);
const CACHE_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(100);
const CACHE_LOCK_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    pub repo_hits: usize,
    pub repo_misses: usize,
    pub architecture_hits: usize,
    pub architecture_misses: usize,
    pub repo_analysis_hits: usize,
    pub repo_analysis_misses: usize,
    pub architecture_analysis_hits: usize,
    pub architecture_analysis_misses: usize,
    pub lock_waits: usize,
    pub stale_lock_recovers: usize,
}

thread_local! {
    static CACHE_STATUS_NOTIFIER: RefCell<Option<Box<dyn Fn(&str)>>> = RefCell::new(None);
}

pub fn reset_cache_stats() {
    *cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned") = CacheStats::default();
}

pub fn snapshot_cache_stats() -> CacheStats {
    *cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned")
}

pub fn format_cache_stats_summary() -> Option<String> {
    let stats = snapshot_cache_stats();
    if stats.repo_hits
        + stats.repo_misses
        + stats.architecture_hits
        + stats.architecture_misses
        + stats.repo_analysis_hits
        + stats.repo_analysis_misses
        + stats.architecture_analysis_hits
        + stats.architecture_analysis_misses
        + stats.lock_waits
        + stats.stale_lock_recovers
        == 0
    {
        return None;
    }
    let mut summary = format!(
        "cache activity: repo annotations reused {}, rebuilt {}; architecture declarations reused {}, rebuilt {}; health analysis reused {}, rebuilt {}; architecture analysis reused {}, rebuilt {}",
        stats.repo_hits,
        stats.repo_misses,
        stats.architecture_hits,
        stats.architecture_misses,
        stats.repo_analysis_hits,
        stats.repo_analysis_misses,
        stats.architecture_analysis_hits,
        stats.architecture_analysis_misses
    );
    if stats.lock_waits > 0 || stats.stale_lock_recovers > 0 {
        summary.push_str(&format!(
            "; waited for another run {} time(s); recovered {} stale cache lock(s)",
            stats.lock_waits, stats.stale_lock_recovers
        ));
    }
    Some(summary)
}

pub fn with_cache_status_notifier<T>(
    notifier: impl Fn(&str) + 'static,
    f: impl FnOnce() -> T,
) -> T {
    CACHE_STATUS_NOTIFIER.with(|cell| {
        let previous = cell.replace(Some(Box::new(notifier)));
        let result = f();
        let _ = cell.replace(previous);
        result
    })
}

pub fn load_or_parse_repo(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<ParsedRepo> {
    let fingerprint = repo_fingerprint(root, ignore_patterns, version)?;
    let cache_path = cache_file_path(root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(parsed) = read_repo_cache(&cache_path, fingerprint)? {
        record_repo_hit();
        return Ok(parsed);
    }

    let _guard = acquire_cache_fill_lock(&cache_path)?;
    if let Some(parsed) = read_repo_cache(&cache_path, fingerprint)? {
        record_repo_hit();
        return Ok(parsed);
    }

    let parsed = parser::parse_repo(root, ignore_patterns, parse_dialect(version))?;
    write_repo_cache(&cache_path, fingerprint, &parsed)?;
    record_repo_miss();
    Ok(parsed)
}

pub fn load_or_parse_architecture(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<ParsedArchitecture> {
    let fingerprint = architecture_fingerprint(root, ignore_patterns)?;
    let cache_path = cache_file_path(
        root,
        &format!("parsed-architecture-v{CACHE_SCHEMA_VERSION}.json"),
    );
    if let Some(parsed) = read_architecture_cache(&cache_path, fingerprint)? {
        record_architecture_hit();
        return Ok(parsed);
    }

    let _guard = acquire_cache_fill_lock(&cache_path)?;
    if let Some(parsed) = read_architecture_cache(&cache_path, fingerprint)? {
        record_architecture_hit();
        return Ok(parsed);
    }

    let parsed = modules::parse_architecture(root, ignore_patterns)?;
    write_architecture_cache(&cache_path, fingerprint, &parsed)?;
    record_architecture_miss();
    Ok(parsed)
}

pub fn load_or_build_repo_analysis_summary(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_architecture: &ParsedArchitecture,
    parsed_repo: &ParsedRepo,
) -> Result<ArchitectureAnalysisSummary> {
    let fingerprint = repo_analysis_fingerprint(root, ignore_patterns, version, parsed_repo)?;
    let cache_path = cache_file_path(root, &format!("repo-analysis-v{CACHE_SCHEMA_VERSION}.json"));
    if let Some(summary) =
        read_analysis_cache::<RepoAnalysisCacheEnvelope, _>(&cache_path, fingerprint, |envelope| {
            envelope.value
        })?
    {
        record_repo_analysis_hit();
        return Ok(summary);
    }

    let _guard = acquire_cache_fill_lock(&cache_path)?;
    if let Some(summary) =
        read_analysis_cache::<RepoAnalysisCacheEnvelope, _>(&cache_path, fingerprint, |envelope| {
            envelope.value
        })?
    {
        record_repo_analysis_hit();
        return Ok(summary);
    }

    let summary = analyze::build_repo_analysis_summary(
        root,
        ignore_patterns,
        parsed_architecture,
        parsed_repo,
        None,
    )?;
    write_analysis_cache(
        &cache_path,
        &RepoAnalysisCacheEnvelope {
            fingerprint,
            value: summary.clone(),
        },
    )?;
    record_repo_analysis_miss();
    Ok(summary)
}

pub fn load_or_build_architecture_analysis(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_architecture: &ParsedArchitecture,
    parsed_repo: Option<&ParsedRepo>,
    options: ModuleAnalysisOptions,
) -> Result<ArchitectureAnalysis> {
    let options = options.normalized();
    let fingerprint = architecture_analysis_fingerprint(
        root,
        ignore_patterns,
        version,
        parsed_repo.is_some(),
        options,
    )?;
    let cache_path = cache_file_path(
        root,
        &format!("architecture-analysis-v{CACHE_SCHEMA_VERSION}.json"),
    );
    if let Some(analysis) = read_analysis_cache::<ArchitectureAnalysisCacheEnvelope, _>(
        &cache_path,
        fingerprint,
        |envelope| envelope.value,
    )? {
        record_architecture_analysis_hit();
        return Ok(analysis);
    }

    let _guard = acquire_cache_fill_lock(&cache_path)?;
    if let Some(analysis) = read_analysis_cache::<ArchitectureAnalysisCacheEnvelope, _>(
        &cache_path,
        fingerprint,
        |envelope| envelope.value,
    )? {
        record_architecture_analysis_hit();
        return Ok(analysis);
    }

    let analysis = analyze::build_architecture_analysis(
        root,
        ignore_patterns,
        parsed_architecture,
        parsed_repo,
        options,
    )?;
    write_analysis_cache(
        &cache_path,
        &ArchitectureAnalysisCacheEnvelope {
            fingerprint,
            value: analysis.clone(),
        },
    )?;
    record_architecture_analysis_miss();
    Ok(analysis)
}

fn repo_fingerprint(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<u64> {
    fingerprint_for_discovered_files(root, ignore_patterns, Some(version))
}

fn architecture_fingerprint(root: &Path, ignore_patterns: &[String]) -> Result<u64> {
    fingerprint_for_discovered_files(root, ignore_patterns, None)
}

fn repo_analysis_fingerprint(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    parsed_repo: &ParsedRepo,
) -> Result<u64> {
    let files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let mut hasher = DefaultHasher::new();
    repo_fingerprint(root, ignore_patterns, version)?.hash(&mut hasher);
    parsed_repo.verifies.len().hash(&mut hasher);
    parsed_repo.attests.len().hash(&mut hasher);
    analyze::analysis_environment_fingerprint(root, &files).hash(&mut hasher);
    Ok(hasher.finish())
}

fn architecture_analysis_fingerprint(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    include_repo: bool,
    options: ModuleAnalysisOptions,
) -> Result<u64> {
    let files = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files;
    let mut hasher = DefaultHasher::new();
    architecture_fingerprint(root, ignore_patterns)?.hash(&mut hasher);
    if include_repo {
        repo_fingerprint(root, ignore_patterns, version)?.hash(&mut hasher);
    }
    include_repo.hash(&mut hasher);
    options.coverage.hash(&mut hasher);
    options.metrics.hash(&mut hasher);
    options.traceability.hash(&mut hasher);
    analyze::analysis_environment_fingerprint(root, &files).hash(&mut hasher);
    Ok(hasher.finish())
}

fn fingerprint_for_discovered_files(
    root: &Path,
    ignore_patterns: &[String],
    version: Option<SpecialVersion>,
) -> Result<u64> {
    let discovered = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?;
    let mut hasher = DefaultHasher::new();
    CACHE_SCHEMA_VERSION.hash(&mut hasher);
    root.hash(&mut hasher);
    ignore_patterns.hash(&mut hasher);
    version.map(SpecialVersion::as_str).hash(&mut hasher);

    for path in discovered
        .source_files
        .iter()
        .chain(discovered.markdown_files.iter())
    {
        path.hash(&mut hasher);
        if let Ok(metadata) = fs::metadata(path) {
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    duration.as_secs().hash(&mut hasher);
                    duration.subsec_nanos().hash(&mut hasher);
                }
            }
        }
        hash_file_contents(path, &mut hasher);
    }

    Ok(hasher.finish())
}

fn hash_file_contents(path: &Path, hasher: &mut DefaultHasher) {
    let Ok(contents) = fs::read(path) else {
        return;
    };
    contents.hash(hasher);
}

fn cache_file_path(root: &Path, file_name: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    root.hash(&mut hasher);
    let root_hash = hasher.finish();
    std::env::temp_dir()
        .join("special-cache")
        .join(format!("{root_hash:016x}"))
        .join(file_name)
}

fn cache_lock_path(cache_path: &Path) -> PathBuf {
    let mut path = cache_path.as_os_str().to_os_string();
    path.push(".lock");
    PathBuf::from(path)
}

fn acquire_cache_fill_lock(cache_path: &Path) -> Result<CacheFillGuard> {
    acquire_cache_fill_lock_with_hooks(cache_path, SystemTime::now, std::thread::sleep)
}

fn acquire_cache_fill_lock_with_hooks(
    cache_path: &Path,
    now: impl Fn() -> SystemTime,
    sleep: impl Fn(Duration),
) -> Result<CacheFillGuard> {
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let lock_path = cache_lock_path(cache_path);
    let mut waited = false;
    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => {
                let owner_token = next_cache_lock_owner();
                refresh_cache_lock(&lock_path, &owner_token);
                return Ok(CacheFillGuard {
                    path: lock_path.clone(),
                    owner_token: owner_token.clone(),
                    heartbeat: Some(start_cache_lock_heartbeat(
                        lock_path,
                        owner_token,
                        CACHE_LOCK_REFRESH_INTERVAL,
                    )),
                });
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                if !waited {
                    record_lock_wait();
                    emit_cache_status(&format!(
                        "another special run is already building {}; waiting to reuse its cached result",
                        cache_entry_label(cache_path)
                    ));
                    waited = true;
                }
                if cache_lock_is_stale_at(&lock_path, now()) {
                    record_stale_lock_recover();
                    emit_cache_status(&format!(
                        "recovered an abandoned cache lock for {}; rebuilding it now",
                        cache_entry_label(cache_path)
                    ));
                    let _ = fs::remove_file(&lock_path);
                    continue;
                }
                sleep(CACHE_LOCK_POLL_INTERVAL);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn cache_lock_is_stale_at(lock_path: &Path, now: SystemTime) -> bool {
    let Ok(metadata) = fs::metadata(lock_path) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    let Ok(elapsed) = now.duration_since(modified) else {
        return false;
    };
    elapsed > CACHE_LOCK_STALE_AFTER
}

struct CacheFillGuard {
    path: PathBuf,
    owner_token: String,
    heartbeat: Option<CacheLockHeartbeat>,
}

impl Drop for CacheFillGuard {
    fn drop(&mut self) {
        if let Some(heartbeat) = self.heartbeat.take() {
            heartbeat.stop();
        }
        if cache_lock_owner(&self.path).as_deref() == Some(self.owner_token.as_str()) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

struct CacheLockHeartbeat {
    stop: Sender<()>,
    handle: JoinHandle<()>,
}

impl CacheLockHeartbeat {
    fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.handle.join();
    }
}

fn start_cache_lock_heartbeat(
    lock_path: PathBuf,
    owner_token: String,
    interval: Duration,
) -> CacheLockHeartbeat {
    let (stop, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        while let Err(mpsc::RecvTimeoutError::Timeout) = rx.recv_timeout(interval) {
            if cache_lock_owner(&lock_path).as_deref() != Some(owner_token.as_str()) {
                break;
            }
            refresh_cache_lock(&lock_path, &owner_token);
        }
    });
    CacheLockHeartbeat { stop, handle }
}

fn refresh_cache_lock(lock_path: &Path, owner_token: &str) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string());
    let _ = fs::write(
        lock_path,
        format!("owner={owner_token}\nupdated_nanos={stamp}\n"),
    );
}

fn cache_lock_owner(lock_path: &Path) -> Option<String> {
    let contents = fs::read_to_string(lock_path).ok()?;
    contents
        .lines()
        .find_map(|line| line.strip_prefix("owner=").map(ToString::to_string))
}

fn next_cache_lock_owner() -> String {
    static LOCK_OWNER_COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = LOCK_OWNER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}-{counter:x}")
}

fn cache_stats() -> &'static Mutex<CacheStats> {
    static CACHE_STATS: OnceLock<Mutex<CacheStats>> = OnceLock::new();
    CACHE_STATS.get_or_init(|| Mutex::new(CacheStats::default()))
}

fn emit_cache_status(message: &str) {
    CACHE_STATUS_NOTIFIER.with(|cell| {
        if let Some(notifier) = cell.borrow().as_ref() {
            notifier(message);
        }
    });
}

fn cache_entry_label(cache_path: &Path) -> &'static str {
    let file_name = cache_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if file_name.starts_with("parsed-repo-") {
        "repo annotations"
    } else if file_name.starts_with("parsed-architecture-") {
        "architecture declarations"
    } else if file_name.starts_with("repo-analysis-") {
        "health analysis"
    } else if file_name.starts_with("architecture-analysis-") {
        "architecture analysis"
    } else {
        "shared analysis"
    }
}

fn record_repo_hit() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.repo_hits += 1;
}

fn record_repo_miss() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.repo_misses += 1;
}

fn record_architecture_hit() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.architecture_hits += 1;
}

fn record_architecture_miss() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.architecture_misses += 1;
}

fn record_repo_analysis_hit() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.repo_analysis_hits += 1;
}

fn record_repo_analysis_miss() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.repo_analysis_misses += 1;
}

fn record_architecture_analysis_hit() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.architecture_analysis_hits += 1;
}

fn record_architecture_analysis_miss() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.architecture_analysis_misses += 1;
}

fn record_lock_wait() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.lock_waits += 1;
}

fn record_stale_lock_recover() {
    let mut stats = cache_stats()
        .lock()
        .expect("cache stats mutex should not be poisoned");
    stats.stale_lock_recovers += 1;
}

fn read_repo_cache(path: &Path, fingerprint: u64) -> Result<Option<ParsedRepo>> {
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

fn write_repo_cache(path: &Path, fingerprint: u64, parsed: &ParsedRepo) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let envelope = RepoCacheEnvelope {
        fingerprint,
        value: CachedParsedRepo::from(parsed),
    };
    let bytes = serde_json::to_vec(&envelope)?;
    write_cache_bytes(path, &bytes)?;
    Ok(())
}

fn read_architecture_cache(path: &Path, fingerprint: u64) -> Result<Option<ParsedArchitecture>> {
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

fn read_analysis_cache<T, U>(
    path: &Path,
    fingerprint: u64,
    into_value: impl FnOnce(T) -> U,
) -> Result<Option<U>>
where
    T: for<'de> Deserialize<'de> + AnalysisCacheEnvelopeValue<U>,
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

fn write_architecture_cache(
    path: &Path,
    fingerprint: u64,
    parsed: &ParsedArchitecture,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let envelope = ArchitectureCacheEnvelope {
        fingerprint,
        value: CachedParsedArchitecture::from(parsed),
    };
    let bytes = serde_json::to_vec(&envelope)?;
    write_cache_bytes(path, &bytes)?;
    Ok(())
}

fn write_analysis_cache<T>(path: &Path, envelope: &T) -> Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let bytes = serde_json::to_vec(envelope)?;
    write_cache_bytes(path, &bytes)?;
    Ok(())
}

fn write_cache_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let temp_path = cache_temp_path(path);
    fs::write(&temp_path, bytes)?;
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error.into());
    }
    Ok(())
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

fn parse_dialect(version: SpecialVersion) -> ParseDialect {
    match version {
        SpecialVersion::V0 => ParseDialect::CompatibilityV0,
        SpecialVersion::V1 => ParseDialect::CurrentV1,
    }
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

trait AnalysisCacheEnvelopeValue<T> {
    fn fingerprint(&self) -> u64;
}

impl AnalysisCacheEnvelopeValue<ArchitectureAnalysisSummary> for RepoAnalysisCacheEnvelope {
    fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}

impl AnalysisCacheEnvelopeValue<ArchitectureAnalysis> for ArchitectureAnalysisCacheEnvelope {
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
            diagnostics: self.diagnostics,
        })
    }
}

impl From<&ParsedArchitecture> for CachedParsedArchitecture {
    fn from(parsed: &ParsedArchitecture) -> Self {
        Self {
            modules: parsed.modules.iter().map(CachedModuleDecl::from).collect(),
            implements: parsed.implements.clone(),
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Barrier};
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        CACHE_LOCK_STALE_AFTER, CACHE_SCHEMA_VERSION, CacheFillGuard, acquire_cache_fill_lock,
        acquire_cache_fill_lock_with_hooks, cache_file_path, cache_lock_owner, cache_lock_path,
        load_or_build_architecture_analysis, load_or_build_repo_analysis_summary,
        load_or_parse_architecture, load_or_parse_repo, reset_cache_stats, snapshot_cache_stats,
        start_cache_lock_heartbeat, with_cache_status_notifier,
    };
    use crate::config::SpecialVersion;
    use crate::model::ModuleAnalysisOptions;

    fn cache_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn temp_root(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        std::fs::create_dir_all(&root).expect("temp root should be created");
        root
    }

    fn write_repo_fixture(root: &Path) {
        std::fs::create_dir_all(root.join("_project")).expect("architecture dir should exist");
        std::fs::create_dir_all(root.join("specs")).expect("specs dir should exist");
        std::fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
            .expect("config should be written");
        std::fs::write(
            root.join("_project/ARCHITECTURE.md"),
            "# Architecture\n\n### `@module APP.CORE`\nCore module.\n",
        )
        .expect("architecture should be written");
        std::fs::write(
            root.join("specs/root.md"),
            "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
        )
        .expect("specs should be written");
        std::fs::write(
            root.join("app.rs"),
            "/**\n@spec APP.LIVE\nLive behavior.\n*/\n\n// @fileimplements APP.CORE\npub fn live_impl() {}\n",
        )
        .expect("source should be written");
    }

    #[test]
    fn parsed_repo_cache_hits_on_second_load() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-repo-hit");
        write_repo_fixture(&root);

        reset_cache_stats();
        let _ = load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("parse should succeed");
        let first = snapshot_cache_stats();
        assert_eq!(first.repo_hits, 0);
        assert_eq!(first.repo_misses, 1);

        let _ = load_or_parse_repo(&root, &[], SpecialVersion::V1)
            .expect("cached parse should succeed");
        let second = snapshot_cache_stats();
        assert_eq!(second.repo_hits, 1);
        assert_eq!(second.repo_misses, 1);

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn parsed_architecture_cache_invalidates_after_file_change() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-arch-invalidate");
        write_repo_fixture(&root);

        reset_cache_stats();
        let _ = load_or_parse_architecture(&root, &[]).expect("parse should succeed");
        let first = snapshot_cache_stats();
        assert_eq!(first.architecture_hits, 0);
        assert_eq!(first.architecture_misses, 1);

        let _ = load_or_parse_architecture(&root, &[]).expect("cached parse should succeed");
        let second = snapshot_cache_stats();
        assert_eq!(second.architecture_hits, 1);
        assert_eq!(second.architecture_misses, 1);

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root.join("_project/ARCHITECTURE.md"),
            "# Architecture\n\n### `@module APP.CORE`\nChanged module text.\n",
        )
        .expect("architecture should be rewritten");

        let _ = load_or_parse_architecture(&root, &[]).expect("reparse should succeed");
        let third = snapshot_cache_stats();
        assert_eq!(third.architecture_hits, 1);
        assert_eq!(third.architecture_misses, 2);

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn repo_analysis_cache_hits_on_second_load() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-repo-analysis-hit");
        write_repo_fixture(&root);
        let parsed_repo =
            load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("parse should succeed");
        let parsed_arch = load_or_parse_architecture(&root, &[]).expect("parse should succeed");

        reset_cache_stats();
        let _ = load_or_build_repo_analysis_summary(
            &root,
            &[],
            SpecialVersion::V1,
            &parsed_arch,
            &parsed_repo,
        )
        .expect("analysis should succeed");
        let first = snapshot_cache_stats();
        assert_eq!(first.repo_analysis_hits, 0);
        assert_eq!(first.repo_analysis_misses, 1);

        let _ = load_or_build_repo_analysis_summary(
            &root,
            &[],
            SpecialVersion::V1,
            &parsed_arch,
            &parsed_repo,
        )
        .expect("cached analysis should succeed");
        let second = snapshot_cache_stats();
        assert_eq!(second.repo_analysis_hits, 1);
        assert_eq!(second.repo_analysis_misses, 1);

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn architecture_analysis_cache_invalidates_after_file_change() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-arch-analysis-invalidate");
        write_repo_fixture(&root);
        let parsed_repo =
            load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("parse should succeed");
        let parsed_arch = load_or_parse_architecture(&root, &[]).expect("parse should succeed");

        reset_cache_stats();
        let _ = load_or_build_architecture_analysis(
            &root,
            &[],
            SpecialVersion::V1,
            &parsed_arch,
            Some(&parsed_repo),
            ModuleAnalysisOptions {
                coverage: true,
                metrics: true,
                traceability: false,
            },
        )
        .expect("analysis should succeed");
        let first = snapshot_cache_stats();
        assert_eq!(first.architecture_analysis_hits, 0);
        assert_eq!(first.architecture_analysis_misses, 1);

        let _ = load_or_build_architecture_analysis(
            &root,
            &[],
            SpecialVersion::V1,
            &parsed_arch,
            Some(&parsed_repo),
            ModuleAnalysisOptions {
                coverage: true,
                metrics: true,
                traceability: false,
            },
        )
        .expect("cached analysis should succeed");
        let second = snapshot_cache_stats();
        assert_eq!(second.architecture_analysis_hits, 1);
        assert_eq!(second.architecture_analysis_misses, 1);

        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            root.join("app.rs"),
            "/**\n@spec APP.LIVE\nLive behavior.\n*/\n\n// @fileimplements APP.CORE\npub fn changed_impl() {}\n",
        )
        .expect("source should be rewritten");
        let parsed_repo =
            load_or_parse_repo(&root, &[], SpecialVersion::V1).expect("reparse should succeed");
        let parsed_arch = load_or_parse_architecture(&root, &[]).expect("reparse should succeed");

        let _ = load_or_build_architecture_analysis(
            &root,
            &[],
            SpecialVersion::V1,
            &parsed_arch,
            Some(&parsed_repo),
            ModuleAnalysisOptions {
                coverage: true,
                metrics: true,
                traceability: false,
            },
        )
        .expect("recomputed analysis should succeed");
        let third = snapshot_cache_stats();
        assert_eq!(third.architecture_analysis_hits, 1);
        assert_eq!(third.architecture_analysis_misses, 2);

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn parsed_repo_cache_single_flights_under_contention() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-repo-contention");
        write_repo_fixture(&root);
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        let blocker = acquire_cache_fill_lock(&cache_path).expect("lock should be acquired");
        let barrier = Arc::new(Barrier::new(3));

        reset_cache_stats();
        let worker = |barrier: Arc<Barrier>, root: PathBuf| {
            std::thread::spawn(move || {
                barrier.wait();
                load_or_parse_repo(&root, &[], SpecialVersion::V1)
                    .expect("concurrent parse should succeed");
            })
        };

        let first = worker(Arc::clone(&barrier), root.clone());
        let second = worker(Arc::clone(&barrier), root.clone());
        barrier.wait();
        std::thread::sleep(std::time::Duration::from_millis(25));
        drop(blocker);

        first.join().expect("first worker should join");
        second.join().expect("second worker should join");

        let stats = snapshot_cache_stats();
        assert_eq!(stats.repo_misses, 1);
        assert_eq!(stats.repo_hits, 1);
        assert_eq!(stats.lock_waits, 2);

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn cache_wait_emits_real_time_status_note() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-wait-note");
        write_repo_fixture(&root);
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        let blocker = acquire_cache_fill_lock(&cache_path).expect("lock should be acquired");

        let messages = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&messages);
        let waiter = std::thread::spawn(move || {
            let guard = with_cache_status_notifier(
                move |message| {
                    captured
                        .lock()
                        .expect("message mutex should not be poisoned")
                        .push(message.to_string());
                },
                || acquire_cache_fill_lock(&cache_path),
            )
            .expect("waiter should acquire lock after blocker drops");
            drop(guard);
        });
        std::thread::sleep(std::time::Duration::from_millis(25));
        drop(blocker);
        waiter.join().expect("waiter should join");

        let messages = messages
            .lock()
            .expect("message mutex should not be poisoned");
        assert!(messages.iter().any(|message| {
            message.contains("another special run is already building repo annotations")
                && message.contains("waiting to reuse its cached result")
        }));

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn stale_lock_is_recovered_before_filling_cache() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-stale-lock");
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).expect("cache dir should exist");
        }
        let lock_path = cache_lock_path(&cache_path);
        std::fs::write(&lock_path, b"stale").expect("lock file should be written");

        reset_cache_stats();
        let future_now =
            SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
        let guard = acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {})
            .expect("stale lock should be recovered");
        let stats = snapshot_cache_stats();
        assert_eq!(stats.lock_waits, 1);
        assert_eq!(stats.stale_lock_recovers, 1);
        assert!(
            lock_path.exists(),
            "fresh lock should exist while guard is held"
        );
        drop(guard);
        assert!(
            !lock_path.exists(),
            "lock should be removed after guard drops"
        );

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn stale_lock_recovery_emits_real_time_status_note() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-stale-note");
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).expect("cache dir should exist");
        }
        let lock_path = cache_lock_path(&cache_path);
        std::fs::write(&lock_path, b"stale").expect("lock file should be written");

        let messages = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&messages);
        let future_now =
            SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
        let guard = with_cache_status_notifier(
            move |message| {
                captured
                    .lock()
                    .expect("message mutex should not be poisoned")
                    .push(message.to_string());
            },
            || acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {}),
        )
        .expect("stale lock should be recovered");
        drop(guard);

        let messages = messages
            .lock()
            .expect("message mutex should not be poisoned");
        assert!(messages.iter().any(|message| {
            message.contains("recovered an abandoned cache lock for repo annotations")
        }));

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn active_cache_lock_heartbeat_refreshes_lock_contents() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-lock-heartbeat");
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).expect("cache dir should exist");
        }
        let lock_path = cache_lock_path(&cache_path);
        std::fs::write(&lock_path, b"owner=test-owner\nupdated_nanos=0\n")
            .expect("initial lock should be written");

        let heartbeat = start_cache_lock_heartbeat(
            lock_path.clone(),
            "test-owner".to_string(),
            std::time::Duration::from_millis(10),
        );
        std::thread::sleep(std::time::Duration::from_millis(30));
        heartbeat.stop();

        let refreshed = std::fs::read_to_string(&lock_path).expect("lock should remain readable");
        assert!(refreshed.contains("owner=test-owner"));
        assert!(!refreshed.contains("updated_nanos=0"));

        std::fs::remove_file(&lock_path).expect("lock should be removable");
        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn stale_lock_recovery_does_not_let_old_owner_remove_new_lock() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-stale-owner");
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).expect("cache dir should exist");
        }
        let lock_path = cache_lock_path(&cache_path);
        std::fs::write(&lock_path, b"owner=stale\nupdated_nanos=0\n")
            .expect("stale lock should be written");

        let future_now =
            SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
        let stale_guard = acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {})
            .expect("stale lock should be recovered");
        let replacement_owner =
            cache_lock_owner(&lock_path).expect("replacement lock should record an owner");

        let old_guard = CacheFillGuard {
            path: lock_path.clone(),
            owner_token: "stale".to_string(),
            heartbeat: None,
        };
        drop(old_guard);

        assert_eq!(
            cache_lock_owner(&lock_path).as_deref(),
            Some(replacement_owner.as_str()),
            "old owner must not remove a newer lock"
        );

        drop(stale_guard);
        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn stale_lock_recovery_stops_old_heartbeat_from_reclaiming_owner_metadata() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-stale-heartbeat");
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).expect("cache dir should exist");
        }
        let lock_path = cache_lock_path(&cache_path);
        std::fs::write(&lock_path, b"owner=stale\nupdated_nanos=0\n")
            .expect("stale lock should be written");

        let stale_heartbeat = start_cache_lock_heartbeat(
            lock_path.clone(),
            "stale".to_string(),
            std::time::Duration::from_millis(10),
        );
        let future_now =
            SystemTime::now() + CACHE_LOCK_STALE_AFTER + std::time::Duration::from_secs(1);
        let fresh_guard = acquire_cache_fill_lock_with_hooks(&cache_path, || future_now, |_| {})
            .expect("stale lock should be recovered");
        let fresh_owner =
            cache_lock_owner(&lock_path).expect("replacement lock should record an owner");

        std::thread::sleep(std::time::Duration::from_millis(30));
        stale_heartbeat.stop();

        assert_eq!(
            cache_lock_owner(&lock_path).as_deref(),
            Some(fresh_owner.as_str()),
            "old heartbeat must not overwrite replacement ownership"
        );

        drop(fresh_guard);
        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn malformed_repo_cache_is_ignored_and_rebuilt_cleanly() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-repo-malformed");
        write_repo_fixture(&root);
        let cache_path =
            cache_file_path(&root, &format!("parsed-repo-v{CACHE_SCHEMA_VERSION}.json"));
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).expect("cache dir should exist");
        }
        std::fs::write(&cache_path, b"{not valid json").expect("malformed cache should be written");

        reset_cache_stats();
        let parsed = load_or_parse_repo(&root, &[], SpecialVersion::V1)
            .expect("repo should rebuild from malformed cache");
        assert_eq!(parsed.specs.len(), 3);
        let first = snapshot_cache_stats();
        assert_eq!(first.repo_misses, 1);
        assert_eq!(first.repo_hits, 0);

        let _ = load_or_parse_repo(&root, &[], SpecialVersion::V1)
            .expect("rebuilt repo cache should be reusable");
        let second = snapshot_cache_stats();
        assert_eq!(second.repo_misses, 1);
        assert_eq!(second.repo_hits, 1);

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }

    #[test]
    fn parsed_repo_cache_invalidates_on_same_size_source_edit() {
        let _guard = cache_test_lock();
        let root = temp_root("special-cache-repo-same-size-edit");
        write_repo_fixture(&root);
        let source_path = root.join("specs/root.md");

        reset_cache_stats();
        let first = load_or_parse_repo(&root, &[], SpecialVersion::V1)
            .expect("initial parse should succeed");
        assert_eq!(first.specs.len(), 3);
        let first_stats = snapshot_cache_stats();
        assert_eq!(first_stats.repo_hits, 0);
        assert_eq!(first_stats.repo_misses, 1);

        let original_source =
            std::fs::read_to_string(&source_path).expect("original source should be readable");
        let updated_source = original_source
            .replace("APP.LIVE", "APP.CURR")
            .replace("Live behavior.", "Same behavior.");
        assert_eq!(
            original_source.len(),
            updated_source.len(),
            "same-size edit should keep the source length stable"
        );
        std::fs::write(&source_path, &updated_source)
            .expect("same-size source should be rewritten");

        let reparsed = load_or_parse_repo(&root, &[], SpecialVersion::V1)
            .expect("same-size edit should invalidate cache");
        let ids = reparsed
            .specs
            .iter()
            .map(|spec| spec.id.clone())
            .collect::<Vec<_>>();
        assert!(ids.iter().any(|id| id == "APP.CURR"));
        let second_stats = snapshot_cache_stats();
        assert_eq!(second_stats.repo_hits, 0);
        assert_eq!(second_stats.repo_misses, 2);

        std::fs::remove_dir_all(&root).expect("temp root should be removed");
    }
}
