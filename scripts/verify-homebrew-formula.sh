#!/usr/bin/env bash
# @module SPECIAL.DISTRIBUTION.HOMEBREW_CHECK
# Homebrew release/install verification in `scripts/verify-homebrew-formula.sh`.
# @fileimplements SPECIAL.DISTRIBUTION.HOMEBREW_CHECK
# @fileverifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA

set -euo pipefail

cd "$(dirname "$0")/.."

version="$(python3 - <<'PY'
import tomllib
from pathlib import Path

data = tomllib.loads(Path("Cargo.toml").read_text())
print(data["package"]["version"])
PY
)"

release_json="$(gh release view "v${version}" --repo LabLeaks/special --json assets)"
formula="$(gh api repos/LabLeaks/homebrew-tap/contents/Formula/special.rb --jq .content | base64 --decode)"

FORMULA_TEXT="$formula" python3 - "$version" "$release_json" <<'PY'
import json
import os
import sys
from pathlib import Path

version = sys.argv[1]
release = json.loads(sys.argv[2])
formula = os.environ["FORMULA_TEXT"]

assets = {asset["name"]: asset for asset in release["assets"]}
required = set(
    json.loads(Path("scripts/release-assets.json").read_text())["homebrew_formula_archives"]
)
missing = sorted(required - assets.keys())

def fail(message, details):
    raise SystemExit(f"{message}: {details}")

if missing:
    fail("missing required release assets for Homebrew formula", missing)
if "class Special < Formula" not in formula:
    fail("formula class declaration missing", "class Special < Formula")
if f'version "{version}"' not in formula:
    fail("formula version mismatch", version)
if 'bin.install "special"' not in formula:
    fail("formula no longer installs special", formula)
for helper in ('on_system_conditional(', 'on_arch_conditional('):
    if helper not in formula:
        fail("formula is missing platform selection helper", helper)

for name in sorted(required):
    asset = assets[name]
    sha256 = asset["digest"].removeprefix("sha256:")
    if not asset["digest"].startswith("sha256:"):
        fail("release asset digest is not sha256", asset)
    if asset["url"] not in formula:
        fail("formula is missing release asset url", asset["url"])
    if f'sha256 "{sha256}"' not in formula:
        fail("formula is missing release asset sha256", sha256)
PY
