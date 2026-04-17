#![allow(dead_code)]
/**
@module SPECIAL.TESTS.SUPPORT.CLI
CLI test helpers and fixture writers in `tests/support/cli.rs`.
*/
// @fileimplements SPECIAL.TESTS.SUPPORT.CLI
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

static TEMP_REPO_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn temp_repo_dir(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let counter = TEMP_REPO_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{timestamp}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temp repo dir should be created");
    path
}

pub fn run_special(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_special"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("special command should run")
}

pub fn run_special_with_input(root: &Path, args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_special"))
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("special command should run");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let _ = child.stdin.take();

    child.wait_with_output().expect("output should be captured")
}

pub fn run_special_with_input_and_env(
    root: &Path,
    args: &[&str],
    input: &str,
    envs: &[(&str, &Path)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_special"));
    command
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }

    let mut child = command.spawn().expect("special command should run");
    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let _ = child.stdin.take();
    child.wait_with_output().expect("output should be captured")
}

pub fn run_special_with_env_removed(
    root: &Path,
    args: &[&str],
    removed_envs: &[&str],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_special"));
    command.args(args).current_dir(root);
    for key in removed_envs {
        command.env_remove(key);
    }
    command.output().expect("special command should run")
}

pub fn write_live_and_planned_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/live_and_planned/specs.txt"),
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../fixtures/cli/live_and_planned/checks.txt"),
    )
    .expect("verify fixture should be written");
}

pub fn write_planned_release_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/planned_release/specs.txt"),
    )
    .expect("planned release fixture should be written");
}

pub fn write_deprecated_release_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/deprecated_release/specs.txt"),
    )
    .expect("deprecated release fixture should be written");
}

pub fn write_file_verify_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        "/**\n@spec DEMO\nDemo root claim.\n*/\n",
    )
    .expect("file verify spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        "// @fileverifies DEMO\nfn verifies_demo_root() {}\n",
    )
    .expect("file verify fixture should be written");
}

pub fn write_file_attest_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("review.md"),
        "### @spec DEMO\nDemo root claim.\n\n### @fileattests DEMO\nartifact: docs/review.md\nowner: qa\nlast_reviewed: 2026-04-16\n\n# Review Notes\n\nThe file-level review body stays attached to the whole markdown artifact.\n",
    )
    .expect("file attest fixture should be written");
}

pub fn write_lint_error_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/lint_error/specs.txt"),
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../fixtures/cli/lint_error/checks.txt"),
    )
    .expect("verify fixture should be written");
}

pub fn write_orphan_verify_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/orphan_verify/specs.txt"),
    )
    .expect("orphan verify spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../fixtures/cli/orphan_verify/checks.txt"),
    )
    .expect("orphan verify fixture should be written");
}

pub fn write_special_toml_root_fixture(root: &Path) {
    let configured_root = root.join("workspace");
    fs::create_dir_all(&configured_root).expect("configured root should be created");
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \"workspace\"\n",
    )
    .expect("special.toml should be written");

    fs::write(
        configured_root.join("specs.rs"),
        "/**\n@spec DEMO\nConfig-root spec.\n*/\n",
    )
    .expect("config-root spec fixture should be written");
    fs::write(
        configured_root.join("checks.rs"),
        "// @verifies DEMO\nfn verifies_demo() {}\n",
    )
    .expect("config-root verify fixture should be written");

    fs::write(
        root.join("outside.rs"),
        "/**\n@spec OUTSIDE\nThis spec should stay outside the configured root.\n*/\n",
    )
    .expect("outside spec fixture should be written");
}

pub fn write_special_toml_dot_root_fixture(root: &Path) -> PathBuf {
    let nested = root.join("nested/deeper");
    fs::create_dir_all(&nested).expect("nested dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");

    fs::write(
        root.join("specs.rs"),
        "/**\n@spec DEMO\nConfig-root spec.\n*/\n",
    )
    .expect("config-root spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        "// @verifies DEMO\nfn verifies_demo() {}\n",
    )
    .expect("config-root verify fixture should be written");

    nested
}

pub fn write_skills_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
}

pub fn write_invalid_skills_root_fixture(root: &Path) {
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \"missing\"\n",
    )
    .expect("special.toml should be written");
}

pub fn write_missing_version_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "root = \".\"\n").expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/missing_version/specs.txt"),
    )
    .expect("missing version fixture should be written");
}

pub fn write_non_adjacent_planned_v1_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/non_adjacent_planned_v1/specs.txt"),
    )
    .expect("planned scope fixture should be written");
}

pub fn write_unsupported_live_fixture(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        include_str!("../fixtures/cli/unsupported_live/specs.txt"),
    )
    .expect("unsupported-live spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        include_str!("../fixtures/cli/unsupported_live/checks.txt"),
    )
    .expect("unsupported-live verify fixture should be written");
}

pub fn write_modules_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo root module.\n\n### `@module DEMO.LIVE`\nLive child module.\n\n### `@module DEMO.PLANNED @planned 0.4.0`\nPlanned child module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\n\n// @implements DEMO.LIVE\nfn implements_demo_live() {}\n",
    )
    .expect("module implementation fixture should be written");
}

pub fn write_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn demo_public() {}\n\nfn demo_private() {}\n",
    )
    .expect("main module implementation fixture should be written");
    fs::write(root.join("hidden.rs"), "fn hidden_subsystem() {}\n")
        .expect("hidden subsystem fixture should be written");
}

pub fn write_item_scoped_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "const BEFORE: usize = 1;\n\n// @implements DEMO\npub fn demo_public() {}\n\nfn hidden_helper() {}\n",
    )
    .expect("item-scoped module analysis fixture should be written");
}

pub fn write_source_local_module_analysis_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("main.rs"),
        "/**\n@module DEMO\nDemo module.\n*/\n// @fileimplements DEMO\npub fn demo_public() {}\n\nfn demo_private() {}\n",
    )
    .expect("source-local module analysis fixture should be written");
}

pub fn write_dependency_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\nuse crate::shared::util::helper;\nuse serde_json::Value;\n\npub fn demo_public() -> Value {\n    helper()\n}\n",
    )
    .expect("dependency analysis fixture should be written");
}

pub fn write_complexity_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn simple() {}\n\nfn branchy(a: bool, b: bool) {\n    if a && b {\n        for _i in 0..1 {}\n    } else if a || b {\n        while a {\n            break;\n        }\n    }\n}\n",
    )
    .expect("complexity analysis fixture should be written");
}

pub fn write_cognitive_complexity_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn simple() {}\n\nfn nested(flag: bool) {\n    if flag {\n        for _i in 0..1 {}\n    }\n}\n",
    )
    .expect("cognitive complexity analysis fixture should be written");
}

pub fn write_quality_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn open_widget(id: &str, name: String, force: bool) {\n    if force {\n        panic!(\"forced\");\n    }\n}\n",
    )
    .expect("quality analysis fixture should be written");
}

pub fn write_coupling_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO`\nDemo root.\n\n### `@module DEMO.API`\nAPI module.\n\n### `@module DEMO.SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("api.rs"),
        "// @fileimplements DEMO.API\nuse crate::shared::helper;\n\npub fn run() {\n    helper();\n}\n",
    )
    .expect("api analysis fixture should be written");
    fs::write(
        root.join("shared.rs"),
        "// @fileimplements DEMO.SHARED\npub fn helper() {}\n",
    )
    .expect("shared analysis fixture should be written");
}

pub fn write_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn entry() {\n    core_helper();\n}\n\nfn core_helper() {\n    helper_leaf();\n    helper_leaf();\n}\n\nfn helper_leaf() {}\n\npub fn outbound_heavy(id: &str, path: String, force: bool) {\n    helper_leaf();\n    if force && id.is_empty() {\n        panic!(\"forced\");\n    }\n    std::env::var(\"X\").ok();\n    std::fs::read_to_string(path).ok();\n}\n\nfn complex_hotspot(flag: bool, extra: bool) {\n    if flag {\n        for _i in 0..1 {\n            if extra || flag {\n                helper_leaf();\n            }\n        }\n    } else if extra {\n        while flag {\n            break;\n        }\n    }\n}\n\nfn isolated_external() {\n    std::process::id();\n}\n",
    )
    .expect("item signals analysis fixture should be written");
}

pub fn write_item_scoped_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO\nfn connected() {\n    shared();\n}\n\n// @implements DEMO\nfn shared() {}\n\n// @implements DEMO\nfn isolated_external() {\n    std::process::id();\n}\n",
    )
    .expect("item-scoped item signals fixture should be written");
}

pub fn write_unreached_code_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn entry() {\n    live_helper();\n}\n\nfn live_helper() {}\n\nfn unreached_cluster_entry() {\n    unreached_cluster_leaf();\n}\n\nfn unreached_cluster_leaf() {}\n",
    )
    .expect("unreached-code implementation fixture should be written");
    fs::write(root.join("hidden.rs"), "fn hidden_unreached() {}\n")
        .expect("unowned unreached-code fixture should be written");
}

pub fn write_typescript_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("src")).expect("src dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("src/app.ts"),
        "// @fileimplements DEMO\nimport { sharedValue } from \"./shared\";\nimport { readFileSync } from \"node:fs\";\n\nexport function entry() {\n    return localHelper() + sharedValue();\n}\n\nexport const render = () => sharedValue();\n\nfunction localHelper() {\n    return 1;\n}\n\nfunction isolatedExternal() {\n    return readFileSync(\"demo.txt\").length;\n}\n\nfunction unreachedClusterEntry() {\n    return unreachedClusterLeaf();\n}\n\nfunction unreachedClusterLeaf() {\n    return 1;\n}\n",
    )
    .expect("typescript implementation fixture should be written");
    fs::write(
        root.join("src/shared.ts"),
        "// @fileimplements SHARED\nexport function sharedValue() {\n    return 1;\n}\n",
    )
    .expect("typescript shared fixture should be written");
}

pub fn write_go_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("shared")).expect("shared dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport \"fmt\"\nimport \"shared\"\n\nfunc Entry() int {\n    return localHelper() + shared.SharedValue()\n}\n\nfunc localHelper() int {\n    return 1\n}\n\nfunc isolatedExternal() {\n    fmt.Println(\"demo\")\n}\n\nfunc unreachedClusterEntry() {\n    unreachedClusterLeaf()\n}\n\nfunc unreachedClusterLeaf() {}\n",
    )
    .expect("go implementation fixture should be written");
    fs::write(
        root.join("shared/shared.go"),
        "// @fileimplements SHARED\npackage shared\n\nfunc SharedValue() int {\n    return 1\n}\n",
    )
    .expect("go shared fixture should be written");
}

pub fn write_traceability_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n\n### `@spec APP.PLANNED`\n### `@planned 0.6.0`\nPlanned behavior.\n\n### `@spec APP.DEPRECATED`\n### `@deprecated 0.6.0`\nDeprecated behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn live_impl() {}\n\npub fn planned_impl() {}\n\npub fn deprecated_impl() {}\n\npub fn unverified_impl() {}\n\npub fn orphan_impl() {}\n",
    )
    .expect("implementation fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.LIVE\n#[test]\nfn verifies_live_impl() {\n    crate::live_impl();\n}\n\n// @verifies APP.PLANNED\n#[test]\nfn verifies_planned_impl() {\n    crate::planned_impl();\n}\n\n// @verifies APP.DEPRECATED\n#[test]\nfn verifies_deprecated_impl() {\n    crate::deprecated_impl();\n}\n\n#[test]\nfn exercises_unverified_impl() {\n    crate::unverified_impl();\n}\n",
    )
    .expect("traceability test fixture should be written");
}

pub fn write_traceability_file_verify_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.FILE`\nFile scoped behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn broad_impl() {}\n\npub fn second_impl() {}\n",
    )
    .expect("implementation fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @fileverifies APP.FILE\n#[test]\nfn covers_broad_impl() {\n    crate::broad_impl();\n}\n\n#[test]\nfn covers_second_impl() {\n    crate::second_impl();\n}\n",
    )
    .expect("file-verify traceability fixture should be written");
}

pub fn write_traceability_name_collision_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.COLLISION`\nCollision behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("one.rs"),
        "// @fileimplements DEMO\npub fn shared() {}\n",
    )
    .expect("first implementation fixture should be written");
    fs::write(
        root.join("two.rs"),
        "// @fileimplements DEMO\npub fn shared() {}\n",
    )
    .expect("second implementation fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.COLLISION\n#[test]\nfn verifies_shared_collision() {\n    crate::shared();\n}\n",
    )
    .expect("name collision traceability fixture should be written");
}

pub fn write_traceability_qualified_match_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.NESTED`\nNested function behavior.\n\n### `@spec APP.METHOD`\nQualified method behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub mod nested {\n    pub fn helper() {}\n\n    pub struct Worker;\n\n    impl Worker {\n        pub fn run() {}\n    }\n}\n\npub mod sibling {\n    pub fn helper() {}\n\n    pub struct Worker;\n\n    impl Worker {\n        pub fn run() {}\n    }\n}\n",
    )
    .expect("qualified match implementation fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.NESTED\n#[test]\nfn verifies_nested_helper() {\n    crate::nested::helper();\n}\n\n// @verifies APP.METHOD\n#[test]\nfn verifies_nested_worker_run() {\n    crate::nested::Worker::run();\n}\n",
    )
    .expect("qualified match traceability fixture should be written");
}

pub fn write_traceability_transitive_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.TRANSITIVE`\nTransitive traceability behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub fn helper_impl() {\n    leaf_impl();\n}\n\npub fn leaf_impl() {}\n\npub fn orphan_impl() {}\n",
    )
    .expect("implementation fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.TRANSITIVE\n#[test]\nfn verifies_transitive_leaf() {\n    crate::helper_impl();\n}\n",
    )
    .expect("transitive traceability fixture should be written");
}

pub fn write_traceability_local_binary_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("src")).expect("src dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo-cli\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[[bin]]\nname = \"app\"\npath = \"src/main.rs\"\n",
    )
    .expect("cargo fixture should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.CLI`\nLocal binary invocation behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("src/main.rs"),
        "// @fileimplements DEMO\nfn main() {\n    app_entry();\n}\n\nfn app_entry() {\n    live_impl();\n}\n\npub fn live_impl() {}\n\npub fn orphan_impl() {}\n",
    )
    .expect("implementation fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "use std::process::Command;\n\nfn run_app() {\n    let _ = Command::new(env!(\"CARGO_BIN_EXE_app\")).arg(\"status\").output();\n}\n\n// @verifies APP.CLI\n#[test]\nfn verifies_cli_entrypoint() {\n    run_app();\n}\n",
    )
    .expect("local binary traceability fixture should be written");
}

pub fn write_traceability_cross_file_module_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("render")).expect("render dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.CROSS_FILE`\nCross-file module-path traceability behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("render/mod.rs"),
        "// @fileimplements DEMO\npub mod common;\npub mod html;\n\npub fn render_entry() {\n    html::render_spec_html();\n}\n",
    )
    .expect("render module fixture should be written");
    fs::write(
        root.join("render/html.rs"),
        "// @fileimplements DEMO\npub fn render_spec_html() {\n    super::common::helper_impl();\n}\n\npub fn orphan_impl() {}\n",
    )
    .expect("render html fixture should be written");
    fs::write(
        root.join("render/common.rs"),
        "// @fileimplements DEMO\npub fn helper_impl() {\n    live_impl();\n}\n\npub fn live_impl() {}\n",
    )
    .expect("render common fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.CROSS_FILE\n#[test]\nfn verifies_cross_file_render_path() {\n    crate::render::render_entry();\n}\n",
    )
    .expect("cross-file traceability fixture should be written");
}

pub fn write_traceability_self_method_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.SELF_METHOD`\nSelf and Self dispatch traceability behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\npub struct Worker;\n\nimpl Worker {\n    pub fn run() {\n        Self::helper();\n    }\n\n    fn helper() {\n        Self::leaf();\n    }\n\n    fn leaf() {}\n\n    fn unknown() {}\n}\n",
    )
    .expect("self method implementation fixture should be written");
    fs::write(
        root.join("tests.rs"),
        "// @verifies APP.SELF_METHOD\n#[test]\nfn verifies_self_method_path() {\n    crate::Worker::run();\n}\n",
    )
    .expect("self method traceability fixture should be written");
}

pub fn write_duplicate_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("alpha.rs"),
        "// @fileimplements DEMO\npub fn first_duplicate(value: i32) -> i32 {\n    let doubled = normalize(value + value);\n    if doubled > 10 {\n        doubled - offset()\n    } else {\n        doubled + offset()\n    }\n}\n\nfn normalize(value: i32) -> i32 {\n    value\n}\n\nfn offset() -> i32 {\n    1\n}\n\npub fn distinct_alpha(input: i32) -> i32 {\n    input * 3\n}\n",
    )
    .expect("first duplicate fixture should be written");
    fs::write(
        root.join("beta.rs"),
        "// @fileimplements DEMO\npub fn second_duplicate(input: i32) -> i32 {\n    let total = normalize(input + input);\n    if total > 10 {\n        total - offset()\n    } else {\n        total + offset()\n    }\n}\n\nfn normalize(value: i32) -> i32 {\n    value\n}\n\nfn offset() -> i32 {\n    1\n}\n",
    )
    .expect("second duplicate fixture should be written");
}

pub fn write_many_duplicate_item_signals_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");

    for name in ["alpha", "beta", "gamma", "delta", "epsilon", "zeta"] {
        fs::write(
            root.join(format!("{name}.rs")),
            format!(
                "// @fileimplements DEMO\npub fn {name}_duplicate(value: i32) -> i32 {{\n    let doubled = normalize(value + value);\n    if doubled > 10 {{\n        doubled - offset()\n    }} else {{\n        doubled + offset()\n    }}\n}}\n\nfn normalize(value: i32) -> i32 {{\n    value\n}}\n\nfn offset() -> i32 {{\n    1\n}}\n"
            ),
        )
        .expect("many duplicate fixture should be written");
    }
}

pub fn write_restricted_visibility_root_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("lib.rs"),
        "// @fileimplements DEMO\npub(super) fn entry() {\n    helper();\n}\n\nfn helper() {}\n",
    )
    .expect("restricted visibility fixture should be written");
}

pub fn write_binary_entrypoint_root_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\nfn main() {\n    helper();\n}\n\nfn helper() {}\n",
    )
    .expect("binary entrypoint fixture should be written");
}

pub fn write_markdown_declarations_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::create_dir_all(root.join("docs")).expect("docs dir should be created");

    fs::write(
        root.join("docs/specs.md"),
        "### `@group DEMO`\nDemo root group.\n\n### `@spec DEMO.MARKDOWN`\nDemo root claim.\n",
    )
    .expect("markdown specs fixture should be written");
    fs::write(
        root.join("docs/architecture.md"),
        "### `@area DEMO`\nDemo architecture root.\n\n### `@area DEMO.AREA`\nDemo architecture area.\n\n### `@module DEMO.MODULE`\nDemo architecture module.\n",
    )
    .expect("markdown modules fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO.MODULE\nfn implements_demo_module() {}\n",
    )
    .expect("markdown module implementation fixture should be written");
}

pub fn write_unsupported_module_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo root module.\n\n### `@module DEMO.UNSUPPORTED`\nUnsupported live child module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\nfn implements_demo() {}\n",
    )
    .expect("module implementation fixture should be written");
}

pub fn write_area_modules_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO`\nDemo architecture area.\n\n### `@module DEMO.LIVE`\nLive child module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO.LIVE\nfn implements_demo_live() {}\n",
    )
    .expect("module implementation fixture should be written");
}

pub fn write_area_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO`\nDemo architecture area.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\nfn implements_demo() {}\n",
    )
    .expect("area implementation fixture should be written");
}

pub fn write_planned_area_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO @planned 0.4.0`\nPlanned area.\n",
    )
    .expect("architecture fixture should be written");
}

pub fn write_planned_area_invalid_suffix_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@area DEMO`\n@plannedness\nDemo area.\n",
    )
    .expect("architecture fixture should be written");
}

pub fn write_unimplemented_module_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
}

pub fn write_unknown_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO.MISSING\nfn implements_missing() {}\n",
    )
    .expect("unknown implementation fixture should be written");
}

pub fn write_implements_with_trailing_content_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo root module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @implements DEMO extra\n// @fileimplements DEMO extra\nfn implements_demo() {}\n",
    )
    .expect("trailing implementation fixture should be written");
}

pub fn write_missing_intermediate_modules_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO.CHILD.LEAF`\nLeaf module without intermediate declarations.\n",
    )
    .expect("architecture fixture should be written");
}

pub fn write_duplicate_file_scoped_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo root module.\n\n### `@module DEMO.OTHER`\nOther module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "// @fileimplements DEMO\n// @fileimplements DEMO.OTHER\nfn implements_demo() {}\n",
    )
    .expect("duplicate file-scoped implementation fixture should be written");
}

pub fn write_duplicate_item_scoped_implements_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO.ONE`\nFirst module.\n\n### `@module DEMO.TWO`\nSecond module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("main.rs"),
        "const BEFORE: usize = 1;\n\n// @implements DEMO.ONE\n// @implements DEMO.TWO\nfn implements_demo() {}\n\nfn untouched() {}\n",
    )
    .expect("duplicate item-scoped implementation fixture should be written");
}

pub fn write_source_local_modules_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("feature.rs"),
        "/**\n@module DEMO\nDemo root module.\n*/\n// @fileimplements DEMO\n\n/**\n@module DEMO.LOCAL\nLocal child module.\n*/\n// @implements DEMO.LOCAL\nfn implements_demo_local() {}\n",
    )
    .expect("source-local architecture fixture should be written");
}

pub fn write_mixed_purpose_source_local_module_fixture(root: &Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    fs::write(
        root.join("feature.rs"),
        "/**\nHuman overview for maintainers.\n@module DEMO\nRenders the demo export surface.\n@param file output path\n*/\n// @fileimplements DEMO\nfn implements_demo() {}\n",
    )
    .expect("mixed-purpose source-local architecture fixture should be written");
}

pub fn write_supported_fixture_without_config(root: &Path) {
    fs::write(
        root.join("specs.rs"),
        "/**\n@spec DEMO\nSupported demo spec.\n*/\n",
    )
    .expect("supported spec fixture should be written");
    fs::write(
        root.join("checks.rs"),
        "// @verifies DEMO\nfn verifies_demo() {}\n",
    )
    .expect("supported verify fixture should be written");
}

pub fn install_skills(root: &Path) -> std::process::Output {
    write_skills_fixture(root);
    run_special_with_input(root, &["skills", "install"], "project\n")
}

pub fn bundled_skill_ids() -> Vec<&'static str> {
    vec![
        "define-product-specs",
        "find-planned-work",
        "inspect-live-spec-state",
        "ship-product-change",
        "validate-architecture-implementation",
        "validate-product-contract",
    ]
}

pub fn bundled_skill_markdown(skill_id: &str) -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir
        .join("templates/skills")
        .join(skill_id)
        .join("SKILL.md");
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("skill template {path:?} should exist: {err}"))
}

pub fn top_level_help_command_names(output: &str) -> Vec<String> {
    top_level_help_commands(output)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

pub fn top_level_help_commands(output: &str) -> Vec<(String, String)> {
    section_items(output, "Commands:")
        .into_iter()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let mut parts = trimmed.split_whitespace();
            let name = parts.next()?;
            let summary = trimmed[name.len()..].trim_start();
            Some((name.to_string(), summary.to_string()))
        })
        .collect()
}

pub fn top_level_help_command_summaries(output: &str) -> Vec<String> {
    top_level_help_commands(output)
        .into_iter()
        .map(|(_, summary)| summary)
        .collect()
}

pub fn skills_command_shape_lines(output: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_section = false;

    for line in output.lines() {
        if line.trim() == "Command shapes:" {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.trim().is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if line.starts_with("  special ") {
            lines.push(line.trim().to_string());
        }
    }

    lines
}

pub fn rendered_spec_node_lines(output: &str) -> Vec<String> {
    let mut lines = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim_start();
        let id = trimmed.split_whitespace().next().unwrap_or_default();
        if !id.is_empty()
            && id.chars().all(|ch| {
                ch.is_ascii_uppercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-')
            })
        {
            lines.push(trimmed.to_string());
        }
    }

    lines
}

pub fn rendered_spec_node_ids(output: &str) -> Vec<String> {
    rendered_spec_node_lines(output)
        .into_iter()
        .filter_map(|line| line.split_whitespace().next().map(|id| id.to_string()))
        .collect()
}

pub fn installed_skill_ids(skills_root: &Path) -> Vec<String> {
    let mut ids = fs::read_dir(skills_root)
        .expect("skills directory should be readable")
        .map(|entry| entry.expect("skill entry should be readable").path())
        .filter(|path| path.is_dir())
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("skill directory name should be utf-8")
                .to_string()
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

pub fn listed_skill_ids(output: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut in_skill_section = false;

    for line in output.lines() {
        if line == "Available skill ids:" {
            in_skill_section = true;
            continue;
        }
        if in_skill_section && line.trim().is_empty() {
            break;
        }
        if in_skill_section {
            let id = line.split_whitespace().next().unwrap_or_default();
            if !id.is_empty() {
                ids.push(id.to_string());
            }
        }
    }

    ids.sort();
    ids
}

pub fn skills_install_destination_lines(output: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_section = false;

    for line in output.lines() {
        if line.trim() == "Install destinations:" {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.trim().is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if line.starts_with("  ") {
            lines.push(line.trim().to_string());
        }
    }

    lines
}

pub fn skills_install_destinations(output: &str) -> Vec<(String, String)> {
    skills_install_destination_lines(output)
        .into_iter()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let mut parts = trimmed.split_whitespace();
            let name = parts.next()?;
            let summary = trimmed[name.len()..].trim_start();
            Some((name.to_string(), summary.to_string()))
        })
        .collect()
}

fn section_items(output: &str, section_heading: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;

    for line in output.lines() {
        if line.trim() == section_heading {
            in_section = true;
            continue;
        }
        if in_section {
            if line.trim().is_empty() {
                break;
            }
            if line.starts_with("  ") && !line.starts_with("    ") {
                items.push(line.trim_start().to_string());
            }
        }
    }

    items
}

pub fn find_node_by_id<'a>(node: &'a Value, id: &str) -> Option<&'a Value> {
    if node["id"].as_str() == Some(id) {
        return Some(node);
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| find_node_by_id(child, id))
}
