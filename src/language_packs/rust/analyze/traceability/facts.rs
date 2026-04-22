/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.FACTS
Cached Rust traceability graph fact serialization for scoped and full traceability analysis.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.TRACEABILITY.FACTS
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::modules::analyze::traceability_core::TraceabilityItemSupport;
use crate::syntax::{ParsedSourceGraph, SourceCall, SourceInvocation, SourceItem, SourceSpan};

use super::RustMediatedReason;

#[derive(Serialize, Deserialize)]
pub(super) struct RustTraceabilityGraphFacts {
    pub(super) source_graphs: BTreeMap<PathBuf, CachedParsedSourceGraph>,
    pub(super) parser_edges: BTreeMap<String, BTreeSet<String>>,
    pub(super) mediated_reasons: BTreeMap<String, CachedRustMediatedReason>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RustTraceabilityScopeFacts {
    pub(super) source_graphs: BTreeMap<PathBuf, CachedParsedSourceGraph>,
    pub(super) edges: BTreeMap<String, BTreeSet<String>>,
    pub(super) mediated_reasons: BTreeMap<String, CachedRustMediatedReason>,
    pub(super) root_supports: BTreeMap<String, CachedTraceabilityItemSupport>,
}

pub(super) type RustGraphFactsDecoded = (
    BTreeMap<PathBuf, ParsedSourceGraph>,
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, RustMediatedReason>,
);

pub(super) fn decode_traceability_graph_facts(
    facts: Option<&[u8]>,
) -> Result<Option<RustGraphFactsDecoded>> {
    let Some(facts) = facts else {
        return Ok(None);
    };
    let facts = serde_json::from_slice::<RustTraceabilityGraphFacts>(facts)?;
    Ok(Some((
        facts
            .source_graphs
            .into_iter()
            .map(|(path, graph)| (path, graph.into_parsed()))
            .collect(),
        facts.parser_edges,
        facts
            .mediated_reasons
            .into_iter()
            .map(|(stable_id, reason)| (stable_id, reason.into_parsed()))
            .collect(),
    )))
}

#[derive(Serialize, Deserialize)]
pub(super) struct CachedParsedSourceGraph {
    items: Vec<CachedSourceItem>,
}

impl CachedParsedSourceGraph {
    pub(super) fn from_parsed(graph: &ParsedSourceGraph) -> Self {
        Self {
            items: graph.items.iter().map(CachedSourceItem::from_parsed).collect(),
        }
    }

    pub(super) fn into_parsed(self) -> ParsedSourceGraph {
        ParsedSourceGraph {
            language: crate::syntax::SourceLanguage::new("rust"),
            items: self
                .items
                .into_iter()
                .map(CachedSourceItem::into_parsed)
                .collect(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct CachedTraceabilityItemSupport {
    name: String,
    has_item_scoped_support: bool,
    has_file_scoped_support: bool,
    current_specs: BTreeSet<String>,
    planned_specs: BTreeSet<String>,
    deprecated_specs: BTreeSet<String>,
}

impl CachedTraceabilityItemSupport {
    pub(super) fn from_runtime(support: TraceabilityItemSupport) -> Self {
        Self {
            name: support.name,
            has_item_scoped_support: support.has_item_scoped_support,
            has_file_scoped_support: support.has_file_scoped_support,
            current_specs: support.current_specs,
            planned_specs: support.planned_specs,
            deprecated_specs: support.deprecated_specs,
        }
    }

    pub(super) fn into_runtime(self) -> TraceabilityItemSupport {
        TraceabilityItemSupport {
            name: self.name,
            has_item_scoped_support: self.has_item_scoped_support,
            has_file_scoped_support: self.has_file_scoped_support,
            current_specs: self.current_specs,
            planned_specs: self.planned_specs,
            deprecated_specs: self.deprecated_specs,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceItem {
    source_path: String,
    stable_id: String,
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    shape_fingerprint: String,
    shape_node_count: usize,
    kind: CachedSourceItemKind,
    span: CachedSourceSpan,
    public: bool,
    root_visible: bool,
    is_test: bool,
    calls: Vec<CachedSourceCall>,
    invocations: Vec<CachedSourceInvocation>,
}

impl CachedSourceItem {
    fn from_parsed(item: &SourceItem) -> Self {
        Self {
            source_path: item.source_path.clone(),
            stable_id: item.stable_id.clone(),
            name: item.name.clone(),
            qualified_name: item.qualified_name.clone(),
            module_path: item.module_path.clone(),
            container_path: item.container_path.clone(),
            shape_fingerprint: item.shape_fingerprint.clone(),
            shape_node_count: item.shape_node_count,
            kind: CachedSourceItemKind::from_parsed(item.kind),
            span: CachedSourceSpan::from_parsed(item.span),
            public: item.public,
            root_visible: item.root_visible,
            is_test: item.is_test,
            calls: item.calls.iter().map(CachedSourceCall::from_parsed).collect(),
            invocations: item
                .invocations
                .iter()
                .map(CachedSourceInvocation::from_parsed)
                .collect(),
        }
    }

    fn into_parsed(self) -> SourceItem {
        SourceItem {
            source_path: self.source_path,
            stable_id: self.stable_id,
            name: self.name,
            qualified_name: self.qualified_name,
            module_path: self.module_path,
            container_path: self.container_path,
            shape_fingerprint: self.shape_fingerprint,
            shape_node_count: self.shape_node_count,
            kind: self.kind.into_parsed(),
            span: self.span.into_parsed(),
            public: self.public,
            root_visible: self.root_visible,
            is_test: self.is_test,
            calls: self
                .calls
                .into_iter()
                .map(CachedSourceCall::into_parsed)
                .collect(),
            invocations: self
                .invocations
                .into_iter()
                .map(CachedSourceInvocation::into_parsed)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CachedSourceItemKind {
    Function,
    Method,
}

impl CachedSourceItemKind {
    fn from_parsed(kind: crate::syntax::SourceItemKind) -> Self {
        match kind {
            crate::syntax::SourceItemKind::Function => Self::Function,
            crate::syntax::SourceItemKind::Method => Self::Method,
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceItemKind {
        match self {
            Self::Function => crate::syntax::SourceItemKind::Function,
            Self::Method => crate::syntax::SourceItemKind::Method,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceSpan {
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,
    start_byte: usize,
    end_byte: usize,
}

impl CachedSourceSpan {
    fn from_parsed(span: SourceSpan) -> Self {
        Self {
            start_line: span.start_line,
            end_line: span.end_line,
            start_column: span.start_column,
            end_column: span.end_column,
            start_byte: span.start_byte,
            end_byte: span.end_byte,
        }
    }

    fn into_parsed(self) -> SourceSpan {
        SourceSpan {
            start_line: self.start_line,
            end_line: self.end_line,
            start_column: self.start_column,
            end_column: self.end_column,
            start_byte: self.start_byte,
            end_byte: self.end_byte,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceCall {
    name: String,
    qualifier: Option<String>,
    syntax: CachedCallSyntaxKind,
    span: CachedSourceSpan,
}

impl CachedSourceCall {
    fn from_parsed(call: &SourceCall) -> Self {
        Self {
            name: call.name.clone(),
            qualifier: call.qualifier.clone(),
            syntax: CachedCallSyntaxKind::from_parsed(&call.syntax),
            span: CachedSourceSpan::from_parsed(call.span),
        }
    }

    fn into_parsed(self) -> SourceCall {
        SourceCall {
            name: self.name,
            qualifier: self.qualifier,
            syntax: self.syntax.into_parsed(),
            span: self.span.into_parsed(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CachedCallSyntaxKind {
    Identifier,
    ScopedIdentifier,
    Field,
}

impl CachedCallSyntaxKind {
    fn from_parsed(kind: &crate::syntax::CallSyntaxKind) -> Self {
        match kind {
            crate::syntax::CallSyntaxKind::Identifier => Self::Identifier,
            crate::syntax::CallSyntaxKind::ScopedIdentifier => Self::ScopedIdentifier,
            crate::syntax::CallSyntaxKind::Field => Self::Field,
        }
    }

    fn into_parsed(self) -> crate::syntax::CallSyntaxKind {
        match self {
            Self::Identifier => crate::syntax::CallSyntaxKind::Identifier,
            Self::ScopedIdentifier => crate::syntax::CallSyntaxKind::ScopedIdentifier,
            Self::Field => crate::syntax::CallSyntaxKind::Field,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedSourceInvocation {
    kind: CachedSourceInvocationKind,
    span: CachedSourceSpan,
}

impl CachedSourceInvocation {
    fn from_parsed(invocation: &SourceInvocation) -> Self {
        Self {
            kind: CachedSourceInvocationKind::from_parsed(&invocation.kind),
            span: CachedSourceSpan::from_parsed(invocation.span),
        }
    }

    fn into_parsed(self) -> SourceInvocation {
        SourceInvocation {
            kind: self.kind.into_parsed(),
            span: self.span.into_parsed(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CachedSourceInvocationKind {
    LocalCargoBinary { binary_name: String },
}

impl CachedSourceInvocationKind {
    fn from_parsed(kind: &crate::syntax::SourceInvocationKind) -> Self {
        match kind {
            crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name } => {
                Self::LocalCargoBinary {
                    binary_name: binary_name.clone(),
                }
            }
        }
    }

    fn into_parsed(self) -> crate::syntax::SourceInvocationKind {
        match self {
            Self::LocalCargoBinary { binary_name } => {
                crate::syntax::SourceInvocationKind::LocalCargoBinary { binary_name }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) enum CachedRustMediatedReason {
    TraitImplEntrypoint,
}

impl CachedRustMediatedReason {
    pub(super) fn from_parsed(reason: RustMediatedReason) -> Self {
        match reason {
            RustMediatedReason::TraitImplEntrypoint => Self::TraitImplEntrypoint,
        }
    }

    pub(super) fn into_parsed(self) -> RustMediatedReason {
        match self {
            Self::TraitImplEntrypoint => RustMediatedReason::TraitImplEntrypoint,
        }
    }
}
