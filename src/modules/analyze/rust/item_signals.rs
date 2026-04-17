/**
@module SPECIAL.MODULES.ANALYZE.RUST.ITEM_SIGNALS
Surfaces per-item Rust evidence inside owned implementation so unusually isolated or outbound-heavy items can be inspected directly without reassigning ownership automatically.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.ITEM_SIGNALS
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;

use syn::visit::Visit;
use syn::{Expr, ImplItemFn, Item};

use crate::model::{ModuleItemKind, ModuleItemSignal, ModuleItemSignalsSummary};
use crate::syntax::{SourceItemKind, parse_source_graph};

use super::item_metrics::{function_metrics, method_metrics};

#[derive(Default)]
pub(super) struct RustItemSignalsSummary {
    items: Vec<ItemSignalRecord>,
}

pub(crate) fn summarize_rust_item_signals(path: &Path, text: &str) -> ModuleItemSignalsSummary {
    let mut summary = RustItemSignalsSummary::default();
    summary.observe(path, text);
    summary.finish()
}

impl RustItemSignalsSummary {
    pub(super) fn observe(&mut self, path: &Path, text: &str) {
        let syntax_items = parse_source_graph(path, text).map(|graph| graph.items);

        if let Ok(file) = syn::parse_file(text) {
            self.observe_items(&file.items, syntax_items.as_deref());
            return;
        }

        if let Ok(item) = syn::parse_str::<Item>(text) {
            self.observe_items(std::slice::from_ref(&item), syntax_items.as_deref());
        }
    }

    pub(super) fn finish(mut self) -> ModuleItemSignalsSummary {
        let local_names = self
            .items
            .iter()
            .map(|item| item.name.clone())
            .collect::<Vec<_>>();
        for item in &mut self.items {
            item.observe_edges(&local_names);
        }

        let mut inbound_counts: BTreeMap<String, usize> = BTreeMap::new();
        for item in &self.items {
            for callee in &item.internal_callees {
                *inbound_counts.entry(callee.clone()).or_default() += 1;
            }
        }
        for item in &mut self.items {
            item.inbound_internal_refs = inbound_counts.get(&item.name).copied().unwrap_or(0);
        }

        let mut connected_items = self
            .items
            .iter()
            .filter(|item| item.internal_refs + item.inbound_internal_refs > 0)
            .cloned()
            .collect::<Vec<_>>();
        connected_items.sort_by(|left, right| {
            (right.internal_refs + right.inbound_internal_refs)
                .cmp(&(left.internal_refs + left.inbound_internal_refs))
                .then_with(|| right.internal_refs.cmp(&left.internal_refs))
                .then_with(|| left.name.cmp(&right.name))
        });

        let mut outbound_heavy_items = self
            .items
            .iter()
            .filter(|item| item.external_refs > item.internal_refs)
            .cloned()
            .collect::<Vec<_>>();
        outbound_heavy_items.sort_by(|left, right| {
            (right.external_refs as isize - right.internal_refs as isize)
                .cmp(&(left.external_refs as isize - left.internal_refs as isize))
                .then_with(|| right.external_refs.cmp(&left.external_refs))
                .then_with(|| left.name.cmp(&right.name))
        });

        let mut isolated_items = self
            .items
            .iter()
            .filter(|item| {
                item.internal_refs == 0 && item.inbound_internal_refs == 0 && item.external_refs > 0
            })
            .cloned()
            .collect::<Vec<_>>();
        isolated_items.sort_by(|left, right| {
            right
                .external_refs
                .cmp(&left.external_refs)
                .then_with(|| left.name.cmp(&right.name))
        });

        let reachable_names = reachable_from_roots(&self.items);
        let mut unreached_items = self
            .items
            .iter()
            .filter(|item| {
                !item.root_visible
                    && !item.is_test
                    && !reachable_names.iter().any(|name| name == &item.name)
            })
            .cloned()
            .collect::<Vec<_>>();
        unreached_items.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.kind.cmp(&right.kind))
        });

        let mut highest_complexity_items = self.items.to_vec();
        highest_complexity_items.sort_by(|left, right| {
            right
                .cognitive
                .cmp(&left.cognitive)
                .then_with(|| right.cyclomatic.cmp(&left.cyclomatic))
                .then_with(|| left.name.cmp(&right.name))
        });
        highest_complexity_items.retain(|item| item.cognitive > 0 || item.cyclomatic > 1);

        let mut parameter_heavy_items = self
            .items
            .iter()
            .filter(|item| item.parameter_count > 1)
            .cloned()
            .collect::<Vec<_>>();
        parameter_heavy_items.sort_by(|left, right| {
            right
                .parameter_count
                .cmp(&left.parameter_count)
                .then_with(|| {
                    right
                        .raw_string_parameter_count
                        .cmp(&left.raw_string_parameter_count)
                })
                .then_with(|| left.name.cmp(&right.name))
        });

        let mut stringly_boundary_items = self
            .items
            .iter()
            .filter(|item| item.public && item.raw_string_parameter_count > 0)
            .cloned()
            .collect::<Vec<_>>();
        stringly_boundary_items.sort_by(|left, right| {
            right
                .raw_string_parameter_count
                .cmp(&left.raw_string_parameter_count)
                .then_with(|| right.parameter_count.cmp(&left.parameter_count))
                .then_with(|| left.name.cmp(&right.name))
        });

        let mut panic_heavy_items = self
            .items
            .iter()
            .filter(|item| item.panic_site_count > 0)
            .cloned()
            .collect::<Vec<_>>();
        panic_heavy_items.sort_by(|left, right| {
            right
                .panic_site_count
                .cmp(&left.panic_site_count)
                .then_with(|| right.cognitive.cmp(&left.cognitive))
                .then_with(|| left.name.cmp(&right.name))
        });

        ModuleItemSignalsSummary {
            analyzed_items: self.items.len(),
            unreached_item_count: unreached_items.len(),
            connected_items: connected_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
            outbound_heavy_items: outbound_heavy_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
            isolated_items: isolated_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
            unreached_items: unreached_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
            highest_complexity_items: highest_complexity_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
            parameter_heavy_items: parameter_heavy_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
            stringly_boundary_items: stringly_boundary_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
            panic_heavy_items: panic_heavy_items
                .into_iter()
                .take(5)
                .map(ItemSignalRecord::into_summary)
                .collect(),
        }
    }

    fn observe_items(
        &mut self,
        items: &[Item],
        syntax_items: Option<&[crate::syntax::SourceItem]>,
    ) {
        let mut records = Vec::new();
        collect_item_records(items, &mut records);
        if let Some(syntax_items) = syntax_items {
            attach_syntax_calls(&mut records, syntax_items);
        }
        for record in records {
            self.items.push(record);
        }
    }
}

#[derive(Clone)]
struct ItemSignalRecord {
    name: String,
    kind: ModuleItemKind,
    public: bool,
    root_visible: bool,
    is_test: bool,
    parameter_count: usize,
    bool_parameter_count: usize,
    raw_string_parameter_count: usize,
    cyclomatic: usize,
    cognitive: usize,
    panic_site_count: usize,
    internal_refs: usize,
    inbound_internal_refs: usize,
    external_refs: usize,
    internal_callees: Vec<String>,
    observed_call_names: Option<Vec<String>>,
    body: syn::Block,
}

impl ItemSignalRecord {
    fn from_function(function: &syn::ItemFn) -> Self {
        let metrics = function_metrics(&function.vis, &function.sig.inputs, &function.block);
        Self {
            name: function.sig.ident.to_string(),
            kind: ModuleItemKind::Function,
            public: metrics.public,
            root_visible: metrics.root_visible || function.sig.ident == "main",
            is_test: has_test_attr(&function.attrs),
            parameter_count: metrics.parameter_count,
            bool_parameter_count: metrics.bool_parameter_count,
            raw_string_parameter_count: metrics.raw_string_parameter_count,
            cyclomatic: metrics.cyclomatic,
            cognitive: metrics.cognitive,
            panic_site_count: metrics.panic_site_count,
            internal_refs: 0,
            inbound_internal_refs: 0,
            external_refs: 0,
            internal_callees: Vec::new(),
            observed_call_names: None,
            body: (*function.block).clone(),
        }
    }

    fn from_method(method: &ImplItemFn) -> Self {
        let metrics = method_metrics(method);
        Self {
            name: method.sig.ident.to_string(),
            kind: ModuleItemKind::Method,
            public: metrics.public,
            root_visible: metrics.root_visible,
            is_test: false,
            parameter_count: metrics.parameter_count,
            bool_parameter_count: metrics.bool_parameter_count,
            raw_string_parameter_count: metrics.raw_string_parameter_count,
            cyclomatic: metrics.cyclomatic,
            cognitive: metrics.cognitive,
            panic_site_count: metrics.panic_site_count,
            internal_refs: 0,
            inbound_internal_refs: 0,
            external_refs: 0,
            internal_callees: Vec::new(),
            observed_call_names: None,
            body: method.block.clone(),
        }
    }

    fn observe_edges(&mut self, local_names: &[String]) {
        if let Some(call_names) = self.observed_call_names.clone() {
            self.observe_named_calls(&call_names, local_names);
            return;
        }

        let mut visitor = CallEdgeVisitor {
            local_names,
            internal_refs: 0,
            external_refs: 0,
            internal_callees: Vec::new(),
        };
        visitor.visit_block(&self.body);
        self.internal_refs = visitor.internal_refs;
        self.external_refs = visitor.external_refs;
        self.internal_callees = visitor.internal_callees;
    }

    fn observe_named_calls(&mut self, call_names: &[String], local_names: &[String]) {
        let mut internal_refs = 0;
        let mut external_refs = 0;
        let mut internal_callees = Vec::new();
        for call_name in call_names {
            if local_names.iter().any(|name| name == call_name) {
                internal_refs += 1;
                internal_callees.push(call_name.clone());
            } else {
                external_refs += 1;
            }
        }
        self.internal_refs = internal_refs;
        self.external_refs = external_refs;
        self.internal_callees = internal_callees;
    }

    fn into_summary(self) -> ModuleItemSignal {
        ModuleItemSignal {
            name: self.name,
            kind: self.kind,
            public: self.public,
            parameter_count: self.parameter_count,
            bool_parameter_count: self.bool_parameter_count,
            raw_string_parameter_count: self.raw_string_parameter_count,
            internal_refs: self.internal_refs,
            inbound_internal_refs: self.inbound_internal_refs,
            external_refs: self.external_refs,
            cyclomatic: self.cyclomatic,
            cognitive: self.cognitive,
            panic_site_count: self.panic_site_count,
        }
    }
}

fn has_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("test"))
}

fn reachable_from_roots(items: &[ItemSignalRecord]) -> Vec<String> {
    let mut reachable = Vec::new();
    let mut queue = VecDeque::new();

    for item in items {
        if item.root_visible || item.is_test {
            reachable.push(item.name.clone());
            queue.push_back(item.name.clone());
        }
    }

    while let Some(name) = queue.pop_front() {
        let Some(item) = items.iter().find(|item| item.name == name) else {
            continue;
        };
        for callee in &item.internal_callees {
            if reachable.iter().any(|seen| seen == callee) {
                continue;
            }
            reachable.push(callee.clone());
            queue.push_back(callee.clone());
        }
    }

    reachable
}

fn collect_item_records(items: &[Item], records: &mut Vec<ItemSignalRecord>) {
    for item in items {
        match item {
            Item::Fn(function) => records.push(ItemSignalRecord::from_function(function)),
            Item::Impl(item_impl) => {
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        records.push(ItemSignalRecord::from_method(method));
                    }
                }
            }
            Item::Mod(item_mod) => {
                if let Some((_, nested)) = &item_mod.content {
                    collect_item_records(nested, records);
                }
            }
            _ => {}
        }
    }
}

fn attach_syntax_calls(
    records: &mut [ItemSignalRecord],
    syntax_items: &[crate::syntax::SourceItem],
) {
    let mut calls_by_item: BTreeMap<(String, SourceItemKind), VecDeque<Vec<String>>> =
        BTreeMap::new();
    for item in syntax_items {
        calls_by_item
            .entry((item.name.clone(), item.kind))
            .or_default()
            .push_back(item.calls.iter().map(|call| call.name.clone()).collect());
    }

    for record in records {
        let key = (record.name.clone(), source_item_kind(record.kind));
        let Some(call_sets) = calls_by_item.get_mut(&key) else {
            continue;
        };
        let Some(call_names) = call_sets.pop_front() else {
            continue;
        };
        record.observed_call_names = Some(call_names);
    }
}

fn source_item_kind(kind: ModuleItemKind) -> SourceItemKind {
    match kind {
        ModuleItemKind::Function => SourceItemKind::Function,
        ModuleItemKind::Method => SourceItemKind::Method,
    }
}

struct CallEdgeVisitor<'a> {
    local_names: &'a [String],
    internal_refs: usize,
    external_refs: usize,
    internal_callees: Vec<String>,
}

impl Visit<'_> for CallEdgeVisitor<'_> {
    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if let Expr::Path(expr_path) = &*node.func {
            let segments = expr_path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if let Some(last) = segments.last() {
                if segments.len() == 1 && self.local_names.iter().any(|name| name == last) {
                    self.internal_refs += 1;
                    self.internal_callees.push(last.clone());
                } else {
                    self.external_refs += 1;
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}
