/**
@module SPECIAL.RENDER.LABELS
Shared rendered labels for source-attached proof and implementation relationships.
*/
// @fileimplements SPECIAL.RENDER.LABELS
use crate::model::{AttestRef, ImplementRef, VerifyRef};

pub(super) fn verify_label(verify: &VerifyRef) -> &'static str {
    if verify.body_location.is_none() && verify.body.is_some() {
        "@fileverifies"
    } else {
        "@verifies"
    }
}

pub(super) fn implementation_label(implementation: &ImplementRef) -> &'static str {
    if implementation.body_location.is_none() {
        "@fileimplements"
    } else {
        "@implements"
    }
}

pub(super) fn attest_label(attest: &AttestRef) -> &'static str {
    attest.scope.as_annotation()
}
