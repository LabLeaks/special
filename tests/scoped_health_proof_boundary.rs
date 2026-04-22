/**
@module SPECIAL.TESTS.SCOPED_HEALTH_PROOF_BOUNDARY
Shared proof-boundary tests that compare full and scoped traceability surfaces across language-pack fixtures.
*/
// @fileimplements SPECIAL.TESTS.SCOPED_HEALTH_PROOF_BOUNDARY
#[allow(dead_code)]
#[path = "../src/language_packs/go/test_fixtures.rs"]
mod go_test_fixtures;
#[allow(dead_code)]
#[path = "../src/language_packs/rust/test_fixtures.rs"]
mod rust_test_fixtures;
#[path = "support/cli.rs"]
mod support;
#[allow(dead_code)]
#[path = "../src/language_packs/typescript/test_fixtures.rs"]
mod typescript_test_fixtures;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use go_test_fixtures::{
    write_go_reference_traceability_fixture, write_go_tool_traceability_fixture,
    write_go_traceability_fixture,
};
use rust_test_fixtures::{
    write_traceability_imported_call_fixture, write_traceability_instance_method_fixture,
    write_traceability_module_context_fixture,
};
use serde_json::Value;
use support::{run_special, temp_repo_dir};
use typescript_test_fixtures::{
    write_typescript_cycle_traceability_fixture, write_typescript_reference_traceability_fixture,
    write_typescript_tool_traceability_fixture, write_typescript_traceability_fixture,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    ScopeRoot,
    Helper,
    Support,
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FileId {
    App,
    Helper,
    Support,
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeRequest {
    File(FileId),
    DirectoryAlpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Bucket {
    CurrentSpec,
    Unexplained,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeBoundary {
    in_scope: BTreeSet<Node>,
    seed: BTreeSet<Node>,
    keep: BTreeSet<Node>,
}

type Graph = BTreeMap<Node, BTreeSet<Node>>;
type Summary = BTreeMap<Bucket, BTreeSet<Node>>;

fn reachable(graph: &Graph, seed: &BTreeSet<Node>) -> BTreeSet<Node> {
    let mut seen = BTreeSet::new();
    let mut pending = seed.iter().copied().collect::<VecDeque<_>>();
    while let Some(node) = pending.pop_front() {
        if !seen.insert(node) {
            continue;
        }
        if let Some(next) = graph.get(&node) {
            pending.extend(next.iter().copied());
        }
    }
    seen
}

fn induced(graph: &Graph, keep: &BTreeSet<Node>) -> Graph {
    let mut reduced = BTreeMap::new();
    for (&from, targets) in graph {
        if !keep.contains(&from) {
            continue;
        }
        let filtered = targets
            .iter()
            .copied()
            .filter(|to| keep.contains(to))
            .collect::<BTreeSet<_>>();
        reduced.insert(from, filtered);
    }
    reduced
}

fn summarize(graph: &Graph, seed: &BTreeSet<Node>, classify: impl Fn(Node) -> Bucket) -> Summary {
    let mut summary = BTreeMap::<Bucket, BTreeSet<Node>>::new();
    for node in reachable(graph, seed) {
        summary.entry(classify(node)).or_default().insert(node);
    }
    summary
}

fn project(summary: &Summary, in_scope: &BTreeSet<Node>) -> Summary {
    summary
        .iter()
        .map(|(&bucket, nodes)| {
            (
                bucket,
                nodes
                    .intersection(in_scope)
                    .copied()
                    .collect::<BTreeSet<_>>(),
            )
        })
        .collect()
}

fn filter_traceability_json_to_path(mut json: Value, scoped_path: &str) -> Value {
    let traceability_value = json
        .get_mut("analysis")
        .and_then(|analysis| analysis.get_mut("traceability"))
        .expect("proof boundary requires analysis.traceability to be present");

    let traceability = traceability_value
        .as_object_mut()
        .expect("traceability summary should be an object");

    for key in [
        "current_spec_items",
        "planned_only_items",
        "deprecated_only_items",
        "file_scoped_only_items",
        "unverified_test_items",
        "statically_mediated_items",
        "unexplained_items",
    ] {
        if let Some(items) = traceability.get_mut(key).and_then(Value::as_array_mut) {
            items.retain(|item| item["path"] == scoped_path);
        }
    }

    let (
        unexplained_len,
        unexplained_review_surface_items,
        unexplained_public_items,
        unexplained_test_file_items,
        unexplained_module_owned_items,
        unexplained_unowned_items,
        unexplained_module_backed_items,
        unexplained_module_connected_items,
        unexplained_module_isolated_items,
    ) = {
        let unexplained_items = traceability["unexplained_items"]
            .as_array()
            .expect("unexplained_items should stay an array");
        let count_true = |field: &str| -> u64 {
            unexplained_items
                .iter()
                .filter(|item| item[field].as_bool().unwrap_or(false))
                .count() as u64
        };
        let count_module_ids = |is_empty: bool| -> u64 {
            unexplained_items
                .iter()
                .filter(|item| {
                    item["module_ids"]
                        .as_array()
                        .map_or(is_empty, |ids| ids.is_empty() == is_empty)
                })
                .count() as u64
        };
        let count_backed_and_connected = |connected: bool| -> u64 {
            unexplained_items
                .iter()
                .filter(|item| {
                    item["module_backed_by_current_specs"]
                        .as_bool()
                        .unwrap_or(false)
                        && item["module_connected_to_current_specs"]
                            .as_bool()
                            .unwrap_or(false)
                            == connected
                })
                .count() as u64
        };

        (
            unexplained_items.len() as u64,
            count_true("review_surface"),
            count_true("public"),
            count_true("test_file"),
            count_module_ids(false),
            count_module_ids(true),
            count_true("module_backed_by_current_specs"),
            count_backed_and_connected(true),
            count_backed_and_connected(false),
        )
    };

    let analyzed_items = [
        "current_spec_items",
        "planned_only_items",
        "deprecated_only_items",
        "file_scoped_only_items",
        "unverified_test_items",
        "statically_mediated_items",
        "unexplained_items",
    ]
    .into_iter()
    .flat_map(|key| {
        traceability[key]
            .as_array()
            .expect("bucket should stay array")
            .iter()
    })
    .map(traceability_identity_key)
    .collect::<std::collections::BTreeSet<_>>()
    .len() as u64;
    traceability.insert("analyzed_items".to_string(), Value::from(analyzed_items));
    traceability.insert(
        "unexplained_review_surface_items".to_string(),
        Value::from(unexplained_review_surface_items),
    );
    traceability.insert(
        "unexplained_public_items".to_string(),
        Value::from(unexplained_public_items),
    );
    traceability.insert(
        "unexplained_internal_items".to_string(),
        Value::from(unexplained_len - unexplained_public_items),
    );
    traceability.insert(
        "unexplained_test_file_items".to_string(),
        Value::from(unexplained_test_file_items),
    );
    traceability.insert(
        "unexplained_module_owned_items".to_string(),
        Value::from(unexplained_module_owned_items),
    );
    traceability.insert(
        "unexplained_unowned_items".to_string(),
        Value::from(unexplained_unowned_items),
    );
    traceability.insert(
        "unexplained_module_backed_items".to_string(),
        Value::from(unexplained_module_backed_items),
    );
    traceability.insert(
        "unexplained_module_connected_items".to_string(),
        Value::from(unexplained_module_connected_items),
    );
    traceability.insert(
        "unexplained_module_isolated_items".to_string(),
        Value::from(unexplained_module_isolated_items),
    );
    json
}

fn traceability_identity_key(item: &Value) -> String {
    serde_json::to_string(item).expect("traceability item identity should serialize")
}

fn assert_scoped_traceability_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    let root = temp_repo_dir(fixture_name);
    fixture_writer(&root);

    let full_output = run_special(&root, &["health", "--json", "--verbose"]);
    assert!(
        full_output.status.success(),
        "full health failed: {}",
        String::from_utf8_lossy(&full_output.stderr)
    );
    let scoped_output = run_special(&root, &["health", scoped_path, "--json", "--verbose"]);
    assert!(
        scoped_output.status.success(),
        "scoped health failed: {}",
        String::from_utf8_lossy(&scoped_output.stderr)
    );

    let full_json: Value =
        serde_json::from_slice(&full_output.stdout).expect("full health output should be json");
    let scoped_json: Value =
        serde_json::from_slice(&scoped_output.stdout).expect("scoped health output should be json");

    let full_unavailable = !full_json["analysis"]["traceability_unavailable_reason"].is_null();
    let scoped_unavailable = !scoped_json["analysis"]["traceability_unavailable_reason"].is_null();
    assert_eq!(
        scoped_unavailable, full_unavailable,
        "scoped and full availability should agree"
    );

    assert!(
        !full_unavailable,
        "proof boundary requires real traceability execution; full unavailable reason: {:?}, scoped unavailable reason: {:?}",
        full_json["analysis"]["traceability_unavailable_reason"].as_str(),
        scoped_json["analysis"]["traceability_unavailable_reason"].as_str(),
    );

    let projected = filter_traceability_json_to_path(full_json, scoped_path);
    assert_eq!(
        scoped_json["analysis"]["traceability"],
        projected["analysis"]["traceability"]
    );

    std::fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn assert_scoped_traceability_matches_full_then_filtered_or_reports_unavailable(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
    expected_unavailable_reason: &str,
) {
    let root = temp_repo_dir(&format!("{fixture_name}-availability-check"));
    fixture_writer(&root);

    let output = run_special(&root, &["health", scoped_path, "--json", "--verbose"]);
    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("scoped health output should be json");
    let unavailable = json["analysis"]["traceability_unavailable_reason"]
        .as_str()
        .is_some_and(|reason| reason.contains(expected_unavailable_reason));

    std::fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
    if unavailable {
        panic!(
            "proof-boundary runtime did not execute for {fixture_name}: expected live scoped equality check, got unavailable traceability surface containing `{expected_unavailable_reason}`"
        );
    }
    assert_scoped_traceability_matches_full_then_filtered(
        fixture_name,
        fixture_writer,
        scoped_path,
    );
}

fn assert_scoped_rust_traceability_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    assert_scoped_traceability_matches_full_then_filtered_or_reports_unavailable(
        fixture_name,
        fixture_writer,
        scoped_path,
        "Rust backward trace",
    )
}

fn assert_scoped_typescript_traceability_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    assert_scoped_traceability_matches_full_then_filtered_or_reports_unavailable(
        fixture_name,
        fixture_writer,
        scoped_path,
        "TypeScript backward trace",
    )
}

fn assert_scoped_go_traceability_matches_full_then_filtered(
    fixture_name: &str,
    fixture_writer: fn(&std::path::Path),
    scoped_path: &str,
) {
    assert_scoped_traceability_matches_full_then_filtered_or_reports_unavailable(
        fixture_name,
        fixture_writer,
        scoped_path,
        "Go backward trace",
    )
}

fn fixture_graph() -> Graph {
    BTreeMap::from([
        (Node::ScopeRoot, BTreeSet::from([Node::Helper])),
        (Node::Helper, BTreeSet::from([Node::Support])),
        (Node::Support, BTreeSet::new()),
        (Node::Detached, BTreeSet::new()),
    ])
}

fn fixture_boundary() -> ScopeBoundary {
    let graph = fixture_graph();
    let seed = BTreeSet::from([Node::ScopeRoot]);
    ScopeBoundary {
        in_scope: BTreeSet::from([Node::ScopeRoot, Node::Helper, Node::Support]),
        seed: seed.clone(),
        keep: reachable(&graph, &seed),
    }
}

fn classify(node: Node) -> Bucket {
    match node {
        Node::Support => Bucket::CurrentSpec,
        Node::ScopeRoot | Node::Helper | Node::Detached => Bucket::Unexplained,
    }
}

fn file_of(node: Node) -> FileId {
    match node {
        Node::ScopeRoot => FileId::App,
        Node::Helper => FileId::Helper,
        Node::Support => FileId::Support,
        Node::Detached => FileId::Detached,
    }
}

fn derive_seed_from_request(request: ScopeRequest) -> BTreeSet<Node> {
    match request {
        ScopeRequest::File(FileId::App) | ScopeRequest::DirectoryAlpha => {
            BTreeSet::from([Node::ScopeRoot])
        }
        ScopeRequest::File(FileId::Helper) => BTreeSet::from([Node::Helper]),
        ScopeRequest::File(FileId::Support) => BTreeSet::from([Node::Support]),
        ScopeRequest::File(FileId::Detached) => BTreeSet::from([Node::Detached]),
    }
}

fn project_for_request(summary: &Summary, request: ScopeRequest) -> Summary {
    let in_scope = match request {
        ScopeRequest::File(file) => summary
            .values()
            .flatten()
            .copied()
            .filter(|node| file_of(*node) == file)
            .collect::<BTreeSet<_>>(),
        ScopeRequest::DirectoryAlpha => {
            BTreeSet::from([Node::ScopeRoot, Node::Helper, Node::Support])
        }
    };
    project(summary, &in_scope)
}

fn project_with_exact_match_only(summary: &Summary, request: ScopeRequest) -> Summary {
    let in_scope = match request {
        ScopeRequest::File(file) => summary
            .values()
            .flatten()
            .copied()
            .filter(|node| file_of(*node) == file)
            .collect::<BTreeSet<_>>(),
        ScopeRequest::DirectoryAlpha => BTreeSet::new(),
    };
    project(summary, &in_scope)
}

#[test]
fn exact_kept_closure_preserves_projected_traceability_summary() {
    let graph = fixture_graph();
    let boundary = fixture_boundary();

    let full = project(
        &summarize(&graph, &boundary.seed, classify),
        &boundary.in_scope,
    );
    let scoped = project(
        &summarize(&induced(&graph, &boundary.keep), &boundary.seed, classify),
        &boundary.in_scope,
    );

    assert_eq!(scoped, full);
}

#[test]
fn inexact_kept_closure_changes_projected_traceability_summary() {
    let graph = fixture_graph();
    let mut boundary = fixture_boundary();
    boundary.keep.remove(&Node::Support);

    let full = project(
        &summarize(&graph, &boundary.seed, classify),
        &boundary.in_scope,
    );
    let scoped = project(
        &summarize(&induced(&graph, &boundary.keep), &boundary.seed, classify),
        &boundary.in_scope,
    );

    assert_ne!(scoped, full);
    assert_eq!(
        full.get(&Bucket::CurrentSpec),
        Some(&BTreeSet::from([Node::Support]))
    );
    assert!(!scoped.contains_key(&Bucket::CurrentSpec));
}

#[test]
fn seed_equivalent_derivations_preserve_traceability_summary() {
    let graph = fixture_graph();
    let direct_seed = BTreeSet::from([Node::ScopeRoot]);
    let derived_but_equivalent_seed = BTreeSet::from([Node::ScopeRoot, Node::Helper]);

    let direct = summarize(&graph, &direct_seed, classify);
    let derived = summarize(&graph, &derived_but_equivalent_seed, classify);

    assert_eq!(direct, derived);
}

#[test]
fn scoped_file_request_with_exact_closure_matches_full_then_filter() {
    let graph = fixture_graph();
    let request = ScopeRequest::File(FileId::App);
    let seed = derive_seed_from_request(request);
    let keep = reachable(&graph, &seed);

    let full_then_filter = project_for_request(&summarize(&graph, &seed, classify), request);
    let closure_then_filter = project_for_request(
        &summarize(&induced(&graph, &keep), &seed, classify),
        request,
    );

    assert_eq!(closure_then_filter, full_then_filter);
}

#[test]
fn inconsistent_scope_matching_changes_the_projected_summary() {
    let graph = fixture_graph();
    let request = ScopeRequest::DirectoryAlpha;
    let seed = derive_seed_from_request(request);
    let full = summarize(&graph, &seed, classify);

    let prefix_projection = project_for_request(&full, request);
    let exact_match_projection = project_with_exact_match_only(&full, request);

    assert_ne!(prefix_projection, exact_match_projection);
    assert_eq!(
        prefix_projection.get(&Bucket::CurrentSpec),
        Some(&BTreeSet::from([Node::Support]))
    );
    assert_eq!(
        exact_match_projection.get(&Bucket::CurrentSpec),
        Some(&BTreeSet::new())
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_typescript_fixture() {
    assert_scoped_typescript_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-typescript",
        write_typescript_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_typescript_tool_fixture() {
    assert_scoped_typescript_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-typescript-tool",
        write_typescript_tool_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_typescript_reference_fixture() {
    assert_scoped_typescript_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-typescript-reference",
        write_typescript_reference_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_typescript_cycle_fixture() {
    assert_scoped_typescript_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-typescript-cycle",
        write_typescript_cycle_traceability_fixture,
        "src/app.ts",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_go_fixture() {
    assert_scoped_go_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-go",
        write_go_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_go_tool_fixture() {
    assert_scoped_go_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-go-tool",
        write_go_tool_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_go_reference_fixture() {
    assert_scoped_go_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-go-reference",
        write_go_reference_traceability_fixture,
        "app/main.go",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_rust_imported_call_fixture() {
    assert_scoped_rust_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-rust-imported-call",
        write_traceability_imported_call_fixture,
        "src/render.rs",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_rust_module_context_fixture() {
    assert_scoped_rust_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-rust-module-context",
        write_traceability_module_context_fixture,
        "src/lib.rs",
    );
}

#[test]
fn scoped_cli_matches_full_then_filtered_traceability_on_rust_instance_method_fixture() {
    assert_scoped_rust_traceability_matches_full_then_filtered(
        "special-scoped-proof-boundary-rust-instance-method",
        write_traceability_instance_method_fixture,
        "src/lib.rs",
    );
}
