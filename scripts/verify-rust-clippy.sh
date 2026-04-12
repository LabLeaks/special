#!/usr/bin/env bash
# @verifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED

set -euo pipefail

exec mise exec -- cargo clippy --all-targets --all-features -- -D warnings
