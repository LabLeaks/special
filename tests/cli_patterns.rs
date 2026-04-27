/**
@module SPECIAL.TESTS.CLI_PATTERNS
`special patterns` command tests in `tests/cli_patterns.rs`.

@group SPECIAL.PATTERNS
special pattern declarations and traversal.

@spec SPECIAL.PATTERNS.DEFINITIONS
special parses `@pattern ID` as a pattern definition.

@spec SPECIAL.PATTERNS.ONE_DEFINITION
special lint reports duplicate `@pattern ID` definitions.

@spec SPECIAL.PATTERNS.DOT_NESTING
special patterns materializes dotted pattern ids as a nested tree.

@spec SPECIAL.PATTERNS.SOURCE_APPLICATIONS
when `@applies ID` is attached to a supported source item, special records the item body as a concrete application of that pattern.

@spec SPECIAL.PATTERNS.FILE_APPLICATIONS
when `@fileapplies ID` appears in a source file, special records a file-scoped application of that pattern.

@spec SPECIAL.PATTERNS.MODULE_JOIN
special derives pattern modules by joining source `@applies` applications to existing `@implements` or `@fileimplements` ownership.

@spec SPECIAL.PATTERNS.NO_MARKDOWN_APPLICATIONS
special rejects `@applies` and `@fileapplies` in markdown declarations because pattern applications must attach to source code.

@spec SPECIAL.PATTERNS.LINT
special lint reports `@applies` references whose target pattern has no matching `@pattern` definition.

@spec SPECIAL.PATTERNS.COMMAND
special patterns lists known pattern ids with definition and source-application counts.

@spec SPECIAL.PATTERNS.ID_SCOPE
special patterns PATTERN.ID shows that pattern's definition, source applications, and modules whose owned implementation contains applications.

@spec SPECIAL.PATTERNS.VERBOSE
special patterns --verbose includes pattern definition text and source application bodies.

@spec SPECIAL.PATTERNS.METRICS
special patterns --metrics reports total patterns, total pattern definitions, total source applications, and modules with applications.

@spec SPECIAL.PATTERNS.STRICTNESS
pattern definitions may declare optional `@strictness high`, `@strictness medium`, or `@strictness low` metadata; omitted strictness defaults to medium.

@spec SPECIAL.PATTERNS.METRICS.SIMILARITY
special patterns --metrics reports advisory per-pattern source similarity decimals and benchmark estimates without turning those estimates into lint failures.

@spec SPECIAL.PATTERNS.METRICS.CONFIGURED_BENCHMARKS
special patterns --metrics reads optional pattern metric benchmark decimals from special.toml.

@spec SPECIAL.PATTERNS.METRICS.MISSING_APPLICATIONS
special patterns --metrics reports advisory possible missing pattern applications from unannotated source code without making them lint failures.

@spec SPECIAL.PATTERNS.METRICS.IGNORES_TEST_CODE
special patterns --metrics does not report advisory pattern candidates from recognized test files.

@group SPECIAL.PATTERNS.METRICS.CLUSTERS
Advisory unannotated source cluster metrics.

@spec SPECIAL.PATTERNS.METRICS.CLUSTERS.INTERPRETATION
special patterns --metrics gives each advisory unannotated source cluster a first-class interpretation so tight duplicate-like clusters can suggest helper or component extraction instead of patternization.

@spec SPECIAL.PATTERNS.METRICS.CLUSTERS.THIN_DELEGATES
special patterns --metrics does not report thin delegate wrappers as advisory pattern clusters after the repeated body has already been extracted into a shared helper.

@spec SPECIAL.PATTERNS.METRICS.TARGET_SCOPE
special patterns --metrics --target PATH limits advisory pattern candidate findings to source items in the requested file or subtree.

@spec SPECIAL.PATTERNS.METRICS.SYMBOL_SCOPE
special patterns --metrics --target PATH --symbol NAME limits advisory pattern candidate findings to source items with that name.

@spec SPECIAL.PATTERNS.ARCH_MODULE_VIEW
special arch MODULE.ID includes the selected module's pattern applications without surfacing pattern definitions that belong in the pattern-centered view.
*/
// @fileimplements SPECIAL.TESTS.CLI_PATTERNS
#[path = "support/cli.rs"]
mod support;

use std::fs;

use serde_json::Value;
use support::{run_special, temp_repo_dir};

#[test]
// @verifies SPECIAL.PATTERNS.DEFINITIONS
fn patterns_show_definition_and_source_application() {
    let root = temp_repo_dir("special-cli-pattern-definitions");
    write_pattern_fixture(&root);

    let output = run_special(&root, &["patterns", "APP.SINGLE_FLIGHT_CACHE_FILL"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("APP.SINGLE_FLIGHT_CACHE_FILL"));
    assert!(stdout.contains("definition: present"));
    assert!(stdout.contains("applications: 1"));
    assert!(!stdout.contains("read-lock-reread-build-write"));
    assert!(!stdout.contains("fn load_or_parse_repo"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.SOURCE_APPLICATIONS
fn source_applications_include_attached_item_body() {
    let root = temp_repo_dir("special-cli-pattern-source-exemplars");
    write_pattern_fixture(&root);

    let output = run_special(
        &root,
        &[
            "patterns",
            "APP.SINGLE_FLIGHT_CACHE_FILL",
            "--verbose",
            "--json",
        ],
    );
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let applications = json["patterns"][0]["applications"]
        .as_array()
        .expect("applications should be an array");
    assert!(applications.iter().any(|application| {
        application["body"]
            .as_str()
            .is_some_and(|body| body.contains("fn load_or_parse_repo"))
    }));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.VERBOSE
fn patterns_verbose_includes_definition_text_and_application_bodies() {
    let root = temp_repo_dir("special-cli-pattern-verbose");
    write_pattern_fixture(&root);

    let output = run_special(
        &root,
        &["patterns", "APP.SINGLE_FLIGHT_CACHE_FILL", "--verbose"],
    );
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("read-lock-reread-build-write"));
    assert!(stdout.contains("fn load_or_parse_repo"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.MODULE_JOIN
fn source_application_materializes_owned_module() {
    let root = temp_repo_dir("special-cli-pattern-applies");
    write_pattern_fixture(&root);

    let output = run_special(
        &root,
        &["patterns", "APP.SINGLE_FLIGHT_CACHE_FILL", "--json"],
    );
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(
        json["patterns"][0]["applications"][0]["module_id"],
        "APP.CACHE"
    );
    assert_eq!(json["patterns"][0]["modules"][0]["id"], "APP.CACHE");

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.FILE_APPLICATIONS
fn file_application_materializes_owned_module_without_item_body() {
    let root = temp_repo_dir("special-cli-pattern-no-implicit-applies");
    write_pattern_fixture_with_fileapplies(&root);

    let output = run_special(
        &root,
        &["patterns", "APP.SINGLE_FLIGHT_CACHE_FILL", "--json"],
    );
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(
        json["patterns"][0]["applications"][0]["module_id"],
        "APP.CACHE"
    );
    assert_eq!(
        json["patterns"][0]["applications"][0]["body"].as_str(),
        None
    );
    assert_eq!(json["patterns"][0]["modules"][0]["id"], "APP.CACHE");

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.LINT
fn lint_reports_applies_to_unknown_pattern() {
    let root = temp_repo_dir("special-cli-pattern-lint");
    write_special_toml(&root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CACHE`\nCache module.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("cache.rs"),
        "// @fileimplements APP.CACHE\n\n// @applies APP.MISSING_PATTERN\nfn load_or_parse_repo() {}\n",
    )
    .expect("source should be written");

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("unknown pattern id `APP.MISSING_PATTERN` referenced by @applies"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.ONE_DEFINITION
fn lint_reports_duplicate_pattern_definitions() {
    let root = temp_repo_dir("special-cli-pattern-duplicate-definition");
    write_special_toml(&root);
    fs::write(
        root.join("patterns.md"),
        "### `@pattern APP.CACHE_FILL`\nFirst definition.\n\n### `@pattern APP.CACHE_FILL`\nSecond definition.\n",
    )
    .expect("patterns should be written");

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("duplicate pattern id `APP.CACHE_FILL`"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.NO_MARKDOWN_APPLICATIONS
fn lint_rejects_markdown_pattern_applications() {
    let root = temp_repo_dir("special-cli-pattern-markdown-applies");
    write_special_toml(&root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CACHE`\nCache module.\n\n@applies APP.SINGLE_FLIGHT_CACHE_FILL\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### `@pattern APP.SINGLE_FLIGHT_CACHE_FILL`\nUse a cache fill pattern.\n",
    )
    .expect("patterns should be written");
    fs::write(root.join("cache.rs"), "// @fileimplements APP.CACHE\n")
        .expect("source should be written");

    let output = run_special(&root, &["lint"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("@applies and @fileapplies must attach to source code"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.COMMAND
fn patterns_command_lists_pattern_ids_with_counts() {
    let root = temp_repo_dir("special-cli-pattern-command");
    write_pattern_fixture(&root);

    let output = run_special(&root, &["patterns"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("APP.SINGLE_FLIGHT_CACHE_FILL"));
    assert!(stdout.contains("definition: present"));
    assert!(stdout.contains("applications: 1"));
    assert!(!stdout.contains("fn load_or_parse_repo"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.DOT_NESTING
fn patterns_command_nests_dotted_pattern_ids() {
    let root = temp_repo_dir("special-cli-pattern-dot-nesting");
    write_special_toml(&root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CACHE`\nCache module.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("nested.md"),
        "### `@pattern APP`\nApplication-wide patterns.\n\n### `@pattern APP.CACHE`\nCache patterns.\n\n### `@pattern APP.CACHE.SINGLE_FLIGHT`\nSingle-flight cache fill.\n",
    )
    .expect("nested patterns should be written");
    fs::write(
        root.join("cache.rs"),
        "// @fileimplements APP.CACHE\n\n// @applies APP.CACHE.SINGLE_FLIGHT\nfn load_or_parse_repo() {}\n",
    )
    .expect("source should be written");

    let output = run_special(&root, &["patterns", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["patterns"][0]["id"], "APP");
    assert_eq!(json["patterns"][0]["children"][0]["id"], "APP.CACHE");
    assert_eq!(
        json["patterns"][0]["children"][0]["children"][0]["id"],
        "APP.CACHE.SINGLE_FLIGHT"
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.ID_SCOPE
fn patterns_id_scope_shows_one_pattern_with_definition_and_modules() {
    let root = temp_repo_dir("special-cli-pattern-id-scope");
    write_pattern_fixture(&root);
    fs::write(
        root.join("other.md"),
        "### `@pattern APP.OTHER_PATTERN`\nAnother pattern.\n",
    )
    .expect("other pattern should be written");

    let output = run_special(&root, &["patterns", "APP.SINGLE_FLIGHT_CACHE_FILL"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("APP.SINGLE_FLIGHT_CACHE_FILL"));
    assert!(stdout.contains("APP.CACHE"));
    assert!(!stdout.contains("APP.OTHER_PATTERN"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS
fn patterns_metrics_reports_definition_and_application_counts() {
    let root = temp_repo_dir("special-cli-pattern-metrics");
    write_pattern_fixture(&root);

    let output = run_special(&root, &["patterns", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["metrics"]["total_patterns"], 1);
    assert_eq!(json["metrics"]["total_definitions"], 1);
    assert_eq!(json["metrics"]["total_applications"], 1);
    assert_eq!(json["metrics"]["modules_with_applications"], 1);

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.STRICTNESS
fn pattern_definitions_parse_strictness_metadata() {
    let root = temp_repo_dir("special-cli-pattern-strictness");
    write_pattern_fixture_with_two_source_applications(&root);

    let output = run_special(&root, &["patterns", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(json["patterns"][0]["definition"]["strictness"], "high");

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.SIMILARITY
fn patterns_metrics_reports_strictness_and_similarity_estimates() {
    let root = temp_repo_dir("special-cli-pattern-similarity-metrics");
    write_pattern_fixture_with_two_source_applications(&root);

    let output = run_special(&root, &["patterns", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let pattern = &json["patterns"][0];
    assert_eq!(pattern["definition"]["strictness"], "high");
    assert_eq!(pattern["metrics"]["strictness"], "high");
    assert_eq!(pattern["metrics"]["scored_applications"], 2);
    assert_eq!(pattern["metrics"]["pair_count"], 1);
    assert!(
        pattern["metrics"]["mean_similarity"]
            .as_f64()
            .is_some_and(|value| (0.0..=1.0).contains(&value))
    );
    assert!(
        pattern["metrics"]["benchmark_estimate"]
            .as_str()
            .is_some_and(|estimate| !estimate.is_empty())
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.CONFIGURED_BENCHMARKS
fn patterns_metrics_use_special_toml_benchmarks() {
    let root = temp_repo_dir("special-cli-pattern-configured-metrics");
    write_pattern_fixture_with_two_source_applications(&root);
    fs::write(
        root.join("special.toml"),
        "version = \"1\"\nroot = \".\"\n\n[patterns.metrics]\nhigh = 0.33\nmedium = 0.22\nlow = 0.11\n",
    )
    .expect("special.toml should be written");

    let output = run_special(&root, &["patterns", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    assert_eq!(
        json["patterns"][0]["metrics"]["expected_similarity"]
            .as_f64()
            .expect("expected similarity should be numeric"),
        0.33
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.MISSING_APPLICATIONS
fn patterns_metrics_reports_possible_missing_applications() {
    let root = temp_repo_dir("special-cli-pattern-missing-applications");
    write_pattern_candidate_fixture(&root);

    let output = run_special(&root, &["patterns", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let candidates = json["metrics"]["possible_missing_applications"]
        .as_array()
        .expect("possible missing applications should be an array");
    assert!(candidates.iter().any(|candidate| {
        candidate["pattern_id"] == "APP.CACHE.FILL"
            && candidate["item_name"] == "load_third"
            && candidate["score"]
                .as_f64()
                .is_some_and(|score| (0.0..=1.0).contains(&score))
    }));

    let lint_output = run_special(&root, &["lint"]);
    assert!(lint_output.status.success());

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.IGNORES_TEST_CODE
fn patterns_metrics_ignores_rust_test_modules() {
    let root = temp_repo_dir("special-cli-pattern-ignore-rust-tests");
    write_pattern_candidate_fixture(&root);
    fs::write(
        root.join("tests.rs"),
        concat!(
            "// ",
            "@fileimplements APP.MAPPERS\n\n",
            "fn helper_alpha() -> String {\n",
            "    let raw = parse_test_source();\n",
            "    let normalized = normalize_test_source(raw);\n",
            "    emit_test_model(normalized)\n",
            "}\n\n",
            "fn helper_beta() -> String {\n",
            "    let raw = parse_test_source();\n",
            "    let normalized = normalize_test_source(raw);\n",
            "    emit_test_model(normalized)\n",
            "}\n",
        ),
    )
    .expect("rust test module should be written");

    let output = run_special(&root, &["patterns", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let clusters = json["metrics"]["possible_pattern_clusters"]
        .as_array()
        .expect("possible pattern clusters should be an array");
    assert!(!clusters.iter().any(|cluster| {
        cluster["items"].as_array().is_some_and(|items| {
            items.iter().any(|item| item["item_name"] == "helper_alpha")
                || items.iter().any(|item| item["item_name"] == "helper_beta")
        })
    }));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.CLUSTERS.INTERPRETATION
fn patterns_metrics_reports_possible_pattern_clusters() {
    let root = temp_repo_dir("special-cli-pattern-clusters");
    write_pattern_candidate_fixture(&root);

    let output = run_special(&root, &["patterns", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let clusters = json["metrics"]["possible_pattern_clusters"]
        .as_array()
        .expect("possible pattern clusters should be an array");
    let mapper_cluster = clusters
        .iter()
        .find(|cluster| {
            cluster["items"].as_array().is_some_and(|items| {
                items.iter().any(|item| item["item_name"] == "map_alpha")
                    && items.iter().any(|item| item["item_name"] == "map_beta")
            })
        })
        .expect("mapper cluster should be reported");
    assert_eq!(
        mapper_cluster["interpretation"],
        Value::from("extraction_candidate")
    );
    assert!(
        mapper_cluster["meaning"]
            .as_str()
            .is_some_and(|meaning| meaning.contains("helper or component"))
    );

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.CLUSTERS.THIN_DELEGATES
fn patterns_metrics_suppresses_thin_delegate_clusters() {
    let root = temp_repo_dir("special-cli-pattern-thin-delegates");
    write_thin_delegate_fixture(&root);

    let output = run_special(&root, &["patterns", "--metrics", "--json"]);
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let clusters = json["metrics"]["possible_pattern_clusters"]
        .as_array()
        .expect("possible pattern clusters should be an array");
    assert!(!clusters.iter().any(|cluster| {
        cluster["items"].as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| item["item_name"] == "render_project_destination")
                && items
                    .iter()
                    .any(|item| item["item_name"] == "render_global_destination")
        })
    }));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.TARGET_SCOPE
fn patterns_metrics_target_limits_candidate_targets() {
    let root = temp_repo_dir("special-cli-pattern-path-scope");
    write_pattern_candidate_fixture(&root);

    let output = run_special(
        &root,
        &["patterns", "--metrics", "--json", "--target", "cluster.rs"],
    );
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let candidates = json["metrics"]["possible_missing_applications"]
        .as_array()
        .expect("possible missing applications should be an array");
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate["item_name"] == "load_third")
    );
    let clusters = json["metrics"]["possible_pattern_clusters"]
        .as_array()
        .expect("possible pattern clusters should be an array");
    assert!(clusters.iter().any(|cluster| {
        cluster["items"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["item_name"] == "map_alpha"))
    }));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.METRICS.SYMBOL_SCOPE
fn patterns_metrics_symbol_limits_candidate_targets() {
    let root = temp_repo_dir("special-cli-pattern-symbol-scope");
    write_pattern_candidate_fixture(&root);

    let output = run_special(
        &root,
        &[
            "patterns",
            "--metrics",
            "--json",
            "--target",
            "cache.rs",
            "--symbol",
            "load_third",
        ],
    );
    assert!(output.status.success());

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("json output should be valid json");
    let candidates = json["metrics"]["possible_missing_applications"]
        .as_array()
        .expect("possible missing applications should be an array");
    assert!(candidates.iter().any(|candidate| {
        candidate["pattern_id"] == "APP.CACHE.FILL" && candidate["item_name"] == "load_third"
    }));
    let clusters = json["metrics"]["possible_pattern_clusters"]
        .as_array()
        .expect("possible pattern clusters should be an array");
    assert!(!clusters.iter().any(|cluster| {
        cluster["items"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["item_name"] == "map_alpha"))
    }));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

#[test]
// @verifies SPECIAL.PATTERNS.ARCH_MODULE_VIEW
fn arch_module_view_includes_pattern_applications_without_definitions() {
    let root = temp_repo_dir("special-cli-pattern-arch-view");
    write_pattern_fixture(&root);

    let output = run_special(&root, &["arch", "APP.CACHE"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("pattern applications: 1"));
    assert!(stdout.contains("APP.SINGLE_FLIGHT_CACHE_FILL"));
    assert!(!stdout.contains("pattern definitions"));

    fs::remove_dir_all(&root).expect("temp repo should be cleaned up");
}

fn write_pattern_fixture(root: &std::path::Path) {
    write_pattern_fixture_with_source_application(root);
}

fn write_pattern_fixture_with_source_application(root: &std::path::Path) {
    write_special_toml(root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CACHE`\nCache module.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### `@pattern APP.SINGLE_FLIGHT_CACHE_FILL`\nUse a read-lock-reread-build-write cache fill shape for expensive reusable computations.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("cache.rs"),
        "// @fileimplements APP.CACHE\n\n// @applies APP.SINGLE_FLIGHT_CACHE_FILL\nfn load_or_parse_repo() {\n    read_cache();\n    acquire_lock();\n    read_cache();\n    build_and_write();\n}\n",
    )
    .expect("source should be written");
}

fn write_pattern_fixture_with_two_source_applications(root: &std::path::Path) {
    write_special_toml(root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CACHE`\nCache module.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### `@pattern APP.SINGLE_FLIGHT_CACHE_FILL`\n@strictness high\nUse a read-lock-reread-build-write cache fill shape for expensive reusable computations.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("cache.rs"),
        "// @fileimplements APP.CACHE\n\n// @applies APP.SINGLE_FLIGHT_CACHE_FILL\nfn load_or_parse_repo() {\n    read_cache();\n    acquire_lock();\n    read_cache();\n    build_and_write();\n}\n\n// @applies APP.SINGLE_FLIGHT_CACHE_FILL\nfn load_or_parse_architecture() {\n    read_cache();\n    acquire_lock();\n    read_cache();\n    build_and_write();\n}\n",
    )
    .expect("source should be written");
}

fn write_pattern_fixture_with_fileapplies(root: &std::path::Path) {
    write_special_toml(root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CACHE`\nCache module.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### `@pattern APP.SINGLE_FLIGHT_CACHE_FILL`\nUse a read-lock-reread-build-write cache fill shape for expensive reusable computations.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("cache.rs"),
        "// @fileimplements APP.CACHE\n\n// @fileapplies APP.SINGLE_FLIGHT_CACHE_FILL\nfn load_or_parse_repo() {}\n",
    )
    .expect("source should be written");
}

fn write_pattern_candidate_fixture(root: &std::path::Path) {
    write_special_toml(root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CACHE`\nCache module.\n\n### `@module APP.MAPPERS`\nMapper module.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### `@pattern APP.CACHE.FILL`\n@strictness high\nUse a read-lock-reread-build-write cache fill shape.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("cache.rs"),
        concat!(
            "// ",
            "@fileimplements APP.CACHE\n\n",
            "// ",
            "@applies APP.CACHE.FILL\n",
            "fn load_first() -> String {\n",
            "    let key = fingerprint();\n",
            "    if let Some(value) = read_cache(key) {\n",
            "        return value;\n",
            "    }\n",
            "    acquire_lock(key);\n",
            "    if let Some(value) = read_cache(key) {\n",
            "        return value;\n",
            "    }\n",
            "    let value = build_value(key);\n",
            "    write_cache(key, value);\n",
            "    value\n",
            "}\n\n",
            "// ",
            "@applies APP.CACHE.FILL\n",
            "fn load_second() -> String {\n",
            "    let key = fingerprint();\n",
            "    if let Some(value) = read_cache(key) {\n",
            "        return value;\n",
            "    }\n",
            "    acquire_lock(key);\n",
            "    if let Some(value) = read_cache(key) {\n",
            "        return value;\n",
            "    }\n",
            "    let value = build_value(key);\n",
            "    write_cache(key, value);\n",
            "    value\n",
            "}\n\n",
            "fn load_third() -> String {\n",
            "    let key = fingerprint();\n",
            "    if let Some(value) = read_cache(key) {\n",
            "        return value;\n",
            "    }\n",
            "    acquire_lock(key);\n",
            "    if let Some(value) = read_cache(key) {\n",
            "        return value;\n",
            "    }\n",
            "    let value = build_value(key);\n",
            "    write_cache(key, value);\n",
            "    value\n",
            "}\n",
        ),
    )
    .expect("source should be written");
    fs::write(
        root.join("cluster.rs"),
        concat!(
            "// ",
            "@fileimplements APP.MAPPERS\n\n",
            "fn map_alpha() -> String {\n",
            "    let raw = parse_source();\n",
            "    let normalized = normalize_source(raw);\n",
            "    emit_model(normalized)\n",
            "}\n\n",
            "fn map_beta() -> String {\n",
            "    let raw = parse_source();\n",
            "    let normalized = normalize_source(raw);\n",
            "    emit_model(normalized)\n",
            "}\n",
        ),
    )
    .expect("cluster source should be written");
}

fn write_thin_delegate_fixture(root: &std::path::Path) {
    write_special_toml(root);
    fs::write(
        root.join("architecture.md"),
        "### `@area APP`\nApp area.\n\n### `@module APP.CLI`\nCLI module.\n",
    )
    .expect("architecture should be written");
    fs::write(
        root.join("patterns.md"),
        "### `@pattern APP.COMMAND`\nA command pattern.\n",
    )
    .expect("patterns should be written");
    fs::write(
        root.join("thin.rs"),
        concat!(
            "// ",
            "@fileimplements APP.CLI\n\n",
            "fn render_project_destination(destination: ProjectDestination) -> String {\n",
            "    let path = destination.path();\n",
            "    let reason = destination.reason();\n",
            "    display_destination_path(path, reason, \"failed to resolve project root\")\n",
            "}\n\n",
            "fn render_global_destination(destination: GlobalDestination) -> String {\n",
            "    let path = destination.path();\n",
            "    let reason = destination.reason();\n",
            "    display_destination_path(path, reason, \"failed to resolve global skills root\")\n",
            "}\n\n",
            "fn display_destination_path(path: Option<&Path>, reason: Option<&str>, fallback: &str) -> String {\n",
            "    path.map(render_path).unwrap_or_else(|| render_unavailable(reason, fallback))\n",
            "}\n",
        ),
    )
    .expect("thin delegate source should be written");
}

fn write_special_toml(root: &std::path::Path) {
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
}
