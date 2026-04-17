/**
@module SPECIAL.MODULES.ANALYZE.SOURCE_ITEM_SIGNALS
Builds generic per-item connectivity and unreached summaries from normalized source-item graphs so lightweight language providers can share one item-signal implementation.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.SOURCE_ITEM_SIGNALS
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::model::{ModuleItemKind, ModuleItemSignal, ModuleItemSignalsSummary};
use crate::syntax::{SourceItem, SourceItemKind};

pub(super) fn summarize_source_item_signals(items: &[SourceItem]) -> ModuleItemSignalsSummary {
    let mut records = items
        .iter()
        .map(ItemSignalRecord::from_source_item)
        .collect::<Vec<_>>();
    let local_names = records
        .iter()
        .map(|item| item.name.clone())
        .collect::<Vec<_>>();
    for item in &mut records {
        item.observe_edges(&local_names);
    }

    let mut inbound_counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in &records {
        for callee in &item.internal_callees {
            *inbound_counts.entry(callee.clone()).or_default() += 1;
        }
    }
    for item in &mut records {
        item.inbound_internal_refs = inbound_counts.get(&item.name).copied().unwrap_or(0);
    }

    let mut connected_items = records
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

    let mut outbound_heavy_items = records
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

    let mut isolated_items = records
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

    let reachable_names = reachable_from_roots(&records);
    let mut unreached_items = records
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

    ModuleItemSignalsSummary {
        analyzed_items: records.len(),
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
        ..ModuleItemSignalsSummary::default()
    }
}

#[derive(Clone)]
struct ItemSignalRecord {
    name: String,
    kind: ModuleItemKind,
    public: bool,
    root_visible: bool,
    is_test: bool,
    internal_refs: usize,
    inbound_internal_refs: usize,
    external_refs: usize,
    internal_callees: Vec<String>,
    observed_call_names: Vec<String>,
}

impl ItemSignalRecord {
    fn from_source_item(item: &SourceItem) -> Self {
        Self {
            name: item.name.clone(),
            kind: match item.kind {
                SourceItemKind::Function => ModuleItemKind::Function,
                SourceItemKind::Method => ModuleItemKind::Method,
            },
            public: item.public,
            root_visible: item.root_visible || is_process_entrypoint(item),
            is_test: item.is_test,
            internal_refs: 0,
            inbound_internal_refs: 0,
            external_refs: 0,
            internal_callees: Vec::new(),
            observed_call_names: item.calls.iter().map(|call| call.name.clone()).collect(),
        }
    }

    fn observe_edges(&mut self, local_names: &[String]) {
        for call_name in &self.observed_call_names {
            if local_names.iter().any(|name| name == call_name) {
                self.internal_refs += 1;
                self.internal_callees.push(call_name.clone());
            } else {
                self.external_refs += 1;
            }
        }
    }

    fn into_summary(self) -> ModuleItemSignal {
        ModuleItemSignal {
            name: self.name,
            kind: self.kind,
            public: self.public,
            parameter_count: 0,
            bool_parameter_count: 0,
            raw_string_parameter_count: 0,
            internal_refs: self.internal_refs,
            inbound_internal_refs: self.inbound_internal_refs,
            external_refs: self.external_refs,
            cyclomatic: 0,
            cognitive: 0,
            panic_site_count: 0,
        }
    }
}

fn reachable_from_roots(items: &[ItemSignalRecord]) -> Vec<String> {
    let adjacency = items
        .iter()
        .map(|item| (item.name.clone(), item.internal_callees.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut queue = items
        .iter()
        .filter(|item| item.root_visible || item.is_test)
        .map(|item| item.name.clone())
        .collect::<VecDeque<_>>();
    let mut seen = BTreeSet::new();

    while let Some(name) = queue.pop_front() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(callees) = adjacency.get(&name) {
            queue.extend(callees.iter().cloned());
        }
    }

    seen.into_iter().collect()
}

fn is_process_entrypoint(item: &SourceItem) -> bool {
    item.kind == SourceItemKind::Function && item.name == "main"
}
