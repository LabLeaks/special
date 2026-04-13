#!/usr/bin/env bash
# @verifies SPECIAL.DISTRIBUTION.HOMEBREW.FORMULA

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
formula="$(gh api repos/LabLeaks/homebrew-tap/contents/special.rb --jq .content | base64 --decode)"

FORMULA_TEXT="$formula" python3 - "$version" "$release_json" <<'PY'
import json
import os
import sys

version = sys.argv[1]
release = json.loads(sys.argv[2])
formula = os.environ["FORMULA_TEXT"]

assets = {asset["name"]: asset for asset in release["assets"]}
expected = {
    name: asset
    for name, asset in assets.items()
    if name in {
        "special-cli-x86_64-apple-darwin.tar.xz",
        "special-cli-aarch64-apple-darwin.tar.xz",
        "special-cli-x86_64-unknown-linux-gnu.tar.xz",
        "special-cli-aarch64-unknown-linux-gnu.tar.xz",
    }
}

assert "class Special < Formula" in formula
assert f'version "{version}"' in formula
assert 'bin.install "special"' in formula

for name, asset in expected.items():
    sha256 = asset["digest"].removeprefix("sha256:")
    assert asset["digest"].startswith("sha256:"), asset
    assert asset["url"] in formula, asset
    assert f'sha256 "{sha256}"' in formula, sha256
PY
