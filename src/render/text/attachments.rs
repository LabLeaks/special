/**
@module SPECIAL.RENDER.TEXT.ATTACHMENTS
Formats verbose support and implementation attachments into human-readable text blocks. This module does not decide which nodes or documents to render.
*/
// @fileimplements SPECIAL.RENDER.TEXT.ATTACHMENTS
use std::fmt::Write;

use crate::model::{AttestRef, ImplementRef, VerifyRef};

use super::super::labels::{attest_label, implementation_label, verify_label};
use super::super::templates::text_indent;

pub(super) fn write_block_text(output: &mut String, body: &str, depth: usize) {
    let indent = text_indent(depth);
    for line in body.lines() {
        writeln!(output, "{indent}{line}").expect("string writes should succeed");
    }
}

pub(super) fn render_verify_section(
    output: &mut String,
    indent: &str,
    depth: usize,
    verifies: &[VerifyRef],
) {
    for verify in verifies {
        writeln!(
            output,
            "{}  {} {}:{}",
            indent,
            verify_label(verify),
            verify.location.path.display(),
            verify.location.line
        )
        .expect("string writes should succeed");
        if let Some(body_location) = &verify.body_location {
            writeln!(
                output,
                "{}    body at: {}:{}",
                indent,
                body_location.path.display(),
                body_location.line
            )
            .expect("string writes should succeed");
        }
        if let Some(body) = &verify.body {
            write_block_text(output, body, depth + 2);
        }
    }
}

pub(super) fn render_attest_section(
    output: &mut String,
    indent: &str,
    depth: usize,
    attests: &[AttestRef],
) {
    for attest in attests {
        writeln!(
            output,
            "{}  {} {}:{}",
            indent,
            attest_label(attest),
            attest.location.path.display(),
            attest.location.line
        )
        .expect("string writes should succeed");
        if let Some(body) = &attest.body {
            write_block_text(output, body, depth + 2);
        }
    }
}

pub(super) fn render_implementation_section(
    output: &mut String,
    indent: &str,
    depth: usize,
    implementations: &[ImplementRef],
) {
    for implementation in implementations {
        writeln!(
            output,
            "{}  {} {}:{}",
            indent,
            implementation_label(implementation),
            implementation.location.path.display(),
            implementation.location.line
        )
        .expect("string writes should succeed");
        if let Some(body_location) = &implementation.body_location {
            writeln!(
                output,
                "{}    body at: {}:{}",
                indent,
                body_location.path.display(),
                body_location.line
            )
            .expect("string writes should succeed");
        }
        if let Some(body) = &implementation.body {
            write_block_text(output, body, depth + 2);
        }
    }
}
