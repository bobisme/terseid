# terseid

Adaptive-length, collision-resistant short IDs for Rust.

Extracted from the ID generation system in [beads_rust](https://github.com/Dicklesworthstone/beads_rust). Produces human-friendly IDs like `bd-a7x3q9` or `tk-r2m.1.3` that automatically grow in length as the ID space fills up.

## ID Format

```
<prefix>-<base36hash>[.<child>.<path>]
```

| Component | Description | Example |
|-----------|-------------|---------|
| prefix | Caller-chosen namespace (required) | `bd`, `tk`, `ev` |
| hash | Base36 lowercase, 3-12 chars | `a7x3q9` |
| child path | Optional dot-separated u32 segments | `.1`, `.1.3.7` |

Full examples: `bd-a7x3q9`, `tk-r2m`, `ev-00plk4.2.1`

Base36 alphabet: `0123456789abcdefghijklmnopqrstuvwxyz`

## Algorithm

### Hashing

1. Caller provides seed bytes (arbitrary content + nonce).
2. SHA256 the seed.
3. Take the first 8 bytes as a big-endian `u64`.
4. Base36-encode the `u64`.
5. Truncate (or zero-pad) to the requested length.

This gives a deterministic, uniformly distributed short string from any input.

### Adaptive Length

The hash length is chosen dynamically based on the current number of existing items, using the birthday problem approximation:

```
P(collision) = 1 - e^(-n^2 / 2d)
```

Where `n` = item count and `d` = 36^length (the size of the ID space at that length).

Starting from `min_hash_length`, the generator picks the shortest length where `P < max_collision_prob`. This means:

| Items | Optimal length (at default 25% threshold) | ID space |
|-------|-------------------------------------------|----------|
| 0-100 | 3 chars | 46,656 |
| ~200 | 4 chars | 1,679,616 |
| ~7,000 | 5 chars | 60,466,176 |
| ~250,000 | 6 chars | 2.18 billion |
| ~1.5M | 7 chars | 78.4 billion |
| ~8M | 8 chars | 2.82 trillion |

### Collision Avoidance

When generating an ID, the generator uses a multi-tier strategy:

1. **Nonce escalation**: Try nonces 0 through 9 at the current optimal length. Each nonce produces a different seed, thus a different hash. The caller's seed function receives the nonce.
2. **Length extension**: If all 10 nonces collide, increment the hash length by 1 and repeat. This continues up to `max_hash_length`.
3. **Long fallback**: If `max_hash_length` is exhausted, switch to 12-character hashes and try nonces 0 through 1000.
4. **Desperate fallback**: If all 1001 long-hash nonces collide (effectively impossible), append the nonce number to the hash string to guarantee uniqueness.

The caller provides an `exists` function that checks whether a candidate ID is already taken. This keeps the generator storage-agnostic.

## API

### Configuration

```rust
pub struct IdConfig {
    pub prefix: String,
    pub min_hash_length: usize,   // default: 3
    pub max_hash_length: usize,   // default: 8
    pub max_collision_prob: f64,   // default: 0.25
}
```

The prefix is required. There is no default — each consumer chooses their own namespace.

```rust
IdConfig::new("bd")
IdConfig::new("tk").min_hash_length(4).max_collision_prob(0.10)
```

### Generation

```rust
pub struct IdGenerator {
    config: IdConfig,
}

impl IdGenerator {
    pub fn new(config: IdConfig) -> Self;
    pub fn prefix(&self) -> &str;

    /// Birthday problem: optimal hash length for a given item count.
    pub fn optimal_length(&self, item_count: usize) -> usize;

    /// Hash seed bytes at a specific length. Returns `<prefix>-<hash>`.
    pub fn candidate(&self, seed: impl AsRef<[u8]>, hash_length: usize) -> String;

    /// Generate with collision avoidance.
    ///
    /// `seed_fn` is called with the nonce (0, 1, 2, ...) and returns seed bytes.
    /// `item_count` is the current number of existing items.
    /// `exists` returns true if a candidate ID is already taken.
    pub fn generate<S, F>(
        &self,
        seed_fn: S,
        item_count: usize,
        exists: F,
    ) -> String
    where
        S: Fn(u32) -> Vec<u8>,
        F: Fn(&str) -> bool;
}
```

Usage:

```rust
let gen = IdGenerator::new(IdConfig::new("bd"));

let id = gen.generate(
    |nonce| format!("my title|my desc|{nonce}").into_bytes(),
    current_count,
    |candidate| db.id_exists(candidate),
);
// => "bd-a7x3q9"
```

### Standalone Hash

For callers who just want a short hash without the full generator:

```rust
/// Base36 hash of arbitrary bytes, truncated to `length` characters.
pub fn hash(input: impl AsRef<[u8]>, length: usize) -> String;
```

### Parsing

```rust
pub struct ParsedId {
    pub prefix: String,
    pub hash: String,
    pub child_path: Vec<u32>,
}

impl ParsedId {
    pub fn is_root(&self) -> bool;
    pub fn depth(&self) -> usize;
    pub fn parent(&self) -> Option<String>;
    pub fn to_id_string(&self) -> String;
    pub fn is_child_of(&self, potential_parent: &str) -> bool;
}

impl Display for ParsedId { ... }

pub fn parse_id(id: &str) -> Result<ParsedId>;
pub fn is_valid_id_format(id: &str) -> bool;
pub fn normalize_id(id: &str) -> String;     // lowercase
pub fn validate_prefix(id: &str, expected: &str, allowed: &[&str]) -> Result<()>;
```

Parsing rules:
- The **last dash** before a valid hash segment separates prefix from hash. This supports hyphenated prefixes like `my-proj-a7x3q9`.
- A hash segment at 3 chars accepts any base36. At 4+ chars it must contain at least one digit (avoids ambiguity with English words like `my-proj-test`).
- Child path segments after dots must be valid `u32` integers.

### Child IDs

```rust
pub fn child_id(parent_id: &str, child_number: u32) -> String;
pub fn is_child_id(id: &str) -> bool;
pub fn id_depth(id: &str) -> usize;
```

`child_id("bd-a7x", 1)` returns `"bd-a7x.1"`. Nesting is unlimited: `bd-a7x.1.3.7`.

### Resolution

Resolves partial or fuzzy user input to a full ID. Useful for CLIs where users type shorthand.

```rust
pub struct IdResolver {
    config: ResolverConfig,
}

pub struct ResolverConfig {
    pub default_prefix: String,
    pub allowed_prefixes: Vec<String>,
    pub allow_substring_match: bool,  // default: true
}

pub enum MatchType { Exact, PrefixNormalized, Substring }

pub struct ResolvedId {
    pub id: String,
    pub match_type: MatchType,
    pub original_input: String,
}

impl IdResolver {
    pub fn new(config: ResolverConfig) -> Self;

    pub fn resolve<F, G>(
        &self,
        input: &str,
        exists_fn: F,
        substring_match_fn: G,
    ) -> Result<ResolvedId>
    where
        F: Fn(&str) -> bool,
        G: Fn(&str) -> Vec<String>;
}

/// Helper: find all IDs in a list whose hash portion contains the substring.
pub fn find_matching_ids(all_ids: &[String], hash_substring: &str) -> Vec<String>;
```

Resolution order:
1. **Exact match** — input matches an existing ID verbatim.
2. **Prefix normalization** — if input has no dash, prepend `default_prefix-` and retry.
3. **Substring match** — search hash portions for the input as a substring. Exactly one match succeeds; multiple matches return `AmbiguousId` error.
4. **Not found** — no match at any stage.

Input is lowercased and trimmed before resolution.

### Errors

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TerseIdError {
    #[error("invalid ID format: {id}")]
    InvalidId { id: String },

    #[error("prefix mismatch: expected '{expected}', found '{found}'")]
    PrefixMismatch { expected: String, found: String },

    #[error("ambiguous ID '{partial}': matches {matches:?}")]
    AmbiguousId { partial: String, matches: Vec<String> },

    #[error("ID not found: {id}")]
    NotFound { id: String },
}

pub type Result<T> = std::result::Result<T, TerseIdError>;
```

## Crate Structure

```
src/
  lib.rs          re-exports, top-level hash() function
  error.rs        TerseIdError, Result alias
  config.rs       IdConfig with builder methods
  hash.rs         compute_hash, base36_encode (pub(crate))
  generate.rs     IdGenerator
  parse.rs        ParsedId, parse_id, validation functions
  children.rs     child_id, is_child_id, id_depth
  resolve.rs      IdResolver, ResolverConfig, MatchType, ResolvedId
```

## Dependencies

```toml
[dependencies]
sha2 = "0.10"
thiserror = "2"

[dev-dependencies]
proptest = "1"
```

Two runtime dependencies. No chrono, no serde, no std feature gates.

## Migration Path for botcrit

botcrit currently generates IDs in `src/events/ids.rs` using UUID v4 → base36 with a fixed 4-char suffix. Three prefixes: `cr` (reviews), `th` (threads), `c` (comments). No collision detection — relies on 36^4 = 1.6M possible values being large enough.

### What changes

1. Replace the `uuid` dependency with `terseid`.

2. Replace `src/events/ids.rs` with terseid calls. Each entity type gets its own generator:

   ```rust
   use terseid::{IdConfig, IdGenerator};

   fn review_generator() -> IdGenerator {
       IdGenerator::new(IdConfig::new("cr"))
   }

   fn thread_generator() -> IdGenerator {
       IdGenerator::new(IdConfig::new("th"))
   }

   fn comment_generator() -> IdGenerator {
       IdGenerator::new(IdConfig::new("c"))
   }
   ```

3. For random (non-deterministic) IDs like botcrit uses today, the seed function just needs to produce unique bytes each call. Use `getrandom` or `rand`:

   ```rust
   pub fn new_review_id(item_count: usize, exists: impl Fn(&str) -> bool) -> String {
       review_generator().generate(
           |nonce| {
               let mut buf = [0u8; 16];
               getrandom::fill(&mut buf).unwrap();
               let mut seed = buf.to_vec();
               seed.extend_from_slice(&nonce.to_le_bytes());
               seed
           },
           item_count,
           exists,
       )
   }
   ```

   This gains adaptive length and collision avoidance over the current approach.

4. If botcrit doesn't need collision checking (current behavior), use the simpler `candidate` method:

   ```rust
   pub fn new_review_id() -> String {
       let mut buf = [0u8; 16];
       getrandom::fill(&mut buf).unwrap();
       review_generator().candidate(&buf, 4)
   }
   ```

   This is a drop-in replacement for the current UUID-based approach, same fixed length, same randomness, just different entropy source.

5. The `is_review_id` / `is_thread_id` / `is_comment_id` validators can use `parse_id` + `validate_prefix` instead of hand-rolled length checks.

### What botcrit gains

- **Adaptive length**: IDs can grow from 4 to 8 chars as the review count increases, instead of being stuck at 4 forever.
- **Collision avoidance**: The `generate` path actively checks for and avoids collisions.
- **Parsing/validation**: `parse_id("cr-a7x3")` returns structured data instead of string matching.
- **Resolution**: Users can type `a7x` in the CLI and the resolver finds `cr-a7x3`.
