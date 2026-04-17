/**
@module SPECIAL.SYNTAX.RUST
Builds shared item and call graphs for Rust source files from tree-sitter syntax trees so higher-level analysis can consume normalized structure instead of raw parser nodes.
*/
// @fileimplements SPECIAL.SYNTAX.RUST
use std::path::Path;

use tree_sitter::{Node, Parser};

use super::{
    CallSyntaxKind, ParsedSourceGraph, SourceCall, SourceInvocation, SourceInvocationKind,
    SourceItem, SourceItemKind, SourceLanguage, SourceSpan, SyntaxProvider, structural_shape,
};

pub(super) struct RustSyntaxProvider;

impl SyntaxProvider for RustSyntaxProvider {
    fn parse(&self, path: &Path, text: &str) -> Option<ParsedSourceGraph> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .ok()?;
        let tree = parser.parse(text, None)?;
        let mut items = Vec::new();
        collect_items(path, tree.root_node(), text.as_bytes(), &mut items);
        Some(ParsedSourceGraph {
            language: SourceLanguage::Rust,
            items,
        })
    }
}

fn collect_items(path: &Path, node: Node<'_>, source: &[u8], items: &mut Vec<SourceItem>) {
    if node.kind() == "function_item"
        && let Some(item) = parse_function_item(path, node, source)
    {
        items.push(item);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_items(path, child, source, items);
    }
}

fn parse_function_item(path: &Path, node: Node<'_>, source: &[u8]) -> Option<SourceItem> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let body = node.child_by_field_name("body")?;
    let module_path = item_module_path(path, node, source);
    let container_path = impl_container_segments(node, source);
    let qualified_name = build_qualified_name(&module_path, &container_path, &name);
    let span = SourceSpan::from_tree_sitter(node);
    let (shape_fingerprint, shape_node_count) = structural_shape(node);
    Some(SourceItem {
        source_path: path.display().to_string(),
        stable_id: format!("{}:{}:{}", path.display(), qualified_name, span.start_line),
        name,
        qualified_name,
        module_path,
        container_path,
        shape_fingerprint,
        shape_node_count,
        kind: item_kind(node),
        span,
        public: has_public_visibility(node, source),
        root_visible: has_root_visibility(node, source),
        is_test: has_test_attribute(node, source),
        calls: collect_calls(body, source),
        invocations: collect_invocations(body, source),
    })
}

fn item_kind(node: Node<'_>) -> SourceItemKind {
    let Some(parent) = node.parent() else {
        return SourceItemKind::Function;
    };
    let Some(grandparent) = parent.parent() else {
        return SourceItemKind::Function;
    };

    if parent.kind() == "declaration_list" && grandparent.kind() == "impl_item" {
        SourceItemKind::Method
    } else {
        SourceItemKind::Function
    }
}

fn has_test_attribute(node: Node<'_>, source: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(source) else {
        return false;
    };
    let lines = text.lines().collect::<Vec<_>>();
    let mut line_index = node.start_position().row;
    while line_index > 0 {
        line_index -= 1;
        let trimmed = lines
            .get(line_index)
            .map(|line| line.trim())
            .unwrap_or_default();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("*/")
        {
            continue;
        }
        if trimmed.starts_with("#[") {
            if trimmed.contains("test") {
                return true;
            }
            continue;
        }
        break;
    }
    false
}

fn has_public_visibility(node: Node<'_>, source: &[u8]) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.kind() == "visibility_modifier"
            && child
                .utf8_text(source)
                .ok()
                .is_some_and(|text| text.trim() == "pub")
    })
}

fn has_root_visibility(node: Node<'_>, source: &[u8]) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.kind() == "visibility_modifier"
            && child
                .utf8_text(source)
                .ok()
                .is_some_and(|text| text.trim().starts_with("pub"))
    })
}

fn collect_calls(node: Node<'_>, source: &[u8]) -> Vec<SourceCall> {
    let mut calls = Vec::new();
    collect_calls_inner(node, source, &mut calls);
    calls
}

fn collect_invocations(node: Node<'_>, source: &[u8]) -> Vec<SourceInvocation> {
    let mut invocations = Vec::new();
    collect_invocations_inner(node, source, &mut invocations);
    invocations
}

fn collect_calls_inner(node: Node<'_>, source: &[u8], calls: &mut Vec<SourceCall>) {
    if node.kind() == "call_expression"
        && let Some(function) = node.child_by_field_name("function")
        && let Some((name, qualifier, syntax)) = function_name(function, source)
    {
        calls.push(SourceCall {
            name,
            qualifier,
            syntax,
            span: SourceSpan::from_tree_sitter(function),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_calls_inner(child, source, calls);
    }
}

fn collect_invocations_inner(
    node: Node<'_>,
    source: &[u8],
    invocations: &mut Vec<SourceInvocation>,
) {
    if node.kind() == "call_expression"
        && let Some(binary_name) = local_cargo_binary_invocation(node, source)
    {
        invocations.push(SourceInvocation {
            kind: SourceInvocationKind::LocalCargoBinary { binary_name },
            span: SourceSpan::from_tree_sitter(node),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_invocations_inner(child, source, invocations);
    }
}

fn local_cargo_binary_invocation(node: Node<'_>, source: &[u8]) -> Option<String> {
    let arguments = node.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    for child in arguments.named_children(&mut cursor) {
        if child.kind() != "macro_invocation" {
            continue;
        }
        let text = child.utf8_text(source).ok()?;
        if let Some(binary_name) = text
            .strip_prefix("env!(\"CARGO_BIN_EXE_")
            .and_then(|rest| rest.strip_suffix("\")"))
        {
            return Some(binary_name.to_string());
        }
    }
    None
}

fn function_name(
    node: Node<'_>,
    source: &[u8],
) -> Option<(String, Option<String>, CallSyntaxKind)> {
    match node.kind() {
        "identifier" => Some((
            node.utf8_text(source).ok()?.to_string(),
            None,
            CallSyntaxKind::Identifier,
        )),
        "scoped_identifier" => Some((
            node.child_by_field_name("name")?
                .utf8_text(source)
                .ok()?
                .to_string(),
            Some(
                node.child_by_field_name("path")?
                    .utf8_text(source)
                    .ok()?
                    .to_string(),
            ),
            CallSyntaxKind::ScopedIdentifier,
        )),
        "field_expression" => Some((
            node.child_by_field_name("field")?
                .utf8_text(source)
                .ok()?
                .to_string(),
            Some(
                node.child_by_field_name("value")?
                    .utf8_text(source)
                    .ok()?
                    .to_string(),
            ),
            CallSyntaxKind::Field,
        )),
        "generic_function" => function_name(node.child_by_field_name("function")?, source),
        _ => None,
    }
}

fn build_qualified_name(module_path: &[String], container_path: &[String], name: &str) -> String {
    let mut segments = module_path.to_vec();
    segments.extend(container_path.iter().cloned());
    segments.push(name.to_string());
    segments.join("::")
}

fn item_module_path(path: &Path, node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut segments = file_module_segments(path);
    segments.extend(nested_mod_segments(node, source));
    segments
}

fn file_module_segments(path: &Path) -> Vec<String> {
    let mut normalized = path.components().collect::<Vec<_>>();
    if let Some(index) = normalized
        .iter()
        .position(|component| component.as_os_str() == "src")
    {
        normalized.drain(..=index);
    }

    let mut segments = normalized
        .iter()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return segments;
    }

    let file_name = segments.pop().unwrap_or_default();
    let stem = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or(file_name);
    if stem != "lib" && stem != "main" && stem != "mod" {
        segments.push(stem);
    }
    segments
}

fn nested_mod_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "mod_item"
            && let Some(name) = parent
                .child_by_field_name("name")
                .and_then(|name| name.utf8_text(source).ok())
        {
            segments.push(name.to_string());
        }
        current = parent.parent();
    }
    segments.reverse();
    segments
}

fn impl_container_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "impl_item"
            && let Some(type_name) = parent
                .child_by_field_name("type")
                .and_then(|node| node.utf8_text(source).ok())
        {
            segments.push(type_name.to_string());
        }
        current = parent.parent();
    }
    segments.reverse();
    segments
}
