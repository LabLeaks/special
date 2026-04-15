/**
@module SPECIAL.PARSER.BLOCK.DECLARATIONS
Handles spec and group declaration parsing, description ownership, and standalone `@planned` application within a source comment block.
*/
// @fileimplements SPECIAL.PARSER.BLOCK.DECLARATIONS
use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::model::{
    CommentBlock, NodeKind, ParsedRepo, PlanState, PlannedRelease, SourceLocation, SpecDecl,
};
use crate::planned_syntax::{PlannedAnnotationError, PlannedSyntax};

use super::super::planned::{
    AdjacentPlanned, DeclHeader, DeclHeaderError, consume_adjacent_planned,
    parse_standalone_planned,
};
use super::super::push_diag;
use super::{ParseRules, collect_description_lines};

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct BlockState {
    last_decl_idx: Option<usize>,
    last_decl_kind: Option<NodeKind>,
    last_decl_line: Option<usize>,
}

impl BlockState {
    fn remember_decl(&mut self, spec_index: usize, kind: NodeKind, line: usize) {
        self.last_decl_idx = Some(spec_index);
        self.last_decl_kind = Some(kind);
        self.last_decl_line = Some(line);
    }
}

pub(super) fn handle_decl_line(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    state: &mut BlockState,
    index: usize,
    line: usize,
    trimmed: &str,
    rules: ParseRules,
) -> Option<usize> {
    let (kind, rest) = reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Spec)
        .map(|rest| (NodeKind::Spec, rest))
        .or_else(|| {
            reserved_special_annotation_rest(trimmed, ReservedSpecialAnnotation::Group)
                .map(|rest| (NodeKind::Group, rest))
        })?;

    let header = parse_decl_header(kind, rest, parsed, block, line, rules)?;
    let (adjacent_planned, adjacent_planned_release, mut cursor) =
        adjacent_planned_state(block, parsed, kind, index, rules);

    if header.planned && adjacent_planned {
        push_diag(
            parsed,
            block,
            block.lines[index + 1].line,
            "@planned must appear only once per owning @spec",
        );
    }

    let description_lines = collect_description_lines(block, &mut cursor);
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
    let spec = match SpecDecl::new(
        header.id.to_string(),
        kind,
        description_lines.join(" "),
        plan,
        SourceLocation {
            path: block.path.clone(),
            line,
        },
    ) {
        Ok(spec) => spec,
        Err(err) => {
            push_diag(parsed, block, line, &err.to_string());
            return Some(cursor);
        }
    };
    parsed.specs.push(spec);
    state.remember_decl(parsed.specs.len() - 1, kind, line);
    Some(cursor)
}

pub(super) fn handle_standalone_planned_line(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    state: &BlockState,
    line: usize,
    trimmed: &str,
    rules: ParseRules,
) -> bool {
    let Some(planned_release) = parse_standalone_planned(trimmed) else {
        return false;
    };

    match planned_release {
        Ok(planned_release) => {
            apply_standalone_planned(parsed, block, state, line, planned_release, rules)
        }
        Err(PlannedAnnotationError::InvalidRelease) => push_diag(
            parsed,
            block,
            line,
            "planned release metadata must not be empty",
        ),
        Err(PlannedAnnotationError::InvalidSuffix) => push_diag(
            parsed,
            block,
            line,
            "use an exact standalone `@planned` marker with no trailing suffix",
        ),
    }

    true
}

fn adjacent_planned_state(
    block: &CommentBlock,
    parsed: &mut ParsedRepo,
    kind: NodeKind,
    index: usize,
    rules: ParseRules,
) -> (bool, Option<PlannedRelease>, usize) {
    match consume_adjacent_planned(block, kind, index, rules.planned) {
        AdjacentPlanned::Absent(cursor) => (false, None, cursor),
        AdjacentPlanned::Parsed(release, cursor) => (true, release, cursor),
        AdjacentPlanned::Invalid(message, cursor) => {
            push_diag(parsed, block, block.lines[index + 1].line, message);
            (false, None, cursor)
        }
    }
}

fn parse_decl_header<'a>(
    kind: NodeKind,
    rest: &'a str,
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    line: usize,
    rules: ParseRules,
) -> Option<DeclHeader<'a>> {
    let rest = rest.trim();
    let mut header = match DeclHeader::parse(rest, rules.planned) {
        Ok(header) => header,
        Err(DeclHeaderError::MissingId) => {
            let annotation = if kind == NodeKind::Spec {
                "@spec"
            } else {
                "@group"
            };
            push_diag(
                parsed,
                block,
                line,
                &format!("missing spec id after {annotation}"),
            );
            return None;
        }
        Err(DeclHeaderError::InvalidTrailingContent) => {
            push_diag(
                parsed,
                block,
                line,
                invalid_trailing_content_message(kind, rules.planned),
            );
            return None;
        }
        Err(DeclHeaderError::InvalidPlannedRelease) => {
            push_diag(
                parsed,
                block,
                line,
                "planned release metadata must not be empty",
            );
            return None;
        }
    };

    if header.planned && kind == NodeKind::Group {
        push_diag(
            parsed,
            block,
            line,
            "@planned may only apply to @spec, not @group",
        );
        header.planned = false;
    }

    Some(header)
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

fn apply_standalone_planned(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    state: &BlockState,
    line: usize,
    planned_release: Option<PlannedRelease>,
    rules: ParseRules,
) {
    match rules.planned {
        PlannedSyntax::LegacyBackward => {
            apply_legacy_planned(parsed, block, state, line, planned_release)
        }
        PlannedSyntax::AdjacentOwnedSpec => {
            apply_adjacent_planned(parsed, block, state, line, planned_release)
        }
    }
}

fn apply_legacy_planned(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    state: &BlockState,
    line: usize,
    planned_release: Option<PlannedRelease>,
) {
    match state.last_decl_idx {
        Some(spec_index) if parsed.specs[spec_index].kind() == NodeKind::Spec => {
            set_decl_plan(parsed, block, line, spec_index, planned_release);
        }
        None => push_diag(
            parsed,
            block,
            line,
            "@planned must appear after a @spec in the same block",
        ),
        Some(_) => push_diag(
            parsed,
            block,
            line,
            "@planned may only apply to @spec, not @group",
        ),
    }
}

fn apply_adjacent_planned(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    state: &BlockState,
    line: usize,
    planned_release: Option<PlannedRelease>,
) {
    let adjacent_line = Some(line.saturating_sub(1));
    if state.last_decl_line == adjacent_line
        && state.last_decl_kind == Some(NodeKind::Spec)
        && let Some(spec_index) = state.last_decl_idx
    {
        set_decl_plan(parsed, block, line, spec_index, planned_release);
        return;
    }

    if state.last_decl_line == adjacent_line && state.last_decl_kind == Some(NodeKind::Group) {
        push_diag(
            parsed,
            block,
            line,
            "@planned may only apply to @spec, not @group",
        );
    } else if state.last_decl_idx.is_some() {
        push_diag(
            parsed,
            block,
            line,
            "@planned must be adjacent to exactly one owning @spec; the backward-looking form is not allowed in version 1",
        );
    } else {
        push_diag(
            parsed,
            block,
            line,
            "@planned must be adjacent to exactly one owning @spec",
        );
    }
}

fn set_decl_plan(
    parsed: &mut ParsedRepo,
    block: &CommentBlock,
    line: usize,
    spec_index: usize,
    planned_release: Option<PlannedRelease>,
) {
    if let Err(err) = parsed.specs[spec_index].set_plan(PlanState::planned(planned_release)) {
        push_diag(parsed, block, line, &err.to_string());
    }
}
