/**
@module SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.SUPPORT
Shared file-writing helpers for pack-owned Rust fixture scenarios.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES.SUPPORT
use std::path::Path;

use super::shared_support as shared;

pub(super) fn create_dirs(root: &Path, dirs: &[&str]) {
    shared::create_dirs(root, dirs);
}

pub(super) fn write_architecture(root: &Path, body: &str) {
    shared::write_architecture(root, body);
}

pub(super) fn write_specs(root: &Path, body: &str) {
    shared::write_specs(root, body);
}

pub(super) fn write_file(root: &Path, relative: &str, body: &str) {
    shared::write_file(root, relative, body);
}

pub(super) fn write_special_toml(root: &Path) {
    shared::write_file(root, "special.toml", "version = \"1\"\nroot = \".\"\n");
}

pub(super) fn write_mise_toolchain_override(root: &Path) {
    shared::write_file(
        root,
        "special.toml",
        "version = \"1\"\nroot = \".\"\n\n[toolchain]\nmanager = \"mise\"\n",
    );
}

pub(super) fn write_rust_toolchain_contract(root: &Path) {
    shared::write_file(root, ".tool-versions", "rust stable\n");
    write_mise_toolchain_override(root);
}
