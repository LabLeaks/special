/**
@module SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.DEPENDENCIES
Extracts Go import-based dependency and coupling evidence for the built-in Go pack analyzer.
*/
// @fileimplements SPECIAL.LANGUAGE_PACKS.GO.ANALYZE.DEPENDENCIES
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser};

use crate::modules::analyze::{ModuleCouplingInput, build_dependency_summary};

use super::toolchain::go_list_packages;

#[derive(Default)]
pub(super) struct GoDependencyResolver {
    package_files: BTreeMap<String, BTreeSet<PathBuf>>,
    tool_backed: bool,
}

impl GoDependencyResolver {
    pub(super) fn from_project(root: &Path) -> Self {
        let Some(packages) = go_list_packages(root) else {
            return Self::default();
        };
        let mut package_files = BTreeMap::<String, BTreeSet<PathBuf>>::new();
        for package in packages {
            for file_name in package.go_files {
                let full_path = package.dir.join(file_name);
                let canonical_full_path =
                    fs::canonicalize(&full_path).unwrap_or_else(|_| full_path.clone());
                let indexed_path = full_path
                    .strip_prefix(root)
                    .map(|relative| root.join(relative))
                    .unwrap_or(canonical_full_path);
                package_files
                    .entry(package.import_path.clone())
                    .or_default()
                    .insert(indexed_path);
            }
        }
        Self {
            package_files,
            tool_backed: true,
        }
    }

    pub(super) fn is_tool_backed(&self) -> bool {
        self.tool_backed
    }

    fn resolve(&self, target: &str) -> Option<&BTreeSet<PathBuf>> {
        self.package_files.get(target)
    }
}

#[derive(Default)]
pub(super) struct GoDependencySummary {
    targets: BTreeMap<String, usize>,
    internal_files: BTreeSet<PathBuf>,
    external_targets: BTreeSet<String>,
}

impl GoDependencySummary {
    pub(super) fn observe(&mut self, root: &Path, text: &str, resolver: &GoDependencyResolver) {
        let mut parser = Parser::new();
        if parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .is_err()
        {
            return;
        }
        let Some(tree) = parser.parse(text, None) else {
            return;
        };
        let mut imports = Vec::new();
        collect_import_sources(tree.root_node(), text.as_bytes(), &mut imports);
        for target in imports {
            *self.targets.entry(target.clone()).or_default() += 1;
            let internal_files = resolver
                .resolve(&target)
                .cloned()
                .unwrap_or_else(|| resolve_internal_imports(root, &target));
            if internal_files.is_empty() {
                self.external_targets.insert(target);
            } else {
                self.internal_files.extend(internal_files);
            }
        }
    }

    pub(super) fn summary(&self) -> crate::model::ModuleDependencySummary {
        build_dependency_summary(&self.targets)
    }

    pub(super) fn coupling_input(&self) -> ModuleCouplingInput {
        ModuleCouplingInput {
            internal_files: self.internal_files.clone(),
            external_targets: self.external_targets.clone(),
        }
    }
}

fn collect_import_sources(node: Node<'_>, source: &[u8], imports: &mut Vec<String>) {
    if node.kind() == "import_spec"
        && let Some(import_source) = node.child_by_field_name("path")
        && let Ok(text) = import_source.utf8_text(source)
    {
        imports.push(text.trim_matches('"').trim_matches('`').to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_sources(child, source, imports);
    }
}

pub(super) fn resolve_internal_imports(root: &Path, target: &str) -> BTreeSet<PathBuf> {
    let segments = target
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return BTreeSet::new();
    }

    let mut matches = BTreeSet::new();
    for start in 0..segments.len() {
        let suffix = PathBuf::from_iter(segments[start..].iter().copied());
        matches.extend(find_go_owned_paths(root, &suffix));
        if !matches.is_empty() {
            break;
        }
    }
    matches
}

fn find_go_owned_paths(root: &Path, suffix: &Path) -> BTreeSet<PathBuf> {
    let mut matches = BTreeSet::new();
    let direct_file = root.join(suffix).with_extension("go");
    if direct_file.exists() {
        matches.insert(direct_file);
    }

    let directory = root.join(suffix);
    if directory.is_dir() {
        matches.extend(read_go_files(&directory));
    }
    matches
}

fn read_go_files(directory: &Path) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("go") {
            files.insert(path);
        }
    }
    files
}

pub(super) fn collect_go_import_aliases(text: &str) -> BTreeMap<String, String> {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .is_err()
    {
        return BTreeMap::new();
    }
    let Some(tree) = parser.parse(text, None) else {
        return BTreeMap::new();
    };
    let mut aliases = BTreeMap::new();
    collect_import_aliases(tree.root_node(), text.as_bytes(), &mut aliases);
    aliases
}

fn collect_import_aliases(node: Node<'_>, source: &[u8], aliases: &mut BTreeMap<String, String>) {
    if node.kind() == "import_spec"
        && let Some(import_source) = node.child_by_field_name("path")
        && let Ok(text) = import_source.utf8_text(source)
    {
        let import_path = text.trim_matches('"').trim_matches('`').to_string();
        let alias = explicit_import_alias(node, import_source, source).unwrap_or_else(|| {
            import_path
                .rsplit('/')
                .next()
                .unwrap_or(import_path.as_str())
                .to_string()
        });
        if alias != "_" && alias != "." {
            aliases.insert(alias, import_path);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_import_aliases(child, source, aliases);
    }
}

fn explicit_import_alias(
    import_spec: Node<'_>,
    import_source: Node<'_>,
    source: &[u8],
) -> Option<String> {
    import_spec
        .child_by_field_name("name")
        .and_then(|name| name.utf8_text(source).ok())
        .map(ToString::to_string)
        .or_else(|| {
            let mut cursor = import_spec.walk();
            import_spec
                .named_children(&mut cursor)
                .find(|child| *child != import_source)
                .and_then(|child| child.utf8_text(source).ok())
                .map(ToString::to_string)
        })
}

#[cfg(test)]
mod tests {
    use super::collect_go_import_aliases;

    #[test]
    fn collect_go_import_aliases_tracks_explicit_aliases() {
        let aliases =
            collect_go_import_aliases("package app\n\nimport l \"example.com/demo/left\"\n");
        assert_eq!(
            aliases.get("l").map(String::as_str),
            Some("example.com/demo/left")
        );
    }
}
