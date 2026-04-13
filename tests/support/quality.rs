#![allow(dead_code)]
/**
@module SPECIAL.TESTS.SUPPORT.QUALITY
Release-review/tag-flow test helpers in `tests/support/quality.rs`.
*/
// @implements SPECIAL.TESTS.SUPPORT.QUALITY
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use serde_json::Value;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn clippy_script() -> String {
    fs::read_to_string(repo_root().join("scripts/verify-rust-clippy.sh"))
        .expect("clippy verification script should be readable")
}

pub fn release_review_script() -> String {
    fs::read_to_string(repo_root().join("scripts/review-rust-release-style.py"))
        .expect("release review script should be readable")
}

pub fn release_tag_script() -> String {
    fs::read_to_string(repo_root().join("scripts/tag-release.py"))
        .expect("release tag script should be readable")
}

pub fn workflow_files() -> Vec<PathBuf> {
    fs::read_dir(repo_root().join(".github/workflows"))
        .expect("workflow directory should be readable")
        .map(|entry| entry.expect("workflow entry should be readable").path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect()
}

pub fn release_review_schema() -> Value {
    serde_json::from_str(
        &fs::read_to_string(repo_root().join("scripts/rust-release-review.schema.json"))
            .expect("release review schema should be readable"),
    )
    .expect("release review schema should be valid json")
}

pub fn release_review_dry_run(args: &[&str]) -> Value {
    let output = Command::new("python3")
        .arg("scripts/review-rust-release-style.py")
        .args(args)
        .arg("--dry-run")
        .current_dir(repo_root())
        .output()
        .expect("release review dry-run should run");
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("dry-run output should be valid json")
}

pub fn current_python_executable() -> String {
    let output = Command::new("python3")
        .arg("-c")
        .arg("import sys; print(sys.executable)")
        .current_dir(repo_root())
        .output()
        .expect("python executable probe should run");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("python executable should be utf-8")
        .trim()
        .to_string()
}

pub fn review_script_path() -> String {
    repo_root()
        .join("scripts/review-rust-release-style.py")
        .display()
        .to_string()
}

pub fn release_review_python_helper(script: &str, args: &[&str]) -> Value {
    let output = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(repo_root())
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("release review helper should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("helper output should be valid json")
}

fn release_review_extract_context_ranges_with_mode(
    path: &str,
    content: &str,
    start: i64,
    end: i64,
    full_scan: bool,
) -> Value {
    let script = r#"
import importlib.util
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
path = sys.argv[2]
content = sys.argv[3]
start = int(sys.argv[4])
end = int(sys.argv[5])
full_scan = sys.argv[6] == "true"
spec = importlib.util.spec_from_file_location(
    "release_review", root / "scripts" / "review-rust-release-style.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(json.dumps(module.extract_context_ranges(path, content, [(start, end)], full_scan)))
"#;

    release_review_python_helper(
        script,
        &[
            path,
            content,
            &start.to_string(),
            &end.to_string(),
            if full_scan { "true" } else { "false" },
        ],
    )
}

pub fn release_review_extract_context_ranges(
    path: &str,
    content: &str,
    start: i64,
    end: i64,
) -> Value {
    release_review_extract_context_ranges_with_mode(path, content, start, end, false)
}

pub fn release_review_extract_full_scan_context_ranges(path: &str, content: &str) -> Value {
    release_review_extract_context_ranges_with_mode(path, content, 1, 1, true)
}

pub fn release_review_chunk_helper(context_chars: usize) -> Value {
    let script = r#"
import importlib.util
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
context_chars = int(sys.argv[2])
spec = importlib.util.spec_from_file_location(
    "release_review", root / "scripts" / "review-rust-release-style.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
review_pass = {"name": "test", "focus": ["budgeting"], "files": ["src/example.rs"]}
contexts = [
    {
        "path": "src/example.rs",
        "start_line": 1,
        "end_line": 1,
        "content": "x" * context_chars,
    },
    {
        "path": "src/example.rs",
        "start_line": 2,
        "end_line": 2,
        "content": "y" * context_chars,
    },
]
chunks, runner_warnings = module.build_pass_chunks(
    root,
    sys.argv[3],
    "jj",
    None,
    "@",
    True,
    review_pass,
    contexts,
)
print(json.dumps({"chunks": chunks, "runner_warnings": runner_warnings}))
"#;

    let version = current_package_version();
    release_review_python_helper(script, &[&context_chars.to_string(), &version])
}

pub fn release_review_changed_line_ranges(diff: &str) -> Value {
    let script = r#"
import importlib.util
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
diff = sys.argv[2]
spec = importlib.util.spec_from_file_location(
    "release_review", root / "scripts" / "review-rust-release-style.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(json.dumps(module.parse_changed_line_ranges(diff)))
"#;

    release_review_python_helper(script, &[diff])
}

pub fn release_review_passes_for(files: &[&str]) -> Value {
    let script = r#"
import importlib.util
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
files = sys.argv[2:]
spec = importlib.util.spec_from_file_location(
    "release_review", root / "scripts" / "review-rust-release-style.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(json.dumps(module.build_review_passes(files)))
"#;

    release_review_python_helper(script, files)
}

pub fn release_review_merge_responses(responses: &Value) -> Value {
    let script = r#"
import importlib.util
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
responses = json.loads(sys.argv[2])
spec = importlib.util.spec_from_file_location(
    "release_review", root / "scripts" / "review-rust-release-style.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(json.dumps(module.merge_pass_responses("v0.2.0", False, responses, [])))
"#;

    let responses_json = serde_json::to_string(responses).expect("responses should serialize");
    release_review_python_helper(script, &[&responses_json])
}

fn release_review_validate_response_shape_output(payload: &str) -> std::process::Output {
    let script = r#"
import importlib.util
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
payload = json.loads(sys.argv[2])
spec = importlib.util.spec_from_file_location(
    "release_review", root / "scripts" / "review-rust-release-style.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
try:
    validated = module.validate_response_shape(payload)
    print(json.dumps(validated))
except SystemExit as err:
    print(str(err), file=sys.stderr)
    raise
"#;

    Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(repo_root())
        .arg(payload)
        .current_dir(repo_root())
        .output()
        .expect("response validation helper should run")
}

pub fn release_review_validate_response_shape_ok(payload: &str) -> Value {
    let output = release_review_validate_response_shape_output(payload);
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("validated response should be valid json")
}

pub fn release_review_validate_response_shape_err(payload: &str) -> String {
    let output = release_review_validate_response_shape_output(payload);
    assert!(
        !output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stderr).expect("stderr should be utf-8")
}

pub fn split_stdout_json_prefix(output: &std::process::Output) -> (Value, String) {
    let mut stream = serde_json::Deserializer::from_slice(&output.stdout).into_iter::<Value>();
    let payload = stream
        .next()
        .expect("stdout should begin with json")
        .expect("stdout prefix should be valid json");
    let offset = stream.byte_offset();
    let remainder = String::from_utf8_lossy(&output.stdout[offset..]).to_string();
    (payload, remainder)
}

fn release_tag_validate_preview_shape_output(payload: &str) -> std::process::Output {
    let script = r#"
import importlib.util
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
payload = json.loads(sys.argv[2])
spec = importlib.util.spec_from_file_location(
    "tag_release", root / "scripts" / "tag-release.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
try:
    validated = module.validate_review_preview(payload)
    print(json.dumps(validated))
except SystemExit as err:
    print(str(err), file=sys.stderr)
    raise
"#;

    Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(repo_root())
        .arg(payload)
        .current_dir(repo_root())
        .output()
        .expect("preview validation helper should run")
}

pub fn release_tag_validate_preview_shape_ok(payload: &str) -> Value {
    let output = release_tag_validate_preview_shape_output(payload);
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("validated preview should be valid json")
}

pub fn release_tag_validate_preview_shape_err(payload: &str) -> String {
    let output = release_tag_validate_preview_shape_output(payload);
    assert!(
        !output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stderr).expect("stderr should be utf-8")
}

pub fn python_entrypoint_runtime_flag(script_name: &str) -> Value {
    let script_path = repo_root().join("scripts").join(script_name);
    let script = r#"
import importlib.util
import json
import pathlib
import sys

script_path = pathlib.Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("entrypoint", script_path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
print(json.dumps({"dont_write_bytecode": sys.dont_write_bytecode}))
"#;
    let script_arg = script_path.to_string_lossy().into_owned();
    let output = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(script_arg)
        .current_dir(repo_root())
        .output()
        .expect("entrypoint runtime flag helper should run");
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .expect("entrypoint runtime flag output should be valid json")
}

pub fn latest_semver_tag() -> String {
    let jj_output = Command::new("jj")
        .args(["tag", "list"])
        .current_dir(repo_root())
        .output()
        .expect("jj tag list should run");
    assert!(jj_output.status.success());

    let mut versions = Vec::new();
    for line in String::from_utf8_lossy(&jj_output.stdout).lines() {
        let Some((tag, _)) = line.split_once(':') else {
            continue;
        };
        let tag = tag.trim();
        let Some(version) = parse_semver_sort_key(tag) else {
            continue;
        };
        versions.push((version, tag.to_string()));
    }

    versions.sort();
    versions
        .last()
        .map(|(_, tag)| tag.clone())
        .expect("repo should have at least one semver tag")
}

type SemverSortKey = (u64, u64, u64, u8, Vec<(u8, String)>);

fn parse_semver_sort_key(tag: &str) -> Option<SemverSortKey> {
    let stripped = tag.strip_prefix('v').unwrap_or(tag);
    let (core, prerelease) = match stripped.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (stripped, None),
    };
    let mut parts = core.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    let (Ok(major), Ok(minor), Ok(patch)) = (
        major.parse::<u64>(),
        minor.parse::<u64>(),
        patch.parse::<u64>(),
    ) else {
        return None;
    };
    let prerelease_key = prerelease
        .map(|value| {
            value
                .split('.')
                .map(|part| {
                    if part.chars().all(|ch| ch.is_ascii_digit()) {
                        (0, format!("{:020}:{}", part.len(), part))
                    } else {
                        (1, part.to_string())
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    Some((
        major,
        minor,
        patch,
        if prerelease.is_none() { 1 } else { 0 },
        prerelease_key,
    ))
}

pub fn current_revision() -> String {
    let output = Command::new("jj")
        .args(["log", "-r", "@", "--no-graph", "-T", "commit_id"])
        .current_dir(repo_root())
        .output()
        .expect("jj log should run");
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("revision should be utf-8")
        .trim()
        .to_string()
}

pub fn release_tag_command_output(
    version: &str,
    extra_args: &[&str],
    mock_output: &str,
    mock_exit_code: Option<&str>,
) -> std::process::Output {
    let mut command = Command::new("python3");
    command
        .arg("scripts/tag-release.py")
        .arg(version)
        .args(extra_args)
        .arg("--dry-run")
        .arg("--allow-mock-review")
        .current_dir(repo_root())
        .env("SPECIAL_RUST_RELEASE_REVIEW_ALLOW_MOCK", "1")
        .env("SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT", mock_output);
    if let Some(code) = mock_exit_code {
        command.env("SPECIAL_RUST_RELEASE_REVIEW_MOCK_EXIT_CODE", code);
    }
    command.output().expect("tag release script should run")
}

pub fn release_tag_dry_run(version: &str, extra_args: &[&str], mock_output: &str) -> Value {
    let output = release_tag_command_output(version, extra_args, mock_output, None);
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("tag dry-run output should be valid json")
}

pub fn release_tag_live_output(
    version: &str,
    extra_args: &[&str],
    mock_output: &str,
    mock_exit_code: Option<&str>,
) -> std::process::Output {
    let mut command = Command::new("python3");
    command
        .arg("scripts/tag-release.py")
        .arg(version)
        .args(extra_args)
        .arg("--allow-mock-review")
        .current_dir(repo_root())
        .env("SPECIAL_RUST_RELEASE_REVIEW_ALLOW_MOCK", "1")
        .env("SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT", mock_output);
    if let Some(code) = mock_exit_code {
        command.env("SPECIAL_RUST_RELEASE_REVIEW_MOCK_EXIT_CODE", code);
    }
    command.output().expect("tag release script should run")
}

pub fn release_tag_live_output_with_input(
    version: &str,
    extra_args: &[&str],
    mock_output: &str,
    mock_exit_code: Option<&str>,
    input: &str,
) -> std::process::Output {
    let mut command = Command::new("python3");
    command
        .arg("scripts/tag-release.py")
        .arg(version)
        .args(extra_args)
        .arg("--allow-mock-review")
        .current_dir(repo_root())
        .env("SPECIAL_RUST_RELEASE_REVIEW_ALLOW_MOCK", "1")
        .env("SPECIAL_RUST_RELEASE_REVIEW_MOCK_OUTPUT", mock_output)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(code) = mock_exit_code {
        command.env("SPECIAL_RUST_RELEASE_REVIEW_MOCK_EXIT_CODE", code);
    }

    let mut child = command.spawn().expect("tag release script should run");
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("warning prompt input should be written");
    let _ = child.stdin.take();
    child
        .wait_with_output()
        .expect("tag release output should be captured")
}

pub fn current_package_version() -> String {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION
        .get_or_init(|| {
            let cargo_toml = fs::read_to_string(repo_root().join("Cargo.toml"))
                .expect("Cargo.toml should be readable");
            let parsed: toml::Value = toml::from_str(&cargo_toml).expect("Cargo.toml should parse");
            parsed["package"]["version"]
                .as_str()
                .expect("Cargo.toml should contain a package.version")
                .to_string()
        })
        .clone()
}
