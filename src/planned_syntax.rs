/**
@module SPECIAL.PLANNED_SYNTAX
Shared parsing rules for `@planned` annotations, including exact boundary handling for standalone and inline forms.
*/
// @implements SPECIAL.PLANNED_SYNTAX
use crate::model::PlannedRelease;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlannedSyntax {
    LegacyBackward,
    AdjacentOwnedSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlannedAnnotationError {
    InvalidSuffix,
    InvalidRelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlannedAnnotationContext {
    Standalone,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclHeaderError {
    MissingId,
    InvalidTrailingContent,
    InvalidPlannedRelease,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedPlannedAnnotation {
    pub(crate) release: Option<PlannedRelease>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedDeclHeader<'a> {
    pub(crate) id: &'a str,
    pub(crate) planned: bool,
    pub(crate) planned_release: Option<PlannedRelease>,
}

pub(crate) fn parse_decl_header<'a>(
    rest: &'a str,
    planned: PlannedSyntax,
) -> Result<ParsedDeclHeader<'a>, DeclHeaderError> {
    match planned {
        PlannedSyntax::LegacyBackward => {
            let mut parts = rest.splitn(2, char::is_whitespace);
            let id = parts.next().unwrap_or_default();
            let suffix = parts.next().map(str::trim).unwrap_or_default();
            if id.is_empty() {
                Err(DeclHeaderError::MissingId)
            } else if suffix.is_empty() {
                Ok(ParsedDeclHeader {
                    id,
                    planned: false,
                    planned_release: None,
                })
            } else {
                Err(DeclHeaderError::InvalidTrailingContent)
            }
        }
        PlannedSyntax::AdjacentOwnedSpec => {
            let mut parts = rest.splitn(2, char::is_whitespace);
            let id = parts.next().unwrap_or_default();
            if id.is_empty() {
                return Err(DeclHeaderError::MissingId);
            }

            let suffix = parts.next().map(str::trim_start).unwrap_or_default();
            let (inline_planned, planned_release) = if suffix.is_empty() {
                (false, None)
            } else {
                match parse_planned_annotation(suffix, PlannedAnnotationContext::Inline) {
                    Some(Ok(annotation)) => (true, annotation.release),
                    Some(Err(PlannedAnnotationError::InvalidRelease)) => {
                        return Err(DeclHeaderError::InvalidPlannedRelease);
                    }
                    Some(Err(PlannedAnnotationError::InvalidSuffix)) | None => {
                        return Err(DeclHeaderError::InvalidTrailingContent);
                    }
                }
            };

            Ok(ParsedDeclHeader {
                id,
                planned: inline_planned,
                planned_release,
            })
        }
    }
}

pub(crate) fn parse_planned_annotation(
    text: &str,
    context: PlannedAnnotationContext,
) -> Option<Result<ParsedPlannedAnnotation, PlannedAnnotationError>> {
    let rest = text.strip_prefix("@planned")?;
    if rest.is_empty() {
        return Some(Ok(ParsedPlannedAnnotation { release: None }));
    }
    if !rest.starts_with(char::is_whitespace) {
        let _ = context;
        return Some(Err(PlannedAnnotationError::InvalidSuffix));
    }
    let release = rest.trim();
    if release.is_empty() {
        Some(Ok(ParsedPlannedAnnotation { release: None }))
    } else {
        Some(match PlannedRelease::new(release) {
            Ok(release) => Ok(ParsedPlannedAnnotation {
                release: Some(release),
            }),
            Err(_) => Err(PlannedAnnotationError::InvalidRelease),
        })
    }
}
