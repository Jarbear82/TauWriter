//! Unit tests for uuid handler.

use crate::handlers::{generate_uuid_ref, generate_uuid_v4, generate_uuid_v7};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_v4_generation_format() {
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
    }

    #[test]
    fn test_uuid_v7_generation_format() {
        let uuid_v7 = generate_uuid_v7();
        assert_eq!(uuid_v7.len(), 36);

        let parts: Vec<&str> = uuid_v7.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);

        // Assert version is 7 (the first character of the 3rd group must be '7')
        assert_eq!(parts[2].chars().next(), Some('7'));

        // Assert variant is RFC 4122 (the first character of the 4th group must be '8', '9', 'a', or 'b')
        let var_char = parts[3].chars().next().unwrap().to_ascii_lowercase();
        assert!(vec!['8', '9', 'a', 'b'].contains(&var_char));

        // Test HubGS ref UUID format (also standard UUID v7 format)
        let uuid_ref = generate_uuid_ref();
        assert_eq!(uuid_ref.len(), 36);
        let ref_parts: Vec<&str> = uuid_ref.split('-').collect();
        assert_eq!(ref_parts.len(), 5);
        assert_eq!(ref_parts[2].chars().next(), Some('7'));
    }

    #[test]
    fn test_uuid_randomness() {
        let u1 = generate_uuid_v4();
        let u2 = generate_uuid_v4();
        let u3 = generate_uuid_v7();
        let u4 = generate_uuid_v7();
        assert_ne!(u1, u2);
        assert_ne!(u3, u4);
        assert_ne!(u1, u3);
    }
}
