use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;

use crate::model::{
    Diagnostic, LintReport, NodeKind, ParsedRepo, SpecDocument, SpecFilter, SpecNode, VerifyRef,
};
use crate::parser::parse_repo;

#[derive(Debug, Clone)]
struct FlatSpecNode {
    id: String,
    kind: NodeKind,
    text: String,
    planned: bool,
    location: crate::model::SourceLocation,
    verifies: Vec<VerifyRef>,
    attests: Vec<crate::model::AttestRef>,
}

pub fn build_spec_document(root: &Path, filter: SpecFilter) -> Result<(SpecDocument, LintReport)> {
    let parsed = parse_repo(root)?;
    let lint = lint_from_parsed(&parsed);
    let document = materialize_spec(&parsed, filter);
    Ok((document, lint))
}

pub fn build_lint_report(root: &Path) -> Result<LintReport> {
    let parsed = parse_repo(root)?;
    Ok(lint_from_parsed(&parsed))
}

fn lint_from_parsed(parsed: &ParsedRepo) -> LintReport {
    let mut diagnostics = parsed.diagnostics.clone();
    let mut declared: BTreeMap<String, (usize, NodeKind)> = BTreeMap::new();

    for spec in &parsed.specs {
        if let Some((previous_line, previous_kind)) =
            declared.insert(spec.id.clone(), (spec.location.line, spec.kind))
        {
            diagnostics.push(Diagnostic {
                path: spec.location.path.clone(),
                line: spec.location.line,
                message: format!(
                    "duplicate node id `{}`; first declared as {} on line {}",
                    spec.id,
                    kind_label(previous_kind),
                    previous_line
                ),
            });
        }
    }

    let ids: BTreeSet<String> = parsed.specs.iter().map(|spec| spec.id.clone()).collect();
    let kinds: BTreeMap<&str, NodeKind> = parsed
        .specs
        .iter()
        .map(|spec| (spec.id.as_str(), spec.kind))
        .collect();

    for spec in &parsed.specs {
        for missing in missing_intermediates(&spec.id, &ids) {
            diagnostics.push(Diagnostic {
                path: spec.location.path.clone(),
                line: spec.location.line,
                message: format!("missing intermediate spec `{missing}`"),
            });
        }
    }

    for verify in &parsed.verifies {
        if !ids.contains(&verify.spec_id) {
            diagnostics.push(Diagnostic {
                path: verify.location.path.clone(),
                line: verify.location.line,
                message: format!(
                    "unknown spec id `{}` referenced by @verifies",
                    verify.spec_id
                ),
            });
        } else if kinds.get(verify.spec_id.as_str()) == Some(&NodeKind::Group) {
            diagnostics.push(Diagnostic {
                path: verify.location.path.clone(),
                line: verify.location.line,
                message: format!(
                    "cannot reference @group `{}` from @verifies",
                    verify.spec_id
                ),
            });
        }
    }

    for attest in &parsed.attests {
        if !ids.contains(&attest.spec_id) {
            diagnostics.push(Diagnostic {
                path: attest.location.path.clone(),
                line: attest.location.line,
                message: format!(
                    "unknown spec id `{}` referenced by @attests",
                    attest.spec_id
                ),
            });
        } else if kinds.get(attest.spec_id.as_str()) == Some(&NodeKind::Group) {
            diagnostics.push(Diagnostic {
                path: attest.location.path.clone(),
                line: attest.location.line,
                message: format!("cannot reference @group `{}` from @attests", attest.spec_id),
            });
        }
    }

    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });
    diagnostics.dedup_by(|left, right| {
        left.path == right.path && left.line == right.line && left.message == right.message
    });

    LintReport { diagnostics }
}

fn materialize_spec(parsed: &ParsedRepo, filter: SpecFilter) -> SpecDocument {
    let mut flat_nodes: BTreeMap<String, FlatSpecNode> = BTreeMap::new();

    for spec in &parsed.specs {
        flat_nodes
            .entry(spec.id.clone())
            .or_insert_with(|| FlatSpecNode {
                id: spec.id.clone(),
                kind: spec.kind,
                text: spec.text.clone(),
                planned: spec.planned,
                location: spec.location.clone(),
                verifies: Vec::new(),
                attests: Vec::new(),
            });
    }

    for verify in &parsed.verifies {
        if verify.body.is_some()
            && let Some(node) = flat_nodes.get_mut(&verify.spec_id)
        {
            node.verifies.push(verify.clone());
        }
    }

    for attest in &parsed.attests {
        if let Some(node) = flat_nodes.get_mut(&attest.spec_id) {
            node.attests.push(attest.clone());
        }
    }

    let directly_visible_ids: BTreeSet<String> = flat_nodes
        .values()
        .filter(|node| {
            node.kind == NodeKind::Spec
                && filter.matches(
                    node.kind,
                    node.planned,
                    node.verifies.is_empty(),
                    node.attests.is_empty(),
                )
        })
        .map(|node| node.id.clone())
        .collect();

    let mut visible_ids = directly_visible_ids.clone();
    for id in &directly_visible_ids {
        let mut parent = immediate_parent(id);
        while let Some(candidate) = parent {
            if flat_nodes.contains_key(candidate) {
                visible_ids.insert(candidate.to_string());
            }
            parent = immediate_parent(candidate);
        }
    }

    let mut children_map: BTreeMap<Option<String>, Vec<String>> = BTreeMap::new();
    for id in &visible_ids {
        let visible_parent = nearest_visible_parent(id, &visible_ids);
        children_map
            .entry(visible_parent)
            .or_default()
            .push(id.clone());
    }

    for children in children_map.values_mut() {
        children.sort();
    }

    let mut nodes = build_children(None, &children_map, &flat_nodes);

    if let Some(scope) = filter.scope.as_deref() {
        nodes = scoped_nodes(nodes, scope);
    }

    SpecDocument { nodes }
}

impl SpecFilter {
    fn matches(
        &self,
        kind: NodeKind,
        planned: bool,
        has_no_verifies: bool,
        has_no_attests: bool,
    ) -> bool {
        if !self.include_planned && planned {
            return false;
        }
        if self.unsupported_only
            && (kind != NodeKind::Spec || planned || !has_no_verifies || !has_no_attests)
        {
            return false;
        }
        true
    }
}

fn nearest_visible_parent(id: &str, visible_ids: &BTreeSet<String>) -> Option<String> {
    let mut parent = immediate_parent(id);
    while let Some(candidate) = parent {
        if visible_ids.contains(candidate) {
            return Some(candidate.to_string());
        }
        parent = immediate_parent(candidate);
    }
    None
}

fn build_children(
    parent: Option<String>,
    children_map: &BTreeMap<Option<String>, Vec<String>>,
    flat_nodes: &BTreeMap<String, FlatSpecNode>,
) -> Vec<SpecNode> {
    let Some(ids) = children_map.get(&parent) else {
        return Vec::new();
    };

    ids.iter()
        .filter_map(|id| flat_nodes.get(id))
        .map(|node| SpecNode {
            id: node.id.clone(),
            kind: node.kind,
            text: node.text.clone(),
            planned: node.planned,
            location: node.location.clone(),
            verifies: node.verifies.clone(),
            attests: node.attests.clone(),
            children: build_children(Some(node.id.clone()), children_map, flat_nodes),
        })
        .collect()
}

fn missing_intermediates(id: &str, declared: &BTreeSet<String>) -> Vec<String> {
    let mut missing = Vec::new();
    let mut cursor = id;
    while let Some(parent) = immediate_parent(cursor) {
        if !declared.contains(parent) {
            missing.push(parent.to_string());
        }
        cursor = parent;
    }
    missing
}

fn immediate_parent(id: &str) -> Option<&str> {
    id.rsplit_once('.').map(|(parent, _)| parent)
}

fn kind_label(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Spec => "@spec",
        NodeKind::Group => "@group",
    }
}

fn scoped_nodes(nodes: Vec<SpecNode>, scope: &str) -> Vec<SpecNode> {
    nodes
        .into_iter()
        .find_map(|node| find_scoped_node(node, scope))
        .into_iter()
        .collect()
}

fn find_scoped_node(node: SpecNode, scope: &str) -> Option<SpecNode> {
    if node.id == scope {
        return Some(node);
    }

    node.children
        .into_iter()
        .find_map(|child| find_scoped_node(child, scope))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::model::{
        AttestRef, NodeKind, ParsedRepo, SourceLocation, SpecDecl, SpecFilter, VerifyRef,
    };

    use super::{build_spec_document, lint_from_parsed, materialize_spec};

    #[test]
    fn filters_planned_specs_by_default() {
        let parsed = ParsedRepo {
            specs: vec![
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Spec,
                    text: "Export root.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 1,
                    },
                },
                SpecDecl {
                    id: "EXPORT.METADATA".to_string(),
                    kind: NodeKind::Spec,
                    text: "Metadata.".to_string(),
                    planned: true,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 3,
                    },
                },
            ],
            verifies: vec![VerifyRef {
                spec_id: "EXPORT".to_string(),
                location: SourceLocation {
                    path: "src/lib.rs".into(),
                    line: 8,
                },
                body_location: None,
                body: Some("fn verifies_export() {}".to_string()),
            }],
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "EXPORT");
    }

    #[test]
    fn includes_planned_specs_with_all_filter() {
        let parsed = ParsedRepo {
            specs: vec![
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Spec,
                    text: "Export root.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 1,
                    },
                },
                SpecDecl {
                    id: "EXPORT.METADATA".to_string(),
                    kind: NodeKind::Spec,
                    text: "Metadata.".to_string(),
                    planned: true,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 3,
                    },
                },
            ],
            verifies: vec![VerifyRef {
                spec_id: "EXPORT".to_string(),
                location: SourceLocation {
                    path: "src/lib.rs".into(),
                    line: 8,
                },
                body_location: None,
                body: Some("fn verifies_export() {}".to_string()),
            }],
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                include_planned: true,
                unsupported_only: false,
                scope: None,
            },
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].children.len(), 1);
        assert_eq!(document.nodes[0].children[0].id, "EXPORT.METADATA");
    }

    #[test]
    fn filters_to_unsupported_live_specs() {
        let parsed = ParsedRepo {
            specs: vec![
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Spec,
                    text: "Export root.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 1,
                    },
                },
                SpecDecl {
                    id: "EXPORT.DOESNTCRASH".to_string(),
                    kind: NodeKind::Spec,
                    text: "No crash.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 3,
                    },
                },
            ],
            verifies: vec![VerifyRef {
                spec_id: "EXPORT.DOESNTCRASH".to_string(),
                location: SourceLocation {
                    path: "src/lib.rs".into(),
                    line: 8,
                },
                body_location: None,
                body: Some("fn verifies_no_crash() {}".to_string()),
            }],
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                include_planned: false,
                unsupported_only: true,
                scope: None,
            },
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "EXPORT");
        assert!(document.nodes[0].is_unsupported());
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.INTERMEDIATE_SPECS
    fn reports_missing_intermediate_specs() {
        let parsed = ParsedRepo {
            specs: vec![SpecDecl {
                id: "EXPORT.DOESNTCRASH".to_string(),
                kind: NodeKind::Spec,
                text: "No crash.".to_string(),
                planned: false,
                location: SourceLocation {
                    path: "src/lib.rs".into(),
                    line: 1,
                },
            }],
            verifies: Vec::new(),
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 1);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("missing intermediate spec `EXPORT`")
        );
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.UNKNOWN_VERIFY_REFS
    fn reports_unknown_verify_refs() {
        let parsed = ParsedRepo {
            specs: vec![SpecDecl {
                id: "EXPORT".to_string(),
                kind: NodeKind::Spec,
                text: "Export root.".to_string(),
                planned: false,
                location: SourceLocation {
                    path: "src/lib.rs".into(),
                    line: 1,
                },
            }],
            verifies: vec![VerifyRef {
                spec_id: "UNKNOWN".to_string(),
                location: SourceLocation {
                    path: "tests/spec.rs".into(),
                    line: 10,
                },
                body_location: None,
                body: Some("fn verifies_unknown() {}".to_string()),
            }],
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 1);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("unknown spec id `UNKNOWN` referenced by @verifies")
        );
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.UNKNOWN_ATTEST_REFS
    fn reports_unknown_attest_refs() {
        let parsed = ParsedRepo {
            specs: vec![SpecDecl {
                id: "EXPORT".to_string(),
                kind: NodeKind::Spec,
                text: "Export root.".to_string(),
                planned: false,
                location: SourceLocation {
                    path: "src/lib.rs".into(),
                    line: 1,
                },
            }],
            verifies: Vec::new(),
            attests: vec![AttestRef {
                spec_id: "UNKNOWN".to_string(),
                artifact: "docs/report.pdf".to_string(),
                owner: "security".to_string(),
                last_reviewed: "2026-04-12".to_string(),
                review_interval_days: None,
                location: SourceLocation {
                    path: "docs/report.txt".into(),
                    line: 10,
                },
                body: Some("@attests UNKNOWN".to_string()),
            }],
            diagnostics: Vec::new(),
        };

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 1);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("unknown spec id `UNKNOWN` referenced by @attests")
        );
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.DUPLICATE_IDS
    fn reports_duplicate_spec_ids() {
        let parsed = ParsedRepo {
            specs: vec![
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Spec,
                    text: "Export root.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 1,
                    },
                },
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Spec,
                    text: "Duplicate export root.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/other.rs".into(),
                        line: 20,
                    },
                },
            ],
            verifies: Vec::new(),
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 1);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("duplicate node id `EXPORT`")
        );
    }

    #[test]
    // @verifies SPECIAL.GROUPS.SPEC_MAY_HAVE_CHILDREN
    fn materializes_nested_tree_structure() {
        let parsed = ParsedRepo {
            specs: vec![
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Spec,
                    text: "Export root.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 1,
                    },
                },
                SpecDecl {
                    id: "EXPORT.DOESNTCRASH".to_string(),
                    kind: NodeKind::Spec,
                    text: "No crash.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 2,
                    },
                },
            ],
            verifies: vec![VerifyRef {
                spec_id: "EXPORT.DOESNTCRASH".to_string(),
                location: SourceLocation {
                    path: "tests/spec.rs".into(),
                    line: 10,
                },
                body_location: None,
                body: Some("fn verifies_export_doesnt_crash() {}".to_string()),
            }],
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "EXPORT");
        assert!(document.nodes[0].is_unsupported());
        assert_eq!(document.nodes[0].children.len(), 1);
        assert_eq!(document.nodes[0].children[0].id, "EXPORT.DOESNTCRASH");
        assert!(!document.nodes[0].children[0].is_unsupported());
    }

    #[test]
    // @verifies SPECIAL.GROUPS.STRUCTURAL_ONLY
    fn materializes_groups_without_marking_them_unsupported() {
        let parsed = ParsedRepo {
            specs: vec![
                SpecDecl {
                    id: "SPECIAL".to_string(),
                    kind: NodeKind::Group,
                    text: "Top-level grouping.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "specs/special.rs".into(),
                        line: 1,
                    },
                },
                SpecDecl {
                    id: "SPECIAL.PARSE".to_string(),
                    kind: NodeKind::Spec,
                    text: "Parses annotated blocks.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "specs/special.rs".into(),
                        line: 3,
                    },
                },
            ],
            verifies: vec![VerifyRef {
                spec_id: "SPECIAL.PARSE".to_string(),
                location: SourceLocation {
                    path: "tests/cli.rs".into(),
                    line: 10,
                },
                body_location: None,
                body: Some("fn verifies_special_parse() {}".to_string()),
            }],
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].kind, NodeKind::Group);
        assert!(!document.nodes[0].is_unsupported());
    }

    #[test]
    // @verifies SPECIAL.GROUPS.MUTUALLY_EXCLUSIVE
    fn rejects_duplicate_ids_across_spec_and_group_kinds() {
        let parsed = ParsedRepo {
            specs: vec![
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Group,
                    text: "Export grouping.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 1,
                    },
                },
                SpecDecl {
                    id: "EXPORT".to_string(),
                    kind: NodeKind::Spec,
                    text: "Export claim.".to_string(),
                    planned: false,
                    location: SourceLocation {
                        path: "src/spec.rs".into(),
                        line: 5,
                    },
                },
            ],
            verifies: Vec::new(),
            attests: Vec::new(),
            diagnostics: Vec::new(),
        };

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 1);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("duplicate node id `EXPORT`")
        );
        assert!(lint.diagnostics[0].message.contains("@group"));
    }

    #[test]
    // @verifies SPECIAL.GROUPS
    fn builds_spec_document_from_repo_files() {
        let root = temp_repo_dir("special-self-host");
        fs::write(
            root.join("spec.rs"),
            r#"/**
@group DEMO
Demo root.

@spec DEMO.OK
Demo child.
*/
"#,
        )
        .expect("spec fixture should be written");
        fs::write(
            root.join("tests.rs"),
            [
                "/",
                "/ @verifies DEMO.OK\n",
                "fn verifies_demo_child() {}\n",
            ]
            .concat(),
        )
        .expect("verify fixture should be written");

        let (document, lint) = build_spec_document(
            &root,
            SpecFilter {
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
        )
        .expect("document build should succeed");

        assert!(lint.diagnostics.is_empty());
        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "DEMO");
        assert_eq!(document.nodes[0].kind, NodeKind::Group);
        assert_eq!(document.nodes[0].verifies.len(), 0);
        assert_eq!(document.nodes[0].children.len(), 1);

        fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.PARSE.MULTI_FILE_TREE
    fn builds_one_tree_from_multiple_files() {
        let root = temp_repo_dir("special-multi-file-tree");
        fs::create_dir_all(root.join("specs")).expect("spec dir should be created");

        fs::write(
            root.join("specs/root.rs"),
            r#"/**
@group DEMO
Demo root.
*/
"#,
        )
        .expect("root spec should be written");
        fs::write(
            root.join("specs/child.rs"),
            r#"/**
@spec DEMO.CHILD
Child claim.
*/
"#,
        )
        .expect("child spec should be written");
        fs::write(
            root.join("checks.rs"),
            [
                "/",
                "/ @verifies DEMO.CHILD\n",
                "fn verifies_demo_child() {}\n",
            ]
            .concat(),
        )
        .expect("verify fixture should be written");

        let (document, lint) = build_spec_document(
            &root,
            SpecFilter {
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
        )
        .expect("document build should succeed");

        assert!(lint.diagnostics.is_empty());
        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "DEMO");
        assert_eq!(document.nodes[0].children.len(), 1);
        assert_eq!(document.nodes[0].children[0].id, "DEMO.CHILD");

        fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
    }

    #[test]
    // @verifies SPECIAL.PARSE.MULTI_FILE_TREE.MIXED_FILE_TYPES
    fn builds_one_tree_from_mixed_supported_file_types() {
        let root = temp_repo_dir("special-mixed-file-tree");
        fs::create_dir_all(root.join("specs")).expect("spec dir should be created");

        fs::write(
            root.join("specs/root.rs"),
            r#"/**
@group DEMO
Demo root.
*/
"#,
        )
        .expect("root spec should be written");
        fs::write(
            root.join("specs/child.sh"),
            "# @spec DEMO.CHILD\n# Child claim.\n",
        )
        .expect("shell child spec should be written");
        fs::write(
            root.join("checks.rs"),
            [
                "/",
                "/ @verifies DEMO.CHILD\n",
                "fn verifies_demo_child() {}\n",
            ]
            .concat(),
        )
        .expect("verify fixture should be written");

        let (document, lint) = build_spec_document(
            &root,
            SpecFilter {
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
        )
        .expect("document build should succeed");

        assert!(lint.diagnostics.is_empty());
        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "DEMO");
        assert_eq!(document.nodes[0].children.len(), 1);
        assert_eq!(document.nodes[0].children[0].id, "DEMO.CHILD");

        fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
    }

    fn temp_repo_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp repo dir should be created");
        path
    }
}
