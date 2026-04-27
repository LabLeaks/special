/**
@module SPECIAL.MODULES.ANALYZE.REGISTRY
Projects the shared `SPECIAL.LANGUAGE_PACKS` registry onto implementation analysis so shared analysis flow can build repo contexts, module analysis, and repo traceability without hardcoding one dispatch branch per language in the analysis core.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.REGISTRY
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cache::load_or_build_language_pack_blob;
use crate::language_packs::{self, LanguagePackAnalysisContext, ScopedTraceabilityPreparation};
use crate::model::{
    ArchitectureTraceabilitySummary, ImplementRef, ModuleAnalysisOptions, ParsedArchitecture,
    ParsedRepo,
};
use crate::syntax::SourceLanguage;

use super::{FileOwnership, ProviderModuleAnalysis, status};

pub(super) type RepoAnalysisContexts =
    BTreeMap<SourceLanguage, Box<dyn LanguagePackAnalysisContext>>;

enum TraceabilityPreparation {
    Ready {
        source_files: Vec<PathBuf>,
        graph_facts: Option<Vec<u8>>,
        include_traceability: bool,
    },
    ExplicitlyUnavailable {
        reason: String,
    },
}

const SCOPED_TRACEABILITY_MODE_ENV: &str = "SPECIAL_SCOPED_TRACEABILITY_MODE";
const EAGER_SCOPED_TRACEABILITY_MODE: &str = "eager";

fn language_label(language: SourceLanguage) -> &'static str {
    match language.id() {
        "typescript" => "TypeScript",
        "rust" => "Rust",
        "python" => "Python",
        "go" => "Go",
        _ => language.id(),
    }
}

fn declared_project_tools(descriptor: &language_packs::LanguagePackDescriptor) -> Option<String> {
    let tooling = descriptor.project_tooling?;
    let tools = tooling
        .requirements
        .iter()
        .map(|requirement| {
            if requirement.probe_args.is_empty() {
                requirement.tool.to_string()
            } else {
                format!("{} {}", requirement.tool, requirement.probe_args.join(" "))
            }
        })
        .collect::<Vec<_>>();
    (!tools.is_empty()).then(|| tools.join(", "))
}

// @applies REGISTRY.PROVIDER_DESCRIPTOR
pub(super) fn build_repo_analysis_contexts(
    root: &Path,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    parsed_repo: &ParsedRepo,
    parsed_architecture: &ParsedArchitecture,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> RepoAnalysisContexts {
    let available_languages = languages_in_files(source_files);
    let mut contexts = BTreeMap::new();
    for descriptor in language_packs::descriptors() {
        if !available_languages.contains(&descriptor.language) {
            continue;
        }
        let language_source_files = source_files
            .iter()
            .filter(|path| SourceLanguage::from_path(path) == Some(descriptor.language))
            .cloned()
            .collect::<Vec<_>>();
        let language_scoped_files = scoped_source_files.map(|files| {
            files
                .iter()
                .filter(|path| SourceLanguage::from_path(path) == Some(descriptor.language))
                .cloned()
                .collect::<Vec<_>>()
        });
        let preparation = prepare_traceability_inputs(
            root,
            descriptor,
            &language_source_files,
            language_scoped_files.as_deref(),
            parsed_repo,
            file_ownership,
            include_traceability,
        );
        match preparation {
            TraceabilityPreparation::Ready {
                source_files,
                graph_facts,
                include_traceability,
            } => {
                status::emit_analysis_status(&format!(
                    "building {} analysis context for {} file(s){}{}",
                    descriptor.language.id(),
                    source_files.len(),
                    if include_traceability {
                        " with traceability"
                    } else {
                        ""
                    },
                    declared_project_tools(descriptor)
                        .map(|tools| format!(" using project tools [{tools}]"))
                        .unwrap_or_default()
                ));
                contexts.insert(
                    descriptor.language,
                    (descriptor.build_repo_analysis_context)(
                        root,
                        &source_files,
                        language_scoped_files.as_deref(),
                        graph_facts.as_deref(),
                        parsed_repo,
                        parsed_architecture,
                        file_ownership,
                        include_traceability,
                    ),
                );
            }
            TraceabilityPreparation::ExplicitlyUnavailable { reason } => {
                status::emit_analysis_status(&format!(
                    "building {} analysis context for {} file(s){}",
                    descriptor.language.id(),
                    language_source_files.len(),
                    declared_project_tools(descriptor)
                        .map(|tools| format!(" using project tools [{tools}]"))
                        .unwrap_or_default()
                ));
                status::emit_analysis_status(&format!(
                    "{} traceability preparation is unavailable: {reason}",
                    descriptor.language.id()
                ));
                let inner = (descriptor.build_repo_analysis_context)(
                    root,
                    &language_source_files,
                    language_scoped_files.as_deref(),
                    None,
                    parsed_repo,
                    parsed_architecture,
                    file_ownership,
                    false,
                );
                contexts.insert(
                    descriptor.language,
                    Box::new(ExplicitTraceabilityUnavailableContext { inner, reason }),
                );
            }
        }
    }
    contexts
}

// @applies REGISTRY.PROVIDER_DESCRIPTOR
fn prepare_traceability_inputs(
    root: &Path,
    descriptor: &language_packs::LanguagePackDescriptor,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    include_traceability: bool,
) -> TraceabilityPreparation {
    if !include_traceability {
        return TraceabilityPreparation::Ready {
            source_files: source_files.to_vec(),
            graph_facts: None,
            include_traceability: false,
        };
    }
    if scoped_graph_discovery_enabled(descriptor, scoped_source_files) {
        status::emit_analysis_status(&format!(
            "{} scoped traceability is using scoped graph discovery",
            descriptor.language.id()
        ));
        return TraceabilityPreparation::Ready {
            source_files: source_files.to_vec(),
            graph_facts: None,
            include_traceability: true,
        };
    }
    let (resolved_source_files, scoped_graph_facts) = match resolve_traceability_source_files(
        root,
        descriptor,
        source_files,
        scoped_source_files,
        parsed_repo,
        file_ownership,
    ) {
        Ok(files) => files,
        Err(error) => {
            return TraceabilityPreparation::ExplicitlyUnavailable {
                reason: format!(
                    "{} backward trace is unavailable because scoped traceability preparation failed: {error}",
                    language_label(descriptor.language)
                ),
            };
        }
    };
    let graph_facts = if scoped_graph_facts.is_some() {
        scoped_graph_facts
    } else {
        match resolve_traceability_graph_facts(root, descriptor, &resolved_source_files, true) {
            Ok(facts) => facts,
            Err(error) => {
                return TraceabilityPreparation::ExplicitlyUnavailable {
                    reason: format!(
                        "{} backward trace is unavailable because traceability graph fact preparation failed: {error}",
                        language_label(descriptor.language)
                    ),
                };
            }
        }
    };

    TraceabilityPreparation::Ready {
        source_files: resolved_source_files,
        graph_facts,
        include_traceability: true,
    }
}

fn scoped_graph_discovery_enabled(
    descriptor: &language_packs::LanguagePackDescriptor,
    scoped_source_files: Option<&[PathBuf]>,
) -> bool {
    descriptor.scoped_traceability_preparation
        == ScopedTraceabilityPreparation::ScopedGraphDiscovery
        && scoped_source_files.is_some_and(|files| !files.is_empty())
        && !eager_scoped_traceability_requested()
}

fn eager_scoped_traceability_requested() -> bool {
    env::var(SCOPED_TRACEABILITY_MODE_ENV).as_deref() == Ok(EAGER_SCOPED_TRACEABILITY_MODE)
}

fn resolve_traceability_source_files(
    root: &Path,
    descriptor: &language_packs::LanguagePackDescriptor,
    source_files: &[PathBuf],
    scoped_source_files: Option<&[PathBuf]>,
    parsed_repo: &ParsedRepo,
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
) -> Result<(Vec<PathBuf>, Option<Vec<u8>>)> {
    let Some(scoped_source_files) = scoped_source_files else {
        return Ok((source_files.to_vec(), None));
    };
    if scoped_source_files.is_empty() {
        return Ok((source_files.to_vec(), None));
    }
    let Some(scope_facts) = descriptor.traceability_scope_facts else {
        return Ok((source_files.to_vec(), None));
    };

    let environment_fingerprint = format!(
        "{}|repo={:016x}|scope={:016x}",
        (descriptor.analysis_environment_fingerprint)(root),
        crate::cache::parsed_repo_contract_fingerprint(parsed_repo),
        scope_fingerprint(scoped_source_files),
    );
    let facts = load_or_build_language_pack_blob(
        root,
        "scope-facts",
        descriptor.language.id(),
        source_files,
        &environment_fingerprint,
        || {
            (scope_facts.build_facts)(
                root,
                source_files,
                scoped_source_files,
                parsed_repo,
                file_ownership,
            )
        },
    )?;
    let resolved =
        (scope_facts.expand_closure)(source_files, scoped_source_files, file_ownership, &facts)?;
    Ok((resolved, Some(facts)))
}

fn scope_fingerprint(scoped_source_files: &[PathBuf]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for path in scoped_source_files {
        path.hash(&mut hasher);
    }
    hasher.finish()
}

fn resolve_traceability_graph_facts(
    root: &Path,
    descriptor: &language_packs::LanguagePackDescriptor,
    source_files: &[PathBuf],
    include_traceability: bool,
) -> Result<Option<Vec<u8>>> {
    if !include_traceability {
        return Ok(None);
    }
    let Some(graph_facts) = descriptor.traceability_graph_facts else {
        return Ok(None);
    };

    let environment_fingerprint = (descriptor.analysis_environment_fingerprint)(root);
    let facts = load_or_build_language_pack_blob(
        root,
        "traceability-graph-facts",
        descriptor.language.id(),
        source_files,
        &environment_fingerprint,
        || (graph_facts.build_facts)(root, source_files),
    )?;
    Ok(Some(facts))
}

pub(super) fn languages_in_files(files: &[PathBuf]) -> BTreeSet<SourceLanguage> {
    files
        .iter()
        .filter_map(|path| SourceLanguage::from_path(path))
        .collect()
}

pub(super) fn analysis_environment_fingerprint(root: &Path, files: &[PathBuf]) -> String {
    let languages = languages_in_files(files);
    let mut parts = Vec::new();
    for descriptor in language_packs::descriptors() {
        if !languages.contains(&descriptor.language) {
            continue;
        }
        parts.push(format!(
            "{}={}",
            descriptor.language.id(),
            (descriptor.analysis_environment_fingerprint)(root)
        ));
    }
    parts.join("|")
}

pub(super) fn summarize_repo_traceability(
    language: SourceLanguage,
    root: &Path,
    contexts: &RepoAnalysisContexts,
) -> Option<ArchitectureTraceabilitySummary> {
    status::emit_analysis_status(&format!("summarizing {} repo traceability", language.id()));
    contexts.get(&language)?.summarize_repo_traceability(root)
}

pub(super) fn traceability_unavailable_reason(
    language: SourceLanguage,
    contexts: &RepoAnalysisContexts,
) -> Option<String> {
    contexts.get(&language)?.traceability_unavailable_reason()
}

pub(super) fn analyze_module_language(
    language: SourceLanguage,
    root: &Path,
    implementations: &[&ImplementRef],
    file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
    contexts: &RepoAnalysisContexts,
    options: ModuleAnalysisOptions,
) -> Result<ProviderModuleAnalysis> {
    let context = contexts.get(&language).ok_or_else(|| {
        anyhow::anyhow!(
            "metrics analysis expected a {} repo context but none was prepared",
            language.id()
        )
    })?;
    context.analyze_module(root, implementations, file_ownership, options)
}

struct ExplicitTraceabilityUnavailableContext {
    inner: Box<dyn LanguagePackAnalysisContext>,
    reason: String,
}

impl LanguagePackAnalysisContext for ExplicitTraceabilityUnavailableContext {
    fn summarize_repo_traceability(&self, _root: &Path) -> Option<ArchitectureTraceabilitySummary> {
        None
    }

    fn traceability_unavailable_reason(&self) -> Option<String> {
        Some(self.reason.clone())
    }

    fn analyze_module(
        &self,
        root: &Path,
        implementations: &[&ImplementRef],
        file_ownership: &BTreeMap<PathBuf, FileOwnership<'_>>,
        options: ModuleAnalysisOptions,
    ) -> Result<ProviderModuleAnalysis> {
        let mut analysis =
            self.inner
                .analyze_module(root, implementations, file_ownership, options)?;
        if options.traceability {
            analysis.traceability = None;
            analysis.traceability_unavailable_reason = Some(self.reason.clone());
        }
        Ok(analysis)
    }
}
