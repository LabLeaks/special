// @fileimplements SPECIAL.SYNTAX
use tree_sitter::Node;

pub(crate) fn direct_operator_matches(node: Node<'_>, source: &[u8], operators: &[&str]) -> bool {
    (0..node.child_count()).any(|index| {
        node.child(index as u32).is_some_and(|child| {
            !child.is_named()
                && child
                    .utf8_text(source)
                    .ok()
                    .is_some_and(|text| operators.contains(&text.trim()))
        })
    })
}

pub(crate) fn is_boolean_binary_expression(node: Node<'_>, source: &[u8]) -> bool {
    node.child_by_field_name("operator")
        .and_then(|operator| operator.utf8_text(source).ok())
        .is_some_and(|operator| matches!(operator.trim(), "&&" | "||"))
        || direct_operator_matches(node, source, &["&&", "||"])
}

pub(crate) fn first_named_child_with_kind<'tree>(
    node: Node<'tree>,
    kind: &str,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}
