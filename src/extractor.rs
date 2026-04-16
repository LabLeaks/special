/**
@spec SPECIAL.PARSE.LINE_COMMENTS
special parses annotation blocks from contiguous line comments.

@spec SPECIAL.PARSE.BLOCK_COMMENTS
special parses annotation blocks from block comments.

@spec SPECIAL.PARSE.GO_LINE_COMMENTS
special parses annotation blocks from Go line comments in `.go` files.

@spec SPECIAL.PARSE.TYPESCRIPT_LINE_COMMENTS
special parses annotation blocks from TypeScript line comments in `.ts` files.

@spec SPECIAL.PARSE.TYPESCRIPT_BLOCK_COMMENTS
special parses annotation blocks from TypeScript block comments in `.ts` files.

@spec SPECIAL.PARSE.MIXED_PURPOSE_COMMENTS
special parses reserved annotations from ordinary mixed-purpose comment blocks without requiring the whole block to be special-only.

@spec SPECIAL.PARSE.LINE_START_RESERVED_TAGS
special interprets reserved annotations only when they begin the normalized comment line after comment markers and leading whitespace are stripped.

@spec SPECIAL.PARSE.FOREIGN_TAGS_NOT_ERRORS
special does not report foreign line-start `@...` and `\\...` tags as lint errors inside mixed-purpose comment blocks.

@spec SPECIAL.PARSE.SHELL_COMMENTS
special parses annotation blocks from shell-style line comments in .sh files.

@spec SPECIAL.PARSE.PYTHON_LINE_COMMENTS
special parses annotation blocks from Python `#` comments in `.py` files instead of docstring ownership.

@spec SPECIAL.PARSE.VERIFIES.ATTACHES_TO_NEXT_ITEM
special attaches a @verifies annotation block to the next supported item in comment-based languages.

@module SPECIAL.EXTRACTOR
Collects contiguous supported source comment blocks, normalizes comment syntax, and captures the next owned code item for attachment. This module does not interpret `special` tag semantics or build spec or module trees.
*/
// @fileimplements SPECIAL.EXTRACTOR
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::discovery::{DiscoveryConfig, discover_annotation_files};
use crate::model::CommentBlock;

mod blocks;
mod owned_items;

pub fn collect_comment_blocks(
    root: &Path,
    ignore_patterns: &[String],
) -> Result<Vec<CommentBlock>> {
    let mut blocks = Vec::new();
    for path in discover_annotation_files(DiscoveryConfig {
        root,
        ignore_patterns,
    })?
    .source_files
    {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("reading source file {}", path.display()))?;
        blocks.extend(extract_blocks_from_text(path, &content));
    }

    Ok(blocks)
}

fn extract_blocks_from_text(path: PathBuf, content: &str) -> Vec<CommentBlock> {
    blocks::extract_blocks_from_text(path, content)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::extract_blocks_from_text;

    #[test]
    // @verifies SPECIAL.PARSE.LINE_COMMENTS
    fn extracts_line_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.rs"),
            "// @spec AUTH\n// Auth works.\nfn main() {}\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[0].text, "@spec AUTH");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "fn main() {}"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.GO_LINE_COMMENTS
    fn extracts_go_line_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.go"),
            "// @spec AUTH.LOGIN\n// Auth works.\nfunc main() {}\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[0].text, "@spec AUTH.LOGIN");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "func main() {}"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.TYPESCRIPT_LINE_COMMENTS
    fn extracts_typescript_line_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.ts"),
            "// @spec AUTH.LOGIN\n// Auth works.\nexport const ok = true;\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[0].text, "@spec AUTH.LOGIN");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "export const ok = true;"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.BLOCK_COMMENTS
    fn extracts_generic_block_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.rs"),
            "/**\n * @spec AUTH.LOGIN\n * Auth works.\n */\nfn main() {}\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[1].text, "@spec AUTH.LOGIN");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "fn main() {}"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.LINE_START_RESERVED_TAGS
    fn ignores_mid_line_tag_text_when_collecting_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.rs"),
            "/// Human prose mentioning @spec EXPORT.CSV inline.\nfn export_csv() {}\n",
        );

        assert!(blocks.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.TYPESCRIPT_BLOCK_COMMENTS
    fn extracts_block_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.ts"),
            "/**\n * @verifies AUTH.LOGIN\n */\nexport {};\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[1].text, "@verifies AUTH.LOGIN");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "export {};"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.MIXED_PURPOSE_COMMENTS
    fn extracts_special_tags_from_ordinary_doc_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.rs"),
            "/// Human overview for maintainers.\n/// @spec EXPORT.CSV\n/// CSV exports include a header row.\nfn export_csv() {}\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[0].text, "Human overview for maintainers.");
        assert_eq!(blocks[0].lines[1].text, "@spec EXPORT.CSV");
        assert_eq!(blocks[0].lines[2].text, "CSV exports include a header row.");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "fn export_csv() {}"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.FOREIGN_TAGS_NOT_ERRORS
    fn ignores_comment_blocks_with_only_foreign_tags() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.ts"),
            "/**\n * @param file output path\n * @returns CSV text\n */\nexport function render() {}\n",
        );

        assert!(blocks.is_empty());
    }

    #[test]
    // @verifies SPECIAL.PARSE.SHELL_COMMENTS
    fn extracts_shell_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("scripts/verify.sh"),
            "#!/usr/bin/env bash\n# @fileverifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED\nset -euo pipefail\n\nexec mise exec -- cargo clippy --all-targets --all-features -- -D warnings\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].lines[1].text,
            "@fileverifies SPECIAL.QUALITY.RUST.CLIPPY.SPEC_OWNED"
        );
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "set -euo pipefail\n\nexec mise exec -- cargo clippy --all-targets --all-features -- -D warnings"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.PYTHON_LINE_COMMENTS
    fn extracts_python_line_comment_blocks() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.py"),
            "# @verifies AUTH.LOGIN\n\ndef test_auth_login():\n    assert True\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].lines[0].text, "@verifies AUTH.LOGIN");
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "def test_auth_login():\n    assert True"
        );
    }

    #[test]
    // @verifies SPECIAL.PARSE.VERIFIES.ATTACHES_TO_NEXT_ITEM
    fn attaches_verify_block_to_next_item() {
        let blocks = extract_blocks_from_text(
            PathBuf::from("src/example.rs"),
            "// @verifies AUTH.LOGIN\n#[test]\nfn verifies_auth_login() {\n    assert!(true);\n}\n",
        );

        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0]
                .owned_item
                .as_ref()
                .expect("owned item should be present")
                .body,
            "#[test]\nfn verifies_auth_login() {\n    assert!(true);\n}"
        );
    }
}
