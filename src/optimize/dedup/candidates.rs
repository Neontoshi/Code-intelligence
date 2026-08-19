//! Candidate generation - first-class phase before comparison

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::core::compute_signature_hash;
use crate::optimize::dedup::types::DedupConfig;
use std::collections::HashMap;

// ============================================================================
// Candidate Types
// ============================================================================

/// A candidate pair for duplicate detection
#[derive(Debug, Clone)]
pub struct CandidatePair {
    pub idx_a: usize,
    pub idx_b: usize,
    pub score: f64,
    pub strategy: CandidateStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CandidateStrategy {
    ExactHash,
    AstHash,
    SignatureHash,
    ParamCount,
    LSH,
}

/// Result of candidate generation
#[derive(Debug, Clone)]
pub struct CandidateResult {
    pub pairs: Vec<CandidatePair>,
    pub total_functions: usize,
    pub total_candidates: usize,
    pub strategies_used: Vec<CandidateStrategy>,
}

// ============================================================================
// Candidate Generator
// ============================================================================

pub struct CandidateGenerator {
    config: DedupConfig,
}

impl CandidateGenerator {
    pub fn new(config: DedupConfig) -> Self {
        Self { config }
    }

    pub fn generate(
        &self,
        functions: &[FunctionNode],
        sources: &crate::optimize::dedup::core::SourceIndex,
    ) -> CandidateResult {
        let mut all_pairs = Vec::new();
        let mut used = std::collections::HashSet::new();
        let mut strategies_used = Vec::new();

        // Signature hash: fast, coarse — catches identical param/return shape.
        let sig_pairs = self.generate_signature_candidates(functions);
        let count = self.add_pairs(
            &mut all_pairs,
            &mut used,
            sig_pairs,
            CandidateStrategy::SignatureHash,
        );
        if count > 0 {
            strategies_used.push(CandidateStrategy::SignatureHash);
        }

        // LSH over body content: catches near-duplicates regardless of
        // signature shape (different param counts, different return types).
        // This is the strategy that actually finds "same logic, different
        // wrapper" duplicates that signature/param bucketing structurally
        // cannot see.
        if self.config.enable_lsh_candidates
            && all_pairs.len() < self.config.max_functions_to_compare
        {
            let lsh_pairs = self.generate_lsh_candidates(functions, sources);
            let count =
                self.add_pairs(&mut all_pairs, &mut used, lsh_pairs, CandidateStrategy::LSH);
            if count > 0 {
                strategies_used.push(CandidateStrategy::LSH);
            }
        }

        // Param count: last-resort fallback for anything still uncovered.
        if all_pairs.len() < self.config.max_functions_to_compare {
            let param_pairs = self.generate_param_candidates(functions, &used);
            let count2 = self.add_pairs(
                &mut all_pairs,
                &mut used,
                param_pairs,
                CandidateStrategy::ParamCount,
            );
            if count2 > 0 {
                strategies_used.push(CandidateStrategy::ParamCount);
            }
        }

        // Limit total pairs
        if all_pairs.len() > self.config.max_functions_to_compare {
            all_pairs.truncate(self.config.max_functions_to_compare);
        }

        let total_candidates = all_pairs.len();

        CandidateResult {
            pairs: all_pairs,
            total_functions: functions.len(),
            total_candidates,
            strategies_used,
        }
    }

    /// Body-content candidates via MinHash/LSH. Assigns a moderate prior
    /// score (0.6) — LSH only proves shingle-bucket collision, not actual
    /// similarity, so downstream comparators (structural/semantic/ML) still
    /// do the real scoring. This is strictly a recall mechanism.
    fn generate_lsh_candidates(
        &self,
        functions: &[FunctionNode],
        sources: &crate::optimize::dedup::core::SourceIndex,
    ) -> Vec<(usize, usize, f64)> {
        let mut index = crate::optimize::dedup::minhash::LshIndex::new();

        for (i, func) in functions.iter().enumerate() {
            if let Some(src) = sources.get(&func.full_path) {
                index.insert(i, src);
            }
        }

        index
            .candidate_pairs(self.config.max_functions_to_compare)
            .into_iter()
            .map(|(a, b)| (a, b, 0.6))
            .collect()
    }

    // ================================================================
    // ⭐ KEEP THESE TWO FUNCTIONS (they're used as fallbacks)
    // ================================================================

    fn generate_signature_candidates(
        &self,
        functions: &[FunctionNode],
    ) -> Vec<(usize, usize, f64)> {
        let mut buckets: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, func) in functions.iter().enumerate() {
            let hash = compute_signature_hash(func);
            buckets.entry(hash).or_default().push(i);
        }

        self.pairs_from_buckets(buckets, 0.85)
    }

    fn generate_param_candidates(
        &self,
        functions: &[FunctionNode],
        used: &std::collections::HashSet<(usize, usize)>,
    ) -> Vec<(usize, usize, f64)> {
        let mut buckets: HashMap<usize, Vec<usize>> = HashMap::new();

        for (i, func) in functions.iter().enumerate() {
            buckets.entry(func.params.len()).or_default().push(i);
        }

        let mut pairs = Vec::new();
        for (_, indices) in buckets {
            for a in 0..indices.len() {
                for b in (a + 1)..indices.len() {
                    let key = (indices[a], indices[b]);
                    if !used.contains(&key) {
                        pairs.push((indices[a], indices[b], 0.7));
                    }
                    if pairs.len() >= self.config.max_functions_to_compare {
                        return pairs;
                    }
                }
            }
        }
        pairs
    }

    // ================================================================
    // Helper Functions
    // ================================================================

    fn pairs_from_buckets(
        &self,
        buckets: HashMap<String, Vec<usize>>,
        score: f64,
    ) -> Vec<(usize, usize, f64)> {
        let mut pairs = Vec::new();

        // Sort keys so results are reproducible across runs on the same input.
        let mut keys: Vec<&String> = buckets.keys().collect();
        keys.sort();

        for key in keys {
            let indices = &buckets[key];
            if indices.len() < 2 {
                continue;
            }

            // Cap pairs generated from any single oversized bucket so one
            // "shape" (e.g. common param/return signature) can't crowd out
            // every other candidate before the global limit is hit.
            let per_bucket_cap =
                self.config.max_functions_to_compare / Self::buckets_len_hint(indices.len());
            let mut emitted_this_bucket = 0;

            for a in 0..indices.len() {
                for b in (a + 1)..indices.len() {
                    pairs.push((indices[a], indices[b], score));
                    emitted_this_bucket += 1;
                    if pairs.len() >= self.config.max_functions_to_compare {
                        return pairs;
                    }
                    if emitted_this_bucket >= per_bucket_cap.max(50) {
                        break;
                    }
                }
            }
        }

        pairs
    }

    fn add_pairs(
        &self,
        all_pairs: &mut Vec<CandidatePair>,
        used: &mut std::collections::HashSet<(usize, usize)>,
        new_pairs: Vec<(usize, usize, f64)>,
        strategy: CandidateStrategy,
    ) -> usize {
        let mut count = 0;
        for (a, b, score) in new_pairs {
            let key = if a < b { (a, b) } else { (b, a) };
            if !used.contains(&key) {
                used.insert(key);
                all_pairs.push(CandidatePair {
                    idx_a: a,
                    idx_b: b,
                    score,
                    strategy: strategy.clone(),
                });
                count += 1;
            }
        }
        count
    }
}

impl CandidateGenerator {
    // Simple heuristic: don't let any one bucket eat the whole global budget.
    fn buckets_len_hint(_bucket_size: usize) -> usize {
        20
    }
}
// ============================================================================
// Default Config
// ============================================================================

impl Default for CandidateGenerator {
    fn default() -> Self {
        Self {
            config: DedupConfig::default(),
        }
    }
}
