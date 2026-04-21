/**
@module SPECIAL.MODULES.ANALYZE.RUST.SEMANTIC
Selects Rust fact sources from probed local-toolchain capabilities without forcing repo-facing analysis layers to know about rustdoc, rust-analyzer, compiler channels, or other source-specific details. This module should only decide which source could be used honestly; it should not fabricate call edges when no source is available.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.RUST.SEMANTIC
use super::toolchain::RustToolchainProject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RustSemanticFactSourceKind {
    RustAnalyzer,
}

pub(super) fn selected_semantic_fact_source(
    project: Option<&RustToolchainProject>,
) -> Option<RustSemanticFactSourceKind> {
    let project = project?;
    if project.capabilities.rust_analyzer_available {
        Some(RustSemanticFactSourceKind::RustAnalyzer)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::{RustSemanticFactSourceKind, selected_semantic_fact_source};
    use crate::modules::analyze::rust::toolchain::{
        RustToolchainCapabilities, RustToolchainChannel, RustToolchainProject,
        RustdocJsonAvailability,
    };

    #[test]
    fn selects_no_semantic_fact_source_on_stable_toolchains() {
        let project = RustToolchainProject {
            workspace_root: PathBuf::from("/tmp/demo"),
            target_sources: BTreeMap::new(),
            capabilities: RustToolchainCapabilities {
                active_channel: RustToolchainChannel::Stable,
                rustdoc_json: RustdocJsonAvailability::RequiresNightly,
                rust_analyzer_available: false,
            },
        };

        assert_eq!(selected_semantic_fact_source(Some(&project)), None);
    }

    #[test]
    fn selects_rust_analyzer_when_toolchain_reports_it_available() {
        let project = RustToolchainProject {
            workspace_root: PathBuf::from("/tmp/demo"),
            target_sources: BTreeMap::new(),
            capabilities: RustToolchainCapabilities {
                active_channel: RustToolchainChannel::Nightly,
                rustdoc_json: RustdocJsonAvailability::Available,
                rust_analyzer_available: true,
            },
        };

        assert_eq!(
            selected_semantic_fact_source(Some(&project)),
            Some(RustSemanticFactSourceKind::RustAnalyzer)
        );
    }
}
