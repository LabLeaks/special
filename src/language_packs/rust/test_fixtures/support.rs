/**
@module SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.SUPPORT
Shared file-writing helpers for pack-owned Rust fixture scenarios.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.SUPPORT
use std::fs;
use std::path::Path;

pub(super) fn create_dirs(root: &Path, dirs: &[&str]) {
    for dir in dirs {
        fs::create_dir_all(root.join(dir)).expect("fixture dir should be created");
    }
}

pub(super) fn write_special_toml(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
}

pub(super) fn write_mise_toolchain_override(root: &Path) {
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\n\n[toolchain]\nmanager = \"mise\"\n",
    )
    .expect("special.toml should be written");
}

pub(super) fn write_rust_toolchain_contract(root: &Path) {
    fs::write(root.join(".tool-versions"), "rust stable\n")
        .expect(".tool-versions should be written");
    write_mise_toolchain_override(root);
}

pub(super) fn write_architecture(root: &Path, body: &str) {
    fs::write(root.join("_project/ARCHITECTURE.md"), body)
        .expect("architecture fixture should be written");
}

pub(super) fn write_specs(root: &Path, body: &str) {
    fs::write(root.join("specs/root.md"), body).expect("spec fixture should be written");
}

pub(super) fn write_file(root: &Path, relative: &str, body: &str) {
    fs::write(root.join(relative), body).expect("fixture file should be written");
}
