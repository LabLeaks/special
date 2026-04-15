/**
@module SPECIAL.PARSER.PLANNED
Planned-marker syntax parsing in `src/parser/planned.rs`.
*/
// @fileimplements SPECIAL.PARSER.PLANNED
use crate::model::{CommentBlock, NodeKind, PlannedRelease};
pub(super) use crate::planned_syntax::DeclHeaderError;
use crate::planned_syntax::{
    ParsedDeclHeader, ParsedPlannedAnnotation, PlannedAnnotationContext, PlannedAnnotationError,
    PlannedSyntax, parse_decl_header, parse_planned_annotation,
};

#[derive(Debug, Clone)]
pub(super) enum AdjacentPlanned {
    Absent(usize),
    Parsed(Option<PlannedRelease>, usize),
    Invalid(&'static str, usize),
}

pub(super) type DeclHeader<'a> = ParsedDeclHeader<'a>;

impl<'a> DeclHeader<'a> {
    pub(super) fn parse(
        rest: &'a str,
        planned: PlannedSyntax,
    ) -> std::result::Result<Self, DeclHeaderError> {
        parse_decl_header(rest, planned)
    }
}

pub(super) fn consume_adjacent_planned(
    block: &CommentBlock,
    kind: NodeKind,
    index: usize,
    syntax: PlannedSyntax,
) -> AdjacentPlanned {
    if syntax != PlannedSyntax::AdjacentOwnedSpec || kind != NodeKind::Spec {
        return AdjacentPlanned::Absent(index + 1);
    }

    let next_index = index + 1;
    let Some(next) = block.lines.get(next_index) else {
        return AdjacentPlanned::Absent(next_index);
    };

    if let Some(planned_release) =
        parse_planned_annotation(next.text.trim(), PlannedAnnotationContext::Standalone)
    {
        return match planned_release {
            Ok(planned_release) => AdjacentPlanned::Parsed(planned_release.release, next_index + 1),
            Err(PlannedAnnotationError::InvalidRelease) => AdjacentPlanned::Invalid(
                "planned release metadata must not be empty",
                next_index + 1,
            ),
            Err(PlannedAnnotationError::InvalidSuffix) => AdjacentPlanned::Invalid(
                "use an exact standalone `@planned` marker with no trailing suffix",
                next_index + 1,
            ),
        };
    }

    AdjacentPlanned::Absent(next_index)
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
