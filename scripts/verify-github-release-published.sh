#!/usr/bin/env bash
# @module SPECIAL.DISTRIBUTION.GITHUB_RELEASE_CHECK
# GitHub release publication verification in `scripts/verify-github-release-published.sh`.
# @fileimplements SPECIAL.DISTRIBUTION.GITHUB_RELEASE_CHECK
# @fileverifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.PUBLISHED

set -euo pipefail

cd "$(dirname "$0")/.."

version="$(python3 - <<'PY'
import tomllib
from pathlib import Path

data = tomllib.loads(Path("Cargo.toml").read_text())
print(data["package"]["version"])
PY
)"

release_tag="$(python3 - "$version" <<'PY'
import re
import sys

version = sys.argv[1]
print(version if re.match(r"^[^/]+/", version) or version.startswith("v") else f"v{version}")
PY
)"

release_json="$(gh release view "$release_tag" --repo LabLeaks/special --json tagName,isDraft,isPrerelease,assets)"

python3 - "$version" "$release_tag" "$release_json" <<'PY'
import json
import sys
from pathlib import Path

version = sys.argv[1]
release_tag = sys.argv[2]
release = json.loads(sys.argv[3])
expected_assets = set(
    json.loads(Path("scripts/release-assets.json").read_text())["github_release_assets"]
)
asset_names = {asset["name"] for asset in release["assets"]}
expected_prerelease = "-" in version

def fail(message, details):
    raise SystemExit(f"{message}: {details}")

if release["tagName"] != release_tag:
    fail("release tag mismatch", release)
if release["isDraft"] is not False:
    fail("release is still a draft", release)
if release["isPrerelease"] is not expected_prerelease:
    fail("release prerelease state mismatch", release)
if asset_names != expected_assets:
    fail(
        "release assets did not match expected set",
        {
            "missing": sorted(expected_assets - asset_names),
            "unexpected": sorted(asset_names - expected_assets),
        },
    )
PY
