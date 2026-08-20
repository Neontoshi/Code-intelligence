// src/optimize/dedup/candidates.rs

use crate::graph::call_graph::FunctionNode;
use crate::optimize::dedup::core::{compute_ast_hash, compute_signature_hash, SourceIndex};
use crate::optimize::dedup::filters::threshold::is_actionable_duplicate_candidate;
use crate::optimize::dedup::minhash::LshIndex;
use crate::optimize::dedup::types::DedupConfig;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CandidatePair {
    pub idx_a: usize,
    pub idx_b: usize,
    pub func_a: FunctionNode,
    pub func_b: FunctionNode,
}

#[derive(Debug, Clone, Default)]
pub struct CandidateResult {
    pub pairs: Vec<CandidatePair>,
    pub candidate_pairs: Vec<CandidatePair>,
    pub total_comparisons: usize,
    pub filtered_comparisons: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CandidateStrategy {
    ExactHash,
    MinHashLsh,
    SignatureBucket,
    AllPairs,
}

pub struct CandidateGenerator {
    #[allow(dead_code)]
    config: DedupConfig,
}

impl CandidateGenerator {
    pub fn new(config: DedupConfig) -> Self {
        Self { config }
    }

    pub fn generate(&self, functions: &[FunctionNode], sources: &SourceIndex) -> CandidateResult {
        let mut pairs = Vec::new();
        let mut seen_pairs = HashSet::new();

        // 1. Collect actionable functions along with their original indices
        let indexed_actionable: Vec<(usize, &FunctionNode)> = functions
            .iter()
            .enumerate()
            .filter(|(_, f)| is_actionable_duplicate_candidate(f))
            .collect();

        // 2. Exact Hash Bucketing
        let mut ast_hash_buckets: HashMap<String, Vec<(usize, &FunctionNode)>> = HashMap::new();
        let mut sig_hash_buckets: HashMap<String, Vec<(usize, &FunctionNode)>> = HashMap::new();

        for (idx, func) in &indexed_actionable {
            if let Some(source) = sources.get(&func.full_path) {
                let ast_hash = compute_ast_hash(func, source);
                if !ast_hash.is_empty() {
                    ast_hash_buckets
                        .entry(ast_hash)
                        .or_default()
                        .push((*idx, func));
                }
            }
            let sig_hash = compute_signature_hash(func);
            if !sig_hash.is_empty() {
                sig_hash_buckets
                    .entry(sig_hash)
                    .or_default()
                    .push((*idx, func));
            }
        }

        for bucket in ast_hash_buckets.values() {
            if bucket.len() > 1 {
                for i in 0..bucket.len() {
                    for j in (i + 1)..bucket.len() {
                        let (idx_1, func_1) = bucket[i];
                        let (idx_2, func_2) = bucket[j];

                        let pair_key = if idx_1 < idx_2 {
                            (idx_1, idx_2)
                        } else {
                            (idx_2, idx_1)
                        };

                        if seen_pairs.insert(pair_key) {
                            pairs.push(CandidatePair {
                                idx_a: pair_key.0,
                                idx_b: pair_key.1,
                                func_a: func_1.clone(),
                                func_b: func_2.clone(),
                            });
                        }
                    }
                }
            }
        }

        // 3. MinHash LSH Indexing
        let mut lsh = LshIndex::new();
        for (orig_idx, func) in &indexed_actionable {
            if let Some(source) = sources.get(&func.full_path) {
                if !source.trim().is_empty() {
                    lsh.insert(*orig_idx, source);
                }
            }
        }

        let lsh_pairs = lsh.candidate_pairs(10000);
        for (idx_a, idx_b) in lsh_pairs {
            if idx_a < functions.len() && idx_b < functions.len() {
                let func_a = &functions[idx_a];
                let func_b = &functions[idx_b];

                if is_actionable_duplicate_candidate(func_a)
                    && is_actionable_duplicate_candidate(func_b)
                {
                    let pair_key = if idx_a < idx_b {
                        (idx_a, idx_b)
                    } else {
                        (idx_b, idx_a)
                    };

                    if seen_pairs.insert(pair_key) {
                        pairs.push(CandidatePair {
                            idx_a: pair_key.0,
                            idx_b: pair_key.1,
                            func_a: func_a.clone(),
                            func_b: func_b.clone(),
                        });
                    }
                }
            }
        }

        let total_possible = if functions.len() > 1 {
            (functions.len() * (functions.len() - 1)) / 2
        } else {
            0
        };

        CandidateResult {
            candidate_pairs: pairs.clone(),
            pairs,
            total_comparisons: total_possible,
            filtered_comparisons: seen_pairs.len(),
        }
    }
}

impl Default for CandidateGenerator {
    fn default() -> Self {
        Self::new(DedupConfig::default())
    }
}
