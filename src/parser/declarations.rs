/**
@module SPECIAL.PARSER.DECLARATIONS
Shared spec declaration semantics across source comment blocks and markdown declarations, including header validation, adjacent lifecycle-marker interpretation, and final spec construction. This module does not scan source blocks or markdown files.
*/
// @fileimplements SPECIAL.PARSER.DECLARATIONS
use crate::model::{
    DeprecatedRelease, NodeKind, PlanState, PlannedRelease, SourceLocation, SpecDecl,
};
use crate::planned_syntax::{PlannedAnnotationError, PlannedSyntax};

use super::planned::{
    DeclHeader, DeclHeaderError, parse_standalone_deprecated, parse_standalone_planned,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AdjacentLifecycle {
    Absent,
    Parsed,
    Invalid,
}

#[derive(Debug)]
pub(super) struct SpecLifecycleMarkers {
    pub(super) planned: bool,
    pub(super) planned_release: Option<PlannedRelease>,
    pub(super) deprecated: bool,
    pub(super) deprecated_release: Option<DeprecatedRelease>,
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
        Err(DeclHeaderError::InvalidDeprecatedRelease) => {
            return Err("deprecated release metadata must not be empty".to_string());
        }
    };

    if kind == NodeKind::Group {
        if header.planned {
            header.planned = false;
            return Ok((
                header,
                Some("@planned may only apply to @spec, not @group".to_string()),
            ));
        }
        if header.deprecated {
            header.deprecated = false;
            return Ok((
                header,
                Some("@deprecated may only apply to @spec, not @group".to_string()),
            ));
        }
    }

    Ok((header, None))
}

pub(super) fn parse_adjacent_spec_planned(
    kind: NodeKind,
    text: &str,
    planned: PlannedSyntax,
) -> (
    AdjacentLifecycle,
    Option<PlannedRelease>,
    Option<&'static str>,
) {
    if planned != PlannedSyntax::AdjacentOwnedSpec || kind != NodeKind::Spec {
        return (AdjacentLifecycle::Absent, None, None);
    }

    let Some(result) = parse_standalone_planned(text) else {
        return (AdjacentLifecycle::Absent, None, None);
    };

    match result {
        Ok(release) => (AdjacentLifecycle::Parsed, release, None),
        Err(PlannedAnnotationError::InvalidRelease) => (
            AdjacentLifecycle::Invalid,
            None,
            Some("planned release metadata must not be empty"),
        ),
        Err(PlannedAnnotationError::InvalidSuffix) => (
            AdjacentLifecycle::Invalid,
            None,
            Some("use an exact standalone `@planned` marker with no trailing suffix"),
        ),
    }
}

pub(super) fn parse_adjacent_spec_deprecated(
    kind: NodeKind,
    text: &str,
    planned: PlannedSyntax,
) -> (
    AdjacentLifecycle,
    Option<DeprecatedRelease>,
    Option<&'static str>,
) {
    if planned != PlannedSyntax::AdjacentOwnedSpec || kind != NodeKind::Spec {
        return (AdjacentLifecycle::Absent, None, None);
    }

    let Some(result) = parse_standalone_deprecated(text) else {
        return (AdjacentLifecycle::Absent, None, None);
    };

    match result {
        Ok(release) => (AdjacentLifecycle::Parsed, release, None),
        Err(PlannedAnnotationError::InvalidRelease) => (
            AdjacentLifecycle::Invalid,
            None,
            Some("deprecated release metadata must not be empty"),
        ),
        Err(PlannedAnnotationError::InvalidSuffix) => (
            AdjacentLifecycle::Invalid,
            None,
            Some("use an exact standalone `@deprecated` marker with no trailing suffix"),
        ),
    }
}

pub(super) fn build_spec_decl(
    header: DeclHeader<'_>,
    kind: NodeKind,
    text: String,
    lifecycle: SpecLifecycleMarkers,
    location: SourceLocation,
) -> Result<SpecDecl, String> {
    let planned_release = if header.planned {
        header.planned_release
    } else {
        lifecycle.planned_release
    };
    let deprecated_release = if header.deprecated {
        header.deprecated_release
    } else {
        lifecycle.deprecated_release
    };
    let plan = if header.planned || lifecycle.planned {
        PlanState::planned(planned_release)
    } else {
        PlanState::current()
    };
    let is_deprecated = header.deprecated || lifecycle.deprecated;

    SpecDecl::new(
        header.id.to_string(),
        kind,
        text,
        plan,
        is_deprecated,
        deprecated_release,
        location,
    )
    .map_err(|err| err.to_string())
}

fn invalid_trailing_content_message(kind: NodeKind, planned: PlannedSyntax) -> &'static str {
    match (kind, planned) {
        (NodeKind::Group, _) => {
            "unexpected trailing content after group id; only the id belongs on the @group line"
        }
        (NodeKind::Spec, PlannedSyntax::LegacyBackward) => {
            "unexpected trailing content after spec id; compatibility parsing does not allow inline lifecycle markers like `@planned` or `@deprecated`"
        }
        (NodeKind::Spec, PlannedSyntax::AdjacentOwnedSpec) => {
            "unexpected trailing content after spec id; use an exact trailing `@planned` or `@deprecated` marker if needed"
        }
    }
}
