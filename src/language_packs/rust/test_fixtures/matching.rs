/**
@module SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.MATCHING
Rust fixture scenarios for direct matching, qualification, and transitive traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.MATCHING
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_special_toml, write_specs,
};

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_file_verify_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs"]);
    write_special_toml(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.FILE`\nFile scoped behavior.\n",
    );
    write_file(
        root,
        "main.rs",
        "// @fileimplements DEMO\npub fn broad_impl() {}\n\npub fn second_impl() {}\n",
    );
    write_file(
        root,
        "tests.rs",
        "// @fileverifies APP.FILE\n#[test]\nfn covers_broad_impl() {\n    crate::broad_impl();\n}\n\n#[test]\nfn covers_second_impl() {\n    crate::second_impl();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_name_collision_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs"]);
    write_special_toml(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.COLLISION`\nCollision behavior.\n",
    );
    write_file(
        root,
        "one.rs",
        "// @fileimplements DEMO\npub fn shared() {}\n",
    );
    write_file(
        root,
        "two.rs",
        "// @fileimplements DEMO\npub fn shared() {}\n",
    );
    write_file(
        root,
        "tests.rs",
        "// @verifies APP.COLLISION\n#[test]\nfn verifies_shared_collision() {\n    crate::shared();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_qualified_match_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs"]);
    write_special_toml(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.NESTED`\nNested function behavior.\n\n### `@spec APP.METHOD`\nQualified method behavior.\n",
    );
    write_file(
        root,
        "main.rs",
        "// @fileimplements DEMO\npub mod nested {\n    pub fn helper() {}\n\n    pub struct Worker;\n\n    impl Worker {\n        pub fn run() {}\n    }\n}\n\npub mod sibling {\n    pub fn helper() {}\n\n    pub struct Worker;\n\n    impl Worker {\n        pub fn run() {}\n    }\n}\n",
    );
    write_file(
        root,
        "tests.rs",
        "// @verifies APP.NESTED\n#[test]\nfn verifies_nested_helper() {\n    crate::nested::helper();\n}\n\n// @verifies APP.METHOD\n#[test]\nfn verifies_nested_worker_run() {\n    crate::nested::Worker::run();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_transitive_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs"]);
    write_special_toml(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.TRANSITIVE`\nTransitive traceability behavior.\n",
    );
    write_file(
        root,
        "main.rs",
        "// @fileimplements DEMO\npub fn helper_impl() {\n    leaf_impl();\n}\n\npub fn leaf_impl() {}\n\npub fn orphan_impl() {}\n",
    );
    write_file(
        root,
        "tests.rs",
        "// @verifies APP.TRANSITIVE\n#[test]\nfn verifies_transitive_leaf() {\n    crate::helper_impl();\n}\n",
    );
}
