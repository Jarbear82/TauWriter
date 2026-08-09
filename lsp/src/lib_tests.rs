//! Unit tests for lib (utf16_idx_to_byte_idx) - migrated from inline block.
//! This is a submodule of the crate root, granting access to private members.

use crate::utf16_idx_to_byte_idx;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf16_idx_to_byte_idx() {
        // ASCII
        assert_eq!(utf16_idx_to_byte_idx("hello", 0), 0);
        assert_eq!(utf16_idx_to_byte_idx("hello", 2), 2);
        assert_eq!(utf16_idx_to_byte_idx("hello", 10), 5);

        // Multi-byte Unicode (curly quotes: 3 bytes each, 1 UTF-16 code unit each)
        let s = "\u{201C}hello\u{201D}";
        assert_eq!(utf16_idx_to_byte_idx(s, 0), 0);
        assert_eq!(utf16_idx_to_byte_idx(s, 1), 3); // after "
        assert_eq!(utf16_idx_to_byte_idx(s, 6), 8); // after o

        // Surrogate pairs (smiley face: 4 bytes, 2 UTF-16 code units)
        let smiley = "a\u{1F60A}b";
        assert_eq!(utf16_idx_to_byte_idx(smiley, 0), 0);
        assert_eq!(utf16_idx_to_byte_idx(smiley, 1), 1); // before 😊
        assert_eq!(utf16_idx_to_byte_idx(smiley, 2), 5); // middle of 😊, snaps to after 😊
        assert_eq!(utf16_idx_to_byte_idx(smiley, 3), 5); // after 😊
        assert_eq!(utf16_idx_to_byte_idx(smiley, 4), 6);
    }

    #[test]
    fn test_utf16_idx_to_byte_idx_2() {
        assert_eq!(utf16_idx_to_byte_idx("h\u{00E9}llo", 2), 3);
        assert_eq!(utf16_idx_to_byte_idx("\u{1F389}\u{1F38A}", 1), 4);
    }
}
