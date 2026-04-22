/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS
Shared TypeScript scoped traceability test context builders.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT.BUILDERS
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::model::ArchitectureTraceabilitySummary;
use crate::modules::analyze::FileOwnership;
use crate::modules::analyze::traceability_core::TraceabilityInputs;

use crate::language_packs::typescript::analyze::boundary::{
    ScopedTraceabilityContract, ScopedTraceabilityReference,
};

#[path = "builders/comparisons.rs"]
mod comparisons;
#[path = "builders/contracts.rs"]
mod contracts;
#[path = "builders/runtime.rs"]
mod runtime;

pub(crate) type TypeScriptContractTestContext = (
    ArchitectureTraceabilitySummary,
    ScopedTraceabilityContract,
    PathBuf,
    crate::model::ParsedRepo,
    crate::model::ParsedArchitecture,
    std::collections::BTreeMap<PathBuf, FileOwnership<'static>>,
);

pub(crate) type TypeScriptContractComparisonContext = (
    ScopedTraceabilityContract,
    ScopedTraceabilityContract,
    PathBuf,
);

pub(crate) type TypeScriptExactTargetContext = (
    ScopedTraceabilityContract,
    TraceabilityInputs,
    PathBuf,
);

pub(crate) type TypeScriptInputComparisonContext = (
    TraceabilityInputs,
    TraceabilityInputs,
    BTreeSet<String>,
    PathBuf,
);

pub(crate) type TypeScriptReferenceComparisonContext = (
    ScopedTraceabilityContract,
    ScopedTraceabilityReference,
    TraceabilityInputs,
    TraceabilityInputs,
    PathBuf,
);

pub(crate) use comparisons::*;
pub(crate) use contracts::*;
pub(crate) use runtime::*;
