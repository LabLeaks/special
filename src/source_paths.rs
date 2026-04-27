/**
@module SPECIAL.SOURCE_PATHS
Shared source-path classification helpers used by source analysis surfaces.
*/
// @fileimplements SPECIAL.SOURCE_PATHS
use std::path::{Path, PathBuf};

pub(crate) fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| extensions.contains(&ext))
}

pub(crate) fn looks_like_test_path(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if path
        .components()
        .any(|component| component.as_os_str() == "tests" || component.as_os_str() == "__tests__")
    {
        return true;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with("_test.go")
                || name == "tests.rs"
                || name.ends_with("_test.rs")
                || name.ends_with("_tests.rs")
                || name.ends_with(".test.ts")
                || name.ends_with(".test.tsx")
                || name.ends_with(".spec.ts")
                || name.ends_with(".spec.tsx")
        })
}

pub(crate) fn normalize_existing_or_joined_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    }

    let joined = root.join(path);
    std::fs::canonicalize(&joined).unwrap_or(joined)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{has_extension, looks_like_test_path, normalize_existing_or_joined_path};

    #[test]
    fn matches_known_extensions() {
        assert!(has_extension(Path::new("src/main.rs"), &["rs"]));
        assert!(has_extension(Path::new("src/app.tsx"), &["ts", "tsx"]));
        assert!(!has_extension(Path::new("src/app.tsx"), &["ts"]));
        assert!(!has_extension(Path::new("Makefile"), &["rs"]));
    }

    #[test]
    fn recognizes_language_test_paths() {
        assert!(looks_like_test_path("pkg/service_test.go"));
        assert!(looks_like_test_path("src/ui/button.test.tsx"));
        assert!(looks_like_test_path("src/ui/button.spec.ts"));
        assert!(looks_like_test_path("src/lib/tests.rs"));
        assert!(looks_like_test_path("src/lib/parser_tests.rs"));
        assert!(looks_like_test_path("tests/integration.rs"));
        assert!(looks_like_test_path("src/__tests__/button.ts"));
        assert!(!looks_like_test_path("src/testing_helpers.rs"));
        assert!(!looks_like_test_path("src/lib/parser.rs"));
    }

    #[test]
    fn normalizes_existing_absolute_paths() {
        let root = std::env::temp_dir().join(format!(
            "special-source-path-normalize-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("temp dir should be created");
        let path = root.join("app.rs");
        std::fs::write(&path, "").expect("source file should be written");

        assert_eq!(
            normalize_existing_or_joined_path(&root, &path),
            std::fs::canonicalize(&path).expect("path should canonicalize")
        );

        std::fs::remove_dir_all(&root).expect("temp dir should be cleaned up");
    }
}
