/**
@module SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.CALLBACKS
TSX fixture scenarios for callback, hook, effect, and context-routed traceability behavior.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.TYPESCRIPT.TEST_FIXTURES.CALLBACKS
use std::path::Path;

use super::support::{
    create_dirs, write_architecture, write_file, write_special_toml, write_specs, write_tsconfig,
};

const JSX_TSCONFIG: &str = "{\n  \"compilerOptions\": {\n    \"target\": \"ES2022\",\n    \"module\": \"ESNext\",\n    \"jsx\": \"preserve\"\n  },\n  \"include\": [\"src/**/*.ts\", \"src/**/*.tsx\"]\n}\n";

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_event_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_tsconfig(root, JSX_TSCONFIG);
    write_architecture(
        root,
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.PAGE`\nTop-level page components.\n\n### `@module APP.UI`\nShared UI components.\n\n### `@module APP.ACTIONS`\nClient-side action helpers.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.BUTTON_ACTION`\nThe page renders a button component and routes its action through the shared action helper stack.\n",
    );
    write_file(
        root,
        "src/actions.ts",
        "// @fileimplements APP.ACTIONS\nexport function handleIncrement() {\n  return updateCount();\n}\n\nexport function updateCount() {\n  return 1;\n}\n\nexport function orphanAction() {\n  return 0;\n}\n",
    );
    write_file(
        root,
        "src/Button.tsx",
        "// @fileimplements APP.UI\nexport type CounterButtonProps = {\n  onPress: () => number;\n};\n\nexport function CounterButton({ onPress }: CounterButtonProps) {\n  return onPress();\n}\n\nexport function OrphanWidget() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.tsx",
        "// @fileimplements APP.PAGE\nimport { handleIncrement } from \"./actions\";\nimport { CounterButton } from \"./Button\";\n\nexport function App() {\n  return <CounterButton onPress={handleIncrement} />;\n}\n\nexport function OrphanPage() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.test.tsx",
        "import { App } from \"./App\";\n\n// @verifies APP.BUTTON_ACTION\nexport function verifies_button_action() {\n  return <App />;\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_forwarded_callback_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_tsconfig(root, JSX_TSCONFIG);
    write_architecture(
        root,
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.PAGE`\nTop-level page components.\n\n### `@module APP.UI`\nShared UI components.\n\n### `@module APP.ACTIONS`\nClient-side action helpers.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.BUTTON_ACTION`\nThe page routes button actions through a forwarded callback prop stack.\n",
    );
    write_file(
        root,
        "src/actions.ts",
        "// @fileimplements APP.ACTIONS\nexport function handleIncrement() {\n  return updateCount();\n}\n\nexport function updateCount() {\n  return 1;\n}\n\nexport function orphanAction() {\n  return 0;\n}\n",
    );
    write_file(
        root,
        "src/Button.tsx",
        "// @fileimplements APP.UI\nexport type CounterButtonProps = {\n  onPress: () => number;\n};\n\nexport function CounterButton({ onPress }: CounterButtonProps) {\n  return onPress();\n}\n\nexport type ToolbarProps = {\n  onAction: () => number;\n};\n\nexport function Toolbar({ onAction }: ToolbarProps) {\n  return <CounterButton onPress={onAction} />;\n}\n\nexport function OrphanWidget() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.tsx",
        "// @fileimplements APP.PAGE\nimport { handleIncrement } from \"./actions\";\nimport { Toolbar } from \"./Button\";\n\nexport function App() {\n  return <Toolbar onAction={handleIncrement} />;\n}\n\nexport function OrphanPage() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.test.tsx",
        "import { App } from \"./App\";\n\n// @verifies APP.BUTTON_ACTION\nexport function verifies_button_action() {\n  return <App />;\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_hook_callback_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_tsconfig(root, JSX_TSCONFIG);
    write_architecture(
        root,
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.PAGE`\nTop-level page components.\n\n### `@module APP.UI`\nShared UI components.\n\n### `@module APP.HOOKS`\nClient-side hooks.\n\n### `@module APP.ACTIONS`\nClient-side action helpers.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.BUTTON_ACTION`\nThe page routes button actions through a hook-provided callback.\n",
    );
    write_file(
        root,
        "src/actions.ts",
        "// @fileimplements APP.ACTIONS\nexport function handleIncrement() {\n  return updateCount();\n}\n\nexport function updateCount() {\n  return 1;\n}\n\nexport function orphanAction() {\n  return 0;\n}\n",
    );
    write_file(
        root,
        "src/hooks.ts",
        "// @fileimplements APP.HOOKS\nimport { handleIncrement } from \"./actions\";\n\nexport function useCounterAction() {\n  return handleIncrement;\n}\n\nexport function orphanHook() {\n  return orphanHook;\n}\n",
    );
    write_file(
        root,
        "src/Button.tsx",
        "// @fileimplements APP.UI\nexport type CounterButtonProps = {\n  onPress: () => number;\n};\n\nexport function CounterButton({ onPress }: CounterButtonProps) {\n  return onPress();\n}\n\nexport function OrphanWidget() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.tsx",
        "// @fileimplements APP.PAGE\nimport { CounterButton } from \"./Button\";\nimport { useCounterAction } from \"./hooks\";\n\nexport function App() {\n  const onPress = useCounterAction();\n  return <CounterButton onPress={onPress} />;\n}\n\nexport function OrphanPage() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.test.tsx",
        "import { App } from \"./App\";\n\n// @verifies APP.BUTTON_ACTION\nexport function verifies_button_action() {\n  return <App />;\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_effect_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_tsconfig(root, JSX_TSCONFIG);
    write_architecture(
        root,
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.PAGE`\nTop-level page components.\n\n### `@module APP.EFFECTS`\nClient-side effect helpers.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.EFFECT_SYNC`\nThe page triggers its shared sync helper from an effect callback.\n",
    );
    write_file(
        root,
        "src/effects.ts",
        "// @fileimplements APP.EFFECTS\nexport function syncCount() {\n  return flushCount();\n}\n\nexport function flushCount() {\n  return 1;\n}\n\nexport function orphanEffect() {\n  return 0;\n}\n",
    );
    write_file(
        root,
        "src/App.tsx",
        "// @fileimplements APP.PAGE\nimport { useEffect } from \"react\";\nimport { syncCount } from \"./effects\";\n\nexport function App() {\n  useEffect(() => {\n    syncCount();\n  }, []);\n  return null;\n}\n\nexport function OrphanPage() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.test.tsx",
        "import { App } from \"./App\";\n\n// @verifies APP.EFFECT_SYNC\nexport function verifies_effect_sync() {\n  return <App />;\n}\n",
    );
}

// @applies TEST_FIXTURE.REPRESENTATIVE_PROJECT
pub fn write_typescript_context_traceability_fixture(root: &Path) {
    create_dirs(root, &["_project", "specs", "src"]);
    write_special_toml(root);
    write_tsconfig(root, JSX_TSCONFIG);
    write_architecture(
        root,
        "# Architecture\n\n### `@area APP`\nApp root.\n\n### `@module APP.PAGE`\nTop-level page components.\n\n### `@module APP.CONTEXT`\nClient-side shared context.\n\n### `@module APP.ACTIONS`\nClient-side action helpers.\n",
    );
    write_specs(
        root,
        "### `@group APP`\nApp root.\n\n### `@spec APP.CONTEXT_ACTION`\nThe page routes button actions through a shared context-provided callback.\n",
    );
    write_file(
        root,
        "src/actions.ts",
        "// @fileimplements APP.ACTIONS\nexport function handleIncrement() {\n  return updateCount();\n}\n\nexport function updateCount() {\n  return 1;\n}\n\nexport function orphanAction() {\n  return 0;\n}\n",
    );
    write_file(
        root,
        "src/context.tsx",
        "// @fileimplements APP.CONTEXT\nimport { createContext, useContext } from \"react\";\nimport { handleIncrement } from \"./actions\";\n\nexport type CounterContextValue = {\n  onPress: () => number;\n};\n\nconst CounterContext = createContext<CounterContextValue>({ onPress: handleIncrement });\n\nexport function CounterProvider({ children }: { children: unknown }) {\n  return <CounterContext.Provider value={{ onPress: handleIncrement }}>{children}</CounterContext.Provider>;\n}\n\nexport function useCounterContext() {\n  return useContext(CounterContext);\n}\n\nexport function orphanContext() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.tsx",
        "// @fileimplements APP.PAGE\nimport { CounterProvider, useCounterContext } from \"./context\";\n\nfunction CounterButton() {\n  const { onPress } = useCounterContext();\n  return onPress();\n}\n\nexport function App() {\n  return <CounterProvider><CounterButton /></CounterProvider>;\n}\n\nexport function OrphanPage() {\n  return null;\n}\n",
    );
    write_file(
        root,
        "src/App.test.tsx",
        "import { App } from \"./App\";\n\n// @verifies APP.CONTEXT_ACTION\nexport function verifies_context_action() {\n  return <App />;\n}\n",
    );
}
