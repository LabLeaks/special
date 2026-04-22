/**
@module SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.RUST_ANALYZER
Queries `rust-analyzer` for Rust call-hierarchy edges and maps them back onto special's owned item ids without teaching higher analysis layers how to speak LSP.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.ANALYZE.RUST_ANALYZER
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::time::Duration;
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;

use crate::config::ProjectToolchain;
use crate::syntax::{SourceCall, SourceSpan};

#[derive(Debug, Clone)]
pub(super) struct RustAnalyzerCallableItem {
    pub(super) stable_id: String,
    pub(super) name: String,
    pub(super) path: PathBuf,
    pub(super) span: SourceSpan,
    pub(super) calls: Vec<SourceCall>,
    pub(super) invocation_targets: BTreeSet<String>,
}

pub(super) fn build_reachable_call_edges(
    root: &Path,
    items: &[RustAnalyzerCallableItem],
    seed_ids: &BTreeSet<String>,
    parser_call_edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let started_at = Instant::now();
    let (mut client, index) = start_client_for_items(
        root,
        items,
        started_at,
        "starting rust-analyzer call graph",
        seed_ids.len(),
    )?;
    let item_by_id = items
        .iter()
        .map(|item| (item.stable_id.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let mut edges = parser_call_edges.clone();
    let mut visited = BTreeSet::new();
    let mut pending = VecDeque::new();

    for seed_id in seed_ids {
        let seed_callees = parser_call_edges.get(seed_id).cloned().unwrap_or_default();
        for callee in &seed_callees {
            if item_by_id.contains_key(callee) {
                pending.push_back(callee.clone());
            }
        }
        edges
            .entry(seed_id.clone())
            .or_default()
            .extend(seed_callees);
    }

    while let Some(caller_id) = pending.pop_front() {
        if !visited.insert(caller_id.clone()) {
            continue;
        }
        let Some(caller) = item_by_id.get(&caller_id) else {
            continue;
        };

        let mut callees = parser_call_edges
            .get(&caller_id)
            .cloned()
            .unwrap_or_default();
        callees.extend(caller.invocation_targets.clone());
        for target in client.definition_targets(caller)? {
            if let Some(stable_id) = index.resolve(&target)
                && stable_id != caller_id
            {
                callees.insert(stable_id);
            }
        }

        for callee in &callees {
            if item_by_id.contains_key(callee) && !visited.contains(callee) {
                pending.push_back(callee.clone());
            }
        }

        edges.entry(caller_id).or_default().extend(callees);
    }

    crate::modules::analyze::emit_analysis_status(&format!(
        "rust-analyzer resolving incoming callers for {} callable item(s)",
        items.len()
    ));
    for callee in items {
        for caller_id in client.reference_callers(callee, &index)? {
            if caller_id != callee.stable_id {
                edges
                    .entry(caller_id)
                    .or_default()
                    .insert(callee.stable_id.clone());
            }
        }
    }

    client.shutdown()?;
    crate::modules::analyze::emit_analysis_status(&format!(
        "rust-analyzer built reachable call graph in {:.1}s",
        started_at.elapsed().as_secs_f32()
    ));
    Ok(edges)
}

pub(super) fn build_reverse_reachable_call_edges(
    root: &Path,
    items: &[RustAnalyzerCallableItem],
    seed_ids: &BTreeSet<String>,
    parser_call_edges: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let started_at = Instant::now();
    let (mut client, index) = start_client_for_items(
        root,
        items,
        started_at,
        "starting rust-analyzer reverse caller walk",
        seed_ids.len(),
    )?;
    let item_by_id = items
        .iter()
        .map(|item| (item.stable_id.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let mut reverse_parser_edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (caller, callees) in parser_call_edges {
        for callee in callees {
            reverse_parser_edges
                .entry(callee.clone())
                .or_default()
                .insert(caller.clone());
        }
    }
    let mut semantic_edges = BTreeMap::<String, BTreeSet<String>>::new();
    let mut visited = BTreeSet::new();
    let mut pending = seed_ids.iter().cloned().collect::<VecDeque<_>>();

    while let Some(callee_id) = pending.pop_front() {
        if !visited.insert(callee_id.clone()) {
            continue;
        }
        let Some(callee) = item_by_id.get(&callee_id) else {
            continue;
        };

        let mut callers = reverse_parser_edges
            .get(&callee_id)
            .cloned()
            .unwrap_or_default();
        for caller_id in client.reference_callers(callee, &index)? {
            if caller_id != callee_id {
                semantic_edges
                    .entry(caller_id.clone())
                    .or_default()
                    .insert(callee_id.clone());
                callers.insert(caller_id);
            }
        }

        for caller_id in callers {
            if item_by_id.contains_key(&caller_id) && !visited.contains(&caller_id) {
                pending.push_back(caller_id);
            }
        }
    }

    client.shutdown()?;
    crate::modules::analyze::emit_analysis_status(&format!(
        "rust-analyzer built reverse reachable callers in {:.1}s",
        started_at.elapsed().as_secs_f32()
    ));
    Ok(semantic_edges)
}

#[derive(Debug, Clone)]
struct RustAnalyzerTarget {
    path: PathBuf,
    name: String,
    line: usize,
}

#[derive(Debug)]
struct RustAnalyzerItemIndex {
    by_path: BTreeMap<PathBuf, Vec<IndexedItem>>,
}

#[derive(Debug, Clone)]
struct IndexedItem {
    stable_id: String,
    name: String,
    span: SourceSpan,
}

impl RustAnalyzerItemIndex {
    fn new(root: &Path, items: &[RustAnalyzerCallableItem]) -> Self {
        let mut by_path: BTreeMap<PathBuf, Vec<IndexedItem>> = BTreeMap::new();
        for item in items {
            let absolute = normalize_path(root.join(&item.path));
            by_path.entry(absolute).or_default().push(IndexedItem {
                stable_id: item.stable_id.clone(),
                name: item.name.clone(),
                span: item.span,
            });
        }
        Self { by_path }
    }

    fn resolve(&self, target: &RustAnalyzerTarget) -> Option<String> {
        let items = self.by_path.get(&target.path)?;
        let mut exact = items.iter().filter(|item| {
            item.name == target.name
                && target.line >= item.span.start_line
                && target.line <= item.span.end_line
        });
        if let Some(item) = exact.next()
            && exact.next().is_none()
        {
            return Some(item.stable_id.clone());
        }

        let mut fallback = items.iter().filter(|item| {
            target.line >= item.span.start_line && target.line <= item.span.end_line
        });
        if let Some(item) = fallback.next()
            && fallback.next().is_none()
        {
            return Some(item.stable_id.clone());
        }

        None
    }

    fn resolve_containing(&self, path: &Path, line: usize) -> Option<String> {
        let items = self.by_path.get(path)?;
        let mut containing = items
            .iter()
            .filter(|item| line >= item.span.start_line && line <= item.span.end_line);
        if let Some(item) = containing.next()
            && containing.next().is_none()
        {
            return Some(item.stable_id.clone());
        }
        None
    }
}

struct RustAnalyzerClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl RustAnalyzerClient {
    fn start(root: &Path) -> Result<Self> {
        let toolchain = ProjectToolchain::discover(root)?
            .ok_or_else(|| anyhow!("project does not declare a supported toolchain contract"))?;
        let mut child = toolchain
            .command("rust-analyzer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("failed to launch rust-analyzer")?;
        let stdin = child.stdin.take().context("rust-analyzer stdin missing")?;
        let stdout = child
            .stdout
            .take()
            .context("rust-analyzer stdout missing")?;
        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        client.initialize(root)?;
        Ok(client)
    }

    fn initialize(&mut self, root: &Path) -> Result<()> {
        let root = normalize_path(root);
        let root_uri = Url::from_file_path(&root).map_err(|_| {
            anyhow!(
                "failed to build rust-analyzer root uri for {}",
                root.display()
            )
        })?;
        let root_uri_text = root_uri.to_string();
        let params = json!({
            "processId": std::process::id(),
            "rootUri": root_uri_text,
            "workspaceFolders": [{
                "uri": root_uri.to_string(),
                "name": root.file_name().and_then(|name| name.to_str()).unwrap_or("workspace"),
            }],
            "capabilities": {
                "textDocument": {
                    "callHierarchy": {}
                }
            }
        });
        let _ = self.request("initialize", params)?;
        self.notify("initialized", json!({}))?;
        Ok(())
    }

    fn definition_targets(
        &mut self,
        item: &RustAnalyzerCallableItem,
    ) -> Result<Vec<RustAnalyzerTarget>> {
        let absolute = normalize_path(&item.path);
        let uri = Url::from_file_path(&absolute).map_err(|_| {
            anyhow!(
                "failed to build rust-analyzer uri for {}",
                absolute.display()
            )
        })?;
        let uri_text = uri.to_string();
        let mut targets = Vec::new();
        for call in &item.calls {
            let character = query_call_character(&absolute, call)?;
            for attempt in 0..10 {
                let response = self.request(
                    "textDocument/definition",
                    json!({
                        "textDocument": { "uri": uri_text },
                        "position": {
                            "line": call.span.start_line.saturating_sub(1),
                            "character": character,
                        }
                    }),
                );
                let response = match response {
                    Ok(response) => response,
                    Err(error) if is_content_modified_error(&error) && attempt < 9 => {
                        std::thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                let definitions = definition_locations(&response);
                if !definitions.is_empty() {
                    targets.extend(definitions.into_iter().filter_map(|definition| {
                        let uri = Url::parse(&definition.uri).ok()?;
                        let path = uri.to_file_path().ok()?;
                        Some(RustAnalyzerTarget {
                            path: normalize_path(path),
                            name: call.name.clone(),
                            line: definition.range.start.line as usize + 1,
                        })
                    }));
                    break;
                }
                if attempt < 9 {
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }
        Ok(targets)
    }

    fn reference_callers(
        &mut self,
        item: &RustAnalyzerCallableItem,
        index: &RustAnalyzerItemIndex,
    ) -> Result<BTreeSet<String>> {
        let absolute = normalize_path(&item.path);
        let uri = Url::from_file_path(&absolute).map_err(|_| {
            anyhow!(
                "failed to build rust-analyzer uri for {}",
                absolute.display()
            )
        })?;
        let uri_text = uri.to_string();
        let character = query_item_character(&absolute, item)?;
        let mut callers = BTreeSet::new();
        for attempt in 0..10 {
            let response = self.request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": uri_text },
                    "position": {
                        "line": item.span.start_line.saturating_sub(1),
                        "character": character,
                    },
                    "context": {
                        "includeDeclaration": false,
                    }
                }),
            );
            let response = match response {
                Ok(response) => response,
                Err(error) if is_content_modified_error(&error) && attempt < 9 => {
                    std::thread::sleep(Duration::from_millis(200));
                    continue;
                }
                Err(error) => return Err(error),
            };
            for reference in definition_locations(&response) {
                let Ok(uri) = Url::parse(&reference.uri) else {
                    continue;
                };
                let Ok(path) = uri.to_file_path() else {
                    continue;
                };
                if let Some(caller_id) = index.resolve_containing(
                    &normalize_path(path),
                    reference.range.start.line as usize + 1,
                ) {
                    callers.insert(caller_id);
                }
            }
            break;
        }
        Ok(callers)
    }

    fn open_files(&mut self, items: &[RustAnalyzerCallableItem]) -> Result<()> {
        let mut seen = BTreeSet::new();
        for item in items {
            let path = normalize_path(&item.path);
            if !seen.insert(path.clone()) {
                continue;
            }
            let uri = Url::from_file_path(&path)
                .map_err(|_| anyhow!("failed to build rust-analyzer uri for {}", path.display()))?;
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            self.notify(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": uri.to_string(),
                        "languageId": "rust",
                        "version": 0,
                        "text": text,
                    }
                }),
            )?;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        let _ = self.request("shutdown", json!({}));
        let _ = self.notify("exit", json!({}));
        let _ = self.child.wait();
        Ok(())
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }))?;

        loop {
            let message = self.read_message()?;
            if message.get("id").and_then(Value::as_i64) != Some(id) {
                continue;
            }
            if let Some(error) = message.get("error") {
                return Err(anyhow!("rust-analyzer request {method} failed: {error}"));
            }
            return Ok(message.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
    }

    fn write_message(&mut self, message: &Value) -> Result<()> {
        let body = serde_json::to_vec(message)?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n", body.len())?;
        self.stdin.write_all(&body)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_message(&mut self) -> Result<Value> {
        let mut content_length = None;
        loop {
            let mut header = String::new();
            let bytes = self.stdout.read_line(&mut header)?;
            if bytes == 0 {
                return Err(anyhow!("rust-analyzer closed the LSP stream"));
            }
            if header == "\r\n" {
                break;
            }
            if let Some(value) = header.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse::<usize>()?);
            }
        }

        let length = content_length.context("missing rust-analyzer content length")?;
        let mut body = vec![0u8; length];
        self.stdout.read_exact(&mut body)?;
        Ok(serde_json::from_slice(&body)?)
    }
}

impl Drop for RustAnalyzerClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_client_for_items(
    root: &Path,
    items: &[RustAnalyzerCallableItem],
    started_at: Instant,
    phase: &str,
    seed_count: usize,
) -> Result<(RustAnalyzerClient, RustAnalyzerItemIndex)> {
    crate::modules::analyze::emit_analysis_status(&format!(
        "{phase} for {} file(s), {} callable item(s), {} seed root(s)",
        items.iter().map(|item| &item.path).collect::<BTreeSet<_>>().len(),
        items.len(),
        seed_count
    ));
    let mut client = RustAnalyzerClient::start(root)?;
    crate::modules::analyze::emit_analysis_status(&format!(
        "rust-analyzer started in {:.1}s",
        started_at.elapsed().as_secs_f32()
    ));
    client.open_files(items)?;
    std::thread::sleep(Duration::from_millis(500));
    crate::modules::analyze::emit_analysis_status(&format!(
        "rust-analyzer opened {} file(s) in {:.1}s",
        items.iter().map(|item| &item.path).collect::<BTreeSet<_>>().len(),
        started_at.elapsed().as_secs_f32()
    ));
    Ok((client, RustAnalyzerItemIndex::new(root, items)))
}

#[derive(Debug)]
struct DefinitionLocation {
    uri: String,
    range: LspRange,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DefinitionLocationWire {
    Location {
        uri: String,
        range: LspRange,
    },
    LocationLink {
        #[serde(rename = "targetUri")]
        target_uri: String,
        #[serde(rename = "targetSelectionRange")]
        target_selection_range: LspRange,
    },
}

impl From<DefinitionLocationWire> for DefinitionLocation {
    fn from(value: DefinitionLocationWire) -> Self {
        match value {
            DefinitionLocationWire::Location { uri, range } => Self { uri, range },
            DefinitionLocationWire::LocationLink {
                target_uri,
                target_selection_range,
            } => Self {
                uri: target_uri,
                range: target_selection_range,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct LspRange {
    start: LspPosition,
}

#[derive(Debug, Deserialize)]
struct LspPosition {
    line: u32,
}

fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .canonicalize()
        .unwrap_or_else(|_| path.as_ref().to_path_buf())
}

fn query_call_character(path: &Path, call: &SourceCall) -> Result<usize> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let line = text
        .lines()
        .nth(call.span.start_line.saturating_sub(1))
        .unwrap_or_default();
    let start = call.span.start_column.min(line.len());
    let end = call.span.end_column.min(line.len()).max(start);
    let (segment_start, segment) = safe_byte_range(line, start, end);
    Ok(segment
        .find(&call.name)
        .map(|offset| segment_start + offset)
        .unwrap_or(segment_start))
}

fn query_item_character(path: &Path, item: &RustAnalyzerCallableItem) -> Result<usize> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let line = text
        .lines()
        .nth(item.span.start_line.saturating_sub(1))
        .unwrap_or_default();
    let start = item.span.start_column.min(line.len());
    Ok(line
        .get(start..)
        .and_then(|tail| tail.find(&item.name).map(|offset| start + offset))
        .or_else(|| line.find(&item.name))
        .unwrap_or(start))
}

fn definition_locations(response: &Value) -> Vec<DefinitionLocation> {
    if response.is_null() {
        return Vec::new();
    }
    if response.is_array() {
        return serde_json::from_value::<Vec<DefinitionLocationWire>>(response.clone())
            .unwrap_or_default()
            .into_iter()
            .map(DefinitionLocation::from)
            .collect();
    }
    serde_json::from_value::<DefinitionLocationWire>(response.clone())
        .map(|location| vec![DefinitionLocation::from(location)])
        .unwrap_or_default()
}

fn safe_byte_range(line: &str, start: usize, end: usize) -> (usize, &str) {
    if let Some(segment) = line.get(start..end) {
        return (start, segment);
    }
    if let Some(segment) = line.get(start..) {
        return (start, segment);
    }
    (0, line)
}

fn is_content_modified_error(error: &anyhow::Error) -> bool {
    error.to_string().contains("content modified")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use serde_json::json;

    use super::{
        RustAnalyzerCallableItem, RustAnalyzerItemIndex, RustAnalyzerTarget, definition_locations,
    };
    use crate::syntax::SourceSpan;

    #[test]
    fn resolves_target_by_name_and_line() {
        let items = vec![RustAnalyzerCallableItem {
            stable_id: "src/lib.rs:demo::run:3".to_string(),
            name: "run".to_string(),
            path: PathBuf::from("src/lib.rs"),
            calls: Vec::new(),
            span: SourceSpan {
                start_line: 3,
                end_line: 5,
                start_column: 0,
                end_column: 0,
                start_byte: 0,
                end_byte: 0,
            },
            invocation_targets: BTreeSet::new(),
        }];
        let index = RustAnalyzerItemIndex::new(PathBuf::from("/tmp/demo").as_path(), &items);

        let resolved = index.resolve(&RustAnalyzerTarget {
            path: PathBuf::from("/tmp/demo/src/lib.rs"),
            name: "run".to_string(),
            line: 3,
        });

        assert_eq!(resolved.as_deref(), Some("src/lib.rs:demo::run:3"));
    }

    #[test]
    fn definition_locations_accept_location_links() {
        let locations = definition_locations(&json!([{
            "targetUri": "file:///tmp/demo/src/lib.rs",
            "targetSelectionRange": {
                "start": { "line": 7 }
            }
        }]));

        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].uri, "file:///tmp/demo/src/lib.rs");
        assert_eq!(locations[0].range.start.line, 7);
    }
}
