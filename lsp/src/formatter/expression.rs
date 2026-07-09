/// Tree-sitter expression formatting for HubGS constraint decorators and instance values.
use tree_sitter::Node;

/// Returns true if `kind` represents an expression node that should be formatted recursively.
pub fn is_expression_kind(kind: &str) -> bool {
    matches!(
        kind,
        "binary_expression"
            | "unary_expression"
            | "member_expression"
            | "call_expression"
            | "arrow_function"
            | "identifier"
            | "number"
            | "string"
            | "template_string"
            | "array"
            | "boolean"
            | "parenthesized_expression"
    )
}

/// Format a tree-sitter expression node into normalized HubGS source text.
pub fn format_expression(node: Node, contents: &str) -> String {
    match node.kind() {
        "identifier" | "number" | "boolean" | "string" => contents[node.byte_range()].to_string(),
        "template_string" => {
            let mut result = String::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if is_expression_kind(child.kind()) {
                    result.push_str(&format_expression(child, contents));
                } else {
                    result.push_str(&contents[child.byte_range()]);
                }
            }
            result
        }
        "array" => {
            let mut exprs = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if is_expression_kind(child.kind()) {
                    exprs.push(format_expression(child, contents));
                }
            }
            format!("[{}]", exprs.join(", "))
        }
        "parenthesized_expression" => {
            if let Some(expr_node) = node.child(1) {
                format!("({})", format_expression(expr_node, contents))
            } else {
                contents[node.byte_range()].to_string()
            }
        }
        "unary_expression" => {
            let operator = node
                .child_by_field_name("operator")
                .map(|n| contents[n.byte_range()].to_string())
                .unwrap_or_default();
            let argument = node
                .child_by_field_name("argument")
                .map(|n| format_expression(n, contents))
                .unwrap_or_default();
            format!("{}{}", operator, argument)
        }
        "binary_expression" => {
            let left = node
                .child_by_field_name("left")
                .map(|n| format_expression(n, contents))
                .unwrap_or_default();
            let operator = node
                .child_by_field_name("operator")
                .map(|n| contents[n.byte_range()].to_string())
                .unwrap_or_default();
            let right = node
                .child_by_field_name("right")
                .map(|n| format_expression(n, contents))
                .unwrap_or_default();
            format!("{} {} {}", left, operator, right)
        }
        "member_expression" => {
            let object = node
                .child_by_field_name("object")
                .map(|n| format_expression(n, contents))
                .unwrap_or_default();
            let property = node
                .child_by_field_name("property")
                .map(|n| contents[n.byte_range()].to_string())
                .unwrap_or_default();
            format!("{}.{}", object, property)
        }
        "call_expression" => {
            let function = node
                .child_by_field_name("function")
                .map(|n| format_expression(n, contents))
                .unwrap_or_default();
            let mut args = Vec::new();
            let mut cursor = node.walk();
            let func_node = node.child_by_field_name("function");
            let func_id = func_node.map(|n| n.id());
            for child in node.children(&mut cursor) {
                if is_expression_kind(child.kind()) && Some(child.id()) != func_id {
                    args.push(format_expression(child, contents));
                }
            }
            format!("{}({})", function, args.join(", "))
        }
        "arrow_function" => {
            let parameter = node
                .child_by_field_name("parameter")
                .map(|n| contents[n.byte_range()].to_string())
                .unwrap_or_default();
            let body = node
                .child_by_field_name("body")
                .map(|n| format_expression(n, contents))
                .unwrap_or_default();
            format!("{} => {}", parameter, body)
        }
        _ => contents[node.byte_range()].to_string(),
    }
}
