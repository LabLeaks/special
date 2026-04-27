/**
@module SPECIAL.LANGUAGE_PACKS.TEST_FIXTURE_SUPPORT
Shared file-writing helpers for language-pack mini-repo fixture builders.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TEST_FIXTURE_SUPPORT
use std::fs;
use std::path::Path;

pub(super) fn create_dirs(root: &Path, dirs: &[&str]) {
    for dir in dirs {
        fs::create_dir_all(root.join(dir)).expect("fixture dir should be created");
    }
}

pub(super) fn write_architecture(root: &Path, body: &str) {
    write_file(root, "_project/ARCHITECTURE.md", body);
}

pub(super) fn write_specs(root: &Path, body: &str) {
    write_file(root, "specs/root.md", body);
}

pub(super) fn write_file(root: &Path, relative: &str, body: &str) {
    fs::write(root.join(relative), body).expect("fixture file should be written");
}
