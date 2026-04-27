/**
@module SPECIAL.MODEL
Canonical Rust domain types grouped by core primitives, spec modeling, architecture modeling, analysis summaries, and rendered report documents.
*/
// @fileimplements SPECIAL.MODEL
mod analysis;
mod architecture;
mod core;
mod overview;
mod spec;

pub use analysis::{
    ArchitectureAnalysisSummary, ArchitectureDuplicateItem, ArchitectureRepoSignalsSummary,
    ArchitectureTraceabilityItem, ArchitectureTraceabilitySummary, ArchitectureUnownedItem,
    ModuleAnalysisOptions, ModuleAnalysisSummary, ModuleComplexitySummary, ModuleCouplingSummary,
    ModuleCoverageSummary, ModuleDependencySummary, ModuleDependencyTargetSummary, ModuleItemKind,
    ModuleItemSignal, ModuleItemSignalsSummary, ModuleMetricsSummary, ModuleQualitySummary,
    ModuleTraceabilityItem, ModuleTraceabilitySummary,
};
pub use architecture::{
    ImplementRef, ModuleDecl, ModuleFilter, ModuleNode, ParsedArchitecture, PatternApplication,
    PatternApplicationNode, PatternBenchmarkEstimate, PatternCandidateConfidence,
    PatternClusterCandidate, PatternClusterInterpretation, PatternClusterItem, PatternDefinition,
    PatternDocument, PatternFilter, PatternMetricsSummary, PatternMissingApplicationCandidate,
    PatternModuleRef, PatternNode, PatternSimilarityMetrics, PatternStrictness,
};
pub use core::{
    ArchitectureKind, BlockLine, CommentBlock, DeclaredStateFilter, DeprecatedRelease,
    DiagnosticSeverity, ModelInvariantError, NodeKind, OwnedItem, PlanState, PlannedRelease,
    SourceLocation,
};
pub use overview::{
    ArchitectureMetricsSummary, GroupedCount, LintReport, ModuleDocument, OverviewArchSummary,
    OverviewDocument, OverviewHealthSummary, OverviewLintSummary, OverviewSpecsSummary,
    RepoDocument, RepoMetricsSummary, RepoTraceabilityMetrics, SpecDocument, SpecMetricsSummary,
};
pub use spec::{
    AttestRef, AttestScope, Diagnostic, ParsedRepo, SpecDecl, SpecFilter, SpecNode, VerifyRef,
};
