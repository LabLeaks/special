/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.ANALYSIS
TypeScript fixture scenarios for pack-owned module-analysis surfaces.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.ANALYSIS
use std::path::Path;

use super::support::{create_dirs, write_architecture, write_file, write_special_toml};

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_module_analysis_fixture(root: &Path) {
    create_dirs(root, &["_project", "src"]);
    write_special_toml(root);
    write_architecture(
        root,
        "# Architecture\n\n### `@module DEMO`\nDemo module.\n\n### `@module SHARED`\nShared module.\n",
    );
    write_file(
        root,
        "src/app.ts",
        "// @fileimplements DEMO\nimport { sharedValue } from \"./shared\";\nimport { readFileSync } from \"node:fs\";\n\nexport function entry(id: string, force: boolean) {\n    if (force) {\n        throw new Error(id);\n    }\n    return localHelper() + sharedValue();\n}\n\nexport const render = (name: string) => sharedValue() + name.length;\n\nfunction localHelper() {\n    return 1;\n}\n\nfunction isolatedExternal() {\n    return readFileSync(\"demo.txt\").length;\n}\n\nfunction unreachedClusterEntry() {\n    return unreachedClusterLeaf();\n}\n\nfunction unreachedClusterLeaf() {\n    return 1;\n}\n",
    );
    write_file(
        root,
        "src/shared.ts",
        "// @fileimplements SHARED\nexport function sharedValue() {\n    return 1;\n}\n",
    );
}
