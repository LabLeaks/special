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

expected = {
    "special-cli-x86_64-apple-darwin.tar.xz": "c6eb6dfb034377ff8fe68dcc364bf45c12244d41bd0f5b3f89f4d77fff00d238",
    "special-cli-aarch64-apple-darwin.tar.xz": "59e394b6bef3bf1eba7577bca1755a740de74e7c06a873c34a1e76645dca802f",
    "special-cli-x86_64-unknown-linux-gnu.tar.xz": "8b91107cb8046c6ad4363a3aa0318bc5b2e79d1fc559ca45206fb11e832ffb72",
    "special-cli-aarch64-unknown-linux-gnu.tar.xz": "ee13029812c4bc6beb59eb2b3733a8894a47b0e49416da0fe1309ac803cf136e",
}

assets = {asset["name"]: asset for asset in release["assets"]}

assert "class Special < Formula" in formula
assert f'version "{version}"' in formula
assert 'bin.install "special"' in formula

for name, sha256 in expected.items():
    asset = assets[name]
    assert asset["digest"] == f"sha256:{sha256}", asset
    assert asset["url"] in formula, asset
    assert f'sha256 "{sha256}"' in formula, sha256
PY
