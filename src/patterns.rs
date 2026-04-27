/**
@module SPECIAL.PATTERNS
Builds pattern-centered views from pattern definitions and source-attached pattern applications.
*/
// @fileimplements SPECIAL.PATTERNS
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cache::load_or_parse_architecture;
use crate::config::PatternMetricBenchmarks;
use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::id_path::nearest_visible_parent_id;
use crate::model::{
    ImplementRef, LintReport, ParsedArchitecture, PatternApplication, PatternApplicationNode,
    PatternBenchmarkEstimate, PatternCandidateConfidence, PatternClusterCandidate,
    PatternClusterInterpretation, PatternClusterItem, PatternDocument, PatternFilter,
    PatternMetricsSummary, PatternMissingApplicationCandidate, PatternModuleRef, PatternNode,
    PatternSimilarityMetrics, PatternStrictness, SourceLocation,
};
use crate::modules;
use crate::source_paths::looks_like_test_path;
use crate::syntax::{SourceInvocationKind, SourceItem, parse_source_graph};

pub fn build_pattern_document(
    root: &Path,
    ignore_patterns: &[String],
    filter: PatternFilter,
    benchmark_config: PatternMetricBenchmarks,
) -> Result<(PatternDocument, LintReport)> {
    let parsed = load_or_parse_architecture(root, ignore_patterns)?;
    let lint = modules::build_module_lint_report_from_parsed(&parsed);
    let document = materialize_patterns(root, ignore_patterns, &parsed, &filter, benchmark_config)?;
    Ok((document, lint))
}

fn materialize_patterns(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    filter: &PatternFilter,
    benchmark_config: PatternMetricBenchmarks,
) -> Result<PatternDocument> {
    let modules_by_id = parsed
        .modules
        .iter()
        .map(|module| (module.id.as_str(), module))
        .collect::<BTreeMap<_, _>>();
    let ids = materialized_pattern_ids(parsed);
    let directly_visible_ids = if let Some(scope) = filter.scope.as_deref() {
        ids.iter()
            .filter(|id| id.as_str() == scope || id.starts_with(&format!("{scope}.")))
            .cloned()
            .collect::<BTreeSet<_>>()
    } else {
        ids
    };
    let mut visible_ids = directly_visible_ids.clone();
    for id in &directly_visible_ids {
        for parent in declared_parents(id, parsed) {
            visible_ids.insert(parent);
        }
    }

    let mut children_map: BTreeMap<Option<String>, Vec<String>> = BTreeMap::new();
    for id in &visible_ids {
        children_map
            .entry(nearest_visible_parent_id(id, &visible_ids))
            .or_default()
            .push(id.clone());
    }
    for children in children_map.values_mut() {
        children.sort();
    }

    let patterns = build_pattern_children(
        None,
        &children_map,
        parsed,
        &modules_by_id,
        filter.metrics,
        benchmark_config,
    );
    let patterns = if let Some(scope) = filter.scope.as_deref() {
        scoped_patterns(patterns, scope)
    } else {
        patterns
    };

    Ok(PatternDocument {
        metrics: if filter.metrics {
            Some(pattern_metrics(
                root,
                ignore_patterns,
                parsed,
                filter,
                benchmark_config,
            )?)
        } else {
            None
        },
        scoped: filter.scope.is_some(),
        patterns,
    })
}

fn materialized_pattern_ids(parsed: &ParsedArchitecture) -> BTreeSet<String> {
    let mut ids = parsed
        .patterns
        .iter()
        .map(|definition| definition.pattern_id.clone())
        .collect::<BTreeSet<_>>();
    ids.extend(
        parsed
            .pattern_applications
            .iter()
            .map(|application| application.pattern_id.clone()),
    );
    ids
}

fn build_pattern_children(
    parent: Option<String>,
    children_map: &BTreeMap<Option<String>, Vec<String>>,
    parsed: &ParsedArchitecture,
    modules_by_id: &BTreeMap<&str, &crate::model::ModuleDecl>,
    include_metrics: bool,
    benchmark_config: PatternMetricBenchmarks,
) -> Vec<PatternNode> {
    let Some(ids) = children_map.get(&parent) else {
        return Vec::new();
    };

    ids.iter()
        .map(|id| PatternNode {
            id: id.clone(),
            definition: parsed
                .patterns
                .iter()
                .find(|definition| definition.pattern_id == *id)
                .cloned(),
            metrics: include_metrics
                .then(|| pattern_similarity_metrics(parsed, id, benchmark_config)),
            applications: applications_for_pattern(parsed, id),
            modules: modules_for_pattern(parsed, id, modules_by_id),
            children: build_pattern_children(
                Some(id.clone()),
                children_map,
                parsed,
                modules_by_id,
                include_metrics,
                benchmark_config,
            ),
        })
        .collect()
}

fn declared_parents(id: &str, parsed: &ParsedArchitecture) -> Vec<String> {
    let declared = parsed
        .patterns
        .iter()
        .map(|definition| definition.pattern_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut parents = Vec::new();
    let mut parent = crate::id_path::immediate_parent_id(id);
    while let Some(candidate) = parent {
        if declared.contains(candidate) {
            parents.push(candidate.to_string());
        }
        parent = crate::id_path::immediate_parent_id(candidate);
    }
    parents
}

fn scoped_patterns(nodes: Vec<PatternNode>, scope: &str) -> Vec<PatternNode> {
    for node in nodes {
        if node.id == scope {
            return vec![node];
        }
        let scoped_children = scoped_patterns(node.children.clone(), scope);
        if !scoped_children.is_empty() {
            return scoped_children;
        }
    }
    Vec::new()
}

fn applications_for_pattern(
    parsed: &ParsedArchitecture,
    pattern_id: &str,
) -> Vec<PatternApplicationNode> {
    parsed
        .pattern_applications
        .iter()
        .filter(|application| application.pattern_id == pattern_id)
        .map(|application| PatternApplicationNode {
            module_id: module_id_for_application(parsed, application),
            location: application.location.clone(),
            body_location: application.body_location.clone(),
            body: application.body.clone(),
        })
        .collect()
}

fn modules_for_pattern(
    parsed: &ParsedArchitecture,
    pattern_id: &str,
    modules_by_id: &BTreeMap<&str, &crate::model::ModuleDecl>,
) -> Vec<PatternModuleRef> {
    let mut modules = BTreeMap::<String, PatternModuleRef>::new();
    for application in parsed
        .pattern_applications
        .iter()
        .filter(|application| application.pattern_id == pattern_id)
    {
        for implementation in &parsed.implements {
            if implementation_contains_application(implementation, application)
                && let Some(module) = modules_by_id.get(implementation.module_id.as_str())
            {
                modules
                    .entry(module.id.clone())
                    .or_insert(PatternModuleRef {
                        id: module.id.clone(),
                        location: application.location.clone(),
                    });
            }
        }
    }
    modules.into_values().collect()
}

fn module_id_for_application(
    parsed: &ParsedArchitecture,
    application: &PatternApplication,
) -> Option<String> {
    parsed
        .implements
        .iter()
        .find(|implementation| implementation_contains_application(implementation, application))
        .map(|implementation| implementation.module_id.clone())
}

pub(crate) fn implementation_contains_application(
    implementation: &ImplementRef,
    application: &PatternApplication,
) -> bool {
    if let (Some(implementation_body), Some(application_body)) =
        (&implementation.body_location, &application.body_location)
        && implementation_body == application_body
    {
        return true;
    }
    implementation.body_location.is_none()
        && implementation.location.path == application.location.path
}

fn pattern_metrics(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    filter: &PatternFilter,
    benchmark_config: PatternMetricBenchmarks,
) -> Result<PatternMetricsSummary> {
    let pattern_ids = parsed
        .patterns
        .iter()
        .map(|definition| definition.pattern_id.as_str())
        .collect::<BTreeSet<_>>();
    let modules_with_applications = parsed
        .pattern_applications
        .iter()
        .flat_map(|application| {
            parsed
                .implements
                .iter()
                .filter(move |implementation| {
                    implementation_contains_application(implementation, application)
                })
                .map(|implementation| implementation.module_id.as_str())
        })
        .collect::<BTreeSet<_>>()
        .len();
    let candidates =
        pattern_metric_candidates(root, ignore_patterns, parsed, filter, benchmark_config)?;

    Ok(PatternMetricsSummary {
        total_patterns: pattern_ids.len(),
        total_definitions: parsed.patterns.len(),
        total_applications: parsed.pattern_applications.len(),
        modules_with_applications,
        possible_missing_applications: candidates.possible_missing_applications,
        possible_pattern_clusters: candidates.possible_pattern_clusters,
    })
}

struct PatternCandidateReport {
    possible_missing_applications: Vec<PatternMissingApplicationCandidate>,
    possible_pattern_clusters: Vec<PatternClusterCandidate>,
}

fn pattern_metric_candidates(
    root: &Path,
    ignore_patterns: &[String],
    parsed: &ParsedArchitecture,
    filter: &PatternFilter,
    benchmark_config: PatternMetricBenchmarks,
) -> Result<PatternCandidateReport> {
    let discovered = discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?;
    let comparison_files = discovered
        .source_files
        .iter()
        .filter(|path| path_matches_scope(path, &filter.comparison_paths))
        .cloned()
        .collect::<Vec<_>>();
    let target_files = comparison_files
        .iter()
        .filter(|path| path_matches_scope(path, &filter.target_paths))
        .cloned()
        .collect::<Vec<_>>();
    let comparison_items = collect_source_feature_items(root, &comparison_files)?;
    let target_paths = target_files.iter().collect::<BTreeSet<_>>();
    let target_items = comparison_items
        .iter()
        .filter(|item| target_paths.contains(&item.location.path))
        .filter(|item| {
            filter
                .symbol
                .as_deref()
                .is_none_or(|symbol| item.item_name == symbol)
        })
        .cloned()
        .collect::<Vec<_>>();
    let annotated = annotated_application_locations(parsed);

    let possible_missing_applications = possible_missing_applications(
        parsed,
        &target_items,
        &annotated,
        &filter.comparison_paths,
        benchmark_config,
    );
    let possible_pattern_clusters =
        possible_pattern_clusters(&comparison_items, &target_items, &annotated);

    Ok(PatternCandidateReport {
        possible_missing_applications,
        possible_pattern_clusters,
    })
}

fn path_matches_scope(path: &Path, scopes: &[PathBuf]) -> bool {
    scopes.is_empty()
        || scopes
            .iter()
            .any(|scope| path == scope || path.starts_with(scope))
}

#[derive(Debug, Clone)]
struct SourceFeatureItem {
    item_name: String,
    location: SourceLocation,
    features: PatternApplicationFeatures,
}

fn collect_source_feature_items(root: &Path, files: &[PathBuf]) -> Result<Vec<SourceFeatureItem>> {
    let mut items = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(path)?;
        let Some(graph) = parse_source_graph(path, &text) else {
            continue;
        };
        for item in graph.items {
            if item.is_test || looks_like_test_path(&item.source_path) {
                continue;
            }
            let location = SourceLocation {
                path: root.join(&item.source_path),
                line: item.span.start_line,
            };
            items.push(SourceFeatureItem {
                item_name: item.name.clone(),
                location,
                features: source_item_features(&item),
            });
        }
    }
    Ok(items)
}

#[derive(Debug, Default)]
struct AnnotatedApplicationLocations {
    by_pattern: BTreeMap<String, BTreeSet<(PathBuf, usize)>>,
    any_item: BTreeSet<(PathBuf, usize)>,
    file_scoped_paths: BTreeSet<PathBuf>,
}

fn annotated_application_locations(parsed: &ParsedArchitecture) -> AnnotatedApplicationLocations {
    let mut annotated = AnnotatedApplicationLocations::default();
    for application in &parsed.pattern_applications {
        if let Some(body_location) = &application.body_location {
            let key = (body_location.path.clone(), body_location.line);
            annotated
                .by_pattern
                .entry(application.pattern_id.clone())
                .or_default()
                .insert(key.clone());
            annotated.any_item.insert(key);
        } else {
            annotated
                .file_scoped_paths
                .insert(application.location.path.clone());
        }
    }
    annotated
}

fn possible_missing_applications(
    parsed: &ParsedArchitecture,
    target_items: &[SourceFeatureItem],
    annotated: &AnnotatedApplicationLocations,
    comparison_paths: &[PathBuf],
    benchmark_config: PatternMetricBenchmarks,
) -> Vec<PatternMissingApplicationCandidate> {
    let mut candidates = Vec::new();
    for definition in &parsed.patterns {
        let applications = parsed
            .pattern_applications
            .iter()
            .filter(|application| application.pattern_id == definition.pattern_id)
            .filter(|application| path_matches_scope(&application.location.path, comparison_paths))
            .filter_map(pattern_application_features)
            .collect::<Vec<_>>();
        let Some(profile) = PatternProfile::from_applications(
            &definition.pattern_id,
            definition.strictness,
            &applications,
            benchmark_config,
        ) else {
            continue;
        };
        for item in target_items {
            let location_key = (item.location.path.clone(), item.location.line);
            if annotated
                .by_pattern
                .get(&definition.pattern_id)
                .is_some_and(|locations| locations.contains(&location_key))
                || annotated.file_scoped_paths.contains(&item.location.path)
            {
                continue;
            }
            if let Some(candidate) = profile.score_item(item) {
                candidates.push(candidate);
            }
        }
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.pattern_id.cmp(&right.pattern_id))
            .then_with(|| left.location.path.cmp(&right.location.path))
            .then_with(|| left.location.line.cmp(&right.location.line))
    });
    candidates.truncate(20);
    candidates
}

struct PatternProfile {
    pattern_id: String,
    strictness: PatternStrictness,
    required_terms: BTreeSet<String>,
    optional_terms: BTreeSet<String>,
    discriminator_terms: BTreeSet<String>,
    expected_similarity: f64,
}

impl PatternProfile {
    fn from_applications(
        pattern_id: &str,
        strictness: PatternStrictness,
        applications: &[PatternApplicationFeatures],
        benchmark_config: PatternMetricBenchmarks,
    ) -> Option<Self> {
        if applications.len() < 2 {
            return None;
        }
        let mut counts = BTreeMap::<String, usize>::new();
        for application in applications {
            for term in application.match_terms() {
                *counts.entry(term).or_default() += 1;
            }
        }
        let required_count = match strictness {
            PatternStrictness::High => ((applications.len() as f64) * 0.67).ceil() as usize,
            PatternStrictness::Medium => ((applications.len() as f64) * 0.50).ceil() as usize,
            PatternStrictness::Low => ((applications.len() as f64) * 0.67).ceil() as usize,
        };
        let required_terms = counts
            .iter()
            .filter(|(term, count)| **count >= required_count && !is_noise_term(term))
            .map(|(term, _)| term.clone())
            .collect::<BTreeSet<_>>();
        let optional_terms = counts
            .iter()
            .filter(|(term, _)| !is_noise_term(term))
            .map(|(term, _)| term.clone())
            .collect::<BTreeSet<_>>();
        let discriminator_terms = optional_terms
            .iter()
            .filter(|term| is_discriminator_term(term))
            .cloned()
            .collect::<BTreeSet<_>>();
        if required_terms.is_empty() || discriminator_terms.is_empty() {
            return None;
        }
        Some(Self {
            pattern_id: pattern_id.to_string(),
            strictness,
            required_terms,
            optional_terms,
            discriminator_terms,
            expected_similarity: expected_similarity(strictness, benchmark_config),
        })
    }

    fn score_item(&self, item: &SourceFeatureItem) -> Option<PatternMissingApplicationCandidate> {
        let item_terms = item.features.match_terms();
        let matched_required = self
            .required_terms
            .intersection(&item_terms)
            .cloned()
            .collect::<BTreeSet<_>>();
        let matched_discriminators = self
            .discriminator_terms
            .intersection(&item_terms)
            .cloned()
            .collect::<BTreeSet<_>>();
        let required_recall = matched_required.len() as f64 / self.required_terms.len() as f64;
        let optional_similarity = jaccard(&self.optional_terms, &item_terms);
        let discriminator_recall =
            matched_discriminators.len() as f64 / self.discriminator_terms.len().max(1) as f64;
        let score = round_similarity(
            0.58 * required_recall + 0.27 * optional_similarity + 0.15 * discriminator_recall,
        );
        let passes = match self.strictness {
            PatternStrictness::High => {
                required_recall >= 0.70
                    && !matched_discriminators.is_empty()
                    && score >= self.expected_similarity.max(0.58)
            }
            PatternStrictness::Medium => {
                required_recall >= 0.50
                    && !matched_discriminators.is_empty()
                    && score >= self.expected_similarity.max(0.48)
            }
            PatternStrictness::Low => {
                matched_discriminators.len() >= 2
                    && discriminator_recall >= 0.50
                    && score >= self.expected_similarity.max(0.36)
            }
        };
        if !passes {
            return None;
        }
        let confidence = match self.strictness {
            PatternStrictness::High if score >= 0.70 => PatternCandidateConfidence::Probable,
            _ => PatternCandidateConfidence::Possible,
        };
        Some(PatternMissingApplicationCandidate {
            pattern_id: self.pattern_id.clone(),
            strictness: self.strictness,
            confidence,
            score,
            item_name: item.item_name.clone(),
            location: item.location.clone(),
            matched_terms: matched_required.into_iter().take(12).collect(),
            missing_terms: self
                .required_terms
                .difference(&item_terms)
                .take(12)
                .cloned()
                .collect(),
        })
    }
}

fn possible_pattern_clusters(
    comparison_items: &[SourceFeatureItem],
    target_items: &[SourceFeatureItem],
    annotated: &AnnotatedApplicationLocations,
) -> Vec<PatternClusterCandidate> {
    let target_locations = target_items
        .iter()
        .map(|item| (item.location.path.clone(), item.location.line))
        .collect::<BTreeSet<_>>();
    let mut groups = BTreeMap::<String, Vec<&SourceFeatureItem>>::new();
    for item in comparison_items {
        let location_key = (item.location.path.clone(), item.location.line);
        if annotated.any_item.contains(&location_key)
            || annotated.file_scoped_paths.contains(&item.location.path)
            || item.features.size < 12
        {
            continue;
        }
        let Some(signature) = cluster_signature(item) else {
            continue;
        };
        groups.entry(signature).or_default().push(item);
    }

    let mut clusters = Vec::new();
    for items in groups.values() {
        if items.len() < 2
            || !items.iter().any(|item| {
                target_locations.contains(&(item.location.path.clone(), item.location.line))
            })
        {
            continue;
        }
        let features = items
            .iter()
            .map(|item| item.features.clone())
            .collect::<Vec<_>>();
        let pair_scores = pairwise_similarities(&features);
        let score = if pair_scores.is_empty() {
            0.0
        } else {
            round_similarity(pair_scores.iter().sum::<f64>() / pair_scores.len() as f64)
        };
        let suggested_strictness = if score >= 0.62 {
            PatternStrictness::High
        } else if score >= 0.36 {
            PatternStrictness::Medium
        } else {
            PatternStrictness::Low
        };
        let shared_cluster_terms = shared_cluster_terms(items);
        if is_thin_delegate_cluster(items, &shared_cluster_terms) {
            continue;
        }
        let shared_terms = shared_cluster_terms
            .into_iter()
            .take(12)
            .collect::<Vec<_>>();
        let interpretation = interpret_pattern_cluster(score, &shared_terms, items);
        clusters.push(PatternClusterCandidate {
            score,
            suggested_strictness,
            interpretation,
            meaning: interpretation.meaning(),
            precise: interpretation.precise(),
            item_count: items.len(),
            shared_terms,
            items: items
                .iter()
                .take(8)
                .map(|item| PatternClusterItem {
                    item_name: item.item_name.clone(),
                    location: item.location.clone(),
                })
                .collect(),
        });
    }
    clusters.sort_by(|left, right| {
        right
            .item_count
            .cmp(&left.item_count)
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                left.items[0]
                    .location
                    .path
                    .cmp(&right.items[0].location.path)
            })
    });
    clusters.truncate(20);
    clusters
}

fn interpret_pattern_cluster(
    score: f64,
    shared_terms: &[String],
    items: &[&SourceFeatureItem],
) -> PatternClusterInterpretation {
    let item_count = items.len();
    let same_item_name = items
        .first()
        .is_some_and(|first| items.iter().all(|item| item.item_name == first.item_name));
    let few_shared_terms = (item_count <= 4 && shared_terms.len() <= 5)
        || (item_count >= 5 && shared_terms.len() <= 4);
    if score >= 0.86 || (score >= 0.82 && (same_item_name || few_shared_terms)) {
        PatternClusterInterpretation::ExtractionCandidate
    } else {
        PatternClusterInterpretation::PossiblePattern
    }
}

fn is_thin_delegate_cluster(items: &[&SourceFeatureItem], shared_terms: &BTreeSet<String>) -> bool {
    let has_control_or_flow = shared_terms.iter().any(|term| {
        term.starts_with("control:") || term.starts_with("flow:") || term == "shape:closure"
    });
    if has_control_or_flow {
        return false;
    }

    let shared_calls = shared_terms
        .iter()
        .filter(|term| term.starts_with("call:"))
        .count();
    let substantial_shared_calls = shared_terms
        .iter()
        .filter(|term| is_unqualified_call_term(term))
        .filter(|term| !is_accessor_or_clone_call(term))
        .count();
    shared_calls <= 4
        && substantial_shared_calls == 1
        && items
            .iter()
            .all(|item| item.features.size <= 30 && item.features.invocations.is_empty())
}

fn is_unqualified_call_term(term: &str) -> bool {
    term.strip_prefix("call:")
        .is_some_and(|call| !call.contains("::"))
}

fn is_accessor_or_clone_call(term: &str) -> bool {
    term.contains("::as_")
        || term.contains("::clone")
        || term.contains("::to_string")
        || term.contains("::display")
}

fn cluster_signature(item: &SourceFeatureItem) -> Option<String> {
    let mut terms = item
        .features
        .match_terms()
        .into_iter()
        .filter(|term| {
            !is_noise_term(term) && !term.starts_with("size:") && !term.starts_with("name:")
        })
        .collect::<Vec<_>>();
    if terms.len() < 2
        || terms
            .iter()
            .filter(|term| is_discriminator_term(term))
            .count()
            < 2
    {
        return None;
    }
    terms.sort();
    terms.truncate(10);
    Some(terms.join("|"))
}

fn shared_cluster_terms(items: &[&SourceFeatureItem]) -> BTreeSet<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for item in items {
        for term in item.features.match_terms() {
            if !is_noise_term(&term) {
                *counts.entry(term).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count == items.len())
        .map(|(term, _)| term)
        .collect()
}

fn pattern_similarity_metrics(
    parsed: &ParsedArchitecture,
    pattern_id: &str,
    benchmark_config: PatternMetricBenchmarks,
) -> PatternSimilarityMetrics {
    let strictness = parsed
        .patterns
        .iter()
        .find(|definition| definition.pattern_id == pattern_id)
        .map(|definition| definition.strictness)
        .unwrap_or_default();
    let features = parsed
        .pattern_applications
        .iter()
        .filter(|application| application.pattern_id == pattern_id)
        .filter_map(pattern_application_features)
        .collect::<Vec<_>>();
    let pair_scores = pairwise_similarities(&features);
    let expected_similarity = expected_similarity(strictness, benchmark_config);
    let (mean_similarity, min_similarity, max_similarity, benchmark_estimate) =
        if pair_scores.is_empty() {
            (None, None, None, None)
        } else {
            let mean = round_similarity(pair_scores.iter().sum::<f64>() / pair_scores.len() as f64);
            let min = round_similarity(pair_scores.iter().copied().fold(f64::INFINITY, f64::min));
            let max = round_similarity(
                pair_scores
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max),
            );
            (
                Some(mean),
                Some(min),
                Some(max),
                Some(benchmark_estimate(
                    strictness,
                    mean,
                    min,
                    max,
                    benchmark_config,
                )),
            )
        };

    PatternSimilarityMetrics {
        strictness,
        scored_applications: features.len(),
        pair_count: pair_scores.len(),
        mean_similarity,
        min_similarity,
        max_similarity,
        expected_similarity: Some(expected_similarity),
        benchmark_estimate,
    }
}

#[derive(Debug, Clone)]
struct PatternApplicationFeatures {
    shape_terms: BTreeSet<String>,
    marker_terms: BTreeSet<String>,
    name_terms: BTreeSet<String>,
    calls: BTreeSet<String>,
    invocations: BTreeSet<String>,
    size: usize,
}

fn pattern_application_features(
    application: &PatternApplication,
) -> Option<PatternApplicationFeatures> {
    let body = application.body.as_ref()?;
    let path = application
        .body_location
        .as_ref()
        .map(|location| location.path.as_path())
        .unwrap_or_else(|| application.location.path.as_path());
    let graph = parse_source_graph(path, body)?;
    let item = graph
        .items
        .iter()
        .max_by_key(|item| item.shape_node_count)?;
    Some(source_item_features(item))
}

fn source_item_features(item: &SourceItem) -> PatternApplicationFeatures {
    PatternApplicationFeatures {
        shape_terms: shape_terms(&item.shape_fingerprint),
        marker_terms: marker_terms(&item.shape_fingerprint, item.size_bucket()),
        name_terms: name_terms(&item.name),
        calls: item
            .calls
            .iter()
            .map(|call| match &call.qualifier {
                Some(qualifier) => format!("{qualifier}::{}", call.name),
                None => call.name.clone(),
            })
            .collect(),
        invocations: item
            .invocations
            .iter()
            .map(|invocation| match &invocation.kind {
                SourceInvocationKind::LocalCargoBinary { binary_name } => {
                    format!("cargo-bin:{binary_name}")
                }
            })
            .collect(),
        size: item.shape_node_count,
    }
}

trait SourceItemSizeBucket {
    fn size_bucket(&self) -> &'static str;
}

impl SourceItemSizeBucket for SourceItem {
    fn size_bucket(&self) -> &'static str {
        match self.shape_node_count {
            0..=24 => "small",
            25..=80 => "medium",
            _ => "large",
        }
    }
}

impl PatternApplicationFeatures {
    fn match_terms(&self) -> BTreeSet<String> {
        let mut terms = self.marker_terms.clone();
        terms.extend(self.name_terms.iter().map(|term| format!("name:{term}")));
        terms.extend(self.calls.iter().map(|call| format!("call:{call}")));
        terms.extend(
            self.invocations
                .iter()
                .map(|invocation| format!("invoke:{invocation}")),
        );
        terms
    }
}

fn shape_terms(shape_fingerprint: &str) -> BTreeSet<String> {
    let nodes = shape_fingerprint.split('>').collect::<Vec<_>>();
    let mut terms = nodes
        .iter()
        .map(|node| format!("node:{node}"))
        .collect::<BTreeSet<_>>();
    for pair in nodes.windows(2) {
        terms.insert(format!("edge:{}>{}", pair[0], pair[1]));
    }
    terms
}

fn marker_terms(shape_fingerprint: &str, size_bucket: &str) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    terms.insert(format!("size:{size_bucket}"));
    add_marker(
        &mut terms,
        shape_fingerprint,
        &["if_expression", "if_statement"],
        "control:if",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &["match_expression", "switch_statement", "conditional_type"],
        "control:branch",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &[
            "for_expression",
            "while_expression",
            "loop_expression",
            "for_statement",
        ],
        "control:loop",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &[
            "let_declaration",
            "lexical_declaration",
            "variable_declarator",
            "short_var_declaration",
        ],
        "data:binding",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &[
            "assignment_expression",
            "augmented_assignment_expression",
            "field_initializer",
        ],
        "data:assignment",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &[
            "closure_expression",
            "arrow_function",
            "func_literal",
            "anonymous_function",
        ],
        "shape:closure",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &["call_expression"],
        "shape:call",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &[
            "field_expression",
            "member_expression",
            "selector_expression",
        ],
        "shape:member",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &["return_expression", "return_statement"],
        "flow:return",
    );
    add_marker(
        &mut terms,
        shape_fingerprint,
        &["try_expression", "await_expression"],
        "flow:defer_or_try",
    );
    terms
}

fn add_marker(terms: &mut BTreeSet<String>, shape_fingerprint: &str, markers: &[&str], term: &str) {
    if markers
        .iter()
        .any(|marker| shape_fingerprint.contains(marker))
    {
        terms.insert(term.to_string());
    }
}

fn name_terms(name: &str) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for token in name
        .split(|character: char| !character.is_ascii_alphanumeric())
        .flat_map(split_camel_case)
    {
        let token = token.to_ascii_lowercase();
        if token.len() >= 3 {
            terms.insert(token);
        }
    }
    terms
}

fn split_camel_case(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in value.chars() {
        if character.is_ascii_uppercase() && !current.is_empty() {
            tokens.push(current);
            current = String::new();
        }
        current.push(character);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn is_noise_term(term: &str) -> bool {
    matches!(
        term,
        "shape:call" | "shape:member" | "data:binding" | "size:small" | "size:medium"
    )
}

fn is_discriminator_term(term: &str) -> bool {
    term.starts_with("call:")
        || term.starts_with("invoke:")
        || term.starts_with("name:")
        || matches!(
            term,
            "control:branch" | "control:loop" | "shape:closure" | "flow:defer_or_try"
        )
}

fn pairwise_similarities(features: &[PatternApplicationFeatures]) -> Vec<f64> {
    let mut scores = Vec::new();
    for left_index in 0..features.len() {
        for right_index in (left_index + 1)..features.len() {
            scores.push(pattern_similarity(
                &features[left_index],
                &features[right_index],
            ));
        }
    }
    scores
}

fn pattern_similarity(
    left: &PatternApplicationFeatures,
    right: &PatternApplicationFeatures,
) -> f64 {
    let size_ratio = if left.size == 0 || right.size == 0 {
        0.0
    } else {
        left.size.min(right.size) as f64 / left.size.max(right.size) as f64
    };
    0.52 * jaccard(&left.shape_terms, &right.shape_terms)
        + 0.34 * jaccard(&left.calls, &right.calls)
        + 0.08 * jaccard(&left.invocations, &right.invocations)
        + 0.06 * size_ratio
}

fn jaccard(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    if left.is_empty() && right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(right).count();
    let union = left.union(right).count();
    intersection as f64 / union as f64
}

fn expected_similarity(strictness: PatternStrictness, config: PatternMetricBenchmarks) -> f64 {
    match strictness {
        PatternStrictness::High => config.high,
        PatternStrictness::Medium => config.medium,
        PatternStrictness::Low => config.low,
    }
}

const DUPLICATE_LIKE_SIMILARITY: f64 = 0.88;
const DIFFUSE_SIMILARITY: f64 = 0.05;
const BENCHMARK_TOLERANCE: f64 = 0.15;
const OUTLIER_MIN_SIMILARITY: f64 = 0.10;
const OUTLIER_RANGE: f64 = 0.60;

fn benchmark_estimate(
    strictness: PatternStrictness,
    mean_similarity: f64,
    min_similarity: f64,
    max_similarity: f64,
    config: PatternMetricBenchmarks,
) -> PatternBenchmarkEstimate {
    if mean_similarity >= DUPLICATE_LIKE_SIMILARITY {
        return PatternBenchmarkEstimate::DuplicateLike;
    }
    if mean_similarity <= DIFFUSE_SIMILARITY {
        return PatternBenchmarkEstimate::Diffuse;
    }
    if strictness != PatternStrictness::Low
        && (min_similarity <= OUTLIER_MIN_SIMILARITY
            || max_similarity - min_similarity >= OUTLIER_RANGE)
    {
        return PatternBenchmarkEstimate::LooserThanExpected;
    }

    let delta = mean_similarity - expected_similarity(strictness, config);
    if delta > BENCHMARK_TOLERANCE {
        PatternBenchmarkEstimate::TighterThanExpected
    } else if delta < -BENCHMARK_TOLERANCE {
        PatternBenchmarkEstimate::LooserThanExpected
    } else {
        PatternBenchmarkEstimate::NearExpected
    }
}

fn round_similarity(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}
