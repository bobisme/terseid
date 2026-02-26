use sha2::{Digest, Sha256};

const BASE36_CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// SHA256 input, take first 8 bytes as big-endian u64
pub(crate) fn compute_hash(input: impl AsRef<[u8]>) -> u64 {
    let hash = Sha256::digest(input.as_ref());
    u64::from_be_bytes(
        hash[..8]
            .try_into()
            .expect("SHA256 always produces at least 8 bytes"),
    )
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
    // Safety: BASE36_CHARS only contains ASCII bytes
    String::from_utf8(result).expect("base36 chars are always valid UTF-8")
}

/// Public standalone hash function: base36 hash truncated/zero-padded to length chars.
///
/// For hashes of 4+ characters, guarantees at least one digit is present.
/// This ensures the output is always accepted by `parse_id`, which rejects
/// 4+ char all-letter hashes to avoid ambiguity with English words.
pub fn hash(input: impl AsRef<[u8]>, length: usize) -> String {
    let h = compute_hash(input);
    let encoded = base36_encode(h);
    let mut result = if encoded.len() >= length {
        encoded[..length].to_string()
    } else {
        format!("{encoded:0>length$}")
    };

    // Ensure 4+ char hashes contain at least one digit (parse_id requirement)
    if length >= 4 && !result.chars().any(|c| c.is_ascii_digit()) {
        // Replace the last character with a digit derived from the hash value
        let digit = char::from(b'0' + (h % 10) as u8);
        let mut chars: Vec<char> = result.chars().collect();
        chars[length - 1] = digit;
        result = chars.into_iter().collect();
    }

    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        // SHA256("hello") = 2cf24dba5fb0a30e...
        // First 8 bytes as big-endian u64: 0x2cf24dba5fb0a30e = 3238736544897475342
        let input = b"hello";
        let hash = compute_hash(input);
        assert_eq!(hash, 3_238_736_544_897_475_342);
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
            assert!(
                c.is_ascii_digit() || c.is_ascii_lowercase(),
                "Invalid base36 character: {c}"
            );
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
            assert_eq!(
                result.len(),
                length,
                "hash() should return exactly {length} characters, got {}",
                result.len()
            );
        }
    }

    #[test]
    fn test_hash_zero_padding() {
        let input = b"x";
        let result = hash(input, 10);
        assert_eq!(result.len(), 10);
        assert!(
            result
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
        );
    }

    #[test]
    fn test_hash_truncation() {
        let input = b"y";
        let hash_long = hash(input, 20);
        let hash_short = hash(input, 5);
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
            assert!(
                c.is_ascii_digit() || c.is_ascii_lowercase(),
                "Invalid base36 character in hash: {c}"
            );
        }
    }

    #[test]
    fn test_hash_4plus_chars_contains_digit() {
        // Test many inputs to find one that would produce an all-letter hash
        for i in 0..1000 {
            let input = format!("seed-{i}");
            for length in 4..=12 {
                let result = hash(input.as_bytes(), length);
                assert!(
                    result.chars().any(|c| c.is_ascii_digit()),
                    "hash of {:?} at length {} produced all-letter hash: {}",
                    input,
                    length,
                    result
                );
            }
        }
    }

    #[test]
    fn test_hash_3_chars_no_digit_requirement() {
        // 3-char hashes don't need a digit — just verify they're valid base36
        let result = hash(b"test", 3);
        assert_eq!(result.len(), 3);
        assert!(result.chars().all(|c| c.is_ascii_digit() || c.is_ascii_lowercase()));
    }

    mod proptests {
        use super::*;
        use proptest::proptest;

        proptest! {
            #[test]
            fn base36_valid_alphabet(value: u64) {
                let encoded = base36_encode(value);
                for c in encoded.chars() {
                    assert!(
                        c.is_ascii_digit() || c.is_ascii_lowercase(),
                        "base36_encode produced invalid character: {c}"
                    );
                }
            }

            #[test]
            fn hash_returns_exact_length(input in ".*", length in 1usize..100) {
                let result = hash(input.as_bytes(), length);
                assert_eq!(result.len(), length);
            }

            #[test]
            fn hash_valid_chars(input in ".*", length in 1usize..50) {
                let result = hash(input.as_bytes(), length);
                for c in result.chars() {
                    assert!(
                        c.is_ascii_digit() || c.is_ascii_lowercase(),
                        "hash produced invalid character: {c}"
                    );
                }
            }

            #[test]
            fn hash_4plus_always_has_digit(input in ".*", length in 4usize..20) {
                let result = hash(input.as_bytes(), length);
                assert!(
                    result.chars().any(|c| c.is_ascii_digit()),
                    "hash of length {} has no digit: {}",
                    length,
                    result
                );
            }
        }
    }
}
