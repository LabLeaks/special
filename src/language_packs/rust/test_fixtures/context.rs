/**
@module SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.CONTEXT
Rust fixture scenarios for repo-health context and support-surface traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.CONTEXT
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_rust_toolchain_contract, write_special_toml,
    write_specs,
};

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_module_analysis_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs"]);
    write_special_toml(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n\n### `@spec APP.PLANNED`\n### `@planned 0.6.0`\nPlanned behavior.\n\n### `@spec APP.DEPRECATED`\n### `@deprecated 0.6.0`\nDeprecated behavior.\n",
    );
    write_file(
        root,
        "main.rs",
        "// @fileimplements DEMO\npub fn live_impl() {}\n\npub fn planned_impl() {}\n\npub fn deprecated_impl() {}\n\npub fn unverified_impl() {}\n\npub fn orphan_impl() {}\n",
    );
    write_file(root, "loose.rs", "pub fn loose_orphan() {}\n");
    write_file(
        root,
        "tests.rs",
        "// @verifies APP.LIVE\n#[test]\nfn verifies_live_impl() {\n    crate::live_impl();\n}\n\n// @verifies APP.PLANNED\n#[test]\nfn verifies_planned_impl() {\n    crate::planned_impl();\n}\n\n// @verifies APP.DEPRECATED\n#[test]\nfn verifies_deprecated_impl() {\n    crate::deprecated_impl();\n}\n\n#[test]\nfn exercises_unverified_impl() {\n    crate::unverified_impl();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_module_context_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src", "tests"]);
    write_rust_toolchain_contract(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.MODULE_CONTEXT`\nRepo traceability exposes whether unexplained items sit in spec-backed modules and whether they connect inside those modules to traced code.\n",
    );
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    );
    write_file(
        root,
        "src/lib.rs",
        "// @fileimplements DEMO\npub fn exercise() {\n    traced_entry();\n}\n\nfn traced_entry() {\n    live_leaf();\n}\n\nfn live_leaf() {}\n\nfn connected_helper() {\n    live_leaf();\n}\n\nfn isolated_helper() {}\n",
    );
    write_file(
        root,
        "tests/module_context.rs",
        "use demo::exercise;\n\n// @verifies APP.MODULE_CONTEXT\n#[test]\nfn verifies_module_context_path() {\n    exercise();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_multiple_supports_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src", "tests"]);
    write_rust_toolchain_contract(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.ALPHA`\nAlpha flow reaches shared implementation.\n\n### `@spec APP.BETA`\nBeta flow reaches shared implementation.\n",
    );
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    );
    write_file(
        root,
        "src/lib.rs",
        "// @fileimplements DEMO\npub fn alpha_entry() {\n    shared_helper();\n}\n\npub fn beta_entry() {\n    shared_helper();\n}\n\nfn shared_helper() {}\n",
    );
    write_file(
        root,
        "tests/alpha.rs",
        "use demo::alpha_entry;\n\n// @verifies APP.ALPHA\n#[test]\nfn verifies_alpha_path() {\n    alpha_entry();\n}\n",
    );
    write_file(
        root,
        "tests/beta.rs",
        "use demo::beta_entry;\n\n// @verifies APP.BETA\n#[test]\nfn verifies_beta_path() {\n    beta_entry();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_review_surface_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src", "tests/support"]);
    write_rust_toolchain_contract(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module DEMO.TESTS`\nDemo test helpers.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.REVIEW_SURFACE`\nRepo traceability review surface excludes public items that only live in test files.\n",
    );
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    );
    write_file(
        root,
        "src/lib.rs",
        "// @fileimplements DEMO\npub fn exercise() {\n    traced_entry();\n}\n\nfn traced_entry() {}\n\npub fn public_orphan() {}\n\nfn internal_orphan() {}\n",
    );
    write_file(
        root,
        "tests/review_surface.rs",
        "use demo::exercise;\n\nmod support;\n\n// @verifies APP.REVIEW_SURFACE\n#[test]\nfn verifies_review_surface_path() {\n    exercise();\n}\n",
    );
    write_file(root, "tests/support/mod.rs", "pub mod helpers;\n");
    write_file(
        root,
        "tests/support/helpers.rs",
        "// @fileimplements DEMO.TESTS\npub fn test_public_orphan() {}\n",
    );
}
