#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import sys
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LEAN_ROOT = ROOT / "lean"
LEAN_TOOLCHAIN = (LEAN_ROOT / "lean-toolchain").read_text().strip()


def mise_elan_lake(*args: str) -> list[str]:
    return [
        "mise",
        "exec",
        "--",
        "elan",
        "run",
        LEAN_TOOLCHAIN,
        "lake",
        *args,
    ]


def release_kernel_path() -> Path:
    suffix = ".exe" if sys.platform == "win32" else ""
    return LEAN_ROOT / ".lake" / "build" / "bin" / f"special_traceability_kernel{suffix}"


def build_release_kernel() -> Path:
    subprocess.run(
        mise_elan_lake("-Krelease", "build", "special_traceability_kernel"),
        cwd=LEAN_ROOT,
        check=True,
    )
    kernel = release_kernel_path()
    if not kernel.is_file():
        raise SystemExit(f"release Lean kernel was not built at {kernel}")
    return kernel


def run_kernel_fixture(kernel: Path) -> None:
    fixture = {
        "schema_version": 1,
        "projected_item_ids": ["app::live", "app::orphan"],
        "preserved_reverse_closure_target_ids": None,
        "edges": {
            "app::helper": ["app::live"],
            "tests::test_helper": ["app::helper"],
            "tests::test_live": ["app::live"],
        },
        "support_root_ids": ["tests::test_live"],
    }
    result = subprocess.run(
        [str(kernel)],
        cwd=LEAN_ROOT,
        input=json.dumps(fixture),
        text=True,
        capture_output=True,
        check=True,
    )
    json_lines = [line for line in result.stdout.splitlines() if line.startswith("{")]
    if not json_lines:
        raise SystemExit(f"Lean kernel did not emit JSON output:\n{result.stdout}")

    output = json.loads(json_lines[-1])
    reference = output["reference"]
    contract = reference["contract"]
    closure = reference["exact_reverse_closure"]

    assert output["schema_version"] == 1
    assert set(contract["projected_item_ids"]) == {"app::live", "app::orphan"}
    assert set(contract["preserved_reverse_closure_target_ids"]) == {"app::live"}
    assert set(closure["target_ids"]) == {"app::live"}
    assert set(closure["node_ids"]) == {
        "app::live",
        "app::helper",
        "tests::test_helper",
        "tests::test_live",
    }
    assert set(closure["internal_edges"]["app::helper"]) == {"app::live"}
    assert set(closure["internal_edges"]["tests::test_helper"]) == {"app::helper"}
    assert set(closure["internal_edges"]["tests::test_live"]) == {"app::live"}


def run_rust_equivalence_tests(kernel: Path) -> None:
    env = os.environ.copy()
    env["SPECIAL_TRACEABILITY_KERNEL_EXE"] = str(kernel)
    env["SPECIAL_REQUIRE_LEAN_KERNEL_TESTS"] = "1"
    for test_name in [
        "projected_traceability_lean_kernel_matches_rust_reference_cases",
        "projected_traceability_kernel_default_uses_lean_when_available",
    ]:
        subprocess.run(
            [
                "mise",
                "exec",
                "--",
                "cargo",
                "test",
                test_name,
            ],
            cwd=ROOT,
            env=env,
            check=True,
        )


def main() -> int:
    kernel = build_release_kernel()
    run_kernel_fixture(kernel)
    run_rust_equivalence_tests(kernel)
    print("Release Lean traceability kernel verified.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
