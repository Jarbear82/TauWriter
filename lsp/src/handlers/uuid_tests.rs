//! Unit tests for uuid handler - migrated from inline block.
//! This is a submodule of handlers, granting access to private members.

use crate::handlers::{generate_uuid_ref, generate_uuid_v4};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_generation_formats() {
        // Test standard UUID v4 format
        let uuid_v4 = generate_uuid_v4();
        assert_eq!(uuid_v4.len(), 36);

        let parts: Vec<&str> = uuid_v4.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);

        // Assert version is 4 (the first character of the 3rd group must be '4')
        assert_eq!(parts[2].chars().next(), Some('4'));

        // Assert variant is RFC 4122 (the first character of the 4th group must be '8', '9', 'a', or 'b')
        let var_char = parts[3].chars().next().unwrap().to_ascii_lowercase();
        assert!(vec!['8', '9', 'a', 'b'].contains(&var_char));

        // Test HubGS ref UUID format
        let uuid_ref = generate_uuid_ref();
        assert_eq!(uuid_ref.len(), 33); // '_' prefix + 32 hex chars
        assert!(uuid_ref.starts_with('_'));

        // Ensure only alphanumeric/underscore chars are present
        assert!(uuid_ref
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'));

        // Entropy check: ensure consecutive UUIDs are different
        let uuid_v4_2 = generate_uuid_v4();
        let uuid_ref_2 = generate_uuid_ref();
        assert_ne!(uuid_v4, uuid_v4_2);
        assert_ne!(uuid_ref, uuid_ref_2);
    }
}
