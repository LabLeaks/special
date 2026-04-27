#[path = "projection/module.rs"]
mod module;
/**
@module SPECIAL.RENDER.PROJECTION
Projects materialized specs, modules, and repo health documents into the visible verbose or non-verbose shape shared by all render backends.
*/
// @fileimplements SPECIAL.RENDER.PROJECTION
mod module_analysis;
#[path = "projection/pattern.rs"]
mod pattern;
#[path = "projection/repo.rs"]
mod repo;
#[path = "projection/spec.rs"]
mod spec;

pub(super) use self::module::project_module_document;
pub(super) use self::pattern::project_pattern_document;
pub(super) use self::repo::{project_repo_document, project_repo_document_json};
pub(super) use self::spec::project_document;

pub(super) use self::module_analysis::{
    ProjectedArchitectureTraceability, ProjectedCount, ProjectedExplanation, ProjectedMetaLine,
    ProjectedModuleAnalysis, ProjectedRepoSignals, project_module_analysis_view,
    project_repo_signals_view, project_repo_traceability_view,
};
