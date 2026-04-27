/**
@module SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.BINARIES
Rust fixture scenarios for process-boundary and Cargo-binary traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.BINARIES
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_special_toml, write_specs,
};

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_local_binary_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo-cli\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[[bin]]\nname = \"app\"\npath = \"src/main.rs\"\n",
    );
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.CLI`\nLocal binary invocation behavior.\n",
    );
    write_file(
        root,
        "src/main.rs",
        "// @fileimplements DEMO\nfn main() {\n    app_entry();\n}\n\nfn app_entry() {\n    live_impl();\n}\n\npub fn live_impl() {}\n\npub fn orphan_impl() {}\n",
    );
    write_file(
        root,
        "tests.rs",
        "use std::process::Command;\n\nfn run_app() {\n    let _ = Command::new(env!(\"CARGO_BIN_EXE_app\")).arg(\"status\").output();\n}\n\n// @verifies APP.CLI\n#[test]\nfn verifies_cli_entrypoint() {\n    run_app();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_lib_crate_binary_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo-cli\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = \"demo\"\n\n[[bin]]\nname = \"app\"\npath = \"src/main.rs\"\n",
    );
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.CLI`\nBinary entrypoint reaches the library crate root honestly.\n",
    );
    write_file(
        root,
        "src/lib.rs",
        "// @fileimplements DEMO\npub fn run_from_env() {\n    live_impl();\n}\n\npub fn live_impl() {}\n\npub fn orphan_impl() {}\n",
    );
    write_file(
        root,
        "src/main.rs",
        "fn main() {\n    demo::run_from_env();\n}\n",
    );
    write_file(
        root,
        "tests.rs",
        "use std::process::Command;\n\nfn run_app() {\n    let _ = Command::new(env!(\"CARGO_BIN_EXE_app\")).arg(\"status\").output();\n}\n\n// @verifies APP.CLI\n#[test]\nfn verifies_cli_entrypoint() {\n    run_app();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_traceability_default_binary_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_file(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo-cli\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write_architecture(root, "# Architecture\n\n### `@module DEMO`\nDemo module.\n");
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.CLI`\nDefault Cargo binary entrypoints participate in process-boundary traceability.\n",
    );
    write_file(
        root,
        "src/main.rs",
        "// @fileimplements DEMO\nfn main() {\n    app_entry();\n}\n\nfn app_entry() {\n    live_impl();\n}\n\npub fn live_impl() {}\n\npub fn orphan_impl() {}\n",
    );
    write_file(
        root,
        "tests.rs",
        "use std::process::Command;\n\nfn run_app() {\n    let _ = Command::new(env!(\"CARGO_BIN_EXE_demo-cli\")).arg(\"status\").output();\n}\n\n// @verifies APP.CLI\n#[test]\nfn verifies_cli_entrypoint() {\n    run_app();\n}\n",
    );
}
