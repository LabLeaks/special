/**
@module SPECIAL.LANGUAGE_PACKS.GO.TEST_FIXTURES
Pack-owned Go integration-test fixtures for module analysis and traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.TEST_FIXTURES
use std::fs;
use std::path::Path;

fn write_go_toolchain_contract(root: &Path) {
    fs::write(
        root.join(".tool-versions"),
        "go 1.23.12\ngo:golang.org/x/tools/gopls 0.21.1\n",
    )
    .expect(".tool-versions should be written");
}

pub fn write_go_module_analysis_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("shared")).expect("shared dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
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

pub fn write_go_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("shared")).expect("shared dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport \"shared\"\n\nfunc LiveImpl() int {\n    return helper() + shared.SharedValue()\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc helper() int {\n    return 1\n}\n",
    )
    .expect("go implementation fixture should be written");
    fs::write(
        root.join("shared/shared.go"),
        "// @fileimplements SHARED\npackage shared\n\nfunc SharedValue() int {\n    return 1\n}\n",
    )
    .expect("go shared fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go traceability test fixture should be written");
}

pub fn write_go_tool_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("left")).expect("left dir should be created");
    fs::create_dir_all(root.join("right")).expect("right dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LEFT`\nLeft shared module.\n\n### `@module RIGHT`\nRight shared module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport l \"example.com/demo/left\"\n\nfunc LiveImpl() int {\n    return helper() + l.SharedValue()\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc helper() int {\n    return 1\n}\n",
    )
    .expect("go implementation fixture should be written");
    fs::write(
        root.join("left/shared.go"),
        "// @fileimplements LEFT\npackage left\n\nfunc SharedValue() int {\n    return 1\n}\n",
    )
    .expect("go left fixture should be written");
    fs::write(
        root.join("right/shared.go"),
        "// @fileimplements RIGHT\npackage right\n\nfunc SharedValue() int {\n    return 2\n}\n",
    )
    .expect("go right fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go tool traceability test fixture should be written");
}

pub fn write_go_reference_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive callback module.\n\n### `@module DEAD`\nDead callback module.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive callback behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\nfunc LiveImpl() int {\n    return invoke(live.LiveCallback)\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(callback func() int) int {\n    return callback()\n}\n",
    )
    .expect("go reference implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\nfunc LiveCallback() int {\n    return 1\n}\n",
    )
    .expect("go live callback fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\nfunc DeadCallback() int {\n    return 2\n}\n",
    )
    .expect("go dead callback fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go reference traceability test fixture should be written");
}

pub fn write_go_interface_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive interface implementation.\n\n### `@module DEAD`\nDead interface implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive interface behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\ntype Runner interface {\n    Run() int\n}\n\nfunc LiveImpl() int {\n    return invoke(live.NewRunner())\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(r Runner) int {\n    return r.Run()\n}\n",
    )
    .expect("go interface implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\ntype LiveRunner struct{}\n\nfunc NewRunner() LiveRunner {\n    return LiveRunner{}\n}\n\nfunc (LiveRunner) Run() int {\n    return 1\n}\n",
    )
    .expect("go live interface fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\ntype DeadRunner struct{}\n\nfunc NewRunner() DeadRunner {\n    return DeadRunner{}\n}\n\nfunc (DeadRunner) Run() int {\n    return 2\n}\n",
    )
    .expect("go dead interface fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go interface traceability test fixture should be written");
}

pub fn write_go_embedding_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("inner")).expect("inner dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive embedding wrapper.\n\n### `@module INNER`\nEmbedded runner implementation.\n\n### `@module DEAD`\nDead unrelated implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive embedding behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\ntype Runner interface {\n    Run() int\n}\n\nfunc LiveImpl() int {\n    return invoke(live.NewRunner())\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(r Runner) int {\n    return r.Run()\n}\n",
    )
    .expect("go embedding implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\nimport \"example.com/demo/inner\"\n\ntype Wrapper struct {\n    inner.Runner\n}\n\nfunc NewRunner() Wrapper {\n    return Wrapper{Runner: inner.Runner{}}\n}\n",
    )
    .expect("go live embedding fixture should be written");
    fs::write(
        root.join("inner/inner.go"),
        "// @fileimplements INNER\npackage inner\n\ntype Runner struct{}\n\nfunc (Runner) Run() int {\n    return 1\n}\n",
    )
    .expect("go inner embedding fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\ntype DeadRunner struct{}\n\nfunc (DeadRunner) Run() int {\n    return 2\n}\n",
    )
    .expect("go dead embedding fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go embedding traceability test fixture should be written");
}

pub fn write_go_method_value_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive method-value implementation.\n\n### `@module DEAD`\nDead unrelated implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive method-value behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\nfunc LiveImpl() int {\n    runner := live.NewRunner()\n    return invoke(runner.Run)\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(callback func() int) int {\n    return callback()\n}\n",
    )
    .expect("go method-value implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\ntype LiveRunner struct{}\n\nfunc NewRunner() LiveRunner {\n    return LiveRunner{}\n}\n\nfunc (LiveRunner) Run() int {\n    return 1\n}\n",
    )
    .expect("go live method-value fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\ntype DeadRunner struct{}\n\nfunc NewRunner() DeadRunner {\n    return DeadRunner{}\n}\n\nfunc (DeadRunner) Run() int {\n    return 2\n}\n",
    )
    .expect("go dead method-value fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go method-value traceability test fixture should be written");
}

pub fn write_go_embedding_method_value_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("inner")).expect("inner dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive embedding wrapper.\n\n### `@module INNER`\nEmbedded runner implementation.\n\n### `@module DEAD`\nDead unrelated implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive embedding method-value behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\nfunc LiveImpl() int {\n    runner := live.NewRunner()\n    return invoke(runner.Run)\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(callback func() int) int {\n    return callback()\n}\n",
    )
    .expect("go embedding method-value implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\nimport \"example.com/demo/inner\"\n\ntype Wrapper struct {\n    inner.Runner\n}\n\nfunc NewRunner() Wrapper {\n    return Wrapper{Runner: inner.Runner{}}\n}\n",
    )
    .expect("go live embedding method-value fixture should be written");
    fs::write(
        root.join("inner/inner.go"),
        "// @fileimplements INNER\npackage inner\n\ntype Runner struct{}\n\nfunc (Runner) Run() int {\n    return 1\n}\n",
    )
    .expect("go inner embedding method-value fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\ntype DeadRunner struct{}\n\nfunc (DeadRunner) Run() int {\n    return 2\n}\n",
    )
    .expect("go dead embedding method-value fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go embedding method-value traceability test fixture should be written");
}

pub fn write_go_method_expression_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive method-expression implementation.\n\n### `@module DEAD`\nDead unrelated implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive method-expression behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\nfunc LiveImpl() int {\n    runner := live.NewRunner()\n    return invoke(func() int { return live.LiveRunner.Run(runner) })\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(callback func() int) int {\n    return callback()\n}\n",
    )
    .expect("go method-expression implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\ntype LiveRunner struct{}\n\nfunc NewRunner() LiveRunner {\n    return LiveRunner{}\n}\n\nfunc (LiveRunner) Run() int {\n    return 1\n}\n",
    )
    .expect("go live method-expression fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\ntype DeadRunner struct{}\n\nfunc NewRunner() DeadRunner {\n    return DeadRunner{}\n}\n\nfunc (DeadRunner) Run() int {\n    return 2\n}\n",
    )
    .expect("go dead method-expression fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go method-expression traceability test fixture should be written");
}

pub fn write_go_receiver_collision_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive receiver-collision implementation.\n\n### `@module DEAD`\nDead receiver-collision implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive receiver-collision behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\nfunc LiveImpl() int {\n    runner := live.NewRunner()\n    return invoke(func() int { return live.Runner.Run(runner) })\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(callback func() int) int {\n    return callback()\n}\n",
    )
    .expect("go receiver-collision implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\ntype Runner struct{}\n\nfunc NewRunner() Runner {\n    return Runner{}\n}\n\nfunc (Runner) Run() int {\n    return 1\n}\n",
    )
    .expect("go live receiver-collision fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\ntype Runner struct{}\n\nfunc NewRunner() Runner {\n    return Runner{}\n}\n\nfunc (Runner) Run() int {\n    return 2\n}\n",
    )
    .expect("go dead receiver-collision fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go receiver-collision traceability test fixture should be written");
}

pub fn write_go_embedded_interface_traceability_fixture(root: &Path) {
    fs::create_dir_all(root.join("_project")).expect("architecture dir should be created");
    fs::create_dir_all(root.join("specs")).expect("spec dir should be created");
    fs::create_dir_all(root.join("app")).expect("app dir should be created");
    fs::create_dir_all(root.join("live")).expect("live dir should be created");
    fs::create_dir_all(root.join("inner")).expect("inner dir should be created");
    fs::create_dir_all(root.join("dead")).expect("dead dir should be created");
    fs::write(root.join("special.toml"), "version = \"1\"\nroot = \".\"\n")
        .expect("special.toml should be written");
    write_go_toolchain_contract(root);
    fs::write(root.join("go.mod"), "module example.com/demo\n\ngo 1.23\n")
        .expect("go.mod should be written");
    fs::write(
        root.join("_project/ARCHITECTURE.md"),
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive embedded interface wrapper.\n\n### `@module INNER`\nEmbedded runner implementation.\n\n### `@module DEAD`\nDead unrelated implementation.\n",
    )
    .expect("architecture fixture should be written");
    fs::write(
        root.join("specs/root.md"),
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive embedded interface behavior.\n",
    )
    .expect("spec fixture should be written");
    fs::write(
        root.join("app/main.go"),
        "// @fileimplements DEMO\npackage app\n\nimport live \"example.com/demo/live\"\n\ntype Runner interface {\n    Run() int\n}\n\nfunc LiveImpl() int {\n    var runner Runner = live.NewRunner()\n    return invoke(runner)\n}\n\nfunc OrphanImpl() int {\n    return 1\n}\n\nfunc invoke(r Runner) int {\n    return r.Run()\n}\n",
    )
    .expect("go embedded-interface implementation fixture should be written");
    fs::write(
        root.join("live/live.go"),
        "// @fileimplements LIVE\npackage live\n\nimport \"example.com/demo/inner\"\n\ntype Wrapper struct {\n    inner.Runner\n}\n\nfunc NewRunner() Wrapper {\n    return Wrapper{Runner: inner.Runner{}}\n}\n",
    )
    .expect("go live embedded-interface fixture should be written");
    fs::write(
        root.join("inner/inner.go"),
        "// @fileimplements INNER\npackage inner\n\ntype Runner struct{}\n\nfunc (Runner) Run() int {\n    return 1\n}\n",
    )
    .expect("go inner embedded-interface fixture should be written");
    fs::write(
        root.join("dead/dead.go"),
        "// @fileimplements DEAD\npackage dead\n\ntype Runner struct{}\n\nfunc (Runner) Run() int {\n    return 2\n}\n",
    )
    .expect("go dead embedded-interface fixture should be written");
    fs::write(
        root.join("app/main_test.go"),
        "package app\n\n// @verifies APP.LIVE\nfunc TestLiveImpl() {\n    LiveImpl()\n}\n",
    )
    .expect("go embedded-interface traceability test fixture should be written");
}
