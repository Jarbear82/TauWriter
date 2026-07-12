/// Generate cryptographically-informed pseudorandom bytes.
pub fn get_pseudorandom_bytes(len: usize) -> Vec<u8> {
    use std::io::Read;
    // Try reading from /dev/urandom first
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let mut buf = vec![0u8; len];
        if file.read_exact(&mut buf).is_ok() {
            return buf;
        }
    }
    // Fallback to LCG seeded with time
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(123456789) as u64;
    let mut buf = vec![0u8; len];
    for byte in buf.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = (seed >> 32) as u8;
    }
    buf
}

/// Generate a standard UUID v7 string (with hyphens).
pub fn generate_uuid_v4() -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let mut bytes = [0u8; 16];
    // Write 48-bit timestamp
    bytes[0..6].copy_from_slice(&ms.to_be_bytes()[2..8]);
    
    // Fill the remaining 10 bytes with pseudorandom data
    let rand = get_pseudorandom_bytes(10);
    bytes[6..16].copy_from_slice(&rand);
    
    // Set version to 7
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    // Set variant to RFC 4122 (0x80)
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Generate a HubGS ref-style UUID (hyphenated standard UUID v7 to unify layout).
pub fn generate_uuid_ref() -> String {
    generate_uuid_v4()
}

#[cfg(test)]
#[path = "uuid_tests.rs"]
mod uuid_test_module;
