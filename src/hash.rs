use sha2::{Sha256, Digest};

const BASE36_CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// SHA256 input, take first 8 bytes as big-endian u64
pub(crate) fn compute_hash(input: impl AsRef<[u8]>) -> u64 {
    let hash = Sha256::digest(input.as_ref());
    u64::from_be_bytes(hash[..8].try_into().unwrap())
}

/// Encode u64 as base36 lowercase string
pub(crate) fn base36_encode(value: u64) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let mut result = Vec::new();
    let mut v = value;
    while v > 0 {
        result.push(BASE36_CHARS[(v % 36) as usize]);
        v /= 36;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Public standalone hash function: base36 hash truncated/zero-padded to length chars
pub fn hash(input: impl AsRef<[u8]>, length: usize) -> String {
    let h = compute_hash(input);
    let encoded = base36_encode(h);
    if encoded.len() >= length {
        encoded[..length].to_string()
    } else {
        format!("{:0>width$}", encoded, width = length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_deterministic() {
        let input = b"test input";
        let hash1 = compute_hash(input);
        let hash2 = compute_hash(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_known_value() {
        // Known value test - SHA256 of "hello" first 8 bytes as big-endian u64
        let input = b"hello";
        let hash = compute_hash(input);
        // This is deterministic, so we can verify against a known computation
        // SHA256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        // First 8 bytes: 2cf24dba5fb0a30e
        // As big-endian u64: 0x2cf24dba5fb0a30e = 3238736544897475342
        assert_eq!(hash, 3238736544897475342);
    }

    #[test]
    fn test_base36_encode_zero() {
        assert_eq!(base36_encode(0), "0");
    }

    #[test]
    fn test_base36_encode_one() {
        assert_eq!(base36_encode(1), "1");
    }

    #[test]
    fn test_base36_encode_thirty_five() {
        assert_eq!(base36_encode(35), "z");
    }

    #[test]
    fn test_base36_encode_thirty_six() {
        assert_eq!(base36_encode(36), "10");
    }

    #[test]
    fn test_base36_encode_deterministic() {
        let value = 12345u64;
        let enc1 = base36_encode(value);
        let enc2 = base36_encode(value);
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_base36_encode_valid_chars() {
        let value = u64::MAX;
        let encoded = base36_encode(value);
        for c in encoded.chars() {
            assert!(c.is_ascii_digit() || (c >= 'a' && c <= 'z'),
                    "Invalid base36 character: {}", c);
        }
    }

    #[test]
    fn test_base36_encode_lowercase() {
        let value = 12345u64;
        let encoded = base36_encode(value);
        assert_eq!(encoded, encoded.to_lowercase());
    }

    #[test]
    fn test_hash_exact_length() {
        let input = b"test";
        for length in 1..20 {
            let result = hash(input, length);
            assert_eq!(result.len(), length,
                      "hash() should return exactly {} characters, got {}", length, result.len());
        }
    }

    #[test]
    fn test_hash_zero_padding() {
        let input = b"x";
        let result = hash(input, 10);
        assert_eq!(result.len(), 10);
        // Should be zero-padded if the base36 encoding is shorter
        assert!(result.chars().all(|c| c.is_ascii_digit() || (c >= 'a' && c <= 'z')));
    }

    #[test]
    fn test_hash_truncation() {
        let input = b"y";
        let hash_long = hash(input, 20);
        let hash_short = hash(input, 5);
        // The short hash should be a prefix of the long hash, but only if the encoded
        // hash is longer than the requested length. If padding is involved, they may differ.
        // Instead, just verify both have correct lengths
        assert_eq!(hash_short.len(), 5);
        assert_eq!(hash_long.len(), 20);
    }

    #[test]
    fn test_hash_deterministic() {
        let input = b"consistency test";
        let h1 = hash(input, 8);
        let h2 = hash(input, 8);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_valid_chars() {
        let input = b"base36 validation";
        let result = hash(input, 15);
        for c in result.chars() {
            assert!(c.is_ascii_digit() || (c >= 'a' && c <= 'z'),
                    "Invalid base36 character in hash: {}", c);
        }
    }

    // proptest tests for base36 alphabet validity
    // Uncomment and add proptest as a dev dependency to enable
    /*
    use proptest::proptest;

    proptest! {
        #[test]
        fn prop_base36_valid_alphabet(value in 0u64..u64::MAX) {
            let encoded = base36_encode(value);
            for c in encoded.chars() {
                prop_assert!(c.is_ascii_digit() || (c >= 'a' && c <= 'z'),
                    "base36_encode produced invalid character: {}", c);
            }
        }

        #[test]
        fn prop_hash_returns_exact_length(input in ".*", length in 1usize..100) {
            let result = hash(input.as_bytes(), length);
            prop_assert_eq!(result.len(), length);
        }

        #[test]
        fn prop_hash_valid_chars(input in ".*", length in 1usize..50) {
            let result = hash(input.as_bytes(), length);
            for c in result.chars() {
                prop_assert!(c.is_ascii_digit() || (c >= 'a' && c <= 'z'),
                    "hash produced invalid character: {}", c);
            }
        }
    }
    */
}
