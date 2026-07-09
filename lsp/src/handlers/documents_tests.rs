//! Unit tests for documents handler - migrated from inline block.
//! This is a submodule of handlers, granting access to private members.

#[cfg(test)]
mod tests {
    use super::super::*;
    use lsp_types::{Position, Range};
    use ropey::Rope;

    #[test]
    fn test_compute_input_edit_single_line_replace() {
        // "hello world" — replace "world" (chars 6..11) with "there"
        let rope = Rope::from_str("hello world");
        let range = Range {
            start: Position {
                line: 0,
                character: 6,
            },
            end: Position {
                line: 0,
                character: 11,
            },
        };
        let new_text = "there";

        let edit = compute_input_edit(&rope, range, new_text, 6, 11, 0, 0);

        assert_eq!(edit.start_byte, 6);
        assert_eq!(edit.old_end_byte, 11);
        assert_eq!(edit.new_end_byte, 11);
        assert_eq!(edit.start_position.row, 0);
        assert_eq!(edit.start_position.column, 6);
        assert_eq!(edit.old_end_position.row, 0);
        assert_eq!(edit.old_end_position.column, 11);
        assert_eq!(edit.new_end_position.row, 0);
        assert_eq!(edit.new_end_position.column, 11);
    }

    #[test]
    fn test_compute_input_edit_multiline_insertion() {
        // "hello\nworld" — insert "FOO\nBAR" at position 5 (between lines)
        let rope = Rope::from_str("hello\nworld");
        let range = Range {
            start: Position {
                line: 0,
                character: 5,
            },
            end: Position {
                line: 1,
                character: 0,
            },
        };
        let new_text = "FOO\nBAR";

        let edit = compute_input_edit(&rope, range, new_text, 5, 5, 0, 1);

        assert_eq!(edit.start_byte, 5);
        assert_eq!(edit.old_end_byte, 5);
        assert_eq!(edit.new_end_byte, 12);
        assert_eq!(edit.start_position.row, 0);
        assert_eq!(edit.start_position.column, 5);
        assert_eq!(edit.new_end_position.row, 1);
        assert_eq!(edit.new_end_position.column, 3);
    }

    #[test]
    fn test_compute_input_edit_no_newlines_in_insertion() {
        let rope = Rope::from_str("abc");
        let range = Range {
            start: Position {
                line: 0,
                character: 1,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        };
        let new_text = "XY";

        let edit = compute_input_edit(&rope, range, new_text, 1, 1, 0, 0);

        assert_eq!(edit.start_position.row, 0);
        assert_eq!(edit.new_end_position.row, 0);
        assert_eq!(edit.new_end_position.column, 3);
    }

    #[test]
    fn test_compute_input_edit_empty_rope() {
        let rope = Rope::from_str("");
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        };
        let new_text = "hello";

        let edit = compute_input_edit(&rope, range, new_text, 0, 0, 0, 0);

        assert_eq!(edit.start_byte, 0);
        assert_eq!(edit.old_end_byte, 0);
        assert_eq!(edit.new_end_byte, 5);
    }

    #[test]
    fn test_compute_input_edit_multiline_old_range() {
        let rope = Rope::from_str("line1\nline2\nline3");
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 2,
                character: 3,
            },
        };
        let new_text = "replaced";

        let edit = compute_input_edit(&rope, range, new_text, 0, 14, 0, 2);

        assert_eq!(edit.old_end_byte, rope.char_to_byte(14));
        assert_eq!(edit.old_end_position.row, 2);
        let end_line_byte = rope.line_to_byte(2);
        assert_eq!(
            edit.old_end_position.column,
            (14u32.saturating_sub(end_line_byte as u32)) as usize
        );
    }
}
