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
    MinHash,
    Fallback,
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
        _sources: &crate::optimize::dedup::core::SourceIndex,
    ) -> CandidateResult {
        let mut all_pairs = Vec::new();
        let mut used = std::collections::HashSet::new();
        let mut strategies_used = Vec::new();

        // Use signature hash as the primary strategy
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

        // Use param count as fallback
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

        for (_, indices) in buckets {
            if indices.len() < 2 {
                continue;
            }
            for a in 0..indices.len() {
                for b in (a + 1)..indices.len() {
                    pairs.push((indices[a], indices[b], score));
                    if pairs.len() >= self.config.max_functions_to_compare {
                        return pairs;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::call_graph::FunctionNode;

    fn create_test_function(name: &str, params: usize, returns: usize) -> FunctionNode {
        FunctionNode {
            name: name.to_string(),
            full_path: format!("test::{}", name),
            file: "test.rs".to_string(),
            line: 1,
            is_public: true,
            is_async: false,
            params: (0..params).map(|i| format!("p{}", i)).collect(),
            returns: (0..returns).map(|i| format!("r{}", i)).collect(),
            complexity: 1.0,
            importance_score: 0.0,
            doc_comment: None,
            writes_to: Vec::new(),
            reads_from: Vec::new(),
            errors: Vec::new(),
            fan_in: 0,
            fan_out: 0,
            is_cycle: false,
            depth: 0,
            layer: String::new(),
            trait_impl: None,
        }
    }

    #[test]
    fn test_candidate_generation() {
        let config = DedupConfig::default();
        let generator = CandidateGenerator::new(config);

        let functions = vec![
            create_test_function("foo", 2, 1),
            create_test_function("bar", 2, 1),
            create_test_function("baz", 3, 1),
        ];

        let sources = crate::optimize::dedup::core::SourceIndex::build(&functions, &[]);
        let result = generator.generate(&functions, &sources);

        assert!(result.total_functions == 3);
        assert!(result.total_candidates > 0);
        assert!(!result.strategies_used.is_empty());
    }
}
