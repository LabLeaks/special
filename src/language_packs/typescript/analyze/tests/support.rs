/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT
Shared TypeScript scoped traceability test builders and comparison helpers.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.SUPPORT
#[path = "support/assertions.rs"]
mod assertions;
#[path = "support/builders.rs"]
mod builders;
#[path = "support/helpers.rs"]
mod helpers;

pub(super) use assertions::*;
pub(super) use builders::*;
pub(super) use helpers::*;
