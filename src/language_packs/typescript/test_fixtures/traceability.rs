/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.TRACEABILITY
TypeScript fixture scenarios for direct, tool-backed, and reference-backed traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.TRACEABILITY
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_special_toml, write_specs,
};

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { sharedValue } from \"./shared\";\n\nexport function liveImpl() {\n    return helper() + sharedValue();\n}\n\nexport function orphanImpl() {\n    return 1;\n}\n\nfunction helper() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/shared.ts",
        "// @fileimplements SHARED\nexport function sharedValue() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/app.test.ts",
        "import { liveImpl } from \"./app\";\n\n// @verifies APP.LIVE\nexport function verifies_live_impl() {\n    return liveImpl();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_tool_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LEFT`\nLeft shared module.\n\n### `@module RIGHT`\nRight shared module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { sharedValue as liveSharedValue } from \"./left\";\n\nexport function liveImpl() {\n    return helper() + liveSharedValue();\n}\n\nexport function orphanImpl() {\n    return 1;\n}\n\nfunction helper() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/left.ts",
        "// @fileimplements LEFT\nexport function sharedValue() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/right.ts",
        "// @fileimplements RIGHT\nexport function sharedValue() {\n    return 2;\n}\n",
    );
    write_file(
        root,
        "src/app.test.ts",
        "import { liveImpl } from \"./app\";\n\n// @verifies APP.LIVE\nexport function verifies_live_impl() {\n    return liveImpl();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_reference_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module LIVE`\nLive callback module.\n\n### `@module DEAD`\nDead callback module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive callback behavior.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { liveCallback } from \"./live\";\n\nexport function runLive() {\n    return invoke(liveCallback);\n}\n\nexport function orphanImpl() {\n    return 1;\n}\n\nfunction invoke(callback: () => number) {\n    return callback();\n}\n",
    );
    write_file(
        root,
        "src/live.ts",
        "// @fileimplements LIVE\nexport function liveCallback() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/dead.ts",
        "// @fileimplements DEAD\nexport function deadCallback() {\n    return 2;\n}\n",
    );
    write_file(
        root,
        "src/app.test.ts",
        "import { runLive } from \"./app\";\n\n// @verifies APP.LIVE\nexport function verifies_run_live() {\n    return runLive();\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_cycle_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@area LIVE`\nLive cycle modules.\n\n### `@module LIVE.A`\nLive cycle entry module.\n\n### `@module LIVE.B`\nLive cycle leaf module.\n\n### `@area DEAD`\nDead cycle modules.\n\n### `@module DEAD.A`\nDead cycle entry module.\n\n### `@module DEAD.B`\nDead cycle leaf module.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.LIVE`\nLive behavior routed through a cyclic module graph should preserve only the reachable cycle.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { liveEntry } from \"./bridge-a\";\n\nexport function runLive() {\n    return liveEntry();\n}\n\nexport function orphanImpl() {\n    return 0;\n}\n",
    );
    write_file(
        root,
        "src/bridge-a.ts",
        "// @fileimplements LIVE.A\nimport { liveLeaf } from \"./bridge-b\";\n\nexport function liveEntry() {\n    return liveLeaf();\n}\n\nexport function bridgeSeed() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/bridge-b.ts",
        "// @fileimplements LIVE.B\nimport { bridgeSeed } from \"./bridge-a\";\n\nexport function liveLeaf() {\n    return bridgeSeed();\n}\n\nexport function orphanLiveLeaf() {\n    return 0;\n}\n",
    );
    write_file(
        root,
        "src/dead-a.ts",
        "// @fileimplements DEAD.A\nimport { deadLeaf } from \"./dead-b\";\n\nexport function deadEntry() {\n    return deadLeaf();\n}\n",
    );
    write_file(
        root,
        "src/dead-b.ts",
        "// @fileimplements DEAD.B\nimport { deadEntry } from \"./dead-a\";\n\nexport function deadLeaf() {\n    return deadEntry();\n}\n",
    );
    write_file(
        root,
        "src/app.test.ts",
        "import { runLive } from \"./app\";\n\n// @verifies APP.LIVE\nexport function verifies_run_live() {\n    return runLive();\n}\n",
    );
}
