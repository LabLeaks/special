/**
@module SPECIAL.PARSER.PLANNED
Raw planned-marker syntax parsing in `src/parser/planned.rs`.
*/
// @fileimplements SPECIAL.PARSER.PLANNED
use crate::model::PlannedRelease;
pub(super) use crate::planned_syntax::DeclHeaderError;
use crate::planned_syntax::{
    ParsedDeclHeader, ParsedPlannedAnnotation, PlannedAnnotationContext, PlannedAnnotationError,
    PlannedSyntax, parse_decl_header, parse_planned_annotation,
};

pub(super) type DeclHeader<'a> = ParsedDeclHeader<'a>;

impl<'a> DeclHeader<'a> {
    pub(super) fn parse(
        rest: &'a str,
        planned: PlannedSyntax,
    ) -> std::result::Result<Self, DeclHeaderError> {
        parse_decl_header(rest, planned)
    }
}

pub(super) fn parse_standalone_planned(
    text: &str,
) -> Option<Result<Option<PlannedRelease>, PlannedAnnotationError>> {
    parse_planned_annotation(text, PlannedAnnotationContext::Standalone).map(
        |result: Result<ParsedPlannedAnnotation, PlannedAnnotationError>| {
            result.map(|annotation| annotation.release)
        },
    )
}
