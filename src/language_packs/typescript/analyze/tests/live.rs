/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.LIVE
TypeScript live-repo exact-contract regression tests.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.ANALYZE.TESTS.LIVE
use std::path::Path;

use super::support::{build_live_scoped_typescript_analysis_pair, summary_identity};

#[test]
#[ignore = "requires local commercebox checkout and TypeScript runtime"]
fn live_typescript_exact_contract_matches_full_then_filtered_on_commercebox() {
    let root = Path::new("/Users/gk/work/paypal/commercebox");
    let scoped_path = "paypal-open-commerce-nextjs/app/page.tsx";

    let Some((full, scoped)) = build_live_scoped_typescript_analysis_pair(root, scoped_path) else {
        return;
    };

    assert_eq!(summary_identity(&full), summary_identity(&scoped));
}
