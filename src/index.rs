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
use std::path::Path;

use anyhow::Result;

use crate::config::SpecialVersion;
use crate::model::{LintReport, SpecDocument, SpecFilter};
use crate::parser::{ParseDialect, parse_repo};

mod lint;
mod materialize;

use self::lint::lint_from_parsed;
use self::materialize::materialize_spec;

pub fn build_spec_document(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
    filter: SpecFilter,
) -> Result<(SpecDocument, LintReport)> {
    let parsed = parse_repo(root, ignore_patterns, parse_dialect(version))?;
    let lint = lint_from_parsed(&parsed);
    let document = materialize_spec(&parsed, filter);
    Ok((document, lint))
}

pub fn build_lint_report(
    root: &Path,
    ignore_patterns: &[String],
    version: SpecialVersion,
) -> Result<LintReport> {
    let parsed = parse_repo(root, ignore_patterns, parse_dialect(version))?;
    Ok(lint_from_parsed(&parsed))
}

fn parse_dialect(version: SpecialVersion) -> ParseDialect {
    match version {
        SpecialVersion::V0 => ParseDialect::CompatibilityV0,
        SpecialVersion::V1 => ParseDialect::CurrentV1,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::SpecialVersion;
    use crate::model::{
        AttestRef, AttestScope, NodeKind, ParsedRepo, PlanState, SourceLocation, SpecDecl,
        SpecFilter, VerifyRef,
    };

    use super::{build_spec_document, lint_from_parsed, materialize_spec};

    fn spec_decl(id: &str, kind: NodeKind, text: &str, planned: bool, line: usize) -> SpecDecl {
        let plan = if planned {
            PlanState::planned(None)
        } else {
            PlanState::live()
        };
        SpecDecl::new(
            id.to_string(),
            kind,
            text.to_string(),
            plan,
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
    fn filters_planned_specs_by_default() {
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
                    PlanState::live(),
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
        let parsed = parsed_repo(
            vec![
                SpecDecl::new(
                    "SPECIAL".to_string(),
                    NodeKind::Group,
                    "Top-level grouping.".to_string(),
                    PlanState::live(),
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
                    PlanState::live(),
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
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
        );

        assert_eq!(document.nodes.len(), 1);
        assert_eq!(document.nodes[0].kind(), NodeKind::Group);
        assert!(!document.nodes[0].is_unsupported());
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
                    PlanState::live(),
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
                    PlanState::live(),
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
                include_planned: false,
                unsupported_only: false,
                scope: None,
            },
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
            &[],
            SpecialVersion::V1,
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
