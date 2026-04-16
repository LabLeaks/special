/**
@module SPECIAL.PARSER.DECLARATIONS
Shared spec declaration semantics across source comment blocks and markdown declarations, including header validation, adjacent planned-marker interpretation, and final spec construction. This module does not scan source blocks or markdown files.
*/
// @fileimplements SPECIAL.PARSER.DECLARATIONS
use crate::model::{NodeKind, PlanState, PlannedRelease, SourceLocation, SpecDecl};
use crate::planned_syntax::{PlannedAnnotationError, PlannedSyntax};

use super::planned::{DeclHeader, DeclHeaderError, parse_standalone_planned};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AdjacentPlanned {
    Absent,
    Parsed,
    Invalid,
}

pub(super) fn parse_spec_decl_header<'a>(
    kind: NodeKind,
    rest: &'a str,
    planned: PlannedSyntax,
) -> Result<(DeclHeader<'a>, Option<String>), String> {
    let rest = rest.trim();
    let mut header = match DeclHeader::parse(rest, planned) {
        Ok(header) => header,
        Err(DeclHeaderError::MissingId) => {
            let annotation = if kind == NodeKind::Spec {
                "@spec"
            } else {
                "@group"
            };
            return Err(format!("missing spec id after {annotation}"));
        }
        Err(DeclHeaderError::InvalidTrailingContent) => {
            return Err(invalid_trailing_content_message(kind, planned).to_string());
        }
        Err(DeclHeaderError::InvalidPlannedRelease) => {
            return Err("planned release metadata must not be empty".to_string());
        }
    };

    if header.planned && kind == NodeKind::Group {
        header.planned = false;
        return Ok((
            header,
            Some("@planned may only apply to @spec, not @group".to_string()),
        ));
    }

    Ok((header, None))
}

pub(super) fn parse_adjacent_spec_planned(
    kind: NodeKind,
    text: &str,
    planned: PlannedSyntax,
) -> (
    AdjacentPlanned,
    Option<PlannedRelease>,
    Option<&'static str>,
) {
    if planned != PlannedSyntax::AdjacentOwnedSpec || kind != NodeKind::Spec {
        return (AdjacentPlanned::Absent, None, None);
    }

    let Some(result) = parse_standalone_planned(text) else {
        return (AdjacentPlanned::Absent, None, None);
    };

    match result {
        Ok(release) => (AdjacentPlanned::Parsed, release, None),
        Err(PlannedAnnotationError::InvalidRelease) => (
            AdjacentPlanned::Invalid,
            None,
            Some("planned release metadata must not be empty"),
        ),
        Err(PlannedAnnotationError::InvalidSuffix) => (
            AdjacentPlanned::Invalid,
            None,
            Some("use an exact standalone `@planned` marker with no trailing suffix"),
        ),
    }
}

pub(super) fn build_spec_decl(
    header: DeclHeader<'_>,
    kind: NodeKind,
    text: String,
    adjacent_planned: bool,
    adjacent_planned_release: Option<PlannedRelease>,
    location: SourceLocation,
) -> Result<SpecDecl, String> {
    let planned_release = if header.planned {
        header.planned_release
    } else {
        adjacent_planned_release
    };
    let plan = if header.planned || adjacent_planned {
        PlanState::planned(planned_release)
    } else {
        PlanState::live()
    };

    SpecDecl::new(header.id.to_string(), kind, text, plan, location).map_err(|err| err.to_string())
}

fn invalid_trailing_content_message(kind: NodeKind, planned: PlannedSyntax) -> &'static str {
    match (kind, planned) {
        (NodeKind::Group, _) => {
            "unexpected trailing content after group id; only the id belongs on the @group line"
        }
        (NodeKind::Spec, PlannedSyntax::LegacyBackward) => {
            "unexpected trailing content after spec id; compatibility parsing does not allow inline `@planned`"
        }
        (NodeKind::Spec, PlannedSyntax::AdjacentOwnedSpec) => {
            "unexpected trailing content after spec id; use an exact trailing `@planned` marker if needed"
        }
    }
}
