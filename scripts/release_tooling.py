# @module SPECIAL.RELEASE_REVIEW.TOOLING
# Shared release-tool helpers in `scripts/release_tooling.py`.
# @implements SPECIAL.RELEASE_REVIEW.TOOLING
from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path

SEMVER_RE = re.compile(r"^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$")


def normalize_tag(value: str, *, subject: str = "release version") -> str:
    match = SEMVER_RE.match(value)
    if not match:
        raise SystemExit(
            f"{subject} must be semver like `0.3.0` or `v0.3.0`, got `{value}`"
        )
    return f"v{match.group(1)}.{match.group(2)}.{match.group(3)}" + (
        f"-{match.group(4)}" if match.group(4) else ""
    )


def package_version(root: Path) -> str:
    data = tomllib.loads(root.joinpath("Cargo.toml").read_text())
    return str(data["package"]["version"])


def run_checked(root: Path, command: list[str]) -> str:
    result = subprocess.run(
        command,
        cwd=root,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)
    return result.stdout


def semver_sort_key(value: str) -> tuple[int, int, int, int, tuple[tuple[int, object], ...]] | None:
    match = SEMVER_RE.match(value)
    if not match:
        return None

    prerelease = match.group(4)
    prerelease_key: tuple[tuple[int, object], ...] = ()
    if prerelease:
        prerelease_key = tuple(_prerelease_identifier_key(item) for item in prerelease.split("."))

    return (
        int(match.group(1)),
        int(match.group(2)),
        int(match.group(3)),
        1 if prerelease is None else 0,
        prerelease_key,
    )


def _prerelease_identifier_key(value: str) -> tuple[int, object]:
    if value.isdigit():
        return (0, int(value))
    return (1, value)
