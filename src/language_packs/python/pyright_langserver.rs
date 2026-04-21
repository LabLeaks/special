/**
@module SPECIAL.LANGUAGE_PACKS.PYTHON.PYRIGHT_LANGSERVER
Queries `pyright-langserver` for Python definition and reference edges and maps them back onto special's owned Python item ids without teaching higher layers how to speak LSP.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON.PYRIGHT_LANGSERVER
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;

use crate::syntax::{SourceCall, SourceSpan};

#[derive(Debug, Clone)]
pub(super) struct PyrightCallableItem {
    pub(super) stable_id: String,
    pub(super) name: String,
    pub(super) qualified_name: String,
    pub(super) path: PathBuf,
    pub(super) span: SourceSpan,
    pub(super) calls: Vec<SourceCall>,
    pub(super) is_test: bool,
}

pub(super) fn available() -> bool {
    tool_path("node").is_some() && tool_path("pyright-langserver").is_some()
}

pub(super) fn environment_fingerprint() -> String {
    let node = tool_path("node")
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unavailable".to_string());
    let pyright = tool_path("pyright-langserver")
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unavailable".to_string());
    format!("node={node};pyright_langserver={pyright}")
}

pub(super) fn build_reachable_call_edges(
    root: &Path,
    items: &[PyrightCallableItem],
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut client = PyrightClient::start(root)?;
    client.open_files(items)?;
    std::thread::sleep(Duration::from_millis(1500));

    let item_by_id = items
        .iter()
        .map(|item| (item.stable_id.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let index = PyrightItemIndex::new(root, items);
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let seed_ids = items
        .iter()
        .filter(|item| item.is_test)
        .map(|item| item.stable_id.clone())
        .collect::<BTreeSet<_>>();
    let mut visited = BTreeSet::new();
    let mut pending = VecDeque::from_iter(seed_ids.iter().cloned());

    while let Some(caller_id) = pending.pop_front() {
        if !visited.insert(caller_id.clone()) {
            continue;
        }
        let Some(caller) = item_by_id.get(&caller_id) else {
            continue;
        };

        let mut callees = BTreeSet::new();
        for target in client.definition_targets(caller)? {
            if let Some(stable_id) = index.resolve(&target) {
                if stable_id != caller_id {
                    callees.insert(stable_id);
                }
            }
        }

        for callee in &callees {
            if item_by_id.contains_key(callee) && !visited.contains(callee) {
                pending.push_back(callee.clone());
            }
        }

        if !callees.is_empty() {
            edges.entry(caller_id).or_default().extend(callees);
        }
    }

    for callee in items.iter().filter(|item| !item.is_test) {
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
    Ok(edges)
}

#[derive(Debug, Clone)]
struct PyrightTarget {
    path: PathBuf,
    name: String,
    line: usize,
}

#[derive(Debug)]
struct PyrightItemIndex {
    by_path: BTreeMap<PathBuf, Vec<IndexedItem>>,
}

#[derive(Debug, Clone)]
struct IndexedItem {
    stable_id: String,
    name: String,
    qualified_name: String,
    span: SourceSpan,
}

impl PyrightItemIndex {
    fn new(root: &Path, items: &[PyrightCallableItem]) -> Self {
        let mut by_path: BTreeMap<PathBuf, Vec<IndexedItem>> = BTreeMap::new();
        for item in items {
            let absolute = normalize_path(root.join(&item.path));
            by_path.entry(absolute).or_default().push(IndexedItem {
                stable_id: item.stable_id.clone(),
                name: item.name.clone(),
                qualified_name: item.qualified_name.clone(),
                span: item.span,
            });
        }
        Self { by_path }
    }

    fn resolve(&self, target: &PyrightTarget) -> Option<String> {
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

        let mut qualified_fallback = items.iter().filter(|item| {
            item.qualified_name
                .rsplit("::")
                .next()
                .is_some_and(|name| name == target.name)
                && target.line >= item.span.start_line
                && target.line <= item.span.end_line
        });
        if let Some(item) = qualified_fallback.next()
            && qualified_fallback.next().is_none()
        {
            return Some(item.stable_id.clone());
        }

        let mut line_only = items.iter().filter(|item| {
            target.line >= item.span.start_line && target.line <= item.span.end_line
        });
        if let Some(item) = line_only.next()
            && line_only.next().is_none()
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

struct PyrightClient {
    child: Child,
    stderr: BufReader<ChildStderr>,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
    workspace_root: PathBuf,
}

impl PyrightClient {
    fn start(root: &Path) -> Result<Self> {
        let node = tool_path("node").context("node is unavailable through mise")?;
        let langserver =
            tool_path("pyright-langserver").context("pyright-langserver is unavailable through mise")?;
        let mut child = Command::new(node)
            .arg(langserver)
            .arg("--stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to launch pyright-langserver")?;
        let stderr = child.stderr.take().context("pyright stderr missing")?;
        let stdin = child.stdin.take().context("pyright stdin missing")?;
        let stdout = child.stdout.take().context("pyright stdout missing")?;
        let mut client = Self {
            child,
            stderr: BufReader::new(stderr),
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            workspace_root: root.to_path_buf(),
        };
        client.initialize(root)?;
        Ok(client)
    }

    fn initialize(&mut self, root: &Path) -> Result<()> {
        let root = normalize_path(root);
        let root_uri = Url::from_file_path(&root)
            .map_err(|_| anyhow!("failed to build pyright root uri for {}", root.display()))?;
        let params = json!({
            "processId": std::process::id(),
            "rootUri": root_uri.to_string(),
            "workspaceFolders": [{
                "uri": root_uri.to_string(),
                "name": root.file_name().and_then(|name| name.to_str()).unwrap_or("workspace"),
            }],
            "capabilities": {
                "textDocument": {}
            }
        });
        let _ = self.request("initialize", params)?;
        self.notify("initialized", json!({}))?;
        Ok(())
    }

    fn definition_targets(&mut self, item: &PyrightCallableItem) -> Result<Vec<PyrightTarget>> {
        let absolute = self.resolve_workspace_path(&item.path);
        let uri = Url::from_file_path(&absolute)
            .map_err(|_| anyhow!("failed to build pyright uri for {}", absolute.display()))?;
        let uri_text = uri.to_string();
        let mut targets = Vec::new();
        for call in &item.calls {
            let character = query_call_character(&absolute, call)?;
            for attempt in 0..5 {
                let response = self.request(
                    "textDocument/definition",
                    json!({
                        "textDocument": { "uri": uri_text },
                        "position": {
                            "line": call.span.start_line.saturating_sub(1),
                            "character": character,
                        }
                    }),
                )?;
                let definitions = definition_locations(&response);
                if definitions.is_empty() && attempt < 4 {
                    std::thread::sleep(Duration::from_millis(300));
                    continue;
                }
                targets.extend(definitions.into_iter().filter_map(|definition| {
                    let uri = Url::parse(&definition.uri).ok()?;
                    let path = uri.to_file_path().ok()?;
                    Some(PyrightTarget {
                        path: normalize_path(path),
                        name: call.name.clone(),
                        line: definition.range.start.line as usize + 1,
                    })
                }));
                break;
            }
        }
        Ok(targets)
    }

    fn reference_callers(
        &mut self,
        item: &PyrightCallableItem,
        index: &PyrightItemIndex,
    ) -> Result<BTreeSet<String>> {
        let absolute = self.resolve_workspace_path(&item.path);
        let uri = Url::from_file_path(&absolute)
            .map_err(|_| anyhow!("failed to build pyright uri for {}", absolute.display()))?;
        let character = query_item_character(&absolute, item)?;
        let mut callers = BTreeSet::new();
        for attempt in 0..5 {
            let response = self.request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": uri.to_string() },
                    "position": {
                        "line": item.span.start_line.saturating_sub(1),
                        "character": character,
                    },
                    "context": {
                        "includeDeclaration": false,
                    }
                }),
            )?;

            let references = definition_locations(&response);
            if references.is_empty() && attempt < 4 {
                std::thread::sleep(Duration::from_millis(300));
                continue;
            }
            for reference in references {
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

    fn open_files(&mut self, items: &[PyrightCallableItem]) -> Result<()> {
        let mut seen = BTreeSet::new();
        for item in items {
            let path = self.resolve_workspace_path(&item.path);
            if !seen.insert(path.clone()) {
                continue;
            }
            let uri = Url::from_file_path(&path)
                .map_err(|_| anyhow!("failed to build pyright uri for {}", path.display()))?;
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            self.notify(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": uri.to_string(),
                        "languageId": "python",
                        "version": 0,
                        "text": text,
                    }
                }),
            )?;
        }
        Ok(())
    }

    fn resolve_workspace_path(&self, path: &Path) -> PathBuf {
        normalize_path(self.workspace_root.join(path))
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
                return Err(anyhow!("pyright request {method} failed: {error}"));
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
                let mut stderr = String::new();
                let _ = self.stderr.read_to_string(&mut stderr);
                if stderr.trim().is_empty() {
                    return Err(anyhow!("pyright closed the LSP stream"));
                }
                return Err(anyhow!(
                    "pyright closed the LSP stream: {}",
                    stderr.trim()
                ));
            }
            if header == "\r\n" {
                break;
            }
            if let Some(value) = header.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse::<usize>()?);
            }
        }

        let length = content_length.context("missing pyright content length")?;
        let mut body = vec![0u8; length];
        self.stdout.read_exact(&mut body)?;
        Ok(serde_json::from_slice(&body)?)
    }
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

fn tool_path(tool: &str) -> Option<PathBuf> {
    which_via_mise(tool).or_else(|| which_on_path(tool))
}

fn which_via_mise(tool: &str) -> Option<PathBuf> {
    let mut command = Command::new("mise");
    command.args(["exec", "--", "which", tool]);
    if let Some(cwd) = special_runtime_root() {
        command.current_dir(cwd);
    }
    let output = command.output().ok()?;
    if output.status.success() {
        parse_which_output(&output.stdout)
    } else {
        None
    }
}

fn which_on_path(tool: &str) -> Option<PathBuf> {
    let output = Command::new("which").arg(tool).output().ok()?;
    if output.status.success() {
        parse_which_output(&output.stdout)
    } else {
        None
    }
}

fn parse_which_output(output: &[u8]) -> Option<PathBuf> {
    let path = String::from_utf8(output.to_vec()).ok()?;
    let trimmed = path.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

fn special_runtime_root() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    executable.ancestors().find_map(|ancestor| {
        let cargo_toml = ancestor.join("Cargo.toml");
        let src = ancestor.join("src");
        (cargo_toml.is_file() && src.is_dir()).then(|| ancestor.to_path_buf())
    })
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
    let segment = &line[start..end];
    Ok(segment
        .find(&call.name)
        .map(|offset| start + offset)
        .unwrap_or(start))
}

fn query_item_character(path: &Path, item: &PyrightCallableItem) -> Result<usize> {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::definition_locations;

    #[test]
    fn definition_locations_accept_location_links() {
        let locations = definition_locations(&json!([{
            "targetUri": "file:///tmp/demo/src/sample.py",
            "targetSelectionRange": {
                "start": { "line": 11 }
            }
        }]));

        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].uri, "file:///tmp/demo/src/sample.py");
        assert_eq!(locations[0].range.start.line, 11);
    }
}
