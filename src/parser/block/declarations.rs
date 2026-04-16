/**
@module SPECIAL.PARSER.BLOCK.DECLARATIONS
Handles spec and group declaration parsing, description ownership, and standalone `@planned` application within a source comment block.
*/
// @fileimplements SPECIAL.PARSER.BLOCK.DECLARATIONS
use crate::annotation_syntax::{ReservedSpecialAnnotation, reserved_special_annotation_rest};
use crate::model::{CommentBlock, NodeKind, ParsedRepo, PlanState, PlannedRelease, SourceLocation};
use crate::planned_syntax::{PlannedAnnotationError, PlannedSyntax};

use super::super::declarations::{
    AdjacentPlanned, build_spec_decl, parse_adjacent_spec_planned, parse_spec_decl_header,
};
use super::super::planned::parse_standalone_planned;
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

    let (header, header_diag) = match parse_spec_decl_header(kind, rest, rules.planned) {
        Ok(parsed) => parsed,
        Err(message) => {
            push_diag(parsed, block, line, &message);
            return None;
        }
    };
    if let Some(message) = header_diag {
        push_diag(parsed, block, line, &message);
    }
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
    let spec = match build_spec_decl(
        header,
        kind,
        description_lines.join(" "),
        adjacent_planned,
        adjacent_planned_release,
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
    let next_index = index + 1;
    let Some(next) = block.lines.get(next_index) else {
        return (false, None, next_index);
    };

    let (state, release, message) =
        parse_adjacent_spec_planned(kind, next.text.trim(), rules.planned);
    match state {
        AdjacentPlanned::Absent => (false, None, next_index),
        AdjacentPlanned::Parsed => (true, release, next_index + 1),
        AdjacentPlanned::Invalid => {
            if let Some(message) = message {
                push_diag(parsed, block, next.line, message);
            }
            (false, None, next_index + 1)
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
