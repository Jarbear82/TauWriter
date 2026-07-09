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

/// Generate a standard UUID v4 string (with hyphens).
pub fn generate_uuid_v4() -> String {
    let mut bytes = get_pseudorandom_bytes(16);
    // Set version to 4
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // Set variant to RFC 4122
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

/// Generate a HubGS ref-style UUID (no hyphens, underscore prefix).
pub fn generate_uuid_ref() -> String {
    let mut bytes = get_pseudorandom_bytes(16);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11],
        bytes[12], bytes[13], bytes[14], bytes[15]
    );
    format!("_{}", hex)
}

#[cfg(test)]
#[path = "uuid_tests.rs"]
mod uuid_test_module;
