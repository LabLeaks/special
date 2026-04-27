/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.QUALITY
Extracts TypeScript quality, complexity, and per-item craftsmanship evidence from owned TypeScript implementation using the same summary categories as the Rust analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.QUALITY
use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{ModuleComplexitySummary, ModuleQualitySummary};
use crate::modules::analyze::source_item_signals::SourceItemMetrics;
use crate::syntax::{ParsedSourceGraph, SourceItem};
use crate::tree_sitter_utils::{first_named_child_with_kind, is_boolean_binary_expression};

type ItemMetricKey = (usize, usize, String);

#[derive(Debug, Default)]
pub(super) struct TypeScriptQualitySummary {
    public_function_count: usize,
    parameter_count: usize,
    bool_parameter_count: usize,
    raw_string_parameter_count: usize,
    panic_site_count: usize,
    function_count: usize,
    total_cyclomatic: usize,
    max_cyclomatic: usize,
    total_cognitive: usize,
    max_cognitive: usize,
    item_metrics: BTreeMap<String, SourceItemMetrics>,
}

impl TypeScriptQualitySummary {
    pub(super) fn observe(&mut self, path: &Path, text: &str, graph: &ParsedSourceGraph) {
        let mut parser = Parser::new();
        let language = match path.extension().and_then(|ext| ext.to_str()) {
            Some("tsx") => tree_sitter_typescript::LANGUAGE_TSX,
            _ => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        };
        if parser.set_language(&language.into()).is_err() {
            return;
        }
        let Some(tree) = parser.parse(text, None) else {
            return;
        };

    let source = text.as_bytes();
    let mut metrics_by_item = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics_by_item);

        for item in &graph.items {
            if let Some(metrics) = metrics_by_item.get(&item_key(item)).copied() {
                self.function_count += 1;
                self.total_cyclomatic += metrics.cyclomatic;
                self.max_cyclomatic = self.max_cyclomatic.max(metrics.cyclomatic);
                self.total_cognitive += metrics.cognitive;
                self.max_cognitive = self.max_cognitive.max(metrics.cognitive);
                self.panic_site_count += metrics.panic_site_count;
                self.item_metrics.insert(item.stable_id.clone(), metrics);
            }
            if item.public {
                self.public_function_count += 1;
            }
            if item.public
                && let Some(metrics) = metrics_by_item.get(&item_key(item))
            {
                self.parameter_count += metrics.parameter_count;
                self.bool_parameter_count += metrics.bool_parameter_count;
                self.raw_string_parameter_count += metrics.raw_string_parameter_count;
            }
        }
    }

    pub(super) fn finish_complexity(&self) -> ModuleComplexitySummary {
        ModuleComplexitySummary {
            function_count: self.function_count,
            total_cyclomatic: self.total_cyclomatic,
            max_cyclomatic: self.max_cyclomatic,
            total_cognitive: self.total_cognitive,
            max_cognitive: self.max_cognitive,
        }
    }

    pub(super) fn item_metrics(&self) -> &BTreeMap<String, SourceItemMetrics> {
        &self.item_metrics
    }

    pub(super) fn finish(self) -> ModuleQualitySummary {
        ModuleQualitySummary {
            public_function_count: self.public_function_count,
            parameter_count: self.parameter_count,
            bool_parameter_count: self.bool_parameter_count,
            raw_string_parameter_count: self.raw_string_parameter_count,
            panic_site_count: self.panic_site_count,
        }
    }
}

fn collect_callable_metrics(
    node: Node<'_>,
    source: &[u8],
    metrics_by_item: &mut BTreeMap<ItemMetricKey, SourceItemMetrics>,
) {
    if matches!(
        node.kind(),
        "function_declaration"
            | "generator_function_declaration"
            | "method_definition"
            | "variable_declarator"
    ) && let Some((line, column, name, callable_node, parameter_node)) = callable_parts(node, source)
    {
        metrics_by_item.insert(
            (line, column, name),
            item_metrics(callable_node, parameter_node, source),
        );
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_callable_metrics(child, source, metrics_by_item);
    }
}

fn callable_parts<'tree>(
    node: Node<'tree>,
    source: &[u8],
) -> Option<(usize, usize, String, Node<'tree>, Node<'tree>)> {
    let callable_node = if node.kind() == "variable_declarator" {
        let value = node.child_by_field_name("value")?;
        if !is_callable_value(value) {
            return None;
        }
        value
    } else {
        node
    };
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .trim_matches('"')
        .to_string();
    let parameters = callable_node
        .child_by_field_name("parameters")
        .or_else(|| first_named_child_with_kind(callable_node, "formal_parameters"))
        .or_else(|| callable_node.child_by_field_name("parameter"))?;
    Some((
        node.start_position().row + 1,
        node.start_position().column,
        name,
        callable_node,
        parameters,
    ))
}

fn item_metrics(
    callable_node: Node<'_>,
    parameter_node: Node<'_>,
    source: &[u8],
) -> SourceItemMetrics {
    let mut metrics = parameter_metrics(parameter_node, source);
    metrics.cyclomatic = cyclomatic_complexity(callable_node, source);
    metrics.cognitive = cognitive_complexity(callable_node, source);
    metrics.panic_site_count = count_throw_sites(callable_node);
    metrics
}

fn parameter_metrics(node: Node<'_>, source: &[u8]) -> SourceItemMetrics {
    let mut metrics = SourceItemMetrics::default();
    if is_standalone_parameter_node(node) {
        observe_parameter_node(&mut metrics, node, source);
        return metrics;
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if !is_parameter_node(child) {
            continue;
        }
        observe_parameter_node(&mut metrics, child, source);
    }
    metrics
}

fn is_parameter_node(node: Node<'_>) -> bool {
    node.kind().contains("parameter") || node.kind() == "identifier"
}

fn is_standalone_parameter_node(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "identifier" | "required_parameter" | "optional_parameter"
    )
}

fn observe_parameter_node(metrics: &mut SourceItemMetrics, node: Node<'_>, source: &[u8]) {
    metrics.parameter_count += 1;
    if parameter_contains_type(node, source, &["boolean", "Boolean"]) {
        metrics.bool_parameter_count += 1;
    }
    if parameter_contains_type(node, source, &["string", "String"]) {
        metrics.raw_string_parameter_count += 1;
    }
}

fn parameter_contains_type(node: Node<'_>, source: &[u8], names: &[&str]) -> bool {
    node.child_by_field_name("type")
        .is_some_and(|type_node| node_contains_type(type_node, source, names))
        || first_named_child_with_kind(node, "type_annotation")
            .is_some_and(|type_node| node_contains_type(type_node, source, names))
}

fn node_contains_type(node: Node<'_>, source: &[u8], names: &[&str]) -> bool {
    let text_matches = matches!(node.kind(), "predefined_type" | "type_identifier")
        && node
            .utf8_text(source)
            .ok()
            .is_some_and(|text| names.contains(&text.trim()));
    if text_matches {
        return true;
    }

    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| node_contains_type(child, source, names))
}

fn count_throw_sites(node: Node<'_>) -> usize {
    let mut count = usize::from(node.kind() == "throw_statement");
    let mut cursor = node.walk();
    count += node
        .named_children(&mut cursor)
        .filter(|child| !is_nested_callable_boundary(*child))
        .map(count_throw_sites)
        .sum::<usize>();
    count
}

fn cyclomatic_complexity(node: Node<'_>, source: &[u8]) -> usize {
    let mut complexity = 1;
    add_cyclomatic_complexity(node, source, &mut complexity);
    complexity
}

fn add_cyclomatic_complexity(node: Node<'_>, source: &[u8], complexity: &mut usize) {
    match node.kind() {
        "if_statement" | "for_statement" | "while_statement" | "do_statement"
        | "for_in_statement" | "catch_clause" | "ternary_expression" => {
            *complexity += 1;
        }
        "switch_case" => {
            *complexity += 1;
        }
        "binary_expression" if is_boolean_binary_expression(node, source) => {
            *complexity += 1;
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if is_nested_callable_boundary(child) {
            continue;
        }
        add_cyclomatic_complexity(child, source, complexity);
    }
}

fn cognitive_complexity(node: Node<'_>, source: &[u8]) -> usize {
    cognitive_complexity_at(node, source, 0)
}

fn cognitive_complexity_at(node: Node<'_>, source: &[u8], nesting: usize) -> usize {
    let structural = matches!(
        node.kind(),
        "if_statement"
            | "for_statement"
            | "for_in_statement"
            | "while_statement"
            | "do_statement"
            | "catch_clause"
            | "ternary_expression"
            | "switch_statement"
    );
    let boolean = node.kind() == "binary_expression" && is_boolean_binary_expression(node, source);

    let mut score = usize::from(boolean);
    let child_nesting = if structural {
        score += 1 + nesting;
        nesting + 1
    } else {
        nesting
    };

    let mut cursor = node.walk();
    score
        + node
            .named_children(&mut cursor)
            .filter(|child| !is_nested_callable_boundary(*child))
            .map(|child| cognitive_complexity_at(child, source, child_nesting))
            .sum::<usize>()
}

fn is_nested_callable_boundary(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "function_declaration"
            | "generator_function_declaration"
            | "method_definition"
            | "arrow_function"
            | "function"
            | "generator_function"
    ) || (node.kind() == "variable_declarator"
        && node
            .child_by_field_name("value")
            .is_some_and(is_callable_value))
}

fn is_callable_value(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "arrow_function" | "function" | "generator_function"
    )
}

fn item_key(item: &SourceItem) -> ItemMetricKey {
    (item.span.start_line, item.span.start_column, item.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric<'a>(
        metrics: &'a BTreeMap<ItemMetricKey, SourceItemMetrics>,
        line: usize,
        name: &str,
    ) -> Option<&'a SourceItemMetrics> {
        metrics
            .iter()
            .find(|((item_line, _, item_name), _)| *item_line == line && item_name == name)
            .map(|(_, metrics)| metrics)
    }

    #[test]
    fn nested_callables_do_not_inflate_outer_item_metrics() {
        let source = br#"
function outer(flag: boolean): number {
  function inner(): number {
    if (flag) {
      throw new Error("nested");
    }
    return 1;
  }
  if (flag) {
    return inner();
  }
  return 0;
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("typescript grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let outer = metric(&metrics, 2, "outer")
            .expect("outer function should be measured");
        assert_eq!(outer.cyclomatic, 2);
        assert_eq!(outer.cognitive, 1);
        assert_eq!(outer.panic_site_count, 0);

        let inner = metric(&metrics, 3, "inner")
            .expect("inner function should still be measured separately");
        assert_eq!(inner.cyclomatic, 2);
        assert_eq!(inner.cognitive, 1);
        assert_eq!(inner.panic_site_count, 1);
    }

    #[test]
    fn single_parameter_arrow_functions_are_measured() {
        let source = br#"
export const identity = value => value;
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("typescript grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let identity = metric(&metrics, 2, "identity")
            .expect("single-parameter arrow should be measured");
        assert_eq!(identity.parameter_count, 1);
        assert_eq!(identity.cyclomatic, 1);
        assert_eq!(identity.cognitive, 0);
    }

    #[test]
    fn parameter_type_counts_ignore_untyped_names() {
        let source = br#"
export function boundary(string, boolean, actual: string, flag: boolean) {
  return `${actual}:${flag}:${string}:${boolean}`;
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("typescript grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let boundary = metric(&metrics, 2, "boundary")
            .expect("boundary function should be measured");
        assert_eq!(boundary.parameter_count, 4);
        assert_eq!(boundary.raw_string_parameter_count, 1);
        assert_eq!(boundary.bool_parameter_count, 1);
    }

    #[test]
    fn nested_generator_functions_do_not_inflate_outer_item_metrics() {
        let source = br#"
export function outer(flag: boolean): number {
  function* nested(): Generator<number> {
    if (flag) {
      throw new Error("nested");
    }
    yield 1;
  }
  if (flag) {
    return nested().next().value ?? 0;
  }
  return 0;
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("typescript grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let outer = metric(&metrics, 2, "outer")
            .expect("outer function should be measured");
        assert_eq!(outer.cyclomatic, 2);
        assert_eq!(outer.cognitive, 1);
        assert_eq!(outer.panic_site_count, 0);

        let nested = metric(&metrics, 3, "nested")
            .expect("nested generator should be measured separately");
        assert_eq!(nested.cyclomatic, 2);
        assert_eq!(nested.cognitive, 1);
        assert_eq!(nested.panic_site_count, 1);
    }

    #[test]
    fn same_line_same_name_callables_keep_distinct_metrics() {
        let source = r#"class A { m() {} } class B { m(value: string) { if (value) { return; } } }"#;
        let path = Path::new("src/demo.ts");
        let graph = crate::syntax::parse_source_graph(path, source).expect("source should parse");
        let mut summary = TypeScriptQualitySummary::default();
        summary.observe(path, source, &graph);

        let method_metrics = graph
            .items
            .iter()
            .filter(|item| item.name == "m")
            .filter_map(|item| summary.item_metrics().get(&item.stable_id))
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(method_metrics.len(), 2);
        assert!(method_metrics.iter().any(|metrics| {
            metrics.parameter_count == 0 && metrics.cyclomatic == 1 && metrics.cognitive == 0
        }));
        assert!(method_metrics.iter().any(|metrics| {
            metrics.parameter_count == 1
                && metrics.raw_string_parameter_count == 1
                && metrics.cyclomatic == 2
                && metrics.cognitive == 1
        }));
    }

    #[test]
    fn for_of_loops_count_as_branches() {
        let source = br#"
export function collect(items) {
  const result = [];
  for (const item of items) {
    result.push(item);
  }
  return result;
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("typescript grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let collect = metric(&metrics, 2, "collect")
            .expect("collect function should be measured");
        assert_eq!(collect.cyclomatic, 2);
        assert_eq!(collect.cognitive, 1);
    }
}
