/**
@module SPECIAL.TESTS.SCOPED_HEALTH_BENCHMARK
Ignored benchmark harness for comparing full and targeted health analysis timing on real repositories.
*/
// @fileimplements SPECIAL.TESTS.SCOPED_HEALTH_BENCHMARK
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::json;

fn cache_bucket_for(root: &Path) -> PathBuf {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut hasher = DefaultHasher::new();
    root.hash(&mut hasher);
    let root_hash = hasher.finish();
    std::env::temp_dir()
        .join("special-cache")
        .join(format!("{root_hash:016x}"))
}

fn clear_cache_bucket(root: &Path) {
    let cache_bucket = cache_bucket_for(root);
    if cache_bucket.exists() {
        fs::remove_dir_all(&cache_bucket).expect("cache bucket should be removable");
    }
}

fn env_or_default(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_or_default_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn stderr_tail(stderr: &[u8], max_lines: usize) -> Vec<String> {
    let text = String::from_utf8_lossy(stderr);
    let lines = text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let keep_from = lines.len().saturating_sub(max_lines);
    lines.into_iter().skip(keep_from).collect()
}

fn run_special(repo_root: &Path, args: &[String], envs: &[(&str, &str)]) -> serde_json::Value {
    let special = std::env::var("CARGO_BIN_EXE_special")
        .expect("cargo should provide the built special binary path");
    let started = Instant::now();
    let mut command = Command::new(special);
    command
        .args(args)
        .current_dir(repo_root)
        .env("SPECIAL_TRACEABILITY_KERNEL", "rust-reference");
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("special should run");
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    let stderr_text = String::from_utf8_lossy(&output.stderr);
    let stderr_line_count = stderr_text.lines().count();
    let stdout_json = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok();
    let traceability_unavailable_reason = stdout_json
        .as_ref()
        .and_then(|json| json["analysis"]["traceability_unavailable_reason"].as_str())
        .map(ToString::to_string);
    let traceability_present = stdout_json
        .as_ref()
        .is_some_and(|json| !json["analysis"]["traceability"].is_null());

    json!({
        "args": args,
        "env": envs.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>(),
        "elapsed_ms": elapsed_ms,
        "success": output.status.success(),
        "exit_code": output.status.code(),
        "stdout_bytes": output.stdout.len(),
        "traceability_present": traceability_present,
        "traceability_unavailable_reason": traceability_unavailable_reason,
        "stderr_line_count": stderr_line_count,
        "stderr_tail": stderr_tail(&output.stderr, 20),
    })
}

fn run_suite(
    repo_root: &Path,
    args: &[String],
    envs: &[(&str, &str)],
    iterations: usize,
) -> Vec<serde_json::Value> {
    let mut runs = Vec::new();
    for _ in 0..iterations {
        runs.push(run_special(repo_root, args, envs));
    }
    runs
}

#[test]
#[ignore = "slow benchmark harness; run with -- --ignored --nocapture"]
fn scoped_health_benchmark_report() {
    let repo_root = std::env::var("SPECIAL_BENCH_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("cwd should resolve"));
    let scope_path = env_or_default(
        "SPECIAL_BENCH_SCOPE",
        "src/language_packs/rust/analyze/traceability.rs",
    );
    let iterations = env_or_default_usize("SPECIAL_BENCH_ITERATIONS", 3);

    let full_args = vec!["health".to_string(), "--json".to_string()];
    let scoped_args = vec![
        "health".to_string(),
        "--target".to_string(),
        scope_path.clone(),
        "--json".to_string(),
    ];

    clear_cache_bucket(&repo_root);
    let scoped_graph_discovery_suite = run_suite(&repo_root, &scoped_args, &[], iterations);

    clear_cache_bucket(&repo_root);
    let eager_scoped_oracle_suite = run_suite(
        &repo_root,
        &scoped_args,
        &[("SPECIAL_SCOPED_TRACEABILITY_MODE", "eager")],
        iterations,
    );

    clear_cache_bucket(&repo_root);
    let full_suite = run_suite(&repo_root, &full_args, &[], iterations);

    let report = json!({
        "repo_root": repo_root,
        "scope_path": scope_path,
        "iterations": iterations,
        "cache_bucket": cache_bucket_for(&repo_root),
        "scoped_graph_discovery_suite": scoped_graph_discovery_suite,
        "eager_scoped_oracle_suite": eager_scoped_oracle_suite,
        "full_suite": full_suite,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("benchmark report should serialize")
    );
}
