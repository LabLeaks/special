/**
@module SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.DISPATCH
Rust fixture scenarios for call routing, mediation, and method-dispatch traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.DISPATCH
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_rust_toolchain_contract, write_special_toml,
    write_specs,
};

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_imported_call_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src", "tests"]);
    write_rust_toolchain_contract(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.RENDER`\nImported function calls participate in traceability.\n",
    );
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    );
    write_file(
        root,
        "src/lib.rs",
        "mod render;\n\nuse render::render_entry;\n\n// @fileimplements DEMO\npub fn run() {\n    render_entry();\n}\n",
    );
    write_file(
        root,
        "src/render.rs",
        "// @fileimplements DEMO\npub fn render_entry() {\n    live_impl();\n}\n\npub fn live_impl() {}\n\npub fn orphan_impl() {}\n",
    );
    write_file(
        root,
        "tests/render.rs",
        "use demo::run;\n\n// @verifies APP.RENDER\n#[test]\nfn verifies_render_path() {\n    run();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_mediated_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_rust_toolchain_contract(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.RENDER`\nStatically mediated trait entrypoints are separated from unexplained items.\n",
    );
    write_file(
        root,
        "src/lib.rs",
        "use std::fmt;\n\n// @fileimplements DEMO\npub fn render_summary() -> String {\n    format!(\"{}\", Report)\n}\n\nstruct Report;\n\nimpl fmt::Display for Report {\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n        f.write_str(\"report\")\n    }\n}\n\npub fn orphan_impl() {}\n",
    );
    write_file(
        root,
        "tests.rs",
        "// @verifies APP.RENDER\n#[test]\nfn verifies_render_summary() {\n    let _ = demo::render_summary();\n}\n",
    );
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_cross_file_module_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "render"]);
    write_special_toml(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.CROSS_FILE`\nCross-file module-path traceability behavior.\n",
    );
    write_file(
        root,
        "render/mod.rs",
        "// @fileimplements DEMO\npub mod common;\npub mod html;\n\npub fn render_entry() {\n    html::render_spec_html();\n}\n",
    );
    write_file(
        root,
        "render/html.rs",
        "// @fileimplements DEMO\npub fn render_spec_html() {\n    super::common::helper_impl();\n}\n\npub fn orphan_impl() {}\n",
    );
    write_file(
        root,
        "render/common.rs",
        "// @fileimplements DEMO\npub fn helper_impl() {\n    live_impl();\n}\n\npub fn live_impl() {}\n",
    );
    write_file(
        root,
        "tests.rs",
        "// @verifies APP.CROSS_FILE\n#[test]\nfn verifies_cross_file_render_path() {\n    crate::render::render_entry();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_self_method_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs"]);
    write_special_toml(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.SELF_METHOD`\nSelf and Self dispatch traceability behavior.\n",
    );
    write_file(
        root,
        "main.rs",
        "// @fileimplements DEMO\npub struct Worker;\n\nimpl Worker {\n    pub fn run() {\n        Self::helper();\n    }\n\n    fn helper() {\n        Self::leaf();\n    }\n\n    fn leaf() {}\n\n    fn unknown() {}\n}\n",
    );
    write_file(
        root,
        "tests.rs",
        "// @verifies APP.SELF_METHOD\n#[test]\nfn verifies_self_method_path() {\n    crate::Worker::run();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_instance_method_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src", "tests"]);
    write_rust_toolchain_contract(root);
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.INSTANCE_METHOD`\nInstance-method dispatch traceability behavior.\n",
    );
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n",
    );
    write_file(
        root,
        "src/lib.rs",
        "// @fileimplements DEMO\npub fn exercise() {\n    let worker = Worker;\n    worker.run();\n}\n\npub struct Worker;\n\nimpl Worker {\n    fn run(&self) {\n        helper();\n    }\n}\n\nfn helper() {}\n\nfn orphan_impl() {}\n",
    );
    write_file(
        root,
        "tests/instance_method.rs",
        "use demo::exercise;\n\n// @verifies APP.INSTANCE_METHOD\n#[test]\nfn verifies_instance_method_path() {\n    exercise();\n}\n",
    );
}
