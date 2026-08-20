use crate::graph::call_graph::FunctionNode;

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct DeduplicationResult {
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub unique_functions: Vec<FunctionNode>,
    pub total_saved_tokens: usize,
    pub accuracy_metrics: AccuracyMetrics,
}

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub functions: Vec<FunctionNode>,
    pub similarity_score: f64,
    pub duplicate_type: DuplicateType,
    pub refactoring_suggestion: String,
    pub estimated_savings: usize,
    pub priority_score: f64,
    pub total_token_savings: usize,
    pub complexity_impact: f64,
    pub confidence_score: f64, // ⭐ NEW: ML confidence score
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DuplicateType {
    Exact,
    Structural,
    Algorithmic,
    Partial,
    FalsePositive,
}

#[derive(Debug, Clone)]
pub struct AccuracyMetrics {
    pub total_comparisons: usize,
    pub exact_matches: usize,
    pub structural_matches: usize,
    pub algorithmic_matches: usize,
    pub false_positives_filtered: usize,
    pub confidence_score: f64,
}

// ============================================================================
// Analysis Results
// ============================================================================

#[derive(Debug, Clone)]
pub struct ASTSignature {
    pub node_types: Vec<String>,
    pub depth: usize,
    pub complexity: f64,
}

#[derive(Debug, Clone)]
pub struct DataFlowSignature {
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub transformations: Vec<String>,
    pub async_depth: usize,
}

#[derive(Debug, Clone)]
pub struct SimilarityScores {
    pub structural: f64,
    pub semantic: f64,
    pub call_graph: f64,
    pub ast: f64,
    pub data_flow: f64,
    pub ml: f64,
    pub context: f64,
}

#[derive(Debug, Clone)]
pub struct FileContext {
    pub directory: String,
    pub filename: String,
    pub extension: String,
    pub module_path: String,
}

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct DedupConfig {
    pub min_similarity_threshold: f64,
    pub enable_call_graph_analysis: bool,
    pub enable_semantic_analysis: bool,
    pub enable_ml_features: bool,
    pub enable_lsh_candidates: bool,
    pub max_functions_to_compare: usize,
    pub adaptive_threshold: bool,
    pub per_signal_threshold: f64,
    pub min_signal_agreement: usize,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            min_similarity_threshold: 0.85,
            enable_call_graph_analysis: true,
            enable_semantic_analysis: true,
            enable_ml_features: true,
            enable_lsh_candidates: true,
            max_functions_to_compare: 10000,
            adaptive_threshold: true,
            per_signal_threshold: 0.75,
            min_signal_agreement: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub structural: f64,
    pub context: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            structural: 0.7,
            context: 0.3,
        }
    }
}

pub fn combine(scores: &SimilarityScores, weights: &ScoreWeights) -> f64 {
    scores.structural * weights.structural + scores.context * weights.context
}

/// One row of independently-computed signals for a single candidate pair.
/// Consensus requires 2+ of the *available* signals to each clear the bar
/// on their own — a single strong signal (e.g. name similarity alone) can
/// no longer carry a pair over the line.
#[derive(Debug, Clone, Default)]
pub struct SignalVerdict {
    pub structural: Option<f64>,
    pub semantic: Option<f64>,
    pub call_graph: Option<f64>,
    pub ml: Option<f64>,
}

impl SignalVerdict {
    pub fn is_duplicate(&self, per_signal_bar: f64, min_agreement: usize) -> bool {
        let mut hits = 0;
        let mut available = 0;

        for score in [self.structural, self.semantic, self.call_graph, self.ml] {
            if let Some(s) = score {
                available += 1;
                if s >= per_signal_bar {
                    hits += 1;
                }
            }
        }

        // `structural` and `semantic` both weight function naming heavily
        // (40% and 60% of their respective scores), so they aren't
        // independent evidence — two same-shaped, similarly-named
        // functions can clear both without sharing any real logic.
        // Require at least one signal that actually looks at behavior
        // (call graph or ML features) before counting agreement.
        let content_aware_hit = self.call_graph.is_some_and(|s| s >= per_signal_bar)
            || self.ml.is_some_and(|s| s >= per_signal_bar);

        available >= min_agreement && hits >= min_agreement && content_aware_hit
    }
    /// Returns how many signals were actually computed for this pair
    pub fn signal_count(&self) -> usize {
        [self.structural, self.semantic, self.call_graph, self.ml]
            .iter()
            .filter(|s| s.is_some())
            .count()
    }

    /// Average of whatever signals were actually computed, for reporting
    /// (DuplicateGroup.similarity_score) — not used for the pass/fail decision.
    pub fn mean_score(&self) -> f64 {
        let scores: Vec<f64> = [self.structural, self.semantic, self.call_graph, self.ml]
            .into_iter()
            .flatten()
            .collect();
        if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        }
    }

    /// Which DuplicateType label best fits this verdict, for reporting.
    pub fn dominant_type(&self) -> DuplicateType {
        let cg = self.call_graph.unwrap_or(0.0);
        let ml = self.ml.unwrap_or(0.0);
        let structural = self.structural.unwrap_or(0.0);

        if cg >= structural && cg >= ml && cg > 0.0 {
            DuplicateType::Algorithmic
        } else if ml >= structural && ml > 0.0 {
            DuplicateType::Partial
        } else {
            DuplicateType::Structural
        }
    }
}
