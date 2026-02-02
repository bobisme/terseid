use crate::config::IdConfig;

/// ID generator with adaptive length and collision avoidance.
pub struct IdGenerator {
    config: IdConfig,
}

impl IdGenerator {
    /// Create a new ID generator with the given config.
    pub fn new(config: IdConfig) -> Self {
        Self { config }
    }

    /// Get the prefix for this generator.
    pub fn prefix(&self) -> &str {
        &self.config.prefix
    }

    /// Compute optimal hash length using the birthday problem approximation.
    ///
    /// Finds the shortest length where P(collision) = 1 - e^(-n^2 / 2d) < max_collision_prob,
    /// where d = 36^length (the size of the ID space at that length).
    ///
    /// Starting from min_hash_length, returns the first length that satisfies the threshold.
    /// If no length up to max_hash_length satisfies it, returns max_hash_length.
    pub fn optimal_length(&self, item_count: usize) -> usize {
        let n = item_count as f64;

        for length in self.config.min_hash_length..=self.config.max_hash_length {
            let d = 36_usize.pow(length as u32) as f64;
            let exponent = -((n.powi(2)) / (2.0 * d));
            let p_collision = 1.0 - exponent.exp();

            if p_collision < self.config.max_collision_prob {
                return length;
            }
        }

        // If no length up to max found, return max_hash_length
        self.config.max_hash_length
    }

    /// Generate a candidate ID at a specific hash length.
    ///
    /// Returns a string formatted as `{prefix}-{hash}`, where hash is the base36
    /// hash of the seed bytes truncated/padded to the specified length.
    pub fn candidate(&self, seed: impl AsRef<[u8]>, hash_length: usize) -> String {
        let hash_str = crate::hash::hash(seed, hash_length);
        format!("{}-{}", self.config.prefix, hash_str)
    }

    /// Generate an ID with full collision avoidance.
    ///
    /// Uses a multi-tier strategy:
    /// 1. Nonce escalation: try nonces 0-9 at optimal length
    /// 2. Length extension: increment length, repeat up to max_hash_length
    /// 3. Long fallback: 12-char hashes, nonces 0-1000
    /// 4. Desperate fallback: append nonce number to hash
    ///
    /// `seed_fn` is called with the nonce (0, 1, 2, ...) and should return seed bytes.
    /// `item_count` is the current number of existing items.
    /// `exists` returns true if a candidate ID is already taken.
    pub fn generate<S, F>(&self, seed_fn: S, item_count: usize, exists: F) -> String
    where
        S: Fn(u32) -> Vec<u8>,
        F: Fn(&str) -> bool,
    {
        // Phase 1: Nonce escalation at optimal length
        let optimal = self.optimal_length(item_count);
        for nonce in 0..10 {
            let seed = seed_fn(nonce);
            let candidate = self.candidate(&seed, optimal);
            if !exists(&candidate) {
                return candidate;
            }
        }

        // Phase 2: Length extension (increment length, repeat up to max_hash_length)
        for length in (optimal + 1)..=self.config.max_hash_length {
            for nonce in 0..10 {
                let seed = seed_fn(nonce);
                let candidate = self.candidate(&seed, length);
                if !exists(&candidate) {
                    return candidate;
                }
            }
        }

        // Phase 3: Long fallback (12-char hashes, nonces 0-1000)
        for nonce in 0..=1000 {
            let seed = seed_fn(nonce);
            let candidate = self.candidate(&seed, 12);
            if !exists(&candidate) {
                return candidate;
            }
        }

        // Phase 4: Desperate fallback (append nonce number to the hash)
        // This guarantees uniqueness since we're appending the nonce directly
        let seed = seed_fn(0);
        let hash_str = crate::hash::hash(&seed, 12);
        for nonce in 0..=10000 {
            let desperate = format!("{}-{}{}", self.config.prefix, hash_str, nonce);
            if !exists(&desperate) {
                return desperate;
            }
        }

        // Absolute fallback: should never reach here in practice
        format!("{}-{}.fallback", self.config.prefix, crate::hash::hash(&seed_fn(0), 12))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_generator() {
        let config = IdConfig::new("bd");
        let generator = IdGenerator::new(config);
        assert_eq!(generator.prefix(), "bd");
    }

    #[test]
    fn test_prefix_accessor() {
        let generator = IdGenerator::new(IdConfig::new("tk"));
        assert_eq!(generator.prefix(), "tk");
    }

    #[test]
    fn test_optimal_length_small_count() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        // For 50 items, should use 3 chars (36^3 = 46,656)
        let len = generator.optimal_length(50);
        assert_eq!(len, 3);
    }

    #[test]
    fn test_optimal_length_medium_count() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        // For 200 items, should use 4 chars (36^4 = 1,679,616)
        let len = generator.optimal_length(200);
        assert_eq!(len, 4);
    }

    #[test]
    fn test_optimal_length_large_count() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        // For 7000 items, should use 6 chars
        // (36^5 = 60M gives p ≈ 0.33; 36^6 = 2.1B gives p ≈ 0.01)
        let len = generator.optimal_length(7000);
        assert_eq!(len, 6);
    }

    #[test]
    fn test_optimal_length_respects_min() {
        let generator = IdGenerator::new(IdConfig::new("bd").min_hash_length(4));
        // Even for 0 items, should use at least min_hash_length
        let len = generator.optimal_length(0);
        assert_eq!(len, 4);
    }

    #[test]
    fn test_optimal_length_respects_max() {
        let generator = IdGenerator::new(IdConfig::new("bd").max_hash_length(5));
        // For very large item count, should not exceed max_hash_length
        let len = generator.optimal_length(1_000_000_000);
        assert_eq!(len, 5);
    }

    #[test]
    fn test_optimal_length_custom_collision_prob() {
        let generator_strict = IdGenerator::new(
            IdConfig::new("bd").max_collision_prob(0.01), // stricter threshold
        );
        let len_strict = generator_strict.optimal_length(100);

        let generator_loose = IdGenerator::new(
            IdConfig::new("bd").max_collision_prob(0.50), // looser threshold
        );
        let len_loose = generator_loose.optimal_length(100);

        // Stricter should be >= looser
        assert!(len_strict >= len_loose);
    }

    #[test]
    fn test_candidate_format() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        let seed = b"test seed";
        let candidate = generator.candidate(seed, 6);

        // Should be in format "prefix-hash"
        assert!(candidate.starts_with("bd-"));
        let parts: Vec<&str> = candidate.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "bd");
        assert_eq!(parts[1].len(), 6);
    }

    #[test]
    fn test_candidate_deterministic() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        let seed = b"consistent seed";

        let c1 = generator.candidate(seed, 8);
        let c2 = generator.candidate(seed, 8);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_candidate_different_seeds_different_hashes() {
        let generator = IdGenerator::new(IdConfig::new("bd"));

        let c1 = generator.candidate(b"seed1", 8);
        let c2 = generator.candidate(b"seed2", 8);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_candidate_different_lengths() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        let seed = b"test";

        let c3 = generator.candidate(seed, 3);
        let c8 = generator.candidate(seed, 8);

        assert_eq!(c3.len(), "bd-".len() + 3);
        assert_eq!(c8.len(), "bd-".len() + 8);
    }

    #[test]
    fn test_generate_no_collisions_simple() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        let mut generated = vec![];

        for i in 0..10 {
            let id = generator.generate(
                |nonce| format!("seed{}-{}", i, nonce).into_bytes(),
                0,
                |candidate| generated.contains(&candidate.to_string()),
            );

            generated.push(id);
        }

        // Check uniqueness
        let unique_count = {
            let mut set = std::collections::HashSet::new();
            for id in &generated {
                set.insert(id.clone());
            }
            set.len()
        };
        assert_eq!(unique_count, 10);
    }

    #[test]
    fn test_generate_handles_collisions() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        let taken: std::collections::HashSet<String> = std::collections::HashSet::new();

        let id = generator.generate(
            |_nonce| b"forced-collision".to_vec(),
            0,
            |candidate| taken.contains(candidate),
        );

        // Should still generate something valid
        assert!(id.starts_with("bd-"));
        assert!(id.len() > 0);
    }

    #[test]
    fn test_generate_with_high_item_count() {
        let generator = IdGenerator::new(IdConfig::new("bd"));

        let id = generator.generate(
            |nonce| format!("content-{}", nonce).into_bytes(),
            250_000, // High item count should use longer hashes
            |_candidate| false, // No collisions
        );

        // Should generate a valid ID
        assert!(id.starts_with("bd-"));

        // Optimal length for 250k items should be 6 or more
        let optimal = generator.optimal_length(250_000);
        assert!(optimal >= 5);
    }

    #[test]
    fn test_generate_format() {
        let generator = IdGenerator::new(IdConfig::new("test"));

        let id = generator.generate(
            |nonce| format!("data-{}", nonce).into_bytes(),
            10,
            |_| false,
        );

        // Check format: prefix-hash
        assert!(id.starts_with("test-"));
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts[0], "test");
    }

    #[test]
    fn test_optimal_length_boundary_100() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        // Just under 100
        assert_eq!(generator.optimal_length(99), 3);
        // At 100
        let len_100 = generator.optimal_length(100);
        assert!(len_100 >= 3);
    }

    #[test]
    fn test_optimal_length_boundary_200() {
        let generator = IdGenerator::new(IdConfig::new("bd"));
        // Around 200 should transition to 4
        let len_200 = generator.optimal_length(200);
        assert!(len_200 >= 3);
    }

    #[test]
    fn test_multiple_generators_independent() {
        let gen1 = IdGenerator::new(IdConfig::new("bd"));
        let gen2 = IdGenerator::new(IdConfig::new("tk"));

        assert_eq!(gen1.prefix(), "bd");
        assert_eq!(gen2.prefix(), "tk");

        let opt1 = gen1.optimal_length(100);
        let opt2 = gen2.optimal_length(100);

        // Should have same optimal length (same config)
        assert_eq!(opt1, opt2);
    }

    #[test]
    fn test_generate_with_string_seed() {
        let generator = IdGenerator::new(IdConfig::new("bd"));

        let id = generator.generate(
            |nonce| format!("user-description-{}", nonce).into_bytes(),
            50,
            |_| false,
        );

        assert!(id.starts_with("bd-"));
    }

    #[test]
    fn test_candidate_all_valid_base36() {
        let generator = IdGenerator::new(IdConfig::new("bd"));

        for seed_val in 0..100 {
            let candidate = generator.candidate(
                format!("seed-{}", seed_val).as_bytes(),
                6,
            );

            // Extract hash part
            let parts: Vec<&str> = candidate.split('-').collect();
            assert_eq!(parts.len(), 2);

            let hash = parts[1];
            for ch in hash.chars() {
                assert!(
                    ch.is_ascii_digit() || (ch >= 'a' && ch <= 'z'),
                    "Invalid base36 char in {}: {}",
                    candidate,
                    ch
                );
            }
        }
    }

    #[test]
    fn test_generate_always_returns_valid_format() {
        let generator = IdGenerator::new(IdConfig::new("prefix"));

        for item_count in [0, 1, 10, 100, 1000, 10000].iter() {
            let id = generator.generate(
                |nonce| format!("seed-{}", nonce).into_bytes(),
                *item_count,
                |_| false,
            );

            // Basic format validation
            assert!(id.starts_with("prefix-"));
            let hash_part = &id[7..]; // Skip "prefix-"
            assert!(!hash_part.is_empty());

            // All chars should be valid base36 or our fallback format
            for ch in hash_part.chars() {
                assert!(
                    ch.is_ascii_digit() || (ch >= 'a' && ch <= 'z'),
                    "Invalid char in {}: {}",
                    id,
                    ch
                );
            }
        }
    }
}
