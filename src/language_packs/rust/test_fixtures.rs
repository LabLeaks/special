#![allow(unused_imports)]
/**
@module SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES
Pack-owned Rust integration-test fixture facade that delegates scenario construction to narrower child modules.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.RUST.TEST_FIXTURES
#[path = "test_fixtures/binaries.rs"]
mod binaries;
#[path = "test_fixtures/context.rs"]
mod context;
#[path = "test_fixtures/dispatch.rs"]
mod dispatch;
#[path = "test_fixtures/matching.rs"]
mod matching;
#[allow(clippy::duplicate_mod)]
#[path = "../shared/test_fixture_support.rs"]
mod shared_support;
#[path = "test_fixtures/support.rs"]
mod support;

pub use binaries::{
    write_traceability_default_binary_fixture, write_traceability_lib_crate_binary_fixture,
    write_traceability_local_binary_fixture,
};
pub use context::{
    write_traceability_module_analysis_fixture, write_traceability_module_context_fixture,
    write_traceability_multiple_supports_fixture, write_traceability_review_surface_fixture,
};
pub use dispatch::{
    write_traceability_cross_file_module_fixture, write_traceability_imported_call_fixture,
    write_traceability_instance_method_fixture, write_traceability_mediated_fixture,
    write_traceability_self_method_fixture,
};
pub use matching::{
    write_traceability_file_verify_fixture, write_traceability_name_collision_fixture,
    write_traceability_qualified_match_fixture, write_traceability_transitive_fixture,
};
