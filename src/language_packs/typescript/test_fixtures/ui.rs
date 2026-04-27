/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.UI
TSX fixture scenarios for rendered component-stack and Next-style client-component traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.UI
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_special_toml, write_specs, write_tsconfig,
};

const JSX_TSCONFIG: &str = "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"CommonJS\",\n    \"jsx\": \"preserve\"\n  },\n  \"include\": [\"src/**/*.ts\", \"src/**/*.tsx\"]\n}\n";
const NEXT_TSCONFIG: &str = "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"ESNext\",\n    \"jsx\": \"preserve\"\n  },\n  \"include\": [\"app/**/*.tsx\", \"components/**/*.tsx\"]\n}\n";

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_react_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_tsconfig(root, JSX_TSCONFIG);
    write_architecture(
        root,
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.PAGE`\nTop-level page components.\n\n### `@module APP.SHARED`\nShared UI components.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.PAGE_RENDER`\nThe page renders through its shared component stack.\n",
    );
    write_file(
        root,
        "src/page.tsx",
        "// @fileimplements APP.PAGE\nimport { Shell } from \"./shared\";\n\nexport function HomePage() {\n    return <Shell />;\n}\n\nexport function OrphanPage() {\n    return null;\n}\n",
    );
    write_file(
        root,
        "src/shared.tsx",
        "// @fileimplements APP.SHARED\nexport function Shell() {\n    return <PrimaryButton />;\n}\n\nexport function PrimaryButton() {\n    return null;\n}\n\nexport function OrphanWidget() {\n    return null;\n}\n",
    );
    write_file(
        root,
        "src/page.test.tsx",
        "import { HomePage } from \"./page\";\n\n// @verifies APP.PAGE_RENDER\nexport function verifies_page_render() {\n    return <HomePage />;\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_next_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "app", "components"]);
    write_special_toml(root);
    write_tsconfig(root, NEXT_TSCONFIG);
    write_architecture(
        root,
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.PAGE`\nTop-level route components.\n\n### `@module APP.CLIENT`\nClient-side interactive components.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.PAGE_RENDER`\nThe page renders through its client component stack.\n",
    );
    write_file(
        root,
        "app/page.tsx",
        "// @fileimplements APP.PAGE\nimport { CounterPanel } from \"../components/counter-panel\";\n\nexport default function Page() {\n    return <CounterPanel />;\n}\n\nexport function OrphanPage() {\n    return null;\n}\n",
    );
    write_file(
        root,
        "components/counter-panel.tsx",
        "// @fileimplements APP.CLIENT\n\"use client\";\n\nimport { useState } from \"react\";\n\nexport function CounterPanel() {\n    const [count] = useState(0);\n    return <CounterButton count={count} />;\n}\n\ntype CounterButtonProps = {\n    count: number;\n};\n\nexport function CounterButton({ count }: CounterButtonProps) {\n    return count;\n}\n\nexport function OrphanWidget() {\n    return null;\n}\n",
    );
    write_file(
        root,
        "app/page.test.tsx",
        "import Page from \"./page\";\n\n// @verifies APP.PAGE_RENDER\nexport function verifies_page_render() {\n    return <Page />;\n}\n",
    );
}
