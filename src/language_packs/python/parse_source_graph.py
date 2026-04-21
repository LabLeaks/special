# @module SPECIAL.LANGUAGE_PACKS.PYTHON.PARSE_SOURCE_GRAPH
# Builds shared item and call graphs for Python source files from Python's own `ast`
# module so higher-level analysis can consume normalized structure without embedding a
# separate parser in the Rust core.
# @fileimplements SPECIAL.LANGUAGE_PACKS.PYTHON.PARSE_SOURCE_GRAPH
import ast
import json
import pathlib
import sys
from dataclasses import dataclass


def is_test_path(path: pathlib.Path) -> bool:
    name = path.name
    return name.startswith("test_") or name.endswith("_test.py")


def file_module_segments(path: pathlib.Path) -> list[str]:
    parts = list(path.parts)
    if "src" in parts:
        parts = parts[parts.index("src") + 1 :]
    if not parts:
        return []
    stem = pathlib.Path(parts[-1]).stem
    parts = parts[:-1]
    if stem != "__init__":
        parts.append(stem)
    return [segment for segment in parts if segment not in ("", ".")]


def line_starts(text: str) -> list[int]:
    starts = [0]
    total = 0
    for line in text.splitlines(keepends=True):
        total += len(line.encode("utf-8"))
        starts.append(total)
    return starts


def byte_offset(starts: list[int], line: int, column: int, text: str) -> int:
    if line <= 0:
        return 0
    lines = text.splitlines(keepends=True)
    prefix = starts[line - 1]
    if line - 1 >= len(lines):
        return prefix
    return prefix + len(lines[line - 1][:column].encode("utf-8"))


def qualify_expr(node: ast.AST) -> str | None:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        base = qualify_expr(node.value)
        if base is None:
            return None
        return f"{base}.{node.attr}"
    return None


def split_qualified_name(qualified_name: str) -> tuple[str, str]:
    if "." not in qualified_name:
        return qualified_name, ""
    qualifier, name = qualified_name.rsplit(".", 1)
    return name, qualifier


@dataclass(frozen=True)
class Binding:
    kind: str
    qualified_target: str

    def instance_target(self) -> str | None:
        if self.kind == "instance":
            return self.qualified_target
        return None


@dataclass
class ItemNode:
    node: ast.FunctionDef | ast.AsyncFunctionDef
    name: str
    qualified_name: str
    module_path: list[str]
    container_path: list[str]
    kind: str
    is_test: bool
    parameters: list[str]

def item_key(module_path: list[str], container_path: list[str], name: str, line: int) -> str:
    return "::".join([*module_path, *container_path, name, str(line)])


def collect_item_nodes(
    module_path: list[str],
    body: list[ast.stmt],
    is_test: bool,
    container_path: list[str] | None = None,
) -> list[ItemNode]:
    container_path = container_path or []
    items: list[ItemNode] = []
    for node in body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            name = node.name
            kind = "method" if container_path else "function"
            items.append(
                ItemNode(
                    node=node,
                    name=name,
                    qualified_name="::".join([*module_path, *container_path, name]),
                    module_path=module_path,
                    container_path=container_path,
                    kind=kind,
                    is_test=is_test,
                    parameters=[arg.arg for arg in node.args.args],
                )
            )
        elif isinstance(node, ast.ClassDef):
            items.extend(
                collect_item_nodes(
                    module_path,
                    node.body,
                    is_test,
                    [*container_path, node.name],
                )
            )
    return items


def relative_import_base(
    path: pathlib.Path,
    module_path: list[str],
    level: int,
) -> list[str]:
    if level <= 0:
        return module_path
    current = module_path[:] if path.stem == "__init__" else module_path[:-1]
    strip_count = max(level - 1, 0)
    if strip_count:
        current = current[: max(len(current) - strip_count, 0)]
    return current


def build_import_aliases(
    path: pathlib.Path,
    module_path: list[str],
    tree: ast.Module,
) -> dict[str, str]:
    aliases: dict[str, str] = {}
    for node in tree.body:
        if isinstance(node, ast.ImportFrom):
            base_segments = relative_import_base(path, module_path, node.level)
            if node.module:
                base_segments = [*base_segments, *node.module.split(".")]
            for alias in node.names:
                local_name = alias.asname or alias.name
                aliases[local_name] = ".".join([*base_segments, alias.name])
        elif isinstance(node, ast.Import):
            for alias in node.names:
                qualified_name = alias.name
                local_name = alias.asname or qualified_name.split(".", 1)[0]
                aliases[local_name] = qualified_name
    return aliases


def local_callable_candidate_keys(item: ItemNode, name: str) -> list[str]:
    keys = []
    if item.container_path:
        keys.append("::".join([*item.module_path, *item.container_path, name]))
    keys.append("::".join([*item.module_path, name]))
    return keys


def evaluate_expr_binding(
    node: ast.AST,
    env: dict[str, Binding],
    import_aliases: dict[str, str],
    return_bindings: dict[str, Binding | None],
    current_item: ItemNode,
) -> Binding | None:
    if isinstance(node, ast.Name):
        return env.get(node.id)

    if isinstance(node, ast.Call):
        if (
            isinstance(node.func, ast.Name)
            and node.func.id == "partial"
            and node.args
        ):
            target = evaluate_expr_binding(
                node.args[0], env, import_aliases, return_bindings, current_item
            )
            if target is not None:
                return Binding("factory", target.qualified_target)
            if isinstance(node.args[0], ast.Name):
                alias_target = import_aliases.get(node.args[0].id)
                if alias_target is not None:
                    return Binding("factory", alias_target)

        callee_binding = evaluate_expr_binding(
            node.func, env, import_aliases, return_bindings, current_item
        )
        if callee_binding is not None:
            if callee_binding.kind == "factory":
                return Binding("instance", callee_binding.qualified_target)
            return callee_binding

        if isinstance(node.func, ast.Name):
            alias_target = import_aliases.get(node.func.id)
            if alias_target is not None:
                return Binding("instance", alias_target)
            for key in local_callable_candidate_keys(current_item, node.func.id):
                if key in return_bindings:
                    return return_bindings[key]

    return None


def call_record(
    node: ast.Call,
    starts: list[int],
    text: str,
    env: dict[str, Binding],
    import_aliases: dict[str, str],
    return_bindings: dict[str, Binding | None],
    current_item: ItemNode,
) -> dict | None:
    target = node.func
    if isinstance(target, ast.Name):
        imported_target = import_aliases.get(target.id)
        if imported_target is not None:
            name, qualifier = split_qualified_name(imported_target)
            syntax = "scoped_identifier" if qualifier else "identifier"
        else:
            name = target.id
            qualifier = None
            syntax = "identifier"
    elif isinstance(target, ast.Attribute):
        inferred_binding = evaluate_expr_binding(
            target.value, env, import_aliases, return_bindings, current_item
        )
        if inferred_binding is not None:
            name = target.attr
            qualifier = inferred_binding.qualified_target
            syntax = "scoped_identifier"
        else:
            name = target.attr
            qualifier = qualify_expr(target.value)
            syntax = "field"
    else:
        return None

    end_line = getattr(target, "end_lineno", target.lineno)
    end_col = getattr(target, "end_col_offset", target.col_offset)
    return {
        "name": name,
        "qualifier": qualifier,
        "syntax": syntax,
        "start_line": target.lineno,
        "end_line": end_line,
        "start_column": target.col_offset,
        "end_column": end_col,
        "start_byte": byte_offset(starts, target.lineno, target.col_offset, text),
        "end_byte": byte_offset(starts, end_line, end_col, text),
    }


def collect_calls(
    item: ItemNode,
    starts: list[int],
    text: str,
    import_aliases: dict[str, str],
    return_bindings: dict[str, Binding | None],
) -> list[dict]:
    calls: list[dict] = []
    env: dict[str, Binding] = {}

    def collect_statement_calls(node: ast.AST) -> None:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)):
            return
        if isinstance(node, ast.Call):
            record = call_record(
                node,
                starts,
                text,
                env,
                import_aliases,
                return_bindings,
                item,
            )
            if record is not None:
                calls.append(record)
        for child in ast.iter_child_nodes(node):
            collect_statement_calls(child)

    for node in item.node.body:
        collect_statement_calls(node)
        if isinstance(node, ast.Assign) and len(node.targets) == 1:
            target = node.targets[0]
            if isinstance(target, ast.Name):
                binding = evaluate_expr_binding(
                    node.value, env, import_aliases, return_bindings, item
                )
                if binding is not None:
                    env[target.id] = binding
    return calls


def item_record(
    item: ItemNode,
    path: pathlib.Path,
    starts: list[int],
    text: str,
    import_aliases: dict[str, str],
    return_bindings: dict[str, Binding | None],
) -> dict:
    node = item.node
    end_line = getattr(node, "end_lineno", node.lineno)
    end_col = getattr(node, "end_col_offset", node.col_offset)
    public = not item.name.startswith("_")
    return {
        "name": item.name,
        "qualified_name": item.qualified_name,
        "module_path": item.module_path,
        "container_path": item.container_path,
        "kind": item.kind,
        "start_line": node.lineno,
        "end_line": end_line,
        "start_column": node.col_offset,
        "end_column": end_col,
        "start_byte": byte_offset(starts, node.lineno, node.col_offset, text),
        "end_byte": byte_offset(starts, end_line, end_col, text),
        "public": public,
        "root_visible": public,
        "is_test": item.is_test,
        "calls": collect_calls(item, starts, text, import_aliases, return_bindings),
    }


def compute_return_bindings(
    items: list[ItemNode],
    import_aliases: dict[str, str],
) -> dict[str, Binding | None]:
    items_by_key = {
        item_key(item.module_path, item.container_path, item.name, item.node.lineno): item
        for item in items
    }
    prefix_index: dict[str, list[str]] = {}
    for key, item in items_by_key.items():
        prefix = "::".join([*item.module_path, *item.container_path, item.name])
        prefix_index.setdefault(prefix, []).append(key)

    cache: dict[str, Binding | None] = {}
    visiting: set[str] = set()

    def resolve_local_callable_by_name(current_item: ItemNode, name: str) -> Binding | None:
        for prefix in local_callable_candidate_keys(current_item, name):
            for key in prefix_index.get(prefix, []):
                binding = binding_for_key(key)
                if binding is not None:
                    return binding
        return None

    def binding_for_key(key: str) -> Binding | None:
        if key in cache:
            return cache[key]
        if key in visiting:
            return None
        visiting.add(key)
        item = items_by_key[key]
        env: dict[str, Binding] = {}
        resolved: Binding | None = None
        for statement in item.node.body:
            if isinstance(statement, ast.Assign) and len(statement.targets) == 1:
                target = statement.targets[0]
                if isinstance(target, ast.Name):
                    binding = evaluate_expr_binding(
                        statement.value, env, import_aliases, cache, item
                    )
                    if binding is not None:
                        env[target.id] = binding
            elif isinstance(statement, ast.Return):
                binding = evaluate_expr_binding(
                    statement.value, env, import_aliases, cache, item
                )
                if binding is not None:
                    resolved = binding
                    break
        visiting.remove(key)
        cache[key] = resolved
        return resolved

    for key in items_by_key:
        binding_for_key(key)

    return cache


def main() -> int:
    if len(sys.argv) != 2:
        return 2

    path = pathlib.Path(sys.argv[1])
    text = sys.stdin.read()
    tree = ast.parse(text)
    starts = line_starts(text)
    module_path = file_module_segments(path)
    items = collect_item_nodes(module_path, tree.body, is_test_path(path))
    import_aliases = build_import_aliases(path, module_path, tree)
    return_bindings = compute_return_bindings(items, import_aliases)
    payload = {
        "items": [
            item_record(item, path, starts, text, import_aliases, return_bindings)
            for item in items
        ]
    }
    json.dump(payload, sys.stdout)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
