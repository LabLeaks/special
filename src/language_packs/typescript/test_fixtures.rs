#![allow(unused_imports)]
/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES
Pack-owned TypeScript fixture facade that delegates analysis, traceability, and UI callback scenarios to narrower child modules.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES
#[path = "test_fixtures/analysis.rs"]
mod analysis;
#[path = "test_fixtures/callbacks.rs"]
mod callbacks;
#[path = "test_fixtures/support.rs"]
mod support;
#[path = "test_fixtures/traceability.rs"]
mod traceability;
#[path = "test_fixtures/ui.rs"]
mod ui;

pub use analysis::write_typescript_module_analysis_fixture;
pub use callbacks::{
    write_typescript_context_traceability_fixture, write_typescript_effect_traceability_fixture,
    write_typescript_event_traceability_fixture,
    write_typescript_forwarded_callback_traceability_fixture,
    write_typescript_hook_callback_traceability_fixture,
};
pub use traceability::{
    write_typescript_cycle_traceability_fixture, write_typescript_reference_traceability_fixture,
    write_typescript_tool_traceability_fixture, write_typescript_traceability_fixture,
};
pub use ui::{
    write_typescript_next_traceability_fixture, write_typescript_react_traceability_fixture,
};
