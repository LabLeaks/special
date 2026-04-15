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
