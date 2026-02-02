pub mod children;
pub mod config;
pub mod error;
pub mod generate;
pub mod hash;
pub mod parse;
pub mod resolve;

pub use error::{TerseIdError, Result};
pub use config::IdConfig;
pub use generate::IdGenerator;
pub use parse::{ParsedId, parse_id, is_valid_id_format, normalize_id, validate_prefix};
pub use children::{child_id, is_child_id, id_depth};
pub use resolve::{IdResolver, ResolverConfig, MatchType, ResolvedId, find_matching_ids};

/// Compute a base36 hash of the input, truncated or zero-padded to `length` characters.
pub fn hash(input: impl AsRef<[u8]>, length: usize) -> String {
    hash::hash(input, length)
}
