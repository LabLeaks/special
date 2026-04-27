#!/usr/bin/env python3
# @module SPECIAL.DISTRIBUTION.RELEASE_FLOW
# Local release publication flow in `scripts/tag-release.py`.
# @fileimplements SPECIAL.DISTRIBUTION.RELEASE_FLOW

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from release_tooling import normalize_tag, package_version, run_checked

GITHUB_RELEASE_POLL_SECONDS = 10
GITHUB_RELEASE_TIMEOUT_SECONDS = 15 * 60
CORE_VALIDATION_COMMANDS = [
    ["mise", "exec", "--", "cargo", "test"],
    ["mise", "exec", "--", "cargo", "run", "--", "lint"],
    ["mise", "exec", "--", "cargo", "run", "--", "specs", "--metrics"],
    [sys.executable, str(SCRIPT_DIR / "verify-lean-traceability-kernel.py")],
]
PRIVATE_TRACKED_PATH_PATTERNS = [
    re.compile(pattern)
    for pattern in [
        r"^_project/",
        r"^BACKLOG(?:\.md)?$",
        r"^\.codex-evals/",
        r"(^|/)__pycache__/",
        r"\.pyc$",
        r"(^|/)\.lake/",
        r"(^|/)lake-manifest\.json$",
        r"^target/",
        r"^\.agents/",
        r"^\.projector/",
    ]
]


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run the deterministic release pipeline: prepare release notes, validate, "
            "then publish by updating main, tagging, pushing, verifying GitHub, and updating Homebrew."
        )
    )
    parser.add_argument("version", help="Release version, with or without a leading `v`.")
    phase = parser.add_mutually_exclusive_group()
    phase.add_argument(
        "--prepare",
        action="store_true",
        help="Write the exact CHANGELOG.md section from interactive release bullets and stop.",
    )
    phase.add_argument(
        "--validate",
        action="store_true",
        help="Run deterministic release validation and record ignored evidence for this revision.",
    )
    phase.add_argument(
        "--publish",
        action="store_true",
        help="Publish the prepared and validated release revision.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the planned pipeline and publication commands without mutating or publishing.",
    )
    parser.add_argument(
        "--skip-checklist",
        dest="skip_checklist",
        action="store_true",
        help=argparse.SUPPRESS,
    )
    parser.add_argument("--yes", dest="skip_checklist", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--allow-mock-publish", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--allow-existing-tag", action="store_true", help=argparse.SUPPRESS)
    return parser.parse_args()


def release_version(value: str) -> str:
    return normalize_tag(value).removeprefix("v")


def existing_tags(root: Path) -> set[str]:
    output = run_checked(root, ["jj", "tag", "list"])
    return {line.split(":", 1)[0].strip() for line in output.splitlines() if ":" in line}


def revision_id(root: Path, revset: str) -> str:
    return run_checked(root, ["jj", "log", "-r", revset, "--no-graph", "-T", "commit_id"]).strip()


def is_empty_revision(root: Path, revset: str) -> bool:
    return run_checked(root, ["jj", "log", "-r", revset, "--no-graph", "-T", "empty"]).strip() == "true"


def is_conflicted_revision(root: Path, revset: str) -> bool:
    return (
        run_checked(root, ["jj", "log", "-r", revset, "--no-graph", "-T", "conflict"]).strip()
        == "true"
    )


def revision_description(root: Path, revset: str) -> str:
    return run_checked(root, ["jj", "log", "-r", revset, "--no-graph", "-T", "description"])


def release_revset(root: Path) -> str:
    return "@-" if is_empty_revision(root, "@") else "@"


def release_revision(root: Path) -> str:
    return revision_id(root, release_revset(root))


def require_clean_working_copy(root: Path) -> None:
    status = run_checked(root, ["jj", "status", "--no-pager"])
    if (
        "The working copy is clean" not in status
        and "The working copy has no changes." not in status
     ):
        raise SystemExit(
            "release publishing requires a clean working copy; commit or revert changes in `@` first"
        )


def require_releasable_revision(root: Path, revset: str) -> None:
    if is_conflicted_revision(root, revset):
        raise SystemExit(f"release revision `{revset}` has conflicts")
    if not revision_description(root, revset).strip():
        raise SystemExit(f"release revision `{revset}` must have a description")


def tag_revision(root: Path, tag: str) -> str:
    return run_checked(root, ["jj", "log", "-r", tag, "--no-graph", "-T", "commit_id"]).strip()


def changelog_section_pattern(version: str) -> re.Pattern[str]:
    escaped = re.escape(version)
    return re.compile(rf"^##\s+v?{escaped}(?:\s+-\s+\d{{4}}-\d{{2}}-\d{{2}})?\s*$", re.M)


def changelog_section_body(text: str, version: str) -> str | None:
    match = changelog_section_pattern(version).search(text)
    if not match:
        return None
    next_heading = re.search(r"^##\s+", text[match.end() :], re.M)
    end = match.end() + next_heading.start() if next_heading else len(text)
    return text[match.end() : end]


def require_changelog_entry(root: Path, version: str) -> None:
    changelog = root / "CHANGELOG.md"
    if not changelog.is_file():
        raise SystemExit("CHANGELOG.md is missing; write release notes before publishing")
    text = changelog.read_text(encoding="utf-8")
    body = changelog_section_body(text, version)
    if body is None:
        today = dt.date.today().isoformat()
        raise SystemExit(
            "CHANGELOG.md is missing the release section for this version; "
            f"add `## {version} - {today}` with release-visible changes before publishing"
        )
    if "TODO" in body or not any(line.strip().startswith("- ") for line in body.splitlines()):
        raise SystemExit(
            f"CHANGELOG.md section for {version} must contain real bullet notes, not placeholders"
        )
    if re.search(r"^##\s+Unreleased\s*$", text, re.M):
        raise SystemExit(
            "CHANGELOG.md contains an `Unreleased` section; release notes must be attached to the exact release version"
        )


def tracked_files(root: Path) -> list[str]:
    output = run_checked(root, ["jj", "file", "list"])
    return [line.strip() for line in output.splitlines() if line.strip()]


def require_no_private_tracked_files(root: Path) -> None:
    matches = [
        path
        for path in tracked_files(root)
        if any(pattern.search(path) for pattern in PRIVATE_TRACKED_PATH_PATTERNS)
    ]
    if matches:
        formatted = "\n".join(f"  - {path}" for path in matches)
        raise SystemExit(
            "release publishing refuses tracked private/generated paths:\n"
            f"{formatted}\n"
            "move private notes under ignored _project/ or remove generated artifacts from version control"
        )


def require_release_preflight(root: Path, version: str) -> None:
    require_changelog_entry(root, version)
    require_no_private_tracked_files(root)


def collect_changelog_bullets(version: str) -> list[str]:
    print(f"Release changelog for {version}.")
    print("Enter release-visible bullet lines. Blank line finishes.")
    print("Do not include local cleanup notes that are not release-visible.")
    bullets: list[str] = []
    while True:
        try:
            line = input("> ").strip()
        except EOFError as err:
            raise SystemExit("changelog entry is required; no bullets were provided") from err
        if not line:
            break
        bullet = line[2:].strip() if line.startswith("- ") else line
        if bullet:
            bullets.append(bullet)
    if not bullets:
        raise SystemExit("at least one release-visible changelog bullet is required")
    if any("TODO" in bullet.upper() or "TBD" in bullet.upper() for bullet in bullets):
        raise SystemExit("changelog bullets must be real release notes, not placeholders")
    return bullets


def upsert_changelog_section(root: Path, version: str, bullets: list[str]) -> None:
    changelog = root / "CHANGELOG.md"
    if not changelog.is_file():
        raise SystemExit("CHANGELOG.md is missing")
    text = changelog.read_text(encoding="utf-8")
    section = "## {} - {}\n\n{}\n\n".format(
        version,
        dt.date.today().isoformat(),
        "\n".join(f"- {bullet}" for bullet in bullets),
    )
    match = changelog_section_pattern(version).search(text)
    if match:
        next_heading = re.search(r"^##\s+", text[match.end() :], re.M)
        end = match.end() + next_heading.start() if next_heading else len(text)
        updated = text[: match.start()] + section + text[end:]
    else:
        title_match = re.match(r"^# .*\n+", text)
        if not title_match:
            raise SystemExit("CHANGELOG.md must start with a top-level title")
        insert_at = title_match.end()
        updated = text[:insert_at] + "\n" + section + text[insert_at:]
    changelog.write_text(updated, encoding="utf-8")
    print(f"Wrote CHANGELOG.md section for {version}.")
    print("Review it, include it in the release revision, rerun validation, then publish.")


def evidence_path(root: Path, version: str) -> Path:
    return root / "_project" / "release" / f"{version}.json"


def write_validation_evidence(root: Path, version: str, revision: str, commands: list[list[str]]) -> None:
    path = evidence_path(root, version)
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "version": version,
        "revision": revision,
        "validated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "commands": commands,
    }
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote validation evidence to {path}")


def require_validation_evidence(root: Path, version: str, revision: str) -> None:
    path = evidence_path(root, version)
    if not path.is_file():
        raise SystemExit(
            f"release validation evidence is missing at {path}; run `python3 scripts/tag-release.py {version} --validate`"
        )
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("version") != version or payload.get("revision") != revision:
        raise SystemExit(
            f"release validation evidence at {path} does not match version {version} and revision {revision}"
        )


def run_command(root: Path, command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=root,
        capture_output=True,
        text=True,
    )


def record_mock_step(root: Path, label: str, command: list[str]) -> None:
    log_override = os.environ.get("SPECIAL_RELEASE_MOCK_LOG_PATH")
    log_path = Path(log_override) if log_override else root / ".tmp-release-mock-log"
    with log_path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps({"label": label, "command": command}) + "\n")


def run_step(root: Path, label: str, command: list[str], args: argparse.Namespace) -> None:
    if args.allow_mock_publish:
        record_mock_step(root, label, command)
        return
    run_checked(root, command)


def run_validation(root: Path, version: str, revision: str, args: argparse.Namespace) -> None:
    completed: list[list[str]] = []
    for command in CORE_VALIDATION_COMMANDS:
        if args.allow_mock_publish:
            record_mock_step(root, "validate", command)
        else:
            print(f"+ {' '.join(command)}")
            run_checked(root, command)
        completed.append(command)
    write_validation_evidence(root, version, revision, completed)


def wait_for_github_release(root: Path, label: str, command: list[str], args: argparse.Namespace) -> None:
    if args.allow_mock_publish:
        record_mock_step(root, label, command)
        return
    deadline = time.monotonic() + GITHUB_RELEASE_TIMEOUT_SECONDS
    last_stderr = ""
    while True:
        result = run_command(root, command)
        if result.returncode == 0:
            return
        last_stderr = result.stderr
        if time.monotonic() >= deadline:
            if last_stderr:
                sys.stderr.write(last_stderr)
            raise SystemExit("timed out waiting for GitHub release publication")
        time.sleep(GITHUB_RELEASE_POLL_SECONDS)


def main() -> int:
    args = parse_args()
    if args.skip_checklist:
        raise SystemExit(
            "`--skip-checklist`/`--yes` was removed; use --prepare, --validate, then --publish"
        )
    if not (args.prepare or args.validate or args.publish or args.dry_run):
        raise SystemExit("choose exactly one release phase: --prepare, --validate, or --publish")
    root = repo_root()
    version = release_version(args.version)
    tag = normalize_tag(args.version)
    manifest_tag = normalize_tag(package_version(root))

    if not root.joinpath(".jj").exists():
        raise SystemExit("release publishing requires a jj repository root")
    target = release_revset(root)
    revision = release_revision(root)
    tag_exists = tag in existing_tags(root)
    if tag_exists and not args.allow_existing_tag:
        raise SystemExit(f"release tag `{tag}` already exists")
    if tag != manifest_tag:
        raise SystemExit(
            f"release tag `{tag}` does not match Cargo.toml version `{manifest_tag}`"
        )

    if args.prepare:
        bullets = collect_changelog_bullets(version)
        upsert_changelog_section(root, version, bullets)
        return 0

    bookmark_command = ["jj", "bookmark", "set", "main", "-r", revision]
    tag_command = ["jj", "tag", "set", tag, "-r", revision]
    if tag_exists and args.allow_existing_tag:
        tag_command.insert(3, "--allow-move")
    push_main_command = ["jj", "git", "push", "--bookmark", "main"]
    push_tag_command = ["git", "push"]
    if tag_exists and args.allow_existing_tag:
        push_tag_command.append("--force")
    push_tag_command.extend(["origin", f"refs/tags/{tag}"])
    verify_github_release_command = [
        "bash",
        str(SCRIPT_DIR / "verify-github-release-published.sh"),
    ]
    update_homebrew_formula_command = [
        sys.executable,
        str(SCRIPT_DIR / "update-homebrew-formula.py"),
    ]
    verify_homebrew_formula_command = [
        "bash",
        str(SCRIPT_DIR / "verify-homebrew-formula.sh"),
    ]

    if args.dry_run:
        print(
            json.dumps(
                {
                    "tag": tag,
                    "version": version,
                    "release_revset": target,
                    "revision": revision,
                    "pipeline": [
                        {
                            "phase": "prepare",
                            "command": ["python3", "scripts/tag-release.py", version, "--prepare"],
                            "produces": "CHANGELOG.md exact-version release section",
                        },
                        {
                            "phase": "validate",
                            "command": ["python3", "scripts/tag-release.py", version, "--validate"],
                            "produces": str(evidence_path(root, version)),
                        },
                        {
                            "phase": "publish",
                            "command": ["python3", "scripts/tag-release.py", version, "--publish"],
                            "requires": [
                                "clean release revision",
                                "CHANGELOG.md exact-version section",
                                "no tracked private/generated paths",
                                "validation evidence for this version and revision",
                            ],
                        },
                    ],
                    "validation_commands": CORE_VALIDATION_COMMANDS,
                    "bookmark_command": bookmark_command,
                    "tag_command": tag_command,
                    "push_main_command": push_main_command,
                    "push_tag_command": push_tag_command,
                    "verify_github_release_command": verify_github_release_command,
                    "update_homebrew_formula_command": update_homebrew_formula_command,
                    "verify_homebrew_formula_command": verify_homebrew_formula_command,
                },
                indent=2,
            )
        )
        return 0

    require_release_preflight(root, version)

    if args.validate:
        run_validation(root, version, revision, args)
        return 0

    if not args.allow_mock_publish:
        require_clean_working_copy(root)
        require_releasable_revision(root, target)
        require_validation_evidence(root, version, revision)

    run_step(root, "bookmark_main", bookmark_command, args)
    if not (tag_exists and tag_revision(root, tag) == revision):
        run_step(root, "set_tag", tag_command, args)
    run_step(root, "push_main", push_main_command, args)
    run_step(root, "push_tag", push_tag_command, args)
    wait_for_github_release(root, "verify_github_release", verify_github_release_command, args)
    run_step(root, "update_homebrew_formula", update_homebrew_formula_command, args)
    run_step(root, "verify_homebrew_formula", verify_homebrew_formula_command, args)
    print(f"Published release {tag}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
