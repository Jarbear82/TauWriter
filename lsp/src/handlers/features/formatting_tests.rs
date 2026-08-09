//! Unit tests for formatting feature - migrated from inline block.
//! This is a submodule of handlers::features, granting access to private members.

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_compute_format_edit_no_change_returns_none() {
        let content = "line1\nline2\nline3";
        assert_eq!(compute_format_edit(content, content), None);
    }

    #[test]
    fn test_compute_format_edit_empty_strings() {
        assert_eq!(compute_format_edit("", ""), None);
    }

    #[test]
    fn test_compute_format_edit_all_lines_changed() {
        let original = "a\nb\nc";
        let formatted = "x\ny\nz";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 0);
        assert_eq!(edit.range.start.character, 0);
        let orig_lines: Vec<&str> = original.lines().collect();
        assert_eq!(edit.range.end.line, (orig_lines.len() - 1) as u32);
        assert_eq!(
            edit.range.end.character,
            orig_lines.last().unwrap().len() as u32
        );
        assert_eq!(edit.new_text, "x\ny\nz");
    }

    #[test]
    fn test_compute_format_edit_prefix_preserved() {
        let original = "unchanged\nold_line\nend";
        let formatted = "unchanged\nnew_line\nend";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 1);
        assert_eq!(edit.new_text, "new_line");
    }

    #[test]
    fn test_compute_format_edit_suffix_preserved() {
        let original = "start\nold";
        let formatted = "start\nnew";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 1);
        assert_eq!(edit.new_text, "new");
    }

    #[test]
    fn test_compute_format_edit_both_prefix_and_suffix() {
        let original = "prefix\nmiddle\nsuffix";
        let formatted = "prefix\nchanged\nsuffix";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 1);
        assert_eq!(edit.new_text, "changed");
    }

    #[test]
    fn test_compute_format_edit_only_prefix_matches() {
        let original = "unchanged\nold_line";
        let formatted = "unchanged\nnew_line";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 1);
        assert_eq!(edit.new_text, "new_line");
    }

    #[test]
    fn test_compute_format_edit_only_suffix_matches() {
        let original = "old\nsuffix";
        let formatted = "new\nsuffix";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 0);
        assert_eq!(edit.new_text, "new");
    }

    #[test]
    fn test_compute_format_edit_multiline_replacement() {
        let original = "a\nold_block";
        let formatted = "a\nfirst\nsecond\nthird";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 1);
        assert_eq!(edit.new_text, "first\nsecond\nthird");
    }

    #[test]
    fn test_compute_format_edit_single_line_to_multiple() {
        let original = "one";
        let formatted = "first\nsecond";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert_eq!(edit.range.start.line, 0);
        assert_eq!(edit.new_text, "first\nsecond");
    }

    #[test]
    fn test_compute_format_edit_trailing_newline() {
        let original = "a\nb\n";
        let formatted = "x\ny\n";

        let edit = compute_format_edit(original, formatted).expect("expected an edit");

        assert!(edit.new_text.ends_with('\n'));
    }
}
