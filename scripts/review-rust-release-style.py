#!/usr/bin/env python3
# @module SPECIAL.RELEASE_REVIEW
# Local-only code-quality review entrypoint that invokes Codex, merges review results, and surfaces release warnings for tagging. Context planning and payload validation live in dedicated helper modules.
# @fileimplements SPECIAL.RELEASE_REVIEW
# @fileverifies SPECIAL.QUALITY.RUST.RELEASE_REVIEW.SPEC_OWNED

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import argparse
import concurrent.futures
import hashlib
import json
import os
import subprocess
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from release_review_contract import validate_review_payload
from release_review_pipeline import (
    MAX_CONCURRENT_REVIEW_CHUNKS,
    build_file_contexts,
    build_pass_chunks,
    build_review_passes,
    changed_files_from_diff,
    command_exists,
    discover_latest_semver_tag,
    diff_text_for_paths,
    extract_context_ranges,
    full_scan_files,
    parse_changed_line_ranges,
)
from release_tooling import package_version


DEFAULT_MODEL = "gpt-5.3-codex"
FAST_MODEL = "gpt-5.3-codex-spark"
SMART_MODEL = "gpt-5.4"
PERMISSIONS_PROFILE = "release_review"
MOCK_ALLOW_ENV = "SPECIAL_RUST_RELEASE_REVIEW_ALLOW_MOCK"
SCHEMA_PATH = Path(__file__).with_name("rust-release-review.schema.json")


class CodexInvocationError(RuntimeError):
    pass


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run a warn-only Codex release review for Rust code-quality issues outside clippy's scope."
        )
    )
    parser.add_argument(
        "--base",
        help="Explicit base tag or revision to diff against. Defaults to the latest semver tag.",
    )
    parser.add_argument(
        "--full",
        action="store_true",
        help="Review the full Rust-relevant code surface instead of only the release diff.",
    )
    parser.add_argument("--head", help=argparse.SUPPRESS)
    model_group = parser.add_mutually_exclusive_group()
    model_group.add_argument(
        "--fast",
        action="store_true",
        help=f"Use the faster {FAST_MODEL} review model.",
    )
    model_group.add_argument(
        "--smart",
        action="store_true",
        help=f"Use the stronger {SMART_MODEL} review model.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the assembled review payload as JSON instead of invoking Codex.",
    )
    parser.add_argument("--allow-mock", action="store_true", help=argparse.SUPPRESS)
    return parser.parse_args()


def running_in_ci() -> bool:
    return any(
        os.environ.get(name, "").strip()
        for name in ("CI", "GITHUB_ACTIONS", "BUILDKITE", "JENKINS_URL")
    )


def selected_model(args: argparse.Namespace) -> tuple[str, str]:
    if args.fast:
        return ("fast", FAST_MODEL)
    if args.smart:
        return ("smart", SMART_MODEL)
    return ("default", DEFAULT_MODEL)


def has_jj_root(root: Path) -> bool:
    return root.joinpath(".jj").exists() and command_exists("jj")


def has_git_root(root: Path) -> bool:
    return root.joinpath(".git").exists() and command_exists("git")


def load_version(root: Path) -> str:
    return package_version(root)


def validate_response_shape(response: dict) -> dict:
    return validate_review_payload(response, subject="review response")


def quota_guidance(model_mode: str) -> str | None:
    if model_mode == "fast":
        return (
            f"{FAST_MODEL} appears quota-limited. Rerun without --fast for the default "
            f"{DEFAULT_MODEL} review, or use --smart for {SMART_MODEL}."
        )
    if model_mode == "smart":
        return (
            f"{SMART_MODEL} appears quota-limited. Rerun without --smart for the default "
            f"{DEFAULT_MODEL} review, or use --fast if Spark quota is available."
        )
    return None


def codex_invocation_config(model: str) -> dict[str, object]:
    return {
        "model": model,
        "sandbox_mode": "read-only",
        "web_search": "disabled",
        "default_permissions": PERMISSIONS_PROFILE,
        "filesystem_permissions": {
            ":project_roots": {
                ".": "read",
            }
        },
    }


def codex_exec_command(model: str) -> list[str]:
    config = codex_invocation_config(model)
    filesystem_toml = '{":project_roots"={"."="read"}}'
    return [
        "codex",
        "exec",
        "--ephemeral",
        "--sandbox",
        str(config["sandbox_mode"]),
        "-c",
        f'web_search="{config["web_search"]}"',
        "-c",
        f'default_permissions="{config["default_permissions"]}"',
        "-c",
        f"permissions.{config['default_permissions']}.filesystem={filesystem_toml}",
        "--skip-git-repo-check",
        "--model",
        model,
        "--output-schema",
        str(SCHEMA_PATH),
        "-",
    ]


def invoke_codex(root: Path, prompt: str, model: str, model_mode: str) -> dict:
    mocked = os.environ.get("SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT")
    if mocked and os.environ.get(MOCK_ALLOW_ENV) == "1":
        try:
            return validate_response_shape(json.loads(mocked))
        except (json.JSONDecodeError, SystemExit) as err:
            raise CodexInvocationError(f"mocked review output was invalid: {err}") from err

    result = subprocess.run(
        codex_exec_command(model),
        cwd=root,
        input=prompt,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        stderr = result.stderr.strip()
        lower = stderr.lower()
        guidance = (
            quota_guidance(model_mode)
            if any(token in lower for token in ("quota", "rate limit", "usage limit", "5hr", "7day"))
            else None
        )
        if guidance:
            raise CodexInvocationError(f"{stderr}\n{guidance}")
        raise CodexInvocationError(stderr or f"codex exited with status {result.returncode}")

    try:
        return validate_response_shape(json.loads(result.stdout))
    except (json.JSONDecodeError, SystemExit) as err:
        raise CodexInvocationError(f"codex returned invalid structured output: {err}") from err


def merge_pass_responses(
    base: str | None,
    full_scan: bool,
    responses: list[tuple[str, int, dict]],
    runner_warnings: list[str],
) -> dict:
    warnings: list[dict] = []
    seen: set[str] = set()

    for pass_name, chunk_index, response in responses:
        for warning in response.get("warnings", []):
            dedupe_key = warning_dedupe_key(warning)
            if dedupe_key in seen:
                continue
            seen.add(dedupe_key)
            merged = dict(warning)
            merged["id"] = stable_warning_id(pass_name, warning)
            warnings.append(merged)

    warnings.sort(key=warning_sort_key)

    if warnings:
        summary = f"{len(warnings)} warn-level issue(s) found across {len(responses)} review chunk(s)."
    else:
        summary = "No warn-level issues found across release review chunks."
    if runner_warnings:
        summary += f" {len(runner_warnings)} runner warning(s) occurred."

    return {
        "baseline": base,
        "full_scan": full_scan,
        "summary": summary,
        "warnings": warnings,
    }


def warning_dedupe_key(warning: dict) -> str:
    evidence = [
        {
            "path": item.get("path"),
            "line": item.get("line"),
            "detail": item.get("detail"),
        }
        for item in warning.get("evidence", [])
    ]
    return json.dumps(
        {
            "category": warning.get("category"),
            "title": warning.get("title"),
            "evidence": evidence,
        },
        sort_keys=True,
    )


def stable_warning_id(pass_name: str, warning: dict) -> str:
    digest = hashlib.sha1(warning_dedupe_key(warning).encode("utf-8")).hexdigest()[:12]
    return f"{pass_name}:{warning['category']}:{digest}"


def warning_sort_key(warning: dict) -> tuple[object, ...]:
    anchors = []
    for item in warning.get("evidence", []):
        anchors.append((item.get("path") or "", item.get("line") or 0, item.get("detail") or ""))
    anchors.sort()
    first_anchor = anchors[0] if anchors else ("", 0, "")
    return (
        warning.get("category", ""),
        first_anchor[0],
        first_anchor[1],
        warning.get("title", ""),
        warning.get("id", ""),
    )


def main() -> int:
    args = parse_args()
    root = repo_root()

    if not args.allow_mock:
        forbidden_mock_envs = [
            name
            for name in (
                MOCK_ALLOW_ENV,
                "SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT",
                "SPECIAL_RUST_RELEASE_REVIEW_MOCK_EXIT_CODE",
            )
            if os.environ.get(name) is not None
        ]
        if forbidden_mock_envs:
            raise SystemExit(
                "release review mock controls are test-only; unset "
                + ", ".join(sorted(forbidden_mock_envs))
            )

    if has_jj_root(root):
        backend = "jj"
    elif has_git_root(root):
        backend = "git"
    else:
        raise SystemExit("repository root must contain .jj or .git")

    review_mode, model = selected_model(args)
    version = load_version(root)
    base = None if args.full else (args.base or discover_latest_semver_tag(root, backend))
    head = args.head or ("@" if backend == "jj" else "HEAD")
    review_files = (
        full_scan_files(root, backend)
        if args.full
        else changed_files_from_diff(root, backend, base, head)
    )
    review_passes = build_review_passes(review_files)

    changed_line_ranges: dict[str, list[tuple[int, int]]] = {}
    if not args.full and review_files and base is not None:
        changed_line_ranges = parse_changed_line_ranges(
            diff_text_for_paths(root, backend, base, head, review_files)
        )

    chunk_records: list[dict[str, object]] = []
    runner_warnings: list[str] = []

    for review_pass in review_passes:
        pass_files = list(review_pass["files"])
        file_contexts = build_file_contexts(root, pass_files, changed_line_ranges, args.full)
        chunks, pass_runner_warnings = build_pass_chunks(
            root,
            version,
            backend,
            base,
            head,
            args.full,
            review_pass,
            file_contexts,
        )
        runner_warnings.extend(pass_runner_warnings)
        chunk_records.extend(chunks)

    if args.dry_run:
        print(
            json.dumps(
                {
                    "model": model,
                    "review_mode": review_mode,
                    "codex_invocation": codex_invocation_config(model),
                    "schema_path": str(SCHEMA_PATH),
                    "backend": backend,
                    "baseline": base,
                    "head": head,
                    "full_scan": args.full,
                    "changed_files": review_files,
                    "runner_warnings": runner_warnings,
                    "review_passes": [
                        {
                            "name": review_pass["name"],
                            "focus": review_pass["focus"],
                            "files": review_pass["files"],
                            "chunks": [
                                {
                                    "chunk_index": chunk["chunk_index"],
                                    "chunk_count": chunk["chunk_count"],
                                    "files": chunk["files"],
                                    "estimated_chars": chunk["estimated_chars"],
                                    "file_contexts": chunk["file_contexts"],
                                    "prompt": chunk["prompt"],
                                }
                                for chunk in chunk_records
                                if chunk["name"] == review_pass["name"]
                            ],
                        }
                        for review_pass in review_passes
                    ],
                },
                indent=2,
            )
        )
        return 0

    if running_in_ci():
        raise SystemExit(
            "rust release review is local-only and must not invoke Codex from CI; "
            "use --dry-run for wrapper verification"
        )

    if not chunk_records:
        payload = merge_pass_responses(base, args.full, [], runner_warnings)
        for warning in runner_warnings:
            print(warning, file=sys.stderr)
        print(json.dumps(payload, indent=2))
        return 1 if runner_warnings else 0

    responses: list[tuple[str, int, dict]] = []
    max_workers = min(len(chunk_records), MAX_CONCURRENT_REVIEW_CHUNKS)

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers or 1) as executor:
        future_to_chunk = {
            executor.submit(
                invoke_codex,
                root,
                str(chunk["prompt"]),
                model,
                review_mode,
            ): chunk
            for chunk in chunk_records
        }
        for future in concurrent.futures.as_completed(future_to_chunk):
            chunk = future_to_chunk[future]
            try:
                response = future.result()
            except CodexInvocationError as error:
                runner_warnings.append(
                    f"review chunk {chunk['name']}#{chunk['chunk_index']} failed: {error}"
                )
                continue
            except Exception as error:
                runner_warnings.append(
                    f"review chunk {chunk['name']}#{chunk['chunk_index']} crashed: {error}"
                )
                continue
            responses.append((str(chunk["name"]), int(chunk["chunk_index"]), response))

    responses.sort(key=lambda item: (item[0], item[1]))
    payload = merge_pass_responses(base, args.full, responses, runner_warnings)
    for warning in runner_warnings:
        print(warning, file=sys.stderr)
    print(json.dumps(payload, indent=2))
    mock_exit_code = os.environ.get("SPECIAL_RUST_RELEASE_REVIEW_MOCK_EXIT_CODE")
    if mock_exit_code is not None and os.environ.get(MOCK_ALLOW_ENV) == "1":
        return int(mock_exit_code)
    return 1 if runner_warnings else 0


if __name__ == "__main__":
    raise SystemExit(main())
