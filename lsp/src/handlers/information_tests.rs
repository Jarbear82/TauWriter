//! Unit tests for information handler - migrated from inline block.
//! This is a submodule of handlers, granting access to private members.

use crate::handlers::{format_hub_value, MarkdownContent};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{HubValue, RawF64};

    // ===== MarkdownContent builder tests =====

    #[test]
    fn test_markdown_content_new_is_empty() {
        let md = MarkdownContent::new();
        assert_eq!(md.to_string(), "");
    }

    #[test]
    fn test_heading_level_2() {
        let mut md = MarkdownContent::new();
        md.heading(2, "Title");
        assert_eq!(md.to_string(), "## Title");
    }

    #[test]
    fn test_heading_level_1() {
        let mut md = MarkdownContent::new();
        md.heading(1, "Root");
        assert_eq!(md.to_string(), "# Root");
    }

    #[test]
    fn test_heading_level_3() {
        let mut md = MarkdownContent::new();
        md.heading(3, "Subsection");
        assert_eq!(md.to_string(), "### Subsection");
    }

    #[test]
    fn test_bold_list_item_format() {
        let mut md = MarkdownContent::new();
        md.bold_list_item("key", "value");
        assert_eq!(md.to_string(), "- **key:** value");
    }

    #[test]
    fn test_separator() {
        let mut md = MarkdownContent::new();
        md.separator();
        assert_eq!(md.to_string(), "---");
    }

    #[test]
    fn test_code_block_with_lang() {
        let mut md = MarkdownContent::new();
        md.code_block("body", "hubgs");
        assert_eq!(md.to_string(), "```hubgs\nbody\n```");
    }

    #[test]
    fn test_code_block_multiline() {
        let mut md = MarkdownContent::new();
        md.code_block("line1\nline2", "rust");
        assert_eq!(md.to_string(), "```rust\nline1\nline2\n```");
    }

    #[test]
    fn test_link_with_uri() {
        let mut md = MarkdownContent::new();
        md.link_with_uri("name", "http://x");
        assert_eq!(md.to_string(), "  - [name](http://x)");
    }

    #[test]
    fn test_text_item() {
        let mut md = MarkdownContent::new();
        md.text_item("child");
        assert_eq!(md.to_string(), "  - child");
    }

    #[test]
    fn test_bold_plain() {
        let mut md = MarkdownContent::new();
        md.bold("text");
        assert_eq!(md.to_string(), "**text**");
    }

    #[test]
    fn test_text_plain() {
        let mut md = MarkdownContent::new();
        md.text("raw content");
        assert_eq!(md.to_string(), "raw content");
    }

    #[test]
    fn test_markdown_content_accumulates_lines() {
        let mut md = MarkdownContent::new();
        md.heading(2, "Header");
        md.separator();
        md.bold_list_item("Field", "Value");
        md.text("Plain line");
        let output = md.to_string();
        let expected = "## Header\n---\n- **Field:** Value\nPlain line";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_roundtrip_multi_section_hover() {
        let mut md = MarkdownContent::new();

        md.heading(2, "MyType: my_instance (Hub)");
        md.separator();
        md.heading(3, "Fields:");
        md.bold_list_item("name", "\"John\"");
        md.bold_list_item("age", "42");
        md.separator();
        md.heading(3, "Roles:");
        md.bold("owner (1)");
        md.text("Count: 1");
        md.link_with_uri("other_instance", "file:///path/to/other.hubgs");
        md.bold("member (*)");
        md.text("Count: 2");
        md.text_item("first_target");
        md.text_item("second_target");

        let output = md.to_string();

        assert!(output.starts_with("## MyType: my_instance (Hub)"));
        assert!(output.contains("- **name:** \"John\""));
        assert!(output.contains("- **age:** 42"));
        assert!(output.contains("---"));
        assert!(output.contains("### Fields:"));
        assert!(output.contains("### Roles:"));
        assert!(output.contains("**owner (1)**"));
        assert!(output.contains("Count: 1"));
        assert!(output.contains("  - [other_instance](file:///path/to/other.hubgs)"));
        assert!(output.contains("**member (*)**"));
        assert!(output.contains("Count: 2"));
        assert!(output.contains("  - first_target"));
        assert!(output.contains("  - second_target"));
    }

    // ===== format_hub_value tests =====

    #[test]
    fn test_format_hub_value_text() {
        let val = HubValue::Text("hello".to_string());
        assert_eq!(format_hub_value(&val), "\"hello\"");
    }

    #[test]
    fn test_format_hub_value_number() {
        let val = HubValue::Number(RawF64::from_f64(42.0));
        assert_eq!(format_hub_value(&val), "42");
    }

    #[test]
    fn test_format_hub_value_boolean_true() {
        let val = HubValue::Boolean(true);
        assert_eq!(format_hub_value(&val), "true");
    }

    #[test]
    fn test_format_hub_value_boolean_false() {
        let val = HubValue::Boolean(false);
        assert_eq!(format_hub_value(&val), "false");
    }

    #[test]
    fn test_format_hub_value_identifier() {
        let val = HubValue::Identifier("my_id".to_string());
        assert_eq!(format_hub_value(&val), "my_id");
    }

    #[test]
    fn test_format_hub_value_array() {
        let val = HubValue::Array(vec![
            HubValue::Number(RawF64::from_f64(1.0)),
            HubValue::Text("a".to_string()),
        ]);
        assert_eq!(format_hub_value(&val), "[1, \"a\"]");
    }

    // ===== hover_instance output format tests =====

    #[test]
    fn test_hover_instance_header_format() {
        let mut md = MarkdownContent::new();
        let type_name = "MyType";
        let name = "my_instance";
        md.heading(2, &format!("{}: {} (Hub)", type_name, name));

        assert_eq!(md.to_string(), "## MyType: my_instance (Hub)");
    }

    #[test]
    fn test_hover_instance_fields_format() {
        let mut md = MarkdownContent::new();
        md.separator();
        md.heading(3, "Fields:");
        md.bold_list_item("name", "\"John\"");
        md.bold_list_item("age", "42");

        let output = md.to_string();
        assert!(output.contains("### Fields:"));
        assert!(output.contains("- **name:** \"John\""));
        assert!(output.contains("- **age:** 42"));
    }

    #[test]
    fn test_hover_instance_roles_multiplicity_format() {
        let mut md = MarkdownContent::new();
        md.separator();
        md.heading(3, "Roles:");
        md.bold("owner (1)");
        md.text("Count: 0");
        md.bold("member (*)");
        md.text("Count: 2");

        let output = md.to_string();
        assert!(output.contains("**owner (1)**"));
        assert!(output.contains("**member (*)**"));
        assert!(output.contains("Count: 0"));
        assert!(output.contains("Count: 2"));
    }

    #[test]
    fn test_hover_instance_role_with_link() {
        let mut md = MarkdownContent::new();
        md.bold("target (1)");
        md.text("Count: 1");
        md.link_with_uri("linked_instance", "file:///path/to/file.hubgs");

        let output = md.to_string();
        assert!(output.contains("**target (1)**"));
        assert!(output.contains("Count: 1"));
        assert!(output.contains("  - [linked_instance](file:///path/to/file.hubgs)"));
    }

    #[test]
    fn test_hover_instance_no_fields_section_when_empty() {
        let mut md = MarkdownContent::new();
        md.heading(2, "MyType: my_instance (Hub)");

        let output = md.to_string();
        assert!(!output.contains("### Fields:"));
        assert_eq!(output.matches("---").count(), 0);
    }

    #[test]
    fn test_hover_instance_no_roles_section_when_empty() {
        let mut md = MarkdownContent::new();
        md.heading(2, "MyType: my_instance (Hub)");

        let output = md.to_string();
        assert!(!output.contains("### Roles:"));
    }
}
