/**
@module SPECIAL.PLANNED_SYNTAX
Shared parsing rules for `@planned` and `@deprecated` annotations, including exact boundary handling for standalone and inline forms.
*/
// @fileimplements SPECIAL.PLANNED_SYNTAX
use crate::model::{DeprecatedRelease, PlannedRelease};

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
    InvalidDeprecatedRelease,
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
    pub(crate) deprecated: bool,
    pub(crate) deprecated_release: Option<DeprecatedRelease>,
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
                    deprecated: false,
                    deprecated_release: None,
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
            let (inline_planned, planned_release, inline_deprecated, deprecated_release) =
                if suffix.is_empty() {
                    (false, None, false, None)
                } else if let Some(result) =
                    parse_planned_annotation(suffix, PlannedAnnotationContext::Inline)
                {
                    match result {
                        Ok(annotation) => (true, annotation.release, false, None),
                        Err(PlannedAnnotationError::InvalidRelease) => {
                            return Err(DeclHeaderError::InvalidPlannedRelease);
                        }
                        Err(PlannedAnnotationError::InvalidSuffix) => {
                            return Err(DeclHeaderError::InvalidTrailingContent);
                        }
                    }
                } else if let Some(result) =
                    parse_deprecated_annotation(suffix, PlannedAnnotationContext::Inline)
                {
                    match result {
                        Ok(annotation) => (false, None, true, annotation.release),
                        Err(PlannedAnnotationError::InvalidRelease) => {
                            return Err(DeclHeaderError::InvalidDeprecatedRelease);
                        }
                        Err(PlannedAnnotationError::InvalidSuffix) => {
                            return Err(DeclHeaderError::InvalidTrailingContent);
                        }
                    }
                } else {
                    return Err(DeclHeaderError::InvalidTrailingContent);
                };

            Ok(ParsedDeclHeader {
                id,
                planned: inline_planned,
                planned_release,
                deprecated: inline_deprecated,
                deprecated_release,
            })
        }
    }
}

pub(crate) fn parse_planned_annotation(
    text: &str,
    context: PlannedAnnotationContext,
) -> Option<Result<ParsedPlannedAnnotation, PlannedAnnotationError>> {
    parse_release_annotation(text, "@planned", context, |release| {
        PlannedRelease::new(release)
    })
    .map(|result| {
        result.map(|annotation| ParsedPlannedAnnotation {
            release: annotation.release,
        })
    })
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedDeprecatedAnnotation {
    pub(crate) release: Option<DeprecatedRelease>,
}

pub(crate) fn parse_deprecated_annotation(
    text: &str,
    context: PlannedAnnotationContext,
) -> Option<Result<ParsedDeprecatedAnnotation, PlannedAnnotationError>> {
    parse_release_annotation(text, "@deprecated", context, |release| {
        DeprecatedRelease::new(release)
    })
    .map(|result| {
        result.map(|annotation| ParsedDeprecatedAnnotation {
            release: annotation.release,
        })
    })
}

fn parse_release_annotation<R>(
    text: &str,
    keyword: &str,
    context: PlannedAnnotationContext,
    parse_release: impl Fn(&str) -> Result<R, crate::model::ModelInvariantError>,
) -> Option<Result<ParsedReleaseAnnotation<R>, PlannedAnnotationError>> {
    let rest = text.strip_prefix(keyword)?;
    if rest.is_empty() {
        return Some(Ok(ParsedReleaseAnnotation { release: None }));
    }
    if !rest.starts_with(char::is_whitespace) {
        let _ = context;
        return Some(Err(PlannedAnnotationError::InvalidSuffix));
    }
    let release = rest.trim();
    if release.is_empty() {
        Some(Ok(ParsedReleaseAnnotation { release: None }))
    } else {
        Some(match parse_release(release) {
            Ok(release) => Ok(ParsedReleaseAnnotation {
                release: Some(release),
            }),
            Err(_) => Err(PlannedAnnotationError::InvalidRelease),
        })
    }
}

#[derive(Debug, Clone)]
struct ParsedReleaseAnnotation<R> {
    release: Option<R>,
}
