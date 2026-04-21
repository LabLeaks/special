/**
@module SPECIAL.INDEX
Coordinates dialect selection, lint assembly, and spec-tree materialization over parsed repo annotations.

@spec SPECIAL.PARSE.MULTI_FILE_TREE
special builds one spec tree from declarations spread across multiple files.

@spec SPECIAL.PARSE.MULTI_FILE_TREE.MIXED_FILE_TYPES
special builds one spec tree from declarations spread across supported file types.

@spec SPECIAL.GROUPS
special supports structural group nodes that organize claims without making direct claims of their own.

@spec SPECIAL.GROUPS.STRUCTURAL_ONLY
special treats @group as structure-only and does not require verifies, attests, or planned markers on group nodes.

@spec SPECIAL.GROUPS.SPEC_MAY_HAVE_CHILDREN
special allows @spec nodes to have children while still remaining direct claims that need their own verifies, attests, or planned marker.

@spec SPECIAL.GROUPS.MUTUALLY_EXCLUSIVE
special does not allow the same node id to be declared as both @spec and @group.
*/
// @fileimplements SPECIAL.INDEX
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::cache::load_or_parse_repo;
use crate::config::SpecialVersion;
use crate::model::{
    AttestScope, GroupedCount, LintReport, ParsedRepo, SpecDocument, SpecFilter, SpecMetricsSummary,
};

mod lint;
mod materialize;

use self::lint::lint_from_parsed;
use self::materialize::materialize_spec;

pub fn build_spec_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    filter: SpecFilter,
    metrics: bool,
) -> Result<(SpecDocument, LintReport)> {
    let parsed = load_or_parse_repo(root, ignore_patterns, version)?;
    let lint = lint_from_parsed(&parsed);
    let document = materialize_spec(&parsed, filter, metrics, Some(root));
    Ok((document, lint))
}

pub fn build_lint_report(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<LintReport> {
    let parsed = load_or_parse_repo(root, ignore_patterns, version)?;
    Ok(lint_from_parsed(&parsed))
}

pub(crate) fn build_spec_document_from_parsed(
    parsed: &ParsedRepo,
    filter: SpecFilter,
    metrics: bool,
) -> SpecDocument {
    materialize_spec(parsed, filter, metrics, None)
}

pub(crate) fn build_lint_report_from_parsed(parsed: &ParsedRepo) -> LintReport {
    lint_from_parsed(parsed)
}

pub(crate) fn build_spec_metrics(
    root: Option<&Path>,
    nodes: &[crate::model::SpecNode],
) -> SpecMetricsSummary {
    let spec_nodes = collect_spec_nodes(nodes);
    SpecMetricsSummary {
        total_specs: spec_nodes.len(),
        planned_specs: spec_nodes.iter().filter(|node| node.is_planned()).count(),
        deprecated_specs: spec_nodes
            .iter()
            .filter(|node| node.is_deprecated())
            .count(),
        unverified_specs: spec_nodes
            .iter()
            .filter(|node| node.is_unverified())
            .count(),
        verified_specs: spec_nodes
            .iter()
            .filter(|node| !node.verifies.is_empty())
            .count(),
        attested_specs: spec_nodes
            .iter()
            .filter(|node| !node.attests.is_empty())
            .count(),
        specs_with_both_supports: spec_nodes
            .iter()
            .filter(|node| !node.verifies.is_empty() && !node.attests.is_empty())
            .count(),
        verifies: spec_nodes.iter().map(|node| node.verifies.len()).sum(),
        item_scoped_verifies: spec_nodes
            .iter()
            .flat_map(|node| node.verifies.iter())
            .filter(|verify| verify.body_location.is_some())
            .count(),
        file_scoped_verifies: spec_nodes
            .iter()
            .flat_map(|node| node.verifies.iter())
            .filter(|verify| verify.body_location.is_none() && verify.body.is_some())
            .count(),
        unattached_verifies: spec_nodes
            .iter()
            .flat_map(|node| node.verifies.iter())
            .filter(|verify| verify.body_location.is_none() && verify.body.is_none())
            .count(),
        attests: spec_nodes.iter().map(|node| node.attests.len()).sum(),
        block_attests: spec_nodes
            .iter()
            .flat_map(|node| node.attests.iter())
            .filter(|attest| attest.scope == AttestScope::Block)
            .count(),
        file_attests: spec_nodes
            .iter()
            .flat_map(|node| node.attests.iter())
            .filter(|attest| attest.scope == AttestScope::File)
            .count(),
        specs_by_file: grouped_counts(
            spec_nodes
                .iter()
                .map(|node| relative_path_display(root, &node.location.path)),
        ),
        current_specs_by_top_level_id: grouped_counts(
            spec_nodes
                .iter()
                .filter(|node| !node.is_planned() && !node.is_deprecated())
                .map(|node| top_level_id(&node.id)),
        ),
    }
}

fn relative_path_display(root: Option<&Path>, path: &Path) -> String {
    root.and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn collect_spec_nodes(nodes: &[crate::model::SpecNode]) -> Vec<&crate::model::SpecNode> {
    let mut collected = Vec::new();
    append_spec_nodes(nodes, &mut collected);
    collected
}

fn append_spec_nodes<'a>(
    nodes: &'a [crate::model::SpecNode],
    collected: &mut Vec<&'a crate::model::SpecNode>,
) {
    nodes.iter().for_each(|node| {
        if node.kind() == crate::model::NodeKind::Spec {
            collected.push(node);
        }
        append_spec_nodes(&node.children, collected);
    });
}

fn top_level_id(id: &str) -> String {
    id.split('.').next().unwrap_or(id).to_string()
}

fn grouped_counts(values: impl Iterator<Item = String>) -> Vec<GroupedCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(value, count)| GroupedCount { value, count })
        .collect()
}
#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::SpecialVersion;
    use crate::model::{
        AttestRef, AttestScope, DeclaredStateFilter, NodeKind, ParsedRepo, PlanState,
        SourceLocation, SpecDecl, SpecFilter, VerifyRef,
    };

    use super::{build_spec_document, lint_from_parsed, materialize_spec};

    fn spec_decl(id: &str, kind: NodeKind, text: &str, planned: bool, line: usize) -> SpecDecl {
        let plan = if planned {
            PlanState::planned(None)
        } else {
            PlanState::current()
        };
        SpecDecl::new(
            id.to_string(),
            kind,
            text.to_string(),
            plan,
            false,
            None,
            SourceLocation {
                path: "src/lib.rs".into(),
                line,
            },
        )
        .expect("test helper should construct valid spec decls")
    }

    fn verify_ref(spec_id: &str, path: &str, line: usize, body: &str) -> VerifyRef {
        VerifyRef {
            spec_id: spec_id.to_string(),
            location: SourceLocation {
                path: path.into(),
                line,
            },
            body_location: None,
            body: Some(body.to_string()),
        }
    }

    fn parsed_repo(
        specs: Vec<SpecDecl>,
        verifies: Vec<VerifyRef>,
        attests: Vec<AttestRef>,
    ) -> ParsedRepo {
        ParsedRepo {
            specs,
            verifies,
            attests,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn filters_planned_specs_with_current_filter() {
        let parsed = parsed_repo(
            vec![
                spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
                spec_decl("EXPORT.METADATA", NodeKind::Spec, "Metadata.", true, 3),
            ],
            vec![verify_ref(
                "EXPORT",
                "src/lib.rs",
                8,
                "fn verifies_export() {}",
            )],
            Vec::new(),
        );

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: false,
                scope: None,
            },
            false,
            None,
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "EXPORT");
    }

    #[test]
    fn includes_planned_specs_by_default() {
        let parsed = parsed_repo(
            vec![
                spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
                spec_decl("EXPORT.METADATA", NodeKind::Spec, "Metadata.", true, 3),
            ],
            vec![verify_ref(
                "EXPORT",
                "src/lib.rs",
                8,
                "fn verifies_export() {}",
            )],
            Vec::new(),
        );

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                state: DeclaredStateFilter::All,
                unverified_only: false,
                scope: None,
            },
            false,
            None,
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].children.len(), 1);
        assert_eq!(document.nodes[0].children[0].id, "EXPORT.METADATA");
    }

    #[test]
    fn filters_to_unverified_current_specs() {
        let parsed = parsed_repo(
            vec![
                spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
                spec_decl("EXPORT.DOESNTCRASH", NodeKind::Spec, "No crash.", false, 3),
            ],
            vec![verify_ref(
                "EXPORT.DOESNTCRASH",
                "src/lib.rs",
                8,
                "fn verifies_no_crash() {}",
            )],
            Vec::new(),
        );

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: true,
                scope: None,
            },
            false,
            None,
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "EXPORT");
        assert!(document.nodes[0].is_unverified());
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.INTERMEDIATE_SPECS
    fn reports_missing_intermediate_specs() {
        let parsed = parsed_repo(
            vec![spec_decl(
                "EXPORT.DOESNTCRASH",
                NodeKind::Spec,
                "No crash.",
                false,
                1,
            )],
            Vec::new(),
            Vec::new(),
        );

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
        let parsed = parsed_repo(
            vec![spec_decl(
                "EXPORT",
                NodeKind::Spec,
                "Export root.",
                false,
                1,
            )],
            vec![verify_ref(
                "UNKNOWN",
                "tests/spec.rs",
                10,
                "fn verifies_unknown() {}",
            )],
            Vec::new(),
        );

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
        let parsed = parsed_repo(
            vec![spec_decl(
                "EXPORT",
                NodeKind::Spec,
                "Export root.",
                false,
                1,
            )],
            Vec::new(),
            vec![
                AttestRef {
                    spec_id: "UNKNOWN".to_string(),
                    artifact: "docs/report.pdf".to_string(),
                    owner: "security".to_string(),
                    last_reviewed: "2026-04-12".to_string(),
                    review_interval_days: None,
                    scope: AttestScope::Block,
                    location: SourceLocation {
                        path: "docs/report.txt".into(),
                        line: 10,
                    },
                    body: Some("@attests UNKNOWN".to_string()),
                },
                AttestRef {
                    spec_id: "ALSO_UNKNOWN".to_string(),
                    artifact: "docs/review.md".to_string(),
                    owner: "security".to_string(),
                    last_reviewed: "2026-04-12".to_string(),
                    review_interval_days: None,
                    scope: AttestScope::File,
                    location: SourceLocation {
                        path: "docs/review.md".into(),
                        line: 1,
                    },
                    body: Some("# Review".to_string()),
                },
            ],
        );

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 2);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("unknown spec id `UNKNOWN` referenced by @attests")
        );
        assert!(
            lint.diagnostics[1]
                .message
                .contains("unknown spec id `ALSO_UNKNOWN` referenced by @fileattests")
        );
    }

    #[test]
    // @verifies SPECIAL.LINT_COMMAND.DUPLICATE_IDS
    fn reports_duplicate_spec_ids() {
        let parsed = parsed_repo(
            vec![
                spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
                SpecDecl::new(
                    "EXPORT".to_string(),
                    NodeKind::Spec,
                    "Duplicate export root.".to_string(),
                    PlanState::current(),
                    false,
                    None,
                    SourceLocation {
                        path: "src/other.rs".into(),
                        line: 20,
                    },
                )
                .expect("test should construct valid spec decl"),
            ],
            Vec::new(),
            Vec::new(),
        );

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 1);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("duplicate node id `EXPORT`")
        );
        assert!(lint.diagnostics[0].message.contains("src/lib.rs:1"));
    }

    #[test]
    // @verifies SPECIAL.GROUPS.SPEC_MAY_HAVE_CHILDREN
    fn materializes_nested_tree_structure() {
        let parsed = parsed_repo(
            vec![
                spec_decl("EXPORT", NodeKind::Spec, "Export root.", false, 1),
                spec_decl("EXPORT.DOESNTCRASH", NodeKind::Spec, "No crash.", false, 2),
            ],
            vec![verify_ref(
                "EXPORT.DOESNTCRASH",
                "tests/spec.rs",
                10,
                "fn verifies_export_doesnt_crash() {}",
            )],
            Vec::new(),
        );

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: false,
                scope: None,
            },
            false,
            None,
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "EXPORT");
        assert!(document.nodes[0].is_unverified());
        assert_eq!(document.nodes[0].children.len(), 1);
        assert_eq!(document.nodes[0].children[0].id, "EXPORT.DOESNTCRASH");
        assert!(!document.nodes[0].children[0].is_unverified());
    }

    #[test]
    // @verifies SPECIAL.GROUPS.STRUCTURAL_ONLY
    fn materializes_groups_without_marking_them_unsupported() {
        let parsed = parsed_repo(
            vec![
                SpecDecl::new(
                    "SPECIAL".to_string(),
                    NodeKind::Group,
                    "Top-level grouping.".to_string(),
                    PlanState::current(),
                    false,
                    None,
                    SourceLocation {
                        path: "specs/special.rs".into(),
                        line: 1,
                    },
                )
                .expect("test should construct valid group decl"),
                SpecDecl::new(
                    "SPECIAL.PARSE".to_string(),
                    NodeKind::Spec,
                    "Parses annotated blocks.".to_string(),
                    PlanState::current(),
                    false,
                    None,
                    SourceLocation {
                        path: "specs/special.rs".into(),
                        line: 3,
                    },
                )
                .expect("test should construct valid spec decl"),
            ],
            vec![verify_ref(
                "SPECIAL.PARSE",
                "tests/cli.rs",
                10,
                "fn verifies_special_parse() {}",
            )],
            Vec::new(),
        );

        let document = materialize_spec(
            &parsed,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: false,
                scope: None,
            },
            false,
            None,
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].kind(), NodeKind::Group);
        assert!(!document.nodes[0].is_unverified());
    }

    #[test]
    // @verifies SPECIAL.GROUPS.MUTUALLY_EXCLUSIVE
    fn rejects_duplicate_ids_across_spec_and_group_kinds() {
        let parsed = parsed_repo(
            vec![
                SpecDecl::new(
                    "EXPORT".to_string(),
                    NodeKind::Group,
                    "Export grouping.".to_string(),
                    PlanState::current(),
                    false,
                    None,
                    SourceLocation {
                        path: "src/lib.rs".into(),
                        line: 1,
                    },
                )
                .expect("test should construct valid group decl"),
                SpecDecl::new(
                    "EXPORT".to_string(),
                    NodeKind::Spec,
                    "Export claim.".to_string(),
                    PlanState::current(),
                    false,
                    None,
                    SourceLocation {
                        path: "src/spec.rs".into(),
                        line: 5,
                    },
                )
                .expect("test should construct valid spec decl"),
            ],
            Vec::new(),
            Vec::new(),
        );

        let lint = lint_from_parsed(&parsed);
        assert_eq!(lint.diagnostics.len(), 1);
        assert!(
            lint.diagnostics[0]
                .message
                .contains("duplicate node id `EXPORT`")
        );
        assert!(lint.diagnostics[0].message.contains("@group"));
        assert!(lint.diagnostics[0].message.contains("src/lib.rs:1"));
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
            &[],
            SpecialVersion::V1,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: false,
                scope: None,
            },
            false,
        )
        .expect("document build should succeed");

        assert!(lint.diagnostics.is_empty());
        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].id, "DEMO");
        assert_eq!(document.nodes[0].kind(), NodeKind::Group);
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
            &[],
            SpecialVersion::V1,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: false,
                scope: None,
            },
            false,
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
            &[],
            SpecialVersion::V1,
            SpecFilter {
                state: DeclaredStateFilter::Current,
                unverified_only: false,
                scope: None,
            },
            false,
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
