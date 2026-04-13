#!/usr/bin/env python3
# @module SPECIAL.RELEASE_REVIEW.TAG
# Release tag flow in `scripts/tag-release.py`.
# @implements SPECIAL.RELEASE_REVIEW.TAG
# @verifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.RELEASE_TAG_FLOW

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import argparse
import json
import subprocess
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from release_review_contract import (
    validate_review_payload as validate_release_review_payload,
    validate_review_preview as validate_release_review_preview,
)
from release_tooling import normalize_tag, package_version, run_checked


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run the Rust release review, then create a jj release tag."
    )
    parser.add_argument("version", help="Release version, with or without a leading `v`.")
    model_group = parser.add_mutually_exclusive_group()
    model_group.add_argument(
        "--fast",
        action="store_true",
        help="Run the release review with the fast Spark model.",
    )
    model_group.add_argument(
        "--smart",
        action="store_true",
        help="Run the release review with the smarter GPT-5.4 model.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run the release review and print the planned tag command without creating the tag.",
    )
    parser.add_argument(
        "--skip-review",
        action="store_true",
        help="Skip the Rust release review and create the tag directly.",
    )
    review_scope_group = parser.add_mutually_exclusive_group()
    review_scope_group.add_argument(
        "--full",
        action="store_true",
        help="Run the release review in full-scan mode instead of the default previous-tag diff.",
    )
    review_scope_group.add_argument(
        "--base",
        help="Run the release review against an explicit base revision instead of the latest semver tag.",
    )
    parser.add_argument(
        "--yes",
        action="store_true",
        help="Create the tag without prompting when the release review returns warnings.",
    )
    parser.add_argument("--allow-mock-review", action="store_true", help=argparse.SUPPRESS)
    return parser.parse_args()

def existing_tags(root: Path) -> set[str]:
    output = run_checked(root, ["jj", "tag", "list"])
    return {line.split(":", 1)[0].strip() for line in output.splitlines() if ":" in line}


def current_revision(root: Path) -> str:
    return run_checked(root, ["jj", "log", "-r", "@", "--no-graph", "-T", "commit_id"]).strip()


def release_review_command(
    args: argparse.Namespace, revision: str, *, dry_run: bool = False
) -> list[str]:
    command = [sys.executable, str(SCRIPT_DIR / "review-rust-release-style.py")]
    if args.fast:
        command.append("--fast")
    elif args.smart:
        command.append("--smart")
    if args.full:
        command.append("--full")
    elif args.base:
        command.extend(["--base", args.base])
    command.extend(["--head", revision])
    if dry_run:
        command.append("--dry-run")
    if getattr(args, "allow_mock_review", False):
        command.append("--allow-mock")
    return command


def validate_review_payload(payload: dict) -> dict:
    return validate_release_review_payload(payload, subject="release review payload")


def validate_review_preview(payload: dict) -> dict:
    return validate_release_review_preview(payload, subject="release review dry-run preview")


def run_release_review(root: Path, command: list[str]) -> dict:
    result = subprocess.run(
        command,
        cwd=root,
        capture_output=True,
        text=True,
    )

    payload = None
    stdout = result.stdout.strip()
    if stdout:
        try:
            payload = json.loads(stdout)
        except json.JSONDecodeError:
            payload = None

    if result.returncode != 0:
        if payload is not None:
            if result.stderr:
                sys.stderr.write(result.stderr)
            print(json.dumps(payload, indent=2))
            raise SystemExit("release review did not complete cleanly; tag not created")

        if result.stderr:
            sys.stderr.write(result.stderr)
        raise SystemExit(result.returncode)

    if payload is None:
        raise SystemExit("release review did not return valid JSON")
    return validate_review_payload(payload)


def run_json_command(root: Path, command: list[str], failure_message: str) -> dict:
    result = subprocess.run(
        command,
        cwd=root,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        if result.stderr:
            sys.stderr.write(result.stderr)
        raise SystemExit(failure_message)

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as err:
        raise SystemExit(f"{failure_message}: invalid JSON output") from err
    return validate_review_preview(payload)


def prompt_to_continue(tag: str, warning_count: int) -> None:
    try:
        answer = input(
            f"Rust release review returned {warning_count} warning(s). Create {tag} anyway? [y/N]: "
        ).strip().lower()
    except EOFError as err:
        raise SystemExit(
            "release review returned warnings and interactive confirmation is unavailable; "
            "rerun with --yes to create the tag"
        ) from err
    if answer not in {"y", "yes"}:
        raise SystemExit("aborted release tagging")


def main() -> int:
    args = parse_args()
    root = repo_root()
    tag = normalize_tag(args.version)
    manifest_tag = normalize_tag(package_version(root))

    if not root.joinpath(".jj").exists():
        raise SystemExit("release tagging requires a jj repository root")
    revision = current_revision(root)
    if tag in existing_tags(root):
        raise SystemExit(f"release tag `{tag}` already exists")
    if tag != manifest_tag:
        raise SystemExit(
            f"release tag `{tag}` does not match Cargo.toml version `{manifest_tag}`"
        )

    tag_command = ["jj", "tag", "set", tag, "-r", revision]

    if args.dry_run:
        payload = {
            "tag": tag,
            "skip_review": args.skip_review,
            "tag_command": tag_command,
        }
        if not args.skip_review:
            review_command = release_review_command(args, revision, dry_run=True)
            review_preview = run_json_command(
                root,
                review_command,
                "release review dry-run did not complete cleanly; tag not created",
            )
            payload["review_command"] = review_command
            payload["review_preview"] = review_preview
        print(json.dumps(payload, indent=2))
        return 0

    if not args.skip_review:
        review_command = release_review_command(args, revision)
        review_result = run_release_review(root, review_command)
        warning_count = len(review_result.get("warnings", []))
        print(json.dumps(review_result, indent=2))

        if warning_count > 0 and not args.yes:
            prompt_to_continue(tag, warning_count)

    run_checked(root, tag_command)
    print(f"Created release tag {tag}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
