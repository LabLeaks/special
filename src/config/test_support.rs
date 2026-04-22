/**
@module SPECIAL.CONFIG.TEST_SUPPORT
Shared test-only temp-directory helpers for config module fixtures.
*/
// @fileimplements SPECIAL.CONFIG.TEST_SUPPORT
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn temp_config_test_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path.canonicalize().expect("temp dir should canonicalize")
}
