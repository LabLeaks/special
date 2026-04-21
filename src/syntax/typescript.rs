/**
@module SPECIAL.SYNTAX.TYPESCRIPT
Builds shared item and call graphs for TypeScript source files from tree-sitter syntax trees so higher-level analysis can consume normalized structure instead of raw parser nodes.
*/
// @fileimplements SPECIAL.SYNTAX.TYPESCRIPT
use std::collections::BTreeSet;
use std::path::Path;

use tree_sitter::{Node, Parser};

use super::{
    CallSyntaxKind, ParsedSourceGraph, SourceItem, SourceItemKind, SourceLanguage, SourceSpan,
    SyntaxProvider, build_qualified_name, collect_calls_with, structural_shape,
};

pub(crate) struct TypeScriptSyntaxProvider;

impl SyntaxProvider for TypeScriptSyntaxProvider {
    fn parse(&self, path: &Path, text: &str) -> Option<ParsedSourceGraph> {
        let mut parser = Parser::new();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("tsx") => parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
                .ok()?,
            _ => parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .ok()?,
        };
        let tree = parser.parse(text, None)?;
        let mut items = Vec::new();
        let exported_names = collect_exported_names(tree.root_node(), text.as_bytes());
        collect_items(
            path,
            tree.root_node(),
            text.as_bytes(),
            &exported_names,
            &mut items,
        );
        Some(ParsedSourceGraph {
            language: SourceLanguage::new("typescript"),
            items,
        })
    }
}

pub(crate) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    TypeScriptSyntaxProvider.parse(path, text)
}

fn collect_items(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    exported_names: &BTreeSet<String>,
    items: &mut Vec<SourceItem>,
) {
    match node.kind() {
        "function_declaration" => {
            if let Some(item) = parse_function_declaration(path, node, source, exported_names) {
                items.push(item);
            }
        }
        "method_definition" => {
            if let Some(item) = parse_method_definition(path, node, source) {
                items.push(item);
            }
        }
        "variable_declarator" => {
            if let Some(item) = parse_function_variable(path, node, source, exported_names) {
                items.push(item);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_items(path, child, source, exported_names, items);
    }
}

fn parse_function_declaration(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    exported_names: &BTreeSet<String>,
) -> Option<SourceItem> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    let body = node.child_by_field_name("body")?;
    Some(build_item(
        path,
        node,
        source,
        name.clone(),
        body,
        TypeScriptItemMeta {
            container_path: Vec::new(),
            kind: SourceItemKind::Function,
            public: is_exported(node, &name, exported_names),
        },
    ))
}

fn parse_method_definition(path: &Path, node: Node<'_>, source: &[u8]) -> Option<SourceItem> {
    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .trim_matches('"')
        .to_string();
    let body = node.child_by_field_name("body")?;
    let container_path = class_container_segments(node, source);
    Some(build_item(
        path,
        node,
        source,
        name,
        body,
        TypeScriptItemMeta {
            container_path,
            kind: SourceItemKind::Method,
            public: method_is_public(node, source),
        },
    ))
}

fn parse_function_variable(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    exported_names: &BTreeSet<String>,
) -> Option<SourceItem> {
    let value = node.child_by_field_name("value")?;
    if value.kind() != "arrow_function" && value.kind() != "function" {
        return None;
    }

    let name = node
        .child_by_field_name("name")?
        .utf8_text(source)
        .ok()?
        .to_string();
    Some(build_item(
        path,
        node,
        source,
        name.clone(),
        value,
        TypeScriptItemMeta {
            container_path: Vec::new(),
            kind: SourceItemKind::Function,
            public: is_exported(node, &name, exported_names),
        },
    ))
}

struct TypeScriptItemMeta {
    container_path: Vec<String>,
    kind: SourceItemKind,
    public: bool,
}

fn build_item(
    path: &Path,
    node: Node<'_>,
    source: &[u8],
    name: String,
    body: Node<'_>,
    meta: TypeScriptItemMeta,
) -> SourceItem {
    let module_path = file_module_segments(path);
    let qualified_name = build_qualified_name(&module_path, &meta.container_path, &name);
    let span = SourceSpan::from_tree_sitter(node);
    let (shape_fingerprint, shape_node_count) = structural_shape(node);
    SourceItem {
        source_path: path.display().to_string(),
        stable_id: format!("{}:{}:{}", path.display(), qualified_name, span.start_line),
        name,
        qualified_name,
        module_path,
        container_path: meta.container_path,
        shape_fingerprint,
        shape_node_count,
        kind: meta.kind,
        span,
        public: meta.public,
        root_visible: meta.public,
        is_test: is_typescript_test_item(path),
        calls: collect_calls_with(body, source, function_name),
        invocations: Vec::new(),
    }
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
        "member_expression" => Some((
            node.child_by_field_name("property")?
                .utf8_text(source)
                .ok()?
                .trim_matches('"')
                .to_string(),
            Some(
                node.child_by_field_name("object")?
                    .utf8_text(source)
                    .ok()?
                    .to_string(),
            ),
            CallSyntaxKind::Field,
        )),
        "parenthesized_expression" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .next()
                .and_then(|child| function_name(child, source))
        }
        _ => None,
    }
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
    if stem != "index" {
        segments.push(stem);
    }
    segments
}

fn is_exported(node: Node<'_>, name: &str, exported_names: &BTreeSet<String>) -> bool {
    let mut current = Some(node);
    while let Some(cursor) = current {
        if cursor.kind() == "export_statement" {
            return true;
        }
        current = cursor.parent();
    }
    exported_names.contains(name)
}

fn collect_exported_names(root: Node<'_>, source: &[u8]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        if child.kind() != "export_statement" {
            continue;
        }
        collect_exported_names_from_statement(child, source, &mut names);
    }
    names
}

fn collect_exported_names_from_statement(
    node: Node<'_>,
    source: &[u8],
    names: &mut BTreeSet<String>,
) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "export_clause" => collect_exported_names_from_clause(child, source, names),
            "function_declaration" | "variable_declarator" | "class_declaration" => {
                if let Some(name) = child
                    .child_by_field_name("name")
                    .and_then(|value| value.utf8_text(source).ok())
                {
                    names.insert(name.trim_matches('"').to_string());
                }
            }
            _ => {}
        }
    }
}

fn collect_exported_names_from_clause(node: Node<'_>, source: &[u8], names: &mut BTreeSet<String>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "export_specifier" {
            if let Some(name) = child
                .child_by_field_name("name")
                .or_else(|| child.child_by_field_name("alias"))
                .or_else(|| {
                    let mut inner = child.walk();
                    child.named_children(&mut inner).next()
                })
                .and_then(|value| value.utf8_text(source).ok())
            {
                names.insert(name.trim_matches('"').to_string());
            }
        }
    }
}

fn class_container_segments(node: Node<'_>, source: &[u8]) -> Vec<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == "class_body"
            && let Some(class_decl) = parent.parent()
            && class_decl.kind() == "class_declaration"
            && let Some(name) = class_decl
                .child_by_field_name("name")
                .and_then(|name| name.utf8_text(source).ok())
        {
            return vec![name.to_string()];
        }
        current = parent.parent();
    }
    Vec::new()
}

fn method_is_public(node: Node<'_>, source: &[u8]) -> bool {
    if !has_export_ancestor(node) {
        return false;
    }
    let mut cursor = node.walk();
    !node.children(&mut cursor).any(|child| {
        child.kind() == "accessibility_modifier"
            && child
                .utf8_text(source)
                .ok()
                .is_some_and(|text| matches!(text.trim(), "private" | "protected"))
    })
}

fn has_export_ancestor(node: Node<'_>) -> bool {
    let mut current = Some(node);
    while let Some(cursor) = current {
        if cursor.kind() == "export_statement" {
            return true;
        }
        current = cursor.parent();
    }
    false
}

fn is_typescript_test_item(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    file_name.ends_with(".test.ts")
        || file_name.ends_with(".test.tsx")
        || file_name.ends_with(".spec.ts")
        || file_name.ends_with(".spec.tsx")
}
