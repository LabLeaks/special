/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON.AST_BRIDGE
Bridges the built-in Python pack to Python's own `ast` parser through a small helper script so Python syntax extraction can stay self-contained under the pack without embedding a separate parser in the Rust core.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON.AST_BRIDGE
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::syntax::{
    CallSyntaxKind, ParsedSourceGraph, SourceCall, SourceItem, SourceItemKind, SourceInvocation,
    SourceLanguage, SourceSpan,
};

macro_rules! include_str_path {
    ($relative:literal) => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/language_packs/python/",
            $relative
        )
    };
}

pub(super) fn parse_source_graph(path: &Path, text: &str) -> Option<ParsedSourceGraph> {
    parse_source_graph_result(path, text).ok()
}

pub(super) fn parse_source_graph_result(path: &Path, text: &str) -> Result<ParsedSourceGraph> {
    let mut child = Command::new("mise")
        .args([
            "exec",
            "--",
            "python3",
            include_str_path!("parse_source_graph.py"),
            &path.display().to_string(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| anyhow!("failed to launch python syntax bridge: {error}"))?;
    {
        use std::io::Write;

        child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("python syntax bridge stdin was unavailable"))?
            .write_all(text.as_bytes())
            .map_err(|error| anyhow!("failed to write python source to syntax bridge: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| anyhow!("failed to wait for python syntax bridge: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!(
            "python syntax bridge exited with status {}{}",
            output.status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        ));
    }

    let payload: PythonParsedSourceGraph = serde_json::from_slice(&output.stdout)?;
    Ok(ParsedSourceGraph {
        language: SourceLanguage::new("python"),
        items: payload.items.into_iter().map(|item| item.into_source_item(path)).collect(),
    })
}

#[derive(Deserialize)]
struct PythonParsedSourceGraph {
    items: Vec<PythonItem>,
}

#[derive(Deserialize)]
struct PythonItem {
    name: String,
    qualified_name: String,
    module_path: Vec<String>,
    container_path: Vec<String>,
    kind: String,
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,
    start_byte: usize,
    end_byte: usize,
    public: bool,
    root_visible: bool,
    is_test: bool,
    calls: Vec<PythonCall>,
}

impl PythonItem {
    fn into_source_item(self, path: &Path) -> SourceItem {
        let span = SourceSpan {
            start_line: self.start_line,
            end_line: self.end_line,
            start_column: self.start_column,
            end_column: self.end_column,
            start_byte: self.start_byte,
            end_byte: self.end_byte,
        };
        SourceItem {
            source_path: path.display().to_string(),
            stable_id: format!(
                "{}:{}:{}",
                path.display(),
                self.qualified_name,
                self.start_line
            ),
            name: self.name,
            qualified_name: self.qualified_name,
            module_path: self.module_path,
            container_path: self.container_path,
            shape_fingerprint: String::new(),
            shape_node_count: 0,
            kind: match self.kind.as_str() {
                "method" => SourceItemKind::Method,
                _ => SourceItemKind::Function,
            },
            span,
            public: self.public,
            root_visible: self.root_visible,
            is_test: self.is_test,
            calls: self.calls.into_iter().map(PythonCall::into_source_call).collect(),
            invocations: Vec::<SourceInvocation>::new(),
        }
    }
}

#[derive(Deserialize)]
struct PythonCall {
    name: String,
    qualifier: Option<String>,
    syntax: String,
    start_line: usize,
    end_line: usize,
    start_column: usize,
    end_column: usize,
    start_byte: usize,
    end_byte: usize,
}

impl PythonCall {
    fn into_source_call(self) -> SourceCall {
        SourceCall {
            name: self.name,
            qualifier: self.qualifier,
            syntax: match self.syntax.as_str() {
                "field" => CallSyntaxKind::Field,
                "scoped_identifier" => CallSyntaxKind::ScopedIdentifier,
                _ => CallSyntaxKind::Identifier,
            },
            span: SourceSpan {
                start_line: self.start_line,
                end_line: self.end_line,
                start_column: self.start_column,
                end_column: self.end_column,
                start_byte: self.start_byte,
                end_byte: self.end_byte,
            },
        }
    }
}
