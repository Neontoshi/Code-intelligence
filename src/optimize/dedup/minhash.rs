// src/optimize/dedup/minhash.rs

//! MinHash + LSH banding for approximate near-duplicate candidate generation.
//!
//! Unlike the signature/param-count buckets in candidates.rs (which only
//! catch functions with identical arity/publicity), this catches functions
//! whose *body content* is similar regardless of signature shape — e.g.
//! two builder methods with different param counts but near-identical logic.
//!
//! Deterministic: hash coefficients are fixed constants (not randomly
//! seeded per run), and bucket iteration is sorted, so results are
//! reproducible across runs on identical input.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const NUM_HASHES: usize = 32;
const SHINGLE_SIZE: usize = 3; // token k-gram size
const NUM_BANDS: usize = 8; // 32 hashes / 8 bands = 4 rows per band
const ROWS_PER_BAND: usize = NUM_HASHES / NUM_BANDS;

/// Deterministic hash function family: a_i * x + b_i mod large prime.
/// Coefficients are derived from a fixed seed, not randomized per run.
struct HashFamily {
    coeffs: Vec<(u64, u64)>,
}

impl HashFamily {
    fn new() -> Self {
        let mut coeffs = Vec::with_capacity(NUM_HASHES);
        let mut state: u64 = 0x9E3779B97F4A7C15;
        for _ in 0..NUM_HASHES {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let a = (state >> 1) | 1; // ensure odd, nonzero
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let b = state;
            coeffs.push((a, b));
        }
        Self { coeffs }
    }

    fn hash(&self, i: usize, x: u64) -> u64 {
        const PRIME: u64 = 0xFFFF_FFFF_FFFF_FFC5; // large prime near u64::MAX
        let (a, b) = self.coeffs[i];
        a.wrapping_mul(x).wrapping_add(b) % PRIME
    }
}

/// Normalize source into a token stream. Deliberately coarse (not the full
/// identifier-substitution used by compute_ast_hash) — this only needs to
/// group "structurally similar" bodies into shingle buckets, not prove
/// exact equivalence.
fn tokenize(source: &str) -> Vec<String> {
    source
        .split(|c: char| c.is_whitespace() || "(){}[];,".contains(c))
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

fn shingle_hashes(tokens: &[String]) -> Vec<u64> {
    if tokens.len() < SHINGLE_SIZE {
        return Vec::new();
    }
    tokens
        .windows(SHINGLE_SIZE)
        .map(|w| {
            let mut hasher = DefaultHasher::new();
            w.hash(&mut hasher);
            hasher.finish()
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct MinHashSignature {
    pub bands: Vec<u64>,
}

fn compute_signature(source: &str, family: &HashFamily) -> Option<MinHashSignature> {
    let tokens = tokenize(source);
    let shingles = shingle_hashes(&tokens);
    if shingles.is_empty() {
        return None;
    }

    let mut mins = vec![u64::MAX; NUM_HASHES];
    for &sh in &shingles {
        for i in 0..NUM_HASHES {
            let h = family.hash(i, sh);
            if h < mins[i] {
                mins[i] = h;
            }
        }
    }

    let mut bands = Vec::with_capacity(NUM_BANDS);
    for b in 0..NUM_BANDS {
        let mut hasher = DefaultHasher::new();
        let start = b * ROWS_PER_BAND;
        mins[start..start + ROWS_PER_BAND].hash(&mut hasher);
        bands.push(hasher.finish());
    }

    Some(MinHashSignature { bands })
}

/// LSH index: functions land in the same bucket for band `b` if their
/// band-hash matches. Sharing any bucket across any band makes them a
/// candidate pair.
pub struct LshIndex {
    family: HashFamily,
    buckets: Vec<HashMap<u64, Vec<usize>>>, // one map per band
}

impl LshIndex {
    pub fn new() -> Self {
        Self {
            family: HashFamily::new(),
            buckets: (0..NUM_BANDS).map(|_| HashMap::new()).collect(),
        }
    }

    /// Insert a function's source into the index, keyed by its position
    /// `idx` in the caller's function list. No-op if the body is too short
    /// to shingle (e.g. one-liners).
    pub fn insert(&mut self, idx: usize, source: &str) {
        if let Some(sig) = compute_signature(source, &self.family) {
            for (b, &band_hash) in sig.bands.iter().enumerate() {
                self.buckets[b].entry(band_hash).or_default().push(idx);
            }
        }
    }

    /// Emit candidate pairs: any two functions sharing a bucket in any
    /// band. Deterministic — buckets and their keys are walked in sorted
    /// order so output doesn't depend on HashMap iteration order.
    pub fn candidate_pairs(&self, max_pairs: usize) -> Vec<(usize, usize)> {
        let mut seen = std::collections::HashSet::new();
        let mut pairs = Vec::new();

        for band in &self.buckets {
            let mut keys: Vec<&u64> = band.keys().collect();
            keys.sort();

            for key in keys {
                let members = &band[key];
                if members.len() < 2 {
                    continue;
                }
                for a in 0..members.len() {
                    for b in (a + 1)..members.len() {
                        let pair = if members[a] < members[b] {
                            (members[a], members[b])
                        } else {
                            (members[b], members[a])
                        };
                        if seen.insert(pair) {
                            pairs.push(pair);
                            if pairs.len() >= max_pairs {
                                return pairs;
                            }
                        }
                    }
                }
            }
        }

        pairs
    }
}

impl Default for LshIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_bodies_land_in_same_bucket() {
        let src = "fn foo(x: i32) -> i32 { let y = x + 1; if y > 0 { return y; } return 0; }";
        let mut index = LshIndex::new();
        index.insert(0, src);
        index.insert(1, src);
        let pairs = index.candidate_pairs(100);
        assert_eq!(pairs, vec![(0, 1)]);
    }

    #[test]
    fn unrelated_bodies_rarely_collide() {
        let src_a = "fn foo() { println!(\"alpha beta gamma delta\"); }";
        let src_b = "fn bar() { std::process::exit(compute_something_else()); }";
        let mut index = LshIndex::new();
        index.insert(0, src_a);
        index.insert(1, src_b);
        assert!(index.candidate_pairs(100).is_empty());
    }

    #[test]
    fn deterministic_across_runs() {
        let src = "fn foo(a: i32, b: i32) -> i32 { a * b + a - b }";
        let mut idx1 = LshIndex::new();
        let mut idx2 = LshIndex::new();
        idx1.insert(0, src);
        idx1.insert(1, src);
        idx2.insert(0, src);
        idx2.insert(1, src);
        assert_eq!(idx1.candidate_pairs(10), idx2.candidate_pairs(10));
    }
}
