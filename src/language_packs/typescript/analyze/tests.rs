/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS
TypeScript scoped traceability regressions that compare direct full-vs-scoped pack behavior and keep the test surface factored into smaller theorem-facing slices.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS
#[path = "../test_fixtures.rs"]
#[allow(dead_code)]
mod test_fixtures;

#[path = "tests/support.rs"]
mod support;
#[path = "tests/summary.rs"]
mod summary;
#[path = "tests/contracts.rs"]
mod contracts;
#[path = "tests/projection.rs"]
mod projection;
