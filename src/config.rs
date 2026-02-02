pub struct IdConfig {
    pub prefix: String,
    pub min_hash_length: usize,
    pub max_hash_length: usize,
    pub max_collision_prob: f64,
}

impl IdConfig {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            min_hash_length: 3,
            max_hash_length: 8,
            max_collision_prob: 0.25,
        }
    }

    pub fn min_hash_length(mut self, len: usize) -> Self {
        self.min_hash_length = len;
        self
    }

    pub fn max_hash_length(mut self, len: usize) -> Self {
        self.max_hash_length = len;
        self
    }

    pub fn max_collision_prob(mut self, prob: f64) -> Self {
        self.max_collision_prob = prob;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_defaults() {
        let config = IdConfig::new("bd");
        assert_eq!(config.prefix, "bd");
        assert_eq!(config.min_hash_length, 3);
        assert_eq!(config.max_hash_length, 8);
        assert_eq!(config.max_collision_prob, 0.25);
    }

    #[test]
    fn test_builder_chain() {
        let config = IdConfig::new("tk")
            .min_hash_length(4)
            .max_collision_prob(0.10);
        assert_eq!(config.prefix, "tk");
        assert_eq!(config.min_hash_length, 4);
        assert_eq!(config.max_hash_length, 8);
        assert_eq!(config.max_collision_prob, 0.10);
    }

    #[test]
    fn test_builder_all_methods() {
        let config = IdConfig::new("test")
            .min_hash_length(2)
            .max_hash_length(10)
            .max_collision_prob(0.5);
        assert_eq!(config.prefix, "test");
        assert_eq!(config.min_hash_length, 2);
        assert_eq!(config.max_hash_length, 10);
        assert_eq!(config.max_collision_prob, 0.5);
    }
}
