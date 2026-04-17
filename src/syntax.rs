/**
@module SPECIAL.SYNTAX
Normalizes parser-specific syntax trees into a shared per-file item and call graph that language modules can populate without leaking raw parser nodes into higher layers.
*/
// @fileimplements SPECIAL.SYNTAX
use std::path::Path;

use tree_sitter::Node;

mod go;
mod rust;
mod typescript;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SourceLanguage {
    Go,
    Rust,
    TypeScript,
}

impl SourceLanguage {
    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("go") => Some(Self::Go),
            Some("rs") => Some(Self::Rust),
            Some("ts" | "tsx") => Some(Self::TypeScript),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SourceItemKind {
    Function,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) start_byte: usize,
    pub(crate) end_byte: usize,
}

impl SourceSpan {
    fn from_tree_sitter(node: tree_sitter::Node<'_>) -> Self {
        Self {
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CallSyntaxKind {
    Identifier,
    ScopedIdentifier,
    Field,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceCall {
    pub(crate) name: String,
    pub(crate) qualifier: Option<String>,
    pub(crate) syntax: CallSyntaxKind,
    pub(crate) span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceInvocationKind {
    LocalCargoBinary { binary_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceInvocation {
    pub(crate) kind: SourceInvocationKind,
    pub(crate) span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceItem {
    pub(crate) source_path: String,
    pub(crate) stable_id: String,
    pub(crate) name: String,
    pub(crate) qualified_name: String,
    pub(crate) module_path: Vec<String>,
    pub(crate) container_path: Vec<String>,
    pub(crate) shape_fingerprint: String,
    pub(crate) shape_node_count: usize,
    pub(crate) kind: SourceItemKind,
    pub(crate) span: SourceSpan,
    pub(crate) public: bool,
    pub(crate) root_visible: bool,
    pub(crate) is_test: bool,
    pub(crate) calls: Vec<SourceCall>,
    pub(crate) invocations: Vec<SourceInvocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedSourceGraph {
    pub(crate) language: SourceLanguage,
    pub(crate) items: Vec<SourceItem>,
}

trait SyntaxProvider {
    fn parse(&self, path: &Path, text: &str) -> Option<ParsedSourceGraph>;
}

pub(crate) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    let language = SourceLanguage::from_path(path)?;
    parse_source_graph_for_language_at_path(language, path, text)
}

#[cfg(test)]
pub(crate) fn parse_source_graph_for_language(
    language: SourceLanguage,
    text: &str,
) -> Option<ParsedSourceGraph> {
    parse_source_graph_for_language_at_path(language, Path::new("lib.rs"), text)
}

fn parse_source_graph_for_language_at_path(
    language: SourceLanguage,
    path: &Path,
    text: &str,
) -> Option<ParsedSourceGraph> {
    match language {
        SourceLanguage::Go => go::GoSyntaxProvider.parse(path, text),
        SourceLanguage::Rust => rust::RustSyntaxProvider.parse(path, text),
        SourceLanguage::TypeScript => typescript::TypeScriptSyntaxProvider.parse(path, text),
    }
}

pub(crate) fn structural_shape(node: Node<'_>) -> (String, usize) {
    let mut kinds = Vec::new();
    collect_structural_shape(node, &mut kinds);
    let node_count = kinds.len();
    (kinds.join(">"), node_count)
}

fn collect_structural_shape(node: Node<'_>, kinds: &mut Vec<String>) {
    kinds.push(node.kind().to_string());
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_structural_shape(child, kinds);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        CallSyntaxKind, SourceInvocationKind, SourceItemKind, SourceLanguage,
        parse_source_graph_for_language, parse_source_graph_for_language_at_path,
    };

    fn item_named<'a>(graph: &'a super::ParsedSourceGraph, name: &str) -> &'a super::SourceItem {
        graph
            .items
            .iter()
            .find(|item| item.name == name)
            .unwrap_or_else(|| panic!("item {name} should be present"))
    }

    fn item_qualified<'a>(
        graph: &'a super::ParsedSourceGraph,
        qualified_name: &str,
    ) -> &'a super::SourceItem {
        graph
            .items
            .iter()
            .find(|item| item.qualified_name == qualified_name)
            .unwrap_or_else(|| panic!("item {qualified_name} should be present"))
    }

    #[test]
    fn go_provider_collects_items_and_calls() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::Go,
            Path::new("app/main.go"),
            r#"
package app

import "fmt"

func Entry() {
    helper()
    fmt.Println("hi")
}

func helper() {}
"#,
        )
        .expect("go graph should parse");

        assert_eq!(graph.items.len(), 2);
        let entry = item_named(&graph, "Entry");
        assert!(entry.public);
        assert_eq!(entry.qualified_name, "app::Entry");
        assert!(entry.calls.iter().any(|call| {
            call.name == "helper"
                && call.qualifier.is_none()
                && call.syntax == CallSyntaxKind::Identifier
        }));
        assert!(entry.calls.iter().any(|call| {
            call.name == "Println"
                && call.qualifier.as_deref() == Some("fmt")
                && call.syntax == CallSyntaxKind::Field
        }));
        let helper = item_named(&graph, "helper");
        assert!(!helper.public);
    }

    #[test]
    fn rust_provider_collects_items_and_calls() {
        let graph = parse_source_graph_for_language(
            SourceLanguage::Rust,
            r#"
use std::process::Command;

fn helper() {}

#[test]
fn verifies_demo() {
    helper();
    crate::shared::work();
    subject.run();
    Command::new(env!("CARGO_BIN_EXE_special")).output();
}
"#,
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 2);
        let helper = item_named(&graph, "helper");
        assert_eq!(helper.qualified_name, "helper");
        assert!(helper.module_path.is_empty());
        assert!(helper.container_path.is_empty());
        assert_eq!(helper.kind, SourceItemKind::Function);
        assert!(!helper.public);
        assert!(!helper.is_test);

        let test_item = item_named(&graph, "verifies_demo");
        assert_eq!(test_item.name, "verifies_demo");
        assert_eq!(test_item.qualified_name, "verifies_demo");
        assert!(test_item.is_test);
        assert!(test_item.calls.iter().any(|call| {
            call.name == "helper"
                && call.qualifier.is_none()
                && call.syntax == CallSyntaxKind::Identifier
        }));
        assert!(test_item.calls.iter().any(|call| {
            call.name == "work"
                && call.qualifier.as_deref() == Some("crate::shared")
                && call.syntax == CallSyntaxKind::ScopedIdentifier
        }));
        assert!(test_item.calls.iter().any(|call| {
            call.name == "run"
                && call.qualifier.as_deref() == Some("subject")
                && call.syntax == CallSyntaxKind::Field
        }));
        assert_eq!(test_item.invocations.len(), 1);
        assert_eq!(
            test_item.invocations[0].kind,
            SourceInvocationKind::LocalCargoBinary {
                binary_name: "special".to_string()
            }
        );
    }

    #[test]
    fn rust_provider_avoids_false_positive_test_and_invocation_detection() {
        let graph = parse_source_graph_for_language(
            SourceLanguage::Rust,
            r#"
#[cfg(not(test))]
fn helper() {
    format!("{}", env!("CARGO_BIN_EXE_special"));
}
"#,
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 1);
        let helper = item_named(&graph, "helper");
        assert!(!helper.is_test);
        assert!(helper.invocations.is_empty());
    }

    #[test]
    fn rust_provider_records_stable_and_qualified_item_names() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::Rust,
            Path::new("src/lib.rs"),
            r#"
mod nested {
    struct Worker;

    impl Worker {
        fn run() {}
    }
}
"#,
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 1);
        let method = item_qualified(&graph, "nested::Worker::run");
        assert_eq!(method.name, "run");
        assert_eq!(method.qualified_name, "nested::Worker::run");
        assert_eq!(method.container_path, vec!["Worker".to_string()]);
        assert!(method.stable_id.contains("nested::Worker::run"));
        assert!(!method.public);
    }

    #[test]
    fn rust_provider_includes_file_module_path_in_qualified_names() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::Rust,
            Path::new("src/render/html.rs"),
            "pub fn render_spec_html() {}\n",
        )
        .expect("rust graph should parse");

        assert_eq!(graph.items.len(), 1);
        let function = item_qualified(&graph, "render::html::render_spec_html");
        assert_eq!(
            function.module_path,
            vec!["render".to_string(), "html".to_string()]
        );
        assert!(function.container_path.is_empty());
        assert_eq!(function.qualified_name, "render::html::render_spec_html");
        assert!(function.public);
        assert!(
            function
                .stable_id
                .contains("render::html::render_spec_html")
        );
    }

    #[test]
    fn typescript_provider_collects_items_and_calls() {
        let graph = parse_source_graph_for_language_at_path(
            SourceLanguage::TypeScript,
            Path::new("src/example.ts"),
            r#"
export function entry() {
    helper();
    api.run();
}

function helper() {}

export const render = () => {
    helper();
};
"#,
        )
        .expect("typescript graph should parse");

        assert_eq!(graph.items.len(), 3);
        let entry = item_named(&graph, "entry");
        assert!(entry.public);
        assert!(
            entry
                .calls
                .iter()
                .any(|call| call.name == "helper" && call.qualifier.is_none())
        );
        assert!(entry.calls.iter().any(|call| {
            call.name == "run"
                && call.qualifier.as_deref() == Some("api")
                && call.syntax == CallSyntaxKind::Field
        }));

        let helper = item_named(&graph, "helper");
        assert!(!helper.public);

        let render = item_named(&graph, "render");
        assert!(render.public);
    }
}
