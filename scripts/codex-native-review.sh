#!/usr/bin/env bash
set -euo pipefail

scope="full"
review_prompt=""
default_full_prompt="Review the current repository state as it exists now, including dirty changes. Audit the whole system in full context rather than only recent edits. Focus on correctness, behavioral regressions, proof and test honesty, cache and toolchain consistency, and any places where the public claims outrun what the code and harnesses actually guarantee. Report findings only."
declare -a passthrough=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scope)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --scope (expected: full, uncommitted, base, commit)" >&2
        exit 2
      fi
      scope="$2"
      shift 2
      ;;
    --prompt)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --prompt" >&2
        exit 2
      fi
      review_prompt="$2"
      shift 2
      ;;
    *)
      passthrough+=("$1")
      shift
      ;;
  esac
done

declare -a review_args=()
case "$scope" in
  full)
    ;;
  uncommitted)
    review_args+=(--uncommitted)
    ;;
  base)
    review_args+=(--base)
    ;;
  commit)
    review_args+=(--commit)
    ;;
  *)
    echo "unsupported scope: $scope (expected: full, uncommitted, base, commit)" >&2
    exit 2
    ;;
esac

review_args+=("${passthrough[@]}")

if [[ "$scope" == "full" && -z "$review_prompt" ]]; then
  review_prompt="$default_full_prompt"
fi

out_dir="${TMPDIR:-/tmp}/special-codex-review"
mkdir -p "$out_dir"
timestamp="$(date +%Y%m%d-%H%M%S)"
log_path="$out_dir/review-$timestamp.log"

set +e
if [[ -n "$review_prompt" ]]; then
  codex exec review "${review_args[@]}" "$review_prompt" >"$log_path" 2>&1
else
  codex exec review "${review_args[@]}" >"$log_path" 2>&1
fi
status=$?
set -e

last_codex_line="$(awk '/^codex$/{ line = NR } END { print line + 0 }' "$log_path")"

if [[ "$last_codex_line" -gt 0 ]]; then
  sed -n "${last_codex_line},\$p" "$log_path"
else
  echo "No final codex findings block found. Raw log saved to: $log_path" >&2
  tail -n 120 "$log_path" >&2
  echo >&2
  echo "Raw review log: $log_path" >&2
  exit 1
fi

echo
echo "Raw review log: $log_path"
exit "$status"
