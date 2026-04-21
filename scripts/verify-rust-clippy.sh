#!/usr/bin/env bash
# @module SPECIAL.DISTRIBUTION.CLIPPY_CHECK
# Pinned clippy verification contract in `scripts/verify-rust-clippy.sh`.
# @fileimplements SPECIAL.DISTRIBUTION.CLIPPY_CHECK
# @fileverifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED

set -euo pipefail

exec mise exec -- cargo clippy --all-targets --all-features -- -D warnings
