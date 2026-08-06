//! Deduplication module - Modular duplicate detection system

pub mod analyzers;
pub mod candidates;
pub mod comparators;
pub mod core;
pub mod filters;
pub mod llm_analyzer;
pub mod reporters;
pub mod types;

// Re-export main types
pub use candidates::{CandidateGenerator, CandidatePair, CandidateResult, CandidateStrategy};
pub use core::{compute_ast_hash, compute_exact_hash, compute_signature_hash, SourceIndex};
pub use llm_analyzer::LLMAnalyzer;
pub use types::{
    AccuracyMetrics, DedupConfig, DeduplicationResult, DuplicateGroup, DuplicateType,
    SimilarityScores,
};

// Internal imports for the main Deduplicator
use crate::graph::call_graph::{CallGraph, FunctionNode};
use crate::optimize::dedup::analyzers::MLAnalyzer;
use crate::optimize::dedup::comparators::{
    CallGraphComparator, SemanticComparator, StructuralComparator,
};
use crate::optimize::dedup::filters::{FalsePositiveFilter, ThresholdTuner};
use crate::optimize::dedup::reporters::ReportGenerator;
use crate::optimize::dedup::types::{combine, ScoreWeights, SignalVerdict};
use crate::parser::tree_sitter::ParsedFile;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Union-Find for cluster merging
// ============================================================================

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }
        if self.rank[rx] < self.rank[ry] {
            self.parent[rx] = ry;
        } else if self.rank[rx] > self.rank[ry] {
            self.parent[ry] = rx;
        } else {
            self.parent[ry] = rx;
            self.rank[rx] += 1;
        }
    }

    fn get_clusters(&mut self, n: usize) -> Vec<Vec<usize>> {
        use std::collections::HashMap;
        let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let root = self.find(i);
            clusters.entry(root).or_default().push(i);
        }
        clusters.into_values().filter(|c| c.len() > 1).collect()
    }
}

pub struct Deduplicator {
    config: DedupConfig,
    weights: ScoreWeights,
}

impl Deduplicator {
    pub fn new() -> Self {
        Self {
            config: DedupConfig::default(),
            weights: ScoreWeights::default(),
        }
    }

    pub fn with_config(mut self, config: DedupConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.config.min_similarity_threshold = threshold;
        self
    }

    pub fn find_duplicates(
        &self,
        call_graph: &CallGraph,
        files: &[ParsedFile],
    ) -> DeduplicationResult {
        let functions: Vec<FunctionNode> = call_graph
            .node_indices()
            .map(|idx| call_graph[idx].clone())
            .collect();

        let mut metrics = AccuracyMetrics {
            total_comparisons: 0,
            exact_matches: 0,
            structural_matches: 0,
            algorithmic_matches: 0,
            false_positives_filtered: 0,
            confidence_score: 0.0,
        };

        if functions.len() < 2 {
            return DeduplicationResult {
                duplicate_groups: Vec::new(),
                unique_functions: functions,
                total_saved_tokens: 0,
                accuracy_metrics: metrics,
            };
        }

        let sources = SourceIndex::build(&functions, files);
        // Enforce max_functions_to_compare instead of only storing it.
        // Cheap pre-bucketing by param-count band keeps this from becoming an
        // unbounded O(n^2) scan on large codebases.
        // Three-tier bucketing: signature hash → AST hash → param count fallback
        let candidate_pairs = self.build_candidate_pairs(&functions, &sources);
        metrics.total_comparisons = candidate_pairs.len();

        let threshold = if self.config.adaptive_threshold {
            ThresholdTuner::auto_tune(&functions)
        } else {
            self.config.min_similarity_threshold
        };

        let mut processed = HashSet::new();
        let mut duplicate_groups = Vec::new();
        let mut all_duplicates = HashSet::new();

        // Phase 1: Exact matches (fast, now actually functional)
        let exact_groups = self.find_exact_duplicates(&functions, &sources);
        self.process_groups(
            exact_groups,
            &sources,
            &mut processed,
            &mut all_duplicates,
            &mut duplicate_groups,
            &mut metrics,
            threshold,
            DuplicateType::Exact,
        );

        // Phase 2: unified consensus pass
        let consensus_groups = self.find_consensus_duplicates(
            &functions,
            call_graph,
            &candidate_pairs,
            &processed,
            &sources,
        );
        for (group, verdict) in consensus_groups {
            let duplicate_type = verdict.dominant_type();
            let _signal_count = verdict.signal_count();
            #[cfg(debug_assertions)]
            {
                let func_a = &group[0];
                let func_b = &group[1];
                eprintln!(
                    "🔍 Duplicate found: {} ↔ {} (signals: {}, score: {:.3})",
                    func_a.name,
                    func_b.name,
                    _signal_count,
                    verdict.mean_score()
                );
            }
            self.process_groups(
                vec![group],
                &sources,
                &mut processed,
                &mut all_duplicates,
                &mut duplicate_groups,
                &mut metrics,
                threshold,
                duplicate_type,
            );
        }

        let unique_functions: Vec<FunctionNode> = functions
            .into_iter()
            .filter(|f| !all_duplicates.contains(&f.full_path))
            .collect();

        let total_saved_tokens: usize = duplicate_groups.iter().map(|g| g.estimated_savings).sum();

        let total_groups = duplicate_groups.len();
        let confidence = if total_groups > 0 {
            let exact_ratio = metrics.exact_matches as f64 / total_groups as f64;
            let structural_ratio = metrics.structural_matches as f64 / total_groups as f64;
            let algorithmic_ratio = metrics.algorithmic_matches as f64 / total_groups as f64;
            (exact_ratio * 1.0 + structural_ratio * 0.95 + algorithmic_ratio * 0.90)
                / (exact_ratio + structural_ratio + algorithmic_ratio + 0.01)
        } else {
            1.0
        };

        metrics.confidence_score = confidence;

        DeduplicationResult {
            duplicate_groups,
            unique_functions,
            total_saved_tokens,
            accuracy_metrics: metrics,
        }
    }

    fn find_exact_duplicates(
        &self,
        functions: &[FunctionNode],
        sources: &SourceIndex,
    ) -> Vec<Vec<FunctionNode>> {
        let mut hash_map: HashMap<String, Vec<FunctionNode>> = HashMap::new();

        for func in functions {
            let hash = compute_exact_hash(func, sources.get(&func.full_path));
            hash_map
                .entry(hash)
                .or_insert_with(Vec::new)
                .push(func.clone());
        }

        hash_map
            .into_iter()
            .filter(|(_, group)| group.len() > 1)
            .map(|(_, group)| group)
            .collect()
    }

    fn build_candidate_pairs(
        &self,
        functions: &[FunctionNode],
        sources: &SourceIndex,
    ) -> Vec<(usize, usize)> {
        let generator = CandidateGenerator::new(self.config.clone());
        let result = generator.generate(functions, sources);

        // Log strategy usage
        let strategy_names: Vec<String> = result
            .strategies_used
            .iter()
            .map(|s| format!("{:?}", s))
            .collect();
        eprintln!(
            "   🎯 Candidate strategies: {} ({} pairs)",
            strategy_names.join(" → "),
            result.total_candidates
        );

        result
            .pairs
            .into_iter()
            .map(|p| (p.idx_a, p.idx_b))
            .collect()
    }

    fn find_consensus_duplicates(
        &self,
        functions: &[FunctionNode],
        call_graph: &CallGraph,
        candidate_pairs: &[(usize, usize)],
        processed: &HashSet<String>,
        sources: &SourceIndex,
    ) -> Vec<(Vec<FunctionNode>, SignalVerdict)> {
        let mut groups = Vec::new();
        let mut used = processed.clone();
        let mut total_pairs_checked = 0;
        let mut pairs_with_consensus = 0;

        for &(i, j) in candidate_pairs {
            let func_a = &functions[i];
            let func_b = &functions[j];

            if used.contains(&func_a.full_path) || used.contains(&func_b.full_path) {
                continue;
            }

            let src_a = sources.get(&func_a.full_path);
            let src_b = sources.get(&func_b.full_path);

            if FalsePositiveFilter::is_likely_false_positive(func_a, src_a)
                || FalsePositiveFilter::is_likely_false_positive(func_b, src_b)
            {
                continue;
            }

            let mut verdict = SignalVerdict::default();

            if self.config.enable_semantic_analysis {
                let scores = StructuralComparator::compare(func_a, func_b);
                verdict.structural = Some(combine(&scores, &self.weights));
                verdict.semantic = Some(SemanticComparator::compare(func_a, func_b));
            }

            if self.config.enable_call_graph_analysis {
                verdict.call_graph = Some(CallGraphComparator::compare(func_a, func_b, call_graph));
            }

            if self.config.enable_ml_features {
                if let (Some(sa), Some(sb)) = (src_a, src_b) {
                    let features_a = MLAnalyzer::extract_features(func_a, sa);
                    let features_b = MLAnalyzer::extract_features(func_b, sb);
                    verdict.ml = Some(MLAnalyzer::cosine_similarity(&features_a, &features_b));
                }
            }

            total_pairs_checked += 1;

            if verdict.is_duplicate(
                self.config.per_signal_threshold,
                self.config.min_signal_agreement,
            ) {
                pairs_with_consensus += 1;
                used.insert(func_a.full_path.clone());
                used.insert(func_b.full_path.clone());
                groups.push((vec![func_a.clone(), func_b.clone()], verdict));
            }
        }

        // Log consensus statistics
        if total_pairs_checked > 0 {
            let consensus_rate = pairs_with_consensus as f64 / total_pairs_checked as f64 * 100.0;
            eprintln!(
                "📊 Consensus: {} pairs checked, {} passed ({:.1}%)",
                total_pairs_checked, pairs_with_consensus, consensus_rate
            );
        }

        groups
    }

    fn process_groups(
        &self,
        groups: Vec<Vec<FunctionNode>>,
        sources: &SourceIndex,
        processed: &mut HashSet<String>,
        all_duplicates: &mut HashSet<String>,
        duplicate_groups: &mut Vec<DuplicateGroup>,
        metrics: &mut AccuracyMetrics,
        threshold: f64,
        duplicate_type: DuplicateType,
    ) {
        // First, collect all function indices from all groups
        let mut all_funcs: Vec<FunctionNode> = Vec::new();
        let mut func_to_idx: HashMap<String, usize> = HashMap::new();

        for group in &groups {
            for func in group {
                if !func_to_idx.contains_key(&func.full_path) {
                    func_to_idx.insert(func.full_path.clone(), all_funcs.len());
                    all_funcs.push(func.clone());
                }
            }
        }

        // Build Union-Find and union functions within each group
        let mut uf = UnionFind::new(all_funcs.len());

        for group in &groups {
            if group.len() < 2 {
                continue;
            }

            // Skip false positives
            if FalsePositiveFilter::filter_duplicate_group(group, Some(sources)) {
                metrics.false_positives_filtered += group.len();
                continue;
            }

            // Calculate similarity for this group
            let similarity = self.calculate_group_similarity(group);
            if similarity < threshold {
                metrics.false_positives_filtered += group.len();
                continue;
            }

            // Union all functions in this group
            let first_idx = func_to_idx[&group[0].full_path];
            for func in &group[1..] {
                let idx = func_to_idx[&func.full_path];
                uf.union(first_idx, idx);
            }
        }

        // Extract clusters and create DuplicateGroups
        // Extract clusters and create DuplicateGroups
        let clusters = uf.get_clusters(all_funcs.len());

        for cluster in clusters {
            let cluster_funcs: Vec<FunctionNode> =
                cluster.iter().map(|&idx| all_funcs[idx].clone()).collect();

            if cluster_funcs.len() < 2 {
                continue;
            }

            // Mark all functions as processed/duplicates
            for func in &cluster_funcs {
                processed.insert(func.full_path.clone());
                all_duplicates.insert(func.full_path.clone());
            }

            // Calculate group similarity
            let similarity = self.calculate_group_similarity(&cluster_funcs);

            let group_size = cluster_funcs.len();
            let avg_complexity: f64 =
                cluster_funcs.iter().map(|f| f.complexity).sum::<f64>() / group_size as f64;
            let avg_fan_in: f64 =
                cluster_funcs.iter().map(|f| f.fan_in as f64).sum::<f64>() / group_size as f64;

            let priority_score = (group_size as f64 * 0.4)
                + ((avg_complexity / 50.0).min(1.0) * 0.3)
                + ((avg_fan_in / 10.0).min(1.0) * 0.3);
            let priority_score = priority_score.min(1.0);

            // Calculate token savings using TokenEstimator
            let mut total_source_len = 0;
            let mut total_compressed_len = 0;
            for func in &cluster_funcs {
                if let Some(source) = sources.get(&func.full_path) {
                    total_source_len += source.len();
                    // Estimate compressed: remove duplicates within group
                    total_compressed_len += source.len() / group_size;
                }
            }
            let total_token_savings = (total_source_len - total_compressed_len) / 4; // rough tokens

            let duplicate_group = DuplicateGroup {
                functions: cluster_funcs.clone(),
                similarity_score: similarity,
                duplicate_type: duplicate_type.clone(),
                refactoring_suggestion: self.get_suggestion(&duplicate_type),
                estimated_savings: self.estimate_savings(&cluster_funcs),
                priority_score,
                total_token_savings,
                complexity_impact: avg_complexity / 50.0,
            };
            duplicate_groups.push(duplicate_group);

            match duplicate_type {
                DuplicateType::Exact => metrics.exact_matches += 1,
                DuplicateType::Structural => metrics.structural_matches += 1,
                DuplicateType::Algorithmic => metrics.algorithmic_matches += 1,
                DuplicateType::Partial => {}
                DuplicateType::FalsePositive => {}
            }
        }
    }

    fn calculate_group_similarity(&self, group: &[FunctionNode]) -> f64 {
        if group.len() < 2 {
            return 0.0;
        }

        let mut total_sim = 0.0;
        let mut comparisons = 0;

        for i in 0..group.len() {
            for j in (i + 1)..group.len() {
                let scores = StructuralComparator::compare(&group[i], &group[j]);
                // Same weights as everywhere else now — previously this used
                // a different 0.6/0.4 blend than auto-tune and phase 2's 0.7/0.3.
                total_sim += combine(&scores, &self.weights);
                comparisons += 1;
            }
        }

        if comparisons == 0 {
            1.0
        } else {
            total_sim / comparisons as f64
        }
    }

    fn get_suggestion(&self, duplicate_type: &DuplicateType) -> String {
        match duplicate_type {
            DuplicateType::Exact => "Extract to shared function".to_string(),
            DuplicateType::Structural => "Extract common structure".to_string(),
            DuplicateType::Algorithmic => "Extract algorithm to shared function".to_string(),
            DuplicateType::Partial => "Consider extracting common logic".to_string(),
            DuplicateType::FalsePositive => "Review - likely not a duplicate".to_string(),
        }
    }

    fn estimate_savings(&self, group: &[FunctionNode]) -> usize {
        let base_cost = 100;
        let duplicate_count = group.len() - 1;
        base_cost * duplicate_count
    }
}

impl Default for Deduplicator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Report Generation (Delegates to ReportGenerator)
// ============================================================================

impl Deduplicator {
    pub fn report(&self, result: &DeduplicationResult) -> String {
        ReportGenerator::generate(result)
    }
}
