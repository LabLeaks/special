#!/usr/bin/env bash
# @verifies SPECIAL.DISTRIBUTION.GITHUB_RELEASES.PUBLISHED

set -euo pipefail

cd "$(dirname "$0")/.."

version="$(python3 - <<'PY'
import tomllib
from pathlib import Path

data = tomllib.loads(Path("Cargo.toml").read_text())
print(data["package"]["version"])
PY
)"

release_json="$(gh release view "v${version}" --repo LabLeaks/special --json tagName,isDraft,isPrerelease,assets)"

python3 - "$version" "$release_json" <<'PY'
import json
import sys

version = sys.argv[1]
release = json.loads(sys.argv[2])

assert release["tagName"] == f"v{version}", release
assert release["isDraft"] is False, release
assert release["isPrerelease"] is False, release
assert len(release["assets"]) > 0, release
PY
