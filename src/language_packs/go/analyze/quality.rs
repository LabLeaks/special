/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.QUALITY
Extracts Go quality, complexity, and per-item craftsmanship evidence from owned Go implementation using the same summary categories as the Rust analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.QUALITY
use std::collections::BTreeMap;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::model::{ModuleComplexitySummary, ModuleQualitySummary};
use crate::modules::analyze::source_item_signals::SourceItemMetrics;
use crate::syntax::{ParsedSourceGraph, SourceItem};
use crate::tree_sitter_utils::{first_named_child_with_kind, is_boolean_binary_expression};

type ItemMetricKey = (usize, usize, String);

#[derive(Debug, Default)]
pub(super) struct GoQualitySummary {
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

impl GoQualitySummary {
    pub(super) fn observe(&mut self, _path: &Path, text: &str, graph: &ParsedSourceGraph) {
        let mut parser = Parser::new();
        if parser.set_language(&tree_sitter_go::LANGUAGE.into()).is_err() {
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
    if matches!(node.kind(), "function_declaration" | "method_declaration")
        && let Some((line, column, name, parameters)) = callable_parts(node, source)
    {
        metrics_by_item.insert((line, column, name), item_metrics(node, parameters, source));
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_callable_metrics(child, source, metrics_by_item);
    }
}

fn callable_parts<'tree>(
    node: Node<'tree>,
    source: &[u8],
) -> Option<(usize, usize, String, Node<'tree>)> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let parameters = node
        .child_by_field_name("parameters")
        .or_else(|| first_named_child_with_kind(node, "parameter_list"))?;
    Some((
        node.start_position().row + 1,
        node.start_position().column,
        name,
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
    metrics.panic_site_count = count_panic_sites(callable_node, source);
    metrics
}

fn parameter_metrics(node: Node<'_>, source: &[u8]) -> SourceItemMetrics {
    let mut metrics = SourceItemMetrics::default();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if !matches!(
            child.kind(),
            "parameter_declaration" | "variadic_parameter_declaration"
        ) {
            continue;
        }
        let parameter_count = parameter_declaration_count(child);
        metrics.parameter_count += parameter_count;
        if parameter_contains_type(child, source, "bool") {
            metrics.bool_parameter_count += parameter_count;
        }
        if parameter_contains_type(child, source, "string") {
            metrics.raw_string_parameter_count += parameter_count;
        }
    }
    metrics
}

fn parameter_declaration_count(node: Node<'_>) -> usize {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|child| child.kind() == "identifier")
        .count()
        .max(1)
}

fn parameter_contains_type(node: Node<'_>, source: &[u8], name: &str) -> bool {
    node.child_by_field_name("type")
        .is_some_and(|type_node| type_node_contains_name(type_node, source, name))
}

fn type_node_contains_name(node: Node<'_>, source: &[u8], name: &str) -> bool {
    let matches_name = node.kind() == "type_identifier"
        && node
            .utf8_text(source)
            .ok()
            .is_some_and(|text| text.trim() == name);
    if matches_name {
        return true;
    }

    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| type_node_contains_name(child, source, name))
}

fn count_panic_sites(node: Node<'_>, source: &[u8]) -> usize {
    let mut count = usize::from(is_panic_call(node, source));
    let mut cursor = node.walk();
    count += node
        .named_children(&mut cursor)
        .filter(|child| !is_nested_callable_boundary(*child))
        .map(|child| count_panic_sites(child, source))
        .sum::<usize>();
    count
}

fn is_panic_call(node: Node<'_>, source: &[u8]) -> bool {
    if node.kind() != "call_expression" {
        return false;
    }
    node.child_by_field_name("function")
        .or_else(|| first_named_child(node))
        .is_some_and(|function| {
            function.kind() == "identifier"
                && function
                    .utf8_text(source)
                    .ok()
                    .is_some_and(|text| text == "panic")
        })
}

fn cyclomatic_complexity(node: Node<'_>, source: &[u8]) -> usize {
    let mut complexity = 1;
    add_cyclomatic_complexity(node, source, &mut complexity);
    complexity
}

fn add_cyclomatic_complexity(node: Node<'_>, source: &[u8], complexity: &mut usize) {
    match node.kind() {
        "if_statement" | "for_statement" | "communication_case" => {
            *complexity += 1;
        }
        "expression_case" | "type_case" => {
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
            | "select_statement"
            | "switch_statement"
            | "type_switch_statement"
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
    node.kind() == "func_literal"
}

fn first_named_child<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).next()
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
    fn range_loop_counts_as_one_branch() {
        let source = br#"
package demo

func Walk(items []string) {
	for _, item := range items {
		_ = item
	}
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("go grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let walk = metric(&metrics, 4, "Walk")
            .expect("Walk function should be measured");
        assert_eq!(walk.cyclomatic, 2);
        assert_eq!(walk.cognitive, 1);
    }

    #[test]
    fn outer_binary_expression_does_not_inherit_nested_boolean_operator() {
        let source = br#"
package demo

func score(base int, a bool, b bool) int {
	return base + boolToInt(a && b)
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("go grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let score = metric(&metrics, 4, "score")
            .expect("score function should be measured");
        assert_eq!(score.cyclomatic, 2);
        assert_eq!(score.cognitive, 1);
    }

    #[test]
    fn parameter_type_counts_ignore_shadowing_names() {
        let source = br#"
package demo

func Boundary(string int, name string, flag bool) {}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("go grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let boundary = metric(&metrics, 4, "Boundary")
            .expect("Boundary function should be measured");
        assert_eq!(boundary.parameter_count, 3);
        assert_eq!(boundary.raw_string_parameter_count, 1);
        assert_eq!(boundary.bool_parameter_count, 1);
    }

    #[test]
    fn func_literals_do_not_inflate_outer_item_metrics() {
        let source = br#"
package demo

func Outer(flag bool) func() {
	inner := func() {
		if flag {
			panic("nested")
		}
	}
	if flag {
		return inner
	}
	return func() {}
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("go grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let outer = metric(&metrics, 4, "Outer")
            .expect("Outer function should be measured");
        assert_eq!(outer.cyclomatic, 2);
        assert_eq!(outer.cognitive, 1);
        assert_eq!(outer.panic_site_count, 0);
    }

    #[test]
    fn type_switch_cases_count_as_branches() {
        let source = br#"
package demo

func Classify(value any) int {
	switch v := value.(type) {
	case string:
		return len(v)
	case int:
		return v
	default:
		return 0
	}
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("go grammar should load");
        let tree = parser.parse(source, None).expect("source should parse");
        let mut metrics = BTreeMap::new();
        collect_callable_metrics(tree.root_node(), source, &mut metrics);

        let classify = metric(&metrics, 4, "Classify")
            .expect("Classify function should be measured");
        assert_eq!(classify.cyclomatic, 3);
        assert_eq!(classify.cognitive, 1);
    }
}
